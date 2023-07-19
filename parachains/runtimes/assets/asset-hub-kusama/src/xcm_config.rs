// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::{
	AccountId, AllPalletsWithSystem, Assets, Authorship, Balance, Balances, ForeignAssets,
	ParachainInfo, ParachainSystem, PolkadotXcm, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin,
	TrustBackedAssetsInstance, WeightToFee, XcmpQueue,
};
use assets_common::matching::{
	Equals, FromSiblingParachain, IsForeignConcreteAsset, StartsWith,
	StartsWithExplicitGlobalConsensus,
};
use frame_support::{
	match_types,
	pallet_prelude::Get,
	parameter_types,
	traits::{ConstU32, Contains, Everything, Nothing, PalletInfoAccess},
};
use frame_system::EnsureRoot;
use pallet_xcm::{
	destination_fees::{DestinationFees, DestinationFeesManager, DestinationFeesSetup},
	XcmPassthrough,
};
use parachains_common::{impls::ToStakingPot, xcm_config::AssetFeeAsExistentialDepositMultiplier};
use polkadot_parachain::primitives::Sibling;
use sp_runtime::{traits::ConvertInto, Perbill};
use xcm::latest::prelude::*;
use xcm_builder::{
	AccountId32Aliases, AllowAliasOriginWithdrawPaidExecutionFrom,
	AllowExplicitUnpaidExecutionFrom, AllowKnownQueryResponses, AllowSubscriptionsFrom,
	AllowTopLevelPaidExecutionFrom, CurrencyAdapter, DenyReserveTransferToRelayChain, DenyThenTry,
	DescribeAllTerminal, DescribeFamily, EnsureXcmOrigin, FungiblesAdapter,
	GlobalConsensusParachainConvertsFor, HashedDescription, IsConcrete, LocalMint, NativeAsset,
	NoChecking, ParentAsSuperuser, ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative,
	SiblingParachainConvertsVia, SignedAccountId32AsNative, SignedToAccountId32,
	SovereignSignedViaLocation, TakeWeightCredit, TrailingSetTopicAsId, UsingComponents,
	WeightInfoBounds, WithComputedOrigin, WithUniqueTopic,
};
use xcm_executor::{traits::WithOriginFilter, XcmExecutor};

parameter_types! {
	pub const KsmLocation: MultiLocation = MultiLocation::parent();
	pub const RelayNetwork: Option<NetworkId> = Some(NetworkId::Kusama);
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub UniversalLocation: InteriorMultiLocation =
		X2(GlobalConsensus(RelayNetwork::get().unwrap()), Parachain(ParachainInfo::parachain_id().into()));
	pub UniversalLocationNetworkId: NetworkId = UniversalLocation::get().global_consensus().unwrap();
	pub TrustBackedAssetsPalletLocation: MultiLocation =
		PalletInstance(<Assets as PalletInfoAccess>::index() as u8).into();
	pub CheckingAccount: AccountId = PolkadotXcm::check_account();
	pub const GovernanceLocation: MultiLocation = MultiLocation::parent();
	pub const FellowshipLocation: MultiLocation = MultiLocation::parent();
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the parent `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
	// Foreign locations alias into accounts according to a hash of their standard description.
	HashedDescription<AccountId, DescribeFamily<DescribeAllTerminal>>,
	// Different global consensus parachain sovereign account.
	// (Used for over-bridge transfers and reserve processing)
	GlobalConsensusParachainConvertsFor<UniversalLocation, AccountId>,
);

/// Means for transacting the native currency on this chain.
pub type CurrencyTransactor = CurrencyAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<KsmLocation>,
	// Convert an XCM MultiLocation into a local account id:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports of `Balances`.
	(),
>;

/// `AssetId/Balance` converter for `TrustBackedAssets`
pub type TrustBackedAssetsConvertedConcreteId =
	assets_common::TrustBackedAssetsConvertedConcreteId<TrustBackedAssetsPalletLocation, Balance>;

/// Means for transacting assets besides the native currency on this chain.
pub type FungiblesTransactor = FungiblesAdapter<
	// Use this fungibles implementation:
	Assets,
	// Use this currency when it is a fungible asset matching the given location or name:
	TrustBackedAssetsConvertedConcreteId,
	// Convert an XCM MultiLocation into a local account id:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We only want to allow teleports of known assets. We use non-zero issuance as an indication
	// that this asset is known.
	LocalMint<parachains_common::impls::NonZeroIssuance<AccountId, Assets>>,
	// The account to use for tracking teleports.
	CheckingAccount,
>;

/// `AssetId/Balance` converter for `TrustBackedAssets`
pub type ForeignAssetsConvertedConcreteId = assets_common::ForeignAssetsConvertedConcreteId<
	(
		// Ignore `TrustBackedAssets` explicitly
		StartsWith<TrustBackedAssetsPalletLocation>,
		// Ignore assets that start explicitly with our `GlobalConsensus(NetworkId)`, means:
		// - foreign assets from our consensus should be: `MultiLocation {parents: 1, X*(Parachain(xyz), ..)}`
		// - foreign assets outside our consensus with the same `GlobalConsensus(NetworkId)` won't be accepted here
		StartsWithExplicitGlobalConsensus<UniversalLocationNetworkId>,
	),
	Balance,
>;

/// Means for transacting foreign assets from different global consensus.
pub type ForeignFungiblesTransactor = FungiblesAdapter<
	// Use this fungibles implementation:
	ForeignAssets,
	// Use this currency when it is a fungible asset matching the given location or name:
	ForeignAssetsConvertedConcreteId,
	// Convert an XCM MultiLocation into a local account id:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We dont need to check teleports here.
	NoChecking,
	// The account to use for tracking teleports.
	CheckingAccount,
>;

/// Means for transacting assets on this chain.
pub type AssetTransactors = (CurrencyTransactor, FungiblesTransactor, ForeignFungiblesTransactor);

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	// Native converter for Relay-chain (Parent) location; will convert to a `Relay` origin when
	// recognised.
	RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognised.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	// Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
	// transaction from the Root origin.
	ParentAsSuperuser<RuntimeOrigin>,
	// Native signed account converter; this just converts an `AccountId32` origin into a normal
	// `RuntimeOrigin::Signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<RuntimeOrigin>,
);

parameter_types! {
	pub const MaxInstructions: u32 = 100;
	pub const MaxAssetsIntoHolding: u32 = 64;
	pub XcmAssetFeesReceiver: Option<AccountId> = Authorship::author();
}

match_types! {
	pub type ParentOrParentsPlurality: impl Contains<MultiLocation> = {
		MultiLocation { parents: 1, interior: Here } |
		MultiLocation { parents: 1, interior: X1(Plurality { .. }) }
	};
	pub type ParentOrSiblings: impl Contains<MultiLocation> = {
		MultiLocation { parents: 1, interior: Here } |
		MultiLocation { parents: 1, interior: X1(_) }
	};
}

/// A call filter for the XCM Transact instruction. This is a temporary measure until we properly
/// account for proof size weights.
///
/// Calls that are allowed through this filter must:
/// 1. Have a fixed weight;
/// 2. Cannot lead to another call being made;
/// 3. Have a defined proof size weight, e.g. no unbounded vecs in call parameters.
pub struct SafeCallFilter;
impl Contains<RuntimeCall> for SafeCallFilter {
	fn contains(call: &RuntimeCall) -> bool {
		#[cfg(feature = "runtime-benchmarks")]
		{
			if matches!(call, RuntimeCall::System(frame_system::Call::remark_with_event { .. })) {
				return true
			}
		}

		matches!(
			call,
			RuntimeCall::PolkadotXcm(pallet_xcm::Call::force_xcm_version { .. }) |
				RuntimeCall::System(
					frame_system::Call::set_heap_pages { .. } |
						frame_system::Call::set_code { .. } |
						frame_system::Call::set_code_without_checks { .. } |
						frame_system::Call::kill_prefix { .. },
				) | RuntimeCall::ParachainSystem(..) |
				RuntimeCall::Timestamp(..) |
				RuntimeCall::Balances(..) |
				RuntimeCall::CollatorSelection(
					pallet_collator_selection::Call::set_desired_candidates { .. } |
						pallet_collator_selection::Call::set_candidacy_bond { .. } |
						pallet_collator_selection::Call::register_as_candidate { .. } |
						pallet_collator_selection::Call::leave_intent { .. } |
						pallet_collator_selection::Call::set_invulnerables { .. } |
						pallet_collator_selection::Call::add_invulnerable { .. } |
						pallet_collator_selection::Call::remove_invulnerable { .. },
				) | RuntimeCall::Session(pallet_session::Call::purge_keys { .. }) |
				RuntimeCall::XcmpQueue(..) |
				RuntimeCall::DmpQueue(..) |
				RuntimeCall::Utility(pallet_utility::Call::as_derivative { .. }) |
				RuntimeCall::Assets(
					pallet_assets::Call::create { .. } |
						pallet_assets::Call::force_create { .. } |
						pallet_assets::Call::start_destroy { .. } |
						pallet_assets::Call::destroy_accounts { .. } |
						pallet_assets::Call::destroy_approvals { .. } |
						pallet_assets::Call::finish_destroy { .. } |
						pallet_assets::Call::mint { .. } |
						pallet_assets::Call::burn { .. } |
						pallet_assets::Call::transfer { .. } |
						pallet_assets::Call::transfer_keep_alive { .. } |
						pallet_assets::Call::force_transfer { .. } |
						pallet_assets::Call::freeze { .. } |
						pallet_assets::Call::thaw { .. } |
						pallet_assets::Call::freeze_asset { .. } |
						pallet_assets::Call::thaw_asset { .. } |
						pallet_assets::Call::transfer_ownership { .. } |
						pallet_assets::Call::set_team { .. } |
						pallet_assets::Call::clear_metadata { .. } |
						pallet_assets::Call::force_clear_metadata { .. } |
						pallet_assets::Call::force_asset_status { .. } |
						pallet_assets::Call::approve_transfer { .. } |
						pallet_assets::Call::cancel_approval { .. } |
						pallet_assets::Call::force_cancel_approval { .. } |
						pallet_assets::Call::transfer_approved { .. } |
						pallet_assets::Call::touch { .. } |
						pallet_assets::Call::refund { .. },
				) | RuntimeCall::ForeignAssets(
				pallet_assets::Call::create { .. } |
					pallet_assets::Call::force_create { .. } |
					pallet_assets::Call::start_destroy { .. } |
					pallet_assets::Call::destroy_accounts { .. } |
					pallet_assets::Call::destroy_approvals { .. } |
					pallet_assets::Call::finish_destroy { .. } |
					pallet_assets::Call::mint { .. } |
					pallet_assets::Call::burn { .. } |
					pallet_assets::Call::transfer { .. } |
					pallet_assets::Call::transfer_keep_alive { .. } |
					pallet_assets::Call::force_transfer { .. } |
					pallet_assets::Call::freeze { .. } |
					pallet_assets::Call::thaw { .. } |
					pallet_assets::Call::freeze_asset { .. } |
					pallet_assets::Call::thaw_asset { .. } |
					pallet_assets::Call::transfer_ownership { .. } |
					pallet_assets::Call::set_team { .. } |
					pallet_assets::Call::set_metadata { .. } |
					pallet_assets::Call::clear_metadata { .. } |
					pallet_assets::Call::force_clear_metadata { .. } |
					pallet_assets::Call::force_asset_status { .. } |
					pallet_assets::Call::approve_transfer { .. } |
					pallet_assets::Call::cancel_approval { .. } |
					pallet_assets::Call::force_cancel_approval { .. } |
					pallet_assets::Call::transfer_approved { .. } |
					pallet_assets::Call::touch { .. } |
					pallet_assets::Call::refund { .. },
			) | RuntimeCall::NftFractionalization(
				pallet_nft_fractionalization::Call::fractionalize { .. } |
					pallet_nft_fractionalization::Call::unify { .. },
			) | RuntimeCall::Nfts(
				pallet_nfts::Call::create { .. } |
					pallet_nfts::Call::force_create { .. } |
					pallet_nfts::Call::destroy { .. } |
					pallet_nfts::Call::mint { .. } |
					pallet_nfts::Call::force_mint { .. } |
					pallet_nfts::Call::burn { .. } |
					pallet_nfts::Call::transfer { .. } |
					pallet_nfts::Call::lock_item_transfer { .. } |
					pallet_nfts::Call::unlock_item_transfer { .. } |
					pallet_nfts::Call::lock_collection { .. } |
					pallet_nfts::Call::transfer_ownership { .. } |
					pallet_nfts::Call::set_team { .. } |
					pallet_nfts::Call::force_collection_owner { .. } |
					pallet_nfts::Call::force_collection_config { .. } |
					pallet_nfts::Call::approve_transfer { .. } |
					pallet_nfts::Call::cancel_approval { .. } |
					pallet_nfts::Call::clear_all_transfer_approvals { .. } |
					pallet_nfts::Call::lock_item_properties { .. } |
					pallet_nfts::Call::set_attribute { .. } |
					pallet_nfts::Call::force_set_attribute { .. } |
					pallet_nfts::Call::clear_attribute { .. } |
					pallet_nfts::Call::approve_item_attributes { .. } |
					pallet_nfts::Call::cancel_item_attributes_approval { .. } |
					pallet_nfts::Call::set_metadata { .. } |
					pallet_nfts::Call::clear_metadata { .. } |
					pallet_nfts::Call::set_collection_metadata { .. } |
					pallet_nfts::Call::clear_collection_metadata { .. } |
					pallet_nfts::Call::set_accept_ownership { .. } |
					pallet_nfts::Call::set_collection_max_supply { .. } |
					pallet_nfts::Call::update_mint_settings { .. } |
					pallet_nfts::Call::set_price { .. } |
					pallet_nfts::Call::buy_item { .. } |
					pallet_nfts::Call::pay_tips { .. } |
					pallet_nfts::Call::create_swap { .. } |
					pallet_nfts::Call::cancel_swap { .. } |
					pallet_nfts::Call::claim_swap { .. },
			) | RuntimeCall::Uniques(
				pallet_uniques::Call::create { .. } |
					pallet_uniques::Call::force_create { .. } |
					pallet_uniques::Call::destroy { .. } |
					pallet_uniques::Call::mint { .. } |
					pallet_uniques::Call::burn { .. } |
					pallet_uniques::Call::transfer { .. } |
					pallet_uniques::Call::freeze { .. } |
					pallet_uniques::Call::thaw { .. } |
					pallet_uniques::Call::freeze_collection { .. } |
					pallet_uniques::Call::thaw_collection { .. } |
					pallet_uniques::Call::transfer_ownership { .. } |
					pallet_uniques::Call::set_team { .. } |
					pallet_uniques::Call::approve_transfer { .. } |
					pallet_uniques::Call::cancel_approval { .. } |
					pallet_uniques::Call::force_item_status { .. } |
					pallet_uniques::Call::set_attribute { .. } |
					pallet_uniques::Call::clear_attribute { .. } |
					pallet_uniques::Call::set_metadata { .. } |
					pallet_uniques::Call::clear_metadata { .. } |
					pallet_uniques::Call::set_collection_metadata { .. } |
					pallet_uniques::Call::clear_collection_metadata { .. } |
					pallet_uniques::Call::set_accept_ownership { .. } |
					pallet_uniques::Call::set_collection_max_supply { .. } |
					pallet_uniques::Call::set_price { .. } |
					pallet_uniques::Call::buy_item { .. }
			)
		)
	}
}

pub type Barrier = TrailingSetTopicAsId<
	DenyThenTry<
		DenyReserveTransferToRelayChain,
		(
			TakeWeightCredit,
			// Expected responses are OK.
			AllowKnownQueryResponses<PolkadotXcm>,
			// Allow XCMs with some computed origins to pass through.
			WithComputedOrigin<
				(
					// If the message is one that immediately attemps to pay for execution, then allow it.
					AllowTopLevelPaidExecutionFrom<Everything>,
					// if we have `AliasOrigin` which attempts to withdraw assets, then allow it.
					AllowAliasOriginWithdrawPaidExecutionFrom<Equals<bridging::AssetHubPolkadot>>,
					// Parent and its pluralities (i.e. governance bodies) get free execution.
					AllowExplicitUnpaidExecutionFrom<ParentOrParentsPlurality>,
					// Subscriptions for version tracking are OK.
					AllowSubscriptionsFrom<ParentOrSiblings>,
				),
				UniversalLocation,
				ConstU32<8>,
			>,
		),
	>,
>;

pub type AssetFeeAsExistentialDepositMultiplierFeeCharger = AssetFeeAsExistentialDepositMultiplier<
	Runtime,
	WeightToFee,
	pallet_assets::BalanceToAssetBalance<Balances, Runtime, ConvertInto, TrustBackedAssetsInstance>,
	TrustBackedAssetsInstance,
>;

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	type AssetTransactor = AssetTransactors;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	// Asset Hub acting _as_ a reserve location for KSM and assets created under `pallet-assets`.
	// For KSM, users must use teleport where allowed (e.g. with the Relay Chain).
	type IsReserve = bridging::IsTrustedBridgedReserveLocationForConcreteAsset;
	// We allow:
	// - teleportation of KSM
	// - teleportation of sibling parachain's assets (as ForeignCreators)
	type IsTeleporter = (
		NativeAsset,
		IsForeignConcreteAsset<FromSiblingParachain<parachain_info::Pallet<Runtime>>>,
	);
	type UniversalLocation = UniversalLocation;
	type Barrier = Barrier;
	type Weigher = WeightInfoBounds<
		crate::weights::xcm::AssetHubKusamaXcmWeight<RuntimeCall>,
		RuntimeCall,
		MaxInstructions,
	>;
	type Trader = (
		UsingComponents<WeightToFee, KsmLocation, AccountId, Balances, ToStakingPot<Runtime>>,
		cumulus_primitives_utility::TakeFirstAssetTrader<
			AccountId,
			AssetFeeAsExistentialDepositMultiplierFeeCharger,
			TrustBackedAssetsConvertedConcreteId,
			Assets,
			cumulus_primitives_utility::XcmFeesTo32ByteAccount<
				FungiblesTransactor,
				AccountId,
				XcmAssetFeesReceiver,
			>,
		>,
	);
	type ResponseHandler = PolkadotXcm;
	type AssetTrap = PolkadotXcm;
	type AssetClaims = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type PalletInstancesInfo = AllPalletsWithSystem;
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type AssetLocker = ();
	type AssetExchanger = ();
	type FeeManager = ();
	type MessageExporter = ();
	type UniversalAliases = bridging::UniversalAliases;
	type CallDispatcher = WithOriginFilter<SafeCallFilter>;
	type SafeCallFilter = SafeCallFilter;
	type Aliasers = bridging::AcceptableAliases;
}

/// Converts a local signed origin into an XCM multilocation.
/// Forms the basis for local origins sending/executing XCMs.
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

/// For routing XCM messages which do not cross local consensus boundary.
type LocalXcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm, ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = WithUniqueTopic<(LocalXcmRouter, bridging::BridgingXcmRouter)>;

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub ReachableDest: Option<MultiLocation> = Some(Parent.into());
}

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	// We want to disallow users sending (arbitrary) XCMs from this chain.
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, ()>;
	type XcmRouter = XcmRouter;
	// We support local origins dispatching XCM executions in principle...
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	// ... but disallow generic XCM execution. As a result only teleports and reserve transfers are allowed.
	type XcmExecuteFilter = Nothing;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Everything;
	type XcmReserveTransferFilter = Everything;
	type Weigher = WeightInfoBounds<
		crate::weights::xcm::AssetHubKusamaXcmWeight<RuntimeCall>,
		RuntimeCall,
		MaxInstructions,
	>;

	type UniversalLocation = UniversalLocation;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type TrustedLockers = ();
	type SovereignAccountOf = LocationToAccountId;
	type MaxLockers = ConstU32<8>;
	type WeightInfo = crate::weights::pallet_xcm::WeightInfo<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type ReachableDest = ReachableDest;
	type AdminOrigin = EnsureRoot<AccountId>;
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
	type DestinationFeesManager = KsmForDotDestinationFeesManager<UniversalLocation, WeightToFee>;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

pub type ForeignCreatorsSovereignAccountOf = (
	SiblingParachainConvertsVia<Sibling, AccountId>,
	AccountId32Aliases<RelayNetwork, AccountId>,
	ParentIsPreset<AccountId>,
);

/// Simple conversion of `u32` into an `AssetId` for use in benchmarking.
pub struct XcmBenchmarkHelper;
#[cfg(feature = "runtime-benchmarks")]
impl pallet_assets::BenchmarkHelper<MultiLocation> for XcmBenchmarkHelper {
	fn create_asset_id_parameter(id: u32) -> MultiLocation {
		MultiLocation { parents: 1, interior: X1(Parachain(id)) }
	}
}

/// All configuration related to bridging
pub mod bridging {
	use super::*;
	use assets_common::{matching, matching::*};
	use frame_support::traits::ContainsPair;
	use sp_std::collections::btree_set::BTreeSet;
	use xcm_builder::{NetworkExportTable, UnpaidRemoteExporter};

	parameter_types! {
		pub BridgeHubKusamaParaId: u32 = 1002;
		pub BridgeHubKusama: MultiLocation = MultiLocation::new(1, X1(Parachain(BridgeHubKusamaParaId::get())));
		pub BridgeHubKusamaWithBridgeHubPolkadotInstance: MultiLocation = MultiLocation::new(1, X2(Parachain(BridgeHubKusamaParaId::get()), PalletInstance(53)));
		pub const PolkadotNetwork: NetworkId = NetworkId::Polkadot;
		pub AssetHubPolkadot: MultiLocation =  MultiLocation::new(2, X2(GlobalConsensus(PolkadotNetwork::get()), Parachain(1000)));
		pub DotLocation: MultiLocation =  MultiLocation::new(2, X1(GlobalConsensus(PolkadotNetwork::get())));

		/// Setup exporters configuration
		pub BridgeTable: sp_std::vec::Vec<(NetworkId, MultiLocation, Option<MultiAsset>)> = sp_std::vec![
			(PolkadotNetwork::get(), BridgeHubKusama::get(), None)
		];

		/// Setup trusted bridged reserve locations
		pub BridgedReserves: sp_std::vec::Vec<FilteredReserveLocation> = sp_std::vec![
			// trust assets from AssetHubPolkadot
			(
				AssetHubPolkadot::get(),
				AssetFilter::ByMultiLocation(
					MultiLocationFilter::default()
						// allow receive DOT
						.add_equals(DotLocation::get())
				)
			)
		];

		/// Universal aliases
		pub UniversalAliases: BTreeSet<(MultiLocation, Junction)> = BTreeSet::from_iter(
			sp_std::vec![
				(BridgeHubKusamaWithBridgeHubPolkadotInstance::get(), GlobalConsensus(PolkadotNetwork::get()))
			]
		);

		/// Acceptable aliases
		pub AcceptableAliases: sp_std::vec::Vec<(MultiLocation, MultiLocation)> = sp_std::vec![
			(BridgeHubKusama::get(), AssetHubPolkadot::get())
		];
	}

	impl Contains<(MultiLocation, Junction)> for UniversalAliases {
		fn contains(alias: &(MultiLocation, Junction)) -> bool {
			UniversalAliases::get().contains(alias)
		}
	}

	impl ContainsPair<MultiLocation, MultiLocation> for AcceptableAliases {
		fn contains(origin: &MultiLocation, target: &MultiLocation) -> bool {
			AcceptableAliases::get().iter().any(|(ao, at)| ao == origin && at == target)
		}
	}

	/// Bridge router, which wraps and sends xcm to BridgeHub to be delivered to the different GlobalConsensus
	pub type BridgingXcmRouter =
		UnpaidRemoteExporter<NetworkExportTable<BridgeTable>, LocalXcmRouter, UniversalLocation>;

	/// Reserve locations filter for `xcm_executor::Config::IsReserve`.
	pub type IsTrustedBridgedReserveLocationForConcreteAsset =
		matching::IsTrustedBridgedReserveLocationForConcreteAsset<
			UniversalLocation,
			BridgedReserves,
		>;

	/// Benchmarks helper for over-bridge transfer.
	#[cfg(feature = "runtime-benchmarks")]
	pub struct BridgingBenchmarksHelper;

	#[cfg(feature = "runtime-benchmarks")]
	impl BridgingBenchmarksHelper {
		pub fn prepare_universal_alias() -> Option<(MultiLocation, Junction)> {
			let alias = UniversalAliases::get().into_iter().find_map(|(location, junction)| {
				match BridgeHubKusama::get().eq(&location) {
					true => Some((location, junction)),
					false => None,
				}
			});
			assert!(alias.is_some(), "we expect here BridgeHubKusama to Polkadot mapping at least");
			Some(alias.unwrap())
		}

		pub fn prepare_alias_origin() -> Option<(MultiLocation, MultiLocation)> {
			let alias = AcceptableAliases::get().into_iter().find_map(|(origin, target)| {
				match BridgeHubKusama::get().eq(&origin) {
					true => Some((origin, target)),
					false => None,
				}
			});
			assert!(
				alias.is_some(),
				"we expect here BridgeHubKusama and AssetHubPolkadot mapping at least"
			);
			Some(alias.unwrap())
		}
	}
}

// TODO: sample implementation of DestinationFeesManager
pub struct KsmForDotDestinationFeesManager<LocalUniversal, WeightToFee>(
	sp_std::marker::PhantomData<(LocalUniversal, WeightToFee)>,
);
impl<
		LocalUniversal: Get<InteriorMultiLocation>,
		WeightToFee: frame_support::weights::WeightToFee<Balance = Balance>,
	> DestinationFeesManager for KsmForDotDestinationFeesManager<LocalUniversal, WeightToFee>
{
	fn decide_for(
		destination: &MultiLocation,
		desired_fee_asset_id: &AssetId,
	) -> DestinationFeesSetup {
		// if somebody wants to pay with KSM on AssetHubPolkadot, we handle fees
		if destination.eq(&bridging::AssetHubPolkadot::get()) {
			if matches!(desired_fee_asset_id, Concrete(asset_location) if asset_location.eq(&KsmLocation::get()))
			{
				return DestinationFeesSetup::ByUniversalLocation {
					// additional fee in KSM goes to this local account (we can set it as treasury maybe?)
					local_account: bridging::BridgeHubKusama::get(),
				}
			}
		}
		// else, just do nothing, e.g. maybe user choose to pay with some sufficient fee on AssetHubPolkadot and so on
		DestinationFeesSetup::ByOrigin
	}

	fn estimate_fee_for(
		destination: &MultiLocation,
		desired_fee_asset_id: &AssetId,
		weight: &WeightLimit,
	) -> Option<DestinationFees> {
		// if somebody wants to pay with KSM on AssetHubPolkadot
		if destination.eq(&bridging::AssetHubPolkadot::get()) {
			if matches!(desired_fee_asset_id, Concrete(asset_location) if asset_location.eq(&KsmLocation::get()))
			{
				//
				let proportional_amount_to_withdraw = match weight {
					Unlimited => {
						// TODO: handle with some limit?
						return None
					},
					Limited(weight) => {
						// TODO: add here some additional transport fee?
						let weight = *weight + (*weight * Perbill::from_percent(25));
						WeightToFee::weight_to_fee(&weight)
					},
				};

				// TODO: later with DEX
				// conversion ration: 1KSM to 100 DOTs
				let proportional_amount_to_buy_execution =
					proportional_amount_to_withdraw.clone() * 100;

				return Some(DestinationFees {
					proportional_amount_to_withdraw: (
						KsmLocation::get(),
						proportional_amount_to_withdraw,
					)
						.into(),
					proportional_amount_to_buy_execution: (
						bridging::DotLocation::get(),
						proportional_amount_to_buy_execution,
					)
						.into(),
				})
			}
		}
		None
	}
}
