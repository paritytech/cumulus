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
	AccountId, AllPalletsWithSystem, AssetId, Authorship, Balance, Balances, ParachainInfo,
	ParachainSystem, PolkadotXcm, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin,
	TrustBackedAssets, TrustBackedAssetsInstance, WeightToFee, XcmpQueue,
};
use frame_support::{
	match_types, parameter_types,
	traits::{
		ConstU32, Contains, EnsureOrigin, EnsureOriginWithArg, Everything, OriginTrait,
		PalletInfoAccess,
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
use sp_runtime::traits::ConvertInto;
use sp_std::marker::PhantomData;
use xcm::latest::{prelude::*, Weight};
use xcm_builder::{
	AccountId32Aliases, AllowExplicitUnpaidExecutionFrom, AllowKnownQueryResponses,
	AllowSubscriptionsFrom, AllowTopLevelPaidExecutionFrom, AsPrefixedGeneralIndex,
	ConvertedConcreteId, CurrencyAdapter, EnsureXcmOrigin, FungiblesAdapter, IsConcrete,
	NativeAsset, ParentAsSuperuser, ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative,
	SiblingParachainConvertsVia, SignedAccountId32AsNative, SignedToAccountId32,
	SovereignSignedViaLocation, TakeWeightCredit, UsingComponents, WeightInfoBounds,
	WithComputedOrigin,
};
use xcm_executor::{
	traits::{Convert, ConvertOrigin, JustTry, ShouldExecute},
	XcmExecutor,
};

parameter_types! {
	pub const WestendLocation: MultiLocation = MultiLocation::parent();
	pub RelayNetwork: NetworkId = NetworkId::Westend;
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub UniversalLocation: InteriorMultiLocation = X2(GlobalConsensus(RelayNetwork::get()), Parachain(ParachainInfo::parachain_id().into()));
	pub const Local: MultiLocation = Here.into_location();
	// todo: accept all instances, perhaps need a type for each instance?
	pub TrustBackedAssetsPalletLocation: MultiLocation =
		PalletInstance(<TrustBackedAssets as PalletInfoAccess>::index() as u8).into();
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
	TrustBackedAssets, // todo: accept all instances
	// Use this currency when it is a fungible asset matching the given location or name:
	ConvertedConcreteId<
		AssetId,
		Balance,
		AsPrefixedGeneralIndex<TrustBackedAssetsPalletLocation, AssetId, JustTry>, // todo: accept all instances
		JustTry,
	>,
	// Convert an XCM MultiLocation into a local account id:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We only want to allow teleports of known assets. We use non-zero issuance as an indication
	// that this asset is known.
	parachains_common::impls::NonZeroIssuance<AccountId, TrustBackedAssets>, // todo: accept all instances
	// The account to use for tracking teleports.
	CheckingAccount,
>;
/// Means for transacting assets on this chain.
pub type AssetTransactors = (CurrencyTransactor, FungiblesTransactor);

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
	// Bridged account origins from different GlobalConsensus converts as SovereignAccount
	BridgedSignedProxyAccountAsNative<
		BridgedProxyAccountId<TrustedBridgedNetworks, AccountId>,
		RuntimeOrigin,
	>,
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
			AssetFeeAsExistentialDepositMultiplier<
				Runtime,
				WeightToFee,
				pallet_assets::BalanceToAssetBalance<
					Balances,
					Runtime,
					ConvertInto,
					TrustBackedAssetsInstance,
				>,
				TrustBackedAssetsInstance,
			>,
			ConvertedConcreteId<
				AssetId,
				Balance,
				AsPrefixedGeneralIndex<TrustBackedAssetsPalletLocation, AssetId, JustTry>, // todo: accept all instances
				JustTry,
			>,
			TrustBackedAssets, // todo: accept all instances
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
	type CallDispatcher = RuntimeCall;
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

// `EnsureOriginWithArg` impl for `CreateOrigin` which allows only XCM origins that are locations
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
			return Err(o);
		}
		SovereignAccountOf::convert(origin_location).map_err(|_| o)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin(a: &MultiLocation) -> RuntimeOrigin {
		pallet_xcm::Origin::Xcm(a.clone()).into()
	}
}

parameter_types! {
	pub TrustedBridgedNetworks: sp_std::vec::Vec<(MultiLocation, Junction)> = sp_std::vec![
		(MultiLocation { parents: 1, interior: X1(Parachain(1014)) }, GlobalConsensus(NetworkId::Rococo))
		// TODO add Ethereum
		// (MultiLocation { parents: 1, interior: X1(Parachain(1014)) }, GlobalConsensus(NetworkId::Ethereum))

	];
}

impl Contains<(MultiLocation, Junction)> for TrustedBridgedNetworks {
	fn contains(t: &(MultiLocation, Junction)) -> bool {
		Self::get().contains(t)
	}
}

impl Contains<MultiLocation> for TrustedBridgedNetworks {
	fn contains(origin: &MultiLocation) -> bool {
		let consensus = match origin {
			MultiLocation {
				parents: 2,
				interior: X2(GlobalConsensus(consensus), AccountId32 { .. }),
			}
			| MultiLocation {
				parents: 2,
				interior: X3(GlobalConsensus(consensus), Parachain(_), AccountId32 { .. }),
			} => consensus,
			_ => {
				log::trace!(target: "xcm::contains_multi_location", "TrustedBridgedNetworks invalid MultiLocation: {:?}", origin);
				return false;
			},
		};

		match Self::get().iter().any(|(_, configured_bridged_network)| {
			match configured_bridged_network {
				GlobalConsensus(bridged_network) => bridged_network.eq(&consensus),
				_ => false,
			}
		}) {
			false => {
				log::trace!(target: "xcm::contains_multi_location", "TrustedBridgedNetworks  GlobalConsensus: {:?} is not Trusted", consensus);
				false
			},
			true => {
				log::trace!(target: "xcm::contains_multi_location", "TrustedBridgedNetworks  GlobalConsensus: {:?} is Trusted", consensus);
				true
			},
		}
	}
}

pub type BridgedCallsBarrier = (
	AllowExecutionForBridgedOperationsFrom<TrustedBridgedNetworks>,
	// Expected responses are OK.
	AllowKnownQueryResponses<PolkadotXcm>,
	// Subscriptions for version tracking are OK.
	AllowSubscriptionsFrom<Everything>,
);

/// Allow execution of Trap & Transact messages from specified bridged networks
/// At first checks `origin` comes from specified networks
/// Then verifies if each instruction is Trap or Transact
pub struct AllowExecutionForBridgedOperationsFrom<BridgedNetworks>(PhantomData<BridgedNetworks>);
impl<BridgedNetworks: Contains<MultiLocation>> ShouldExecute
	for AllowExecutionForBridgedOperationsFrom<BridgedNetworks>
{
	fn should_execute<RuntimeCall>(
		origin: &MultiLocation,
		instructions: &mut [Instruction<RuntimeCall>],
		_max_weight: Weight,
		_weight_credit: &mut Weight,
	) -> Result<(), ()> {
		log::trace!(
			target: "xcm::barriers",
			"AllowExecutionForBridgedOperationsFrom origin: {:?}, instructions: {:?}, max_weight: {:?}, weight_credit: {:?}",
			origin, instructions, _max_weight, _weight_credit,
		);

		if !BridgedNetworks::contains(origin) {
			log::trace!(target: "xcm::barriers", "AllowExecutionForBridgedOperationsFrom barrier failed on invalid Origin: {:?}", origin);
			return Err(());
		}

		match instructions.iter().all(|instruction| match instruction {
			Trap { .. } | Transact { .. } => true,
			_ => false,
		}) {
			true => {
				log::trace!(target: "xcm::barriers", "AllowExecutionForBridgedOperationsFrom barrier passed for origin: {:?}, instructions: {:?}", origin, instructions);
				Ok(())
			},
			false => {
				log::trace!(target: "xcm::barriers", "AllowExecutionForBridgedOperationsFrom barrier failed for origin: {:?}, instructions: {:?}", origin, instructions);
				Err(())
			},
		}
	}
}

pub struct BridgedSignedProxyAccountAsNative<LocationConverter, RuntimeOrigin>(
	PhantomData<(LocationConverter, RuntimeOrigin)>,
);
impl<
		LocationConverter: Convert<MultiLocation, RuntimeOrigin::AccountId>,
		RuntimeOrigin: OriginTrait,
	> ConvertOrigin<RuntimeOrigin>
	for BridgedSignedProxyAccountAsNative<LocationConverter, RuntimeOrigin>
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
			"BridgedSignedProxyAccountAsNative origin: {:?}, kind: {:?}",
			origin, kind,
		);

		if let OriginKind::SovereignAccount = kind {
			let location = LocationConverter::convert(origin)?;
			Ok(RuntimeOrigin::signed(location).into())
		} else {
			log::trace!(
				target: "xcm::origin_conversion",
				"BridgedSignedProxyAccountAsNative origin: {:?} is not a SovereignAccount, kind: {:?}",
				origin, kind
			);

			Err(origin)
		}
	}
}

/// Extracts the `AccountId32` from the bridged `MultiLocation` if the network matches.
pub struct BridgedProxyAccountId<BridgedNetworks, AccountId>(
	PhantomData<(BridgedNetworks, AccountId)>,
);
impl<
		BridgedNetworks: Contains<MultiLocation>,
		AccountId: From<[u8; 32]> + Into<[u8; 32]> + Clone,
	> Convert<MultiLocation, AccountId> for BridgedProxyAccountId<BridgedNetworks, AccountId>
{
	fn convert(location: MultiLocation) -> Result<AccountId, MultiLocation> {
		log::trace!(target: "xcm::location_conversion", "BridgedProxyAccountId source: {:?}", location);

		if !BridgedNetworks::contains(&location) {
			log::trace!(target: "xcm::location_conversion", "BridgedProxyAccountId MultiLocation: {:?} is not Trusted", location);
			return Err(location);
		}

		match location {
			MultiLocation {
				parents: 2,
				interior: X2(GlobalConsensus(_), AccountId32 { id, network: _ }),
			}
			| MultiLocation {
				parents: 2,
				interior: X3(GlobalConsensus(_), Parachain(_), AccountId32 { id, network: _ }),
			} => Ok(id.into()),
			_ => {
				log::trace!(target: "xcm::location_conversion", "BridgedProxyAccountId cannot extract AccountId from MultiLocation: {:?}", location);
				Err(location)
			},
		}
	}

	fn reverse(who: AccountId) -> Result<MultiLocation, AccountId> {
		Ok(AccountId32 { id: who.into(), network: None }.into())
	}
}
