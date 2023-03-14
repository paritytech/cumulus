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
	AccountId, AllPalletsWithSystem, AssetIdForTrustBackedAssets, Assets, Authorship, Balance,
	Balances, ForeignAssets, ParachainInfo, ParachainSystem, PolkadotXcm, Runtime, RuntimeCall,
	RuntimeEvent, RuntimeOrigin, TrustBackedAssetsInstance, WeightToFee, XcmpQueue,
};
use frame_support::{
	match_types, parameter_types,
	traits::{
		ConstU32, Contains, ContainsPair, EnsureOrigin, EnsureOriginWithArg, Everything,
		OriginTrait, PalletInfoAccess,
	},
};
use pallet_xcm::{EnsureXcm, XcmPassthrough};
use parachains_common::{
	impls::ToStakingPot,
	xcm_config::{
		AssetFeeAsExistentialDepositMultiplier, DenyReserveTransferToRelayChain, DenyThenTry,
	},
};
use polkadot_parachain::primitives::{Id as ParaId, Sibling};
use sp_core::Get;
use sp_runtime::traits::ConvertInto;
use xcm::latest::prelude::*;
use xcm_builder::{
	AccountId32Aliases, AllowExplicitUnpaidExecutionFrom, AllowKnownQueryResponses,
	AllowSubscriptionsFrom, AllowTopLevelPaidExecutionFrom, AsPrefixedGeneralIndex,
	ConvertedConcreteId, CurrencyAdapter, EnsureXcmOrigin, FungiblesAdapter, IsConcrete, LocalMint,
	NativeAsset, NoChecking, ParentAsSuperuser, ParentIsPreset, RelayChainAsNative,
	SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
	SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit, UsingComponents,
	WeightInfoBounds, WithComputedOrigin,
};
use xcm_executor::{
	traits::{Convert, ConvertOrigin, Identity, JustTry, ShouldExecute, WithOriginFilter},
	XcmExecutor,
};

parameter_types! {
	pub const WestendLocation: MultiLocation = MultiLocation::parent();
	pub RelayNetwork: Option<NetworkId> = Some(NetworkId::Westend);
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub UniversalLocation: InteriorMultiLocation =
		X2(GlobalConsensus(RelayNetwork::get().unwrap()), Parachain(ParachainInfo::parachain_id().into()));
	pub const Local: MultiLocation = Here.into_location();
	// todo: accept all instances, perhaps need a type for each instance?
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

/// Means for transacting assets besides the native currency on this chain.
pub type FungiblesTransactor = FungiblesAdapter<
	// Use this fungibles implementation:
	Assets,
	// Use this currency when it is a fungible asset matching the given location or name:
	ConvertedConcreteId<
		AssetIdForTrustBackedAssets,
		Balance,
		AsPrefixedGeneralIndex<
			TrustBackedAssetsPalletLocation,
			AssetIdForTrustBackedAssets,
			JustTry,
		>,
		JustTry,
	>,
	// Convert an XCM MultiLocation into a local account id:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We only want to allow teleports of known assets. We use non-zero issuance as an indication
	// that this asset is known.
	LocalMint<parachains_common::impls::NonZeroIssuance<AccountId, Assets>>, // todo: accept all instances
	// The account to use for tracking teleports.
	CheckingAccount,
>;

/// Means for transacting foreign assets from different global consensus.
pub type ForeignFungiblesTransactor = FungiblesAdapter<
	// Use this fungibles implementation:
	ForeignAssets,
	// Use this currency when it is a fungible asset matching the given location or name:
	ConvertedConcreteId<MultiLocationForAssetId, Balance, Identity, JustTry>,
	// Convert an XCM MultiLocation into a local account id:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// TODO:check-parameter - no teleports
	NoChecking,
	// The account to use for tracking teleports.
	CheckingAccount,
>;

/// Means for transacting assets on this chain.
// TODO:check-paramter - FungiblesTransactor cannot be in the middle, because stops tuple execution, check/fix?
// TODO:check-paramter - possible bug for matches_fungibles and return error and tuple processing?
pub type AssetTransactors = (CurrencyTransactor, ForeignFungiblesTransactor, FungiblesTransactor);

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
	// TODO:check-parameter - find a better solution
	// Bridged account origins from different globalConsensus converts as SovereignAccount
	BridgedSignedAccountId32AsNative<LocationToAccountId, RuntimeOrigin, TrustedBridgedNetworks>,
	// TODO: add here alternative for BridgedRelayChainAs... or BridgedParachainAs...
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
			RuntimeCall::System(
				frame_system::Call::set_heap_pages { .. } |
				frame_system::Call::set_code { .. } |
				frame_system::Call::set_code_without_checks { .. } |
				// TODO:check-parameter - verify, if we need for production (remark_with_event)
				frame_system::Call::remark_with_event { .. } |
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
				// Specific barrier for bridged calls from different globalConsensus/network
				BridgedCallsBarrier,
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
	// Westmint is acting _as_ a reserve location for WND and assets created under `pallet-assets`.
	// For WND, users must use teleport where allowed (e.g. with the Relay Chain).
	type IsReserve =
		ConcreteFungibleAssetsFromTrustedBridgedReserves<TrustedBridgedReserveLocations>;
	type IsTeleporter = NativeAsset; // <- should be enough to allow teleportation of WND
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
			ConvertedConcreteId<
				AssetIdForTrustBackedAssets,
				Balance,
				AsPrefixedGeneralIndex<
					TrustBackedAssetsPalletLocation,
					AssetIdForTrustBackedAssets,
					JustTry,
				>,
				JustTry,
			>,
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
	type UniversalAliases = TrustedBridgedNetworks;
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
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

pub type MultiLocationForAssetId = MultiLocation;

pub type SovereignAccountOf = (
	SiblingParachainConvertsVia<ParaId, AccountId>,
	AccountId32Aliases<RelayNetwork, AccountId>,
	ParentIsPreset<AccountId>,
);

// `EnsureOriginWithArg` impl for `CreateOrigin` that allows only XCM origins that are locations
// containing the class location.
pub struct ForeignCreators;
impl EnsureOriginWithArg<RuntimeOrigin, MultiLocation> for ForeignCreators {
	type Success = AccountId;

	fn try_origin(
		o: RuntimeOrigin,
		a: &MultiLocation,
	) -> sp_std::result::Result<Self::Success, RuntimeOrigin> {
		let origin_location = EnsureXcm::<Everything>::try_origin(o.clone())?;
		if !a.starts_with(&origin_location) {
			return Err(o)
		}
		SovereignAccountOf::convert(origin_location).map_err(|_| o)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin(a: &MultiLocation) -> RuntimeOrigin {
		pallet_xcm::Origin::Xcm(a.clone()).into()
	}
}

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

parameter_types! {
	// TODO:check-parameter - join all together in one on-chain cfg (statemine/t, eth(chain_ids), ...)

	// TODO:check-parameter - add new pallet and persist/manage this via governance?
	// Means, that we accept some `GlobalConsensus` from some `MultiLocation` (which is supposed to be our bridge-hub)
	pub TrustedBridgedNetworks: sp_std::vec::Vec<(MultiLocation, Junction)> = sp_std::vec![
		(MultiLocation { parents: 1, interior: X1(Parachain(1014)) }, GlobalConsensus(NetworkId::Rococo))
	];
	// TODO:check-parameter - add new pallet and persist/manage this via governance?
	// TODO:check-parameter - we specify here just trusted location, we can extend this with some AssetFilter patterns to trust only to several assets
	pub TrustedBridgedReserveLocations: sp_std::vec::Vec<MultiLocation> = sp_std::vec![
		// TODO:check-parameter - tmp values that cover local/live Rococo/Wococo run
		MultiLocation { parents: 2, interior: X2(GlobalConsensus(Rococo), Parachain(1000)) },
		MultiLocation { parents: 2, interior: X2(GlobalConsensus(Kusama), Parachain(1000)) },
		MultiLocation { parents: 2, interior: X2(GlobalConsensus(Rococo), Parachain(1015)) },
		MultiLocation { parents: 2, interior: X2(GlobalConsensus(Kusama), Parachain(1015)) },
	];
}

impl Contains<(MultiLocation, Junction)> for TrustedBridgedNetworks {
	fn contains(t: &(MultiLocation, Junction)) -> bool {
		Self::get().contains(t)
	}
}

impl Contains<MultiLocation> for TrustedBridgedReserveLocations {
	fn contains(t: &MultiLocation) -> bool {
		Self::get().contains(t)
	}
}

pub type BridgedCallsBarrier = (
	// TODO:check-parameter - verify, if we need for production (usefull at least for testing connection in production)
	AllowExecutionForTrapFrom<Everything>,
	// TODO:check-parameter - verify, if we need for production
	AllowExecutionForTransactFrom<Everything>,
	// TODO:check-parameter - setup fess
	// TODO:check-parameter - change Everything to some Contains with trusted BridgeHub configuration
	// Configured trusted BridgeHub gets free execution.
	AllowExplicitUnpaidExecutionFrom<Everything>,
	// Expected responses are OK.
	AllowKnownQueryResponses<PolkadotXcm>,
	// Subscriptions for version tracking are OK.
	AllowSubscriptionsFrom<Everything>,
);

/// Asset filter that allows all assets from trusted bridge location
pub struct ConcreteFungibleAssetsFromTrustedBridgedReserves<TrustedReserverLocations>(
	sp_std::marker::PhantomData<TrustedReserverLocations>,
);
impl<TrustedReserverLocations: Contains<MultiLocation>> ContainsPair<MultiAsset, MultiLocation>
	for ConcreteFungibleAssetsFromTrustedBridgedReserves<TrustedReserverLocations>
{
	fn contains(asset: &MultiAsset, origin: &MultiLocation) -> bool {
		log::trace!(
			target: "xcm::barriers",
			"ConcreteFungibleAssetsFromTrustedBridgedReserves origin: {:?}, asset: {:?}",
			origin, asset,
		);
		if !TrustedReserverLocations::contains(origin) {
			return false
		}
		// TODO:check-parameter - better assets filtering
		matches!(asset, MultiAsset { id: AssetId::Concrete(_), fun: Fungible(_) })
	}
}

pub struct AllowExecutionForTrapFrom<T>(sp_std::marker::PhantomData<T>);
impl<T: Contains<MultiLocation>> ShouldExecute for AllowExecutionForTrapFrom<T> {
	fn should_execute<RuntimeCall>(
		origin: &MultiLocation,
		instructions: &mut [Instruction<RuntimeCall>],
		max_weight: xcm::latest::Weight,
		_weight_credit: &mut xcm::latest::Weight,
	) -> Result<(), ()> {
		log::warn!(
			target: "xcm::barriers",
			"(TODO:remove-in-production) AllowExecutionForTrapFrom origin: {:?}, instructions: {:?}, max_weight: {:?}, weight_credit: {:?}",
			origin, instructions, max_weight, _weight_credit,
		);

		match instructions.first() {
			Some(Trap { .. }) => Ok(()),
			_ => Err(()),
		}
	}
}

pub struct AllowExecutionForTransactFrom<T>(sp_std::marker::PhantomData<T>);
impl<T: Contains<MultiLocation>> ShouldExecute for AllowExecutionForTransactFrom<T> {
	fn should_execute<RuntimeCall>(
		origin: &MultiLocation,
		instructions: &mut [Instruction<RuntimeCall>],
		max_weight: xcm::latest::Weight,
		_weight_credit: &mut xcm::latest::Weight,
	) -> Result<(), ()> {
		log::error!(
			target: "xcm::barriers",
			"(TODO:change/remove-in-production) AllowExecutionForTransactFrom origin: {:?}, instructions: {:?}, max_weight: {:?}, weight_credit: {:?}",
			origin, instructions, max_weight, _weight_credit,
		);

		match instructions.first() {
			// TODO:check-parameter - filter just remark/remark_with_event
			Some(Transact { .. }) => Ok(()),
			_ => Err(()),
		}
	}
}

pub struct BridgedSignedAccountId32AsNative<LocationConverter, RuntimeOrigin, BridgedNetworks>(
	sp_std::marker::PhantomData<(LocationConverter, RuntimeOrigin, BridgedNetworks)>,
);
impl<
		LocationConverter: Convert<MultiLocation, RuntimeOrigin::AccountId>,
		RuntimeOrigin: OriginTrait,
		BridgedNetworks: Get<sp_std::vec::Vec<(MultiLocation, Junction)>>,
	> ConvertOrigin<RuntimeOrigin>
	for BridgedSignedAccountId32AsNative<LocationConverter, RuntimeOrigin, BridgedNetworks>
where
	RuntimeOrigin::AccountId: Clone,
{
	fn convert_origin(
		origin: impl Into<MultiLocation>,
		kind: OriginKind,
	) -> Result<RuntimeOrigin, MultiLocation> {
		let origin = origin.into();
		log::trace!(
			target: "xcm::origin_conversion",
			"BridgedSignedAccountId32AsNative origin: {:?}, kind: {:?}",
			origin, kind,
		);
		if let OriginKind::SovereignAccount = kind {
			match origin {
				// this represents remote relaychain
				MultiLocation {
					parents: 2,
					interior:
						X2(
							GlobalConsensus(remote_network),
							AccountId32 { network: _network, id: _id },
						),
				} |
				// this represents remote parachain
				MultiLocation {
					parents: 2,
					interior:
					X3(
						GlobalConsensus(remote_network),
						Parachain(_),
						AccountId32 { network: _network, id: _id },
					),
				} => {
					// TODO:check-parameter - hack - configured local bridge-hub behaves on behalf of any origin from configured bridged network (just to pass Transact/System::remark_with_event - ensure_signed)
					// find configured local bridge_hub for remote network
					let bridge_hub_location = BridgedNetworks::get()
						.iter()
						.find(|(_, configured_bridged_network)| match configured_bridged_network {
							GlobalConsensus(bridged_network) => bridged_network.eq(&remote_network),
							_ => false,
						})
						.map(|(bridge_hub_location, _)| bridge_hub_location.clone());

					// try to convert local bridge-hub location
					match bridge_hub_location {
						Some(bridge_hub_location) => {
							let new_origin = bridge_hub_location;
							log::error!(
								target: "xcm::origin_conversion",
								"BridgedSignedAccountId32AsNative replacing origin: {:?} to new_origin: {:?}, kind: {:?}",
								origin, new_origin, kind,
							);
							let location = LocationConverter::convert(new_origin)?;
							Ok(RuntimeOrigin::signed(location).into())
						},
						_ => Err(origin),
					}
				},
				_ => Err(origin),
			}
		} else {
			Err(origin)
		}
	}
}
