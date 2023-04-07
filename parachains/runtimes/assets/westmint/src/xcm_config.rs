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
	AccountId, AllPalletsWithSystem, Assets, Authorship, Balance, Balances, ParachainInfo,
	ParachainSystem, PolkadotXcm, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin,
	TrustBackedAssetsInstance, WeightToFee, XcmpQueue,
};
use crate::ForeignAssets;
use assets_common::matching::{
	FromSiblingParachain, IsForeignConcreteAsset, StartsWith, StartsWithExplicitGlobalConsensus,
};
use frame_support::{
	match_types, parameter_types,
	traits::{ConstU32, Contains, Everything, Nothing, PalletInfoAccess},
};
use frame_system::EnsureRoot;
use pallet_xcm::XcmPassthrough;
use parachains_common::{
	impls::ToStakingPot,
	xcm_config::{
		AssetFeeAsExistentialDepositMultiplier, DenyReserveTransferToRelayChain, DenyThenTry,
	},
	TREASURY_PALLET_ID,
};
use polkadot_parachain::primitives::Sibling;
use sp_runtime::traits::{AccountIdConversion, ConvertInto};
use xcm::latest::prelude::*;
use xcm_builder::{
	AccountId32Aliases, AllowExplicitUnpaidExecutionFrom, AllowKnownQueryResponses,
	AllowSubscriptionsFrom, AllowTopLevelPaidExecutionFrom, CurrencyAdapter, EnsureXcmOrigin,
	FungiblesAdapter, IsConcrete, LocalMint, NativeAsset, NoChecking, ParentAsSuperuser,
	ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
	SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit,
	UsingComponents, WeightInfoBounds, WithComputedOrigin, XcmFeesToAccount,
};
use xcm_executor::{traits::WithOriginFilter, XcmExecutor};

parameter_types! {
	pub const WestendLocation: MultiLocation = MultiLocation::parent();
	pub RelayNetwork: Option<NetworkId> = Some(NetworkId::Westend);
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub UniversalLocation: InteriorMultiLocation =
		X2(GlobalConsensus(RelayNetwork::get().unwrap()), Parachain(ParachainInfo::parachain_id().into()));
	pub UniversalLocationNetworkId: NetworkId = UniversalLocation::get().global_consensus().unwrap();
	pub TrustBackedAssetsPalletLocation: MultiLocation =
		PalletInstance(<Assets as PalletInfoAccess>::index() as u8).into();
	pub CheckingAccount: AccountId = PolkadotXcm::check_account();
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
);

/// Means for transacting the native currency on this chain.
pub type CurrencyTransactor = CurrencyAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<WestendLocation>,
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
		// Ignore asset which starts explicitly with our `GlobalConsensus(NetworkId)`, means:
		// - foreign assets from our consensus should be: `MultiLocation {parent: 1, X*(Parachain(xyz))}
		// - foreign assets outside our consensus with the same `GlobalConsensus(NetworkId)` wont be accepted here
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

	pub TreasuryAccount: Option<AccountId> = Some(TREASURY_PALLET_ID.into_account_truncating());
}

match_types! {
	pub type ParentOrParentsPlurality: impl Contains<MultiLocation> = {
		MultiLocation { parents: 1, interior: Here } |
		MultiLocation { parents: 1, interior: X1(Plurality { .. }) }
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

		match call {
			RuntimeCall::PolkadotXcm(pallet_xcm::Call::force_xcm_version { .. }) |
			RuntimeCall::System(
				frame_system::Call::set_heap_pages { .. } |
				frame_system::Call::set_code { .. } |
				frame_system::Call::set_code_without_checks { .. } |
				frame_system::Call::kill_prefix { .. },
			) |
			RuntimeCall::ParachainSystem(..) |
			RuntimeCall::Timestamp(..) |
			RuntimeCall::Balances(..) |
			RuntimeCall::CollatorSelection(
				pallet_collator_selection::Call::set_desired_candidates { .. } |
				pallet_collator_selection::Call::set_candidacy_bond { .. } |
				pallet_collator_selection::Call::register_as_candidate { .. } |
				pallet_collator_selection::Call::leave_intent { .. },
			) |
			RuntimeCall::Session(pallet_session::Call::purge_keys { .. }) |
			RuntimeCall::XcmpQueue(..) |
			RuntimeCall::DmpQueue(..) |
			RuntimeCall::Utility(
				pallet_utility::Call::as_derivative { .. } |
				pallet_utility::Call::batch { .. } |
				pallet_utility::Call::batch_all { .. },
			) |
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
			) |
			RuntimeCall::ForeignAssets(
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
			) |
			RuntimeCall::Nfts(
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
			) |
			RuntimeCall::Uniques(
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
				pallet_uniques::Call::buy_item { .. },
			) => true,
			_ => false,
		}
	}
}

pub type Barrier = DenyThenTry<
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
				// Parent or its plurality (i.e. governance bodies) gets free execution.
				AllowExplicitUnpaidExecutionFrom<ParentOrParentsPlurality>,
				// Subscriptions for version tracking are OK.
				AllowSubscriptionsFrom<Everything>,
			),
			UniversalLocation,
			ConstU32<8>,
		>,
	),
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
	// Westmint does not recognize a reserve location for any asset. This does not prevent
	// Westmint acting _as_ a reserve location for WND and assets created under `pallet-assets`.
	// For WND, users must use teleport where allowed (e.g. with the Relay Chain).
	type IsReserve = ();
	// We allow:
	// - teleportation of WND
	// - teleportation of sibling parachain's assets (as ForeignCreators)
	type IsTeleporter = (
		NativeAsset,
		IsForeignConcreteAsset<FromSiblingParachain<parachain_info::Pallet<Runtime>>>,
	);
	type UniversalLocation = UniversalLocation;
	type Barrier = Barrier;
	type Weigher = WeightInfoBounds<
		crate::weights::xcm::WestmintXcmWeight<RuntimeCall>,
		RuntimeCall,
		MaxInstructions,
	>;
	type Trader = (
		UsingComponents<WeightToFee, WestendLocation, AccountId, Balances, ToStakingPot<Runtime>>,
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
	type FeeManager =
		XcmFeesToAccount<Self, westend_common::RelayOrOtherSystemParachains<Runtime>, AccountId, TreasuryAccount>;
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type CallDispatcher = WithOriginFilter<SafeCallFilter>;
	type SafeCallFilter = SafeCallFilter;
}

/// Local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm, ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub ReachableDest: Option<MultiLocation> = Some(Parent.into());
}

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmExecuteFilter = Everything;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Everything;
	type XcmReserveTransferFilter = Everything;
	type Weigher = WeightInfoBounds<
		crate::weights::xcm::WestmintXcmWeight<RuntimeCall>,
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
use pallet_assets::BenchmarkHelper;
#[cfg(feature = "runtime-benchmarks")]
impl BenchmarkHelper<MultiLocation> for XcmBenchmarkHelper {
	fn create_asset_id_parameter(id: u32) -> MultiLocation {
		MultiLocation { parents: 1, interior: X1(Parachain(id)) }
	}
}
