// Copyright (C) 2023 Parity Technologies (UK) Ltd.
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

use crate as bridge_transfer;
use frame_support::traits::{AsEnsureOriginWithArg, ConstU32, Contains, ContainsPair, Currency};

use crate::{
	features::{
		AllowedUniversalAliasesOf, ConfiguredConcreteAssetTransferKindResolver,
		IsTrustedBridgedReserveForConcreteAsset,
	},
	filter::{AssetFilter, MultiLocationFilter},
	pallet::{AllowedReserveLocations, AllowedUniversalAliases},
	AllowedExporters, AssetTransferKind, BridgeConfig, Config, Error, Event,
	LatestVersionedMultiLocation, MaybePaidLocation, ReachableDestination,
	ResolveAssetTransferKind,
};
use frame_support::{
	assert_noop, assert_ok, dispatch::DispatchError, parameter_types, sp_io, sp_tracing, BoundedVec,
};
use frame_system::EnsureRoot;
use polkadot_parachain::primitives::Sibling;
use sp_runtime::{
	testing::{Header, H256},
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32, ModuleError,
};
use sp_version::RuntimeVersion;
use xcm::prelude::*;
use xcm_builder::{
	AccountId32Aliases, CurrencyAdapter, EnsureXcmOrigin, ExporterFor,
	GlobalConsensusParachainConvertsFor, IsConcrete, SiblingParachainConvertsVia,
	SignedToAccountId32, UnpaidRemoteExporter,
};
use xcm_executor::traits::ConvertLocation;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

frame_support::construct_runtime!(
	pub enum TestRuntime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		BridgeTransfer: bridge_transfer::{Pallet, Call, Event<T>} = 52,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub Version: RuntimeVersion = RuntimeVersion {
		spec_name: sp_version::create_runtime_str!("test"),
		impl_name: sp_version::create_runtime_str!("system-test"),
		authoring_version: 1,
		spec_version: 1,
		impl_version: 1,
		apis: sp_version::create_apis_vec!([]),
		transaction_version: 1,
		state_version: 1,
	};
}

pub type AccountId = AccountId32;

impl frame_system::Config for TestRuntime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type BlockLength = ();
	type BlockWeights = ();
	type Version = Version;
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = frame_support::traits::Everything;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 5;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for TestRuntime {
	type Balance = u64;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = RuntimeHoldReason;
	type FreezeIdentifier = ();
	type MaxHolds = ConstU32<0>;
	type MaxFreezes = ConstU32<0>;
}

parameter_types! {
	pub const BridgedNetwork: NetworkId = NetworkId::ByGenesis([4; 32]);
	pub const RelayNetwork: NetworkId = NetworkId::ByGenesis([9; 32]);
	pub UniversalLocation: InteriorMultiLocation = X2(GlobalConsensus(RelayNetwork::get()), Parachain(1000));
	// Relay chain currency/balance location (e.g. KsmLocation, DotLocation, ..)
	pub const RelayLocation: MultiLocation = MultiLocation::parent();
}

std::thread_local! {
	static ROUTED_MESSAGE: std::cell::RefCell<Option<Xcm<()>>> = std::cell::RefCell::new(None);
}

pub struct ThreadLocalXcmRouter;
impl SendXcm for ThreadLocalXcmRouter {
	type Ticket = Option<Xcm<()>>;

	fn validate(
		destination: &mut Option<MultiLocation>,
		message: &mut Option<Xcm<()>>,
	) -> SendResult<Self::Ticket> {
		log::info!(
			target: super::LOG_TARGET,
			"[ThreadLocalXcmRouter]: destination: {:?}, message: {:?}",
			destination,
			message
		);
		Ok((message.take(), MultiAssets::default()))
	}

	fn deliver(ticket: Self::Ticket) -> Result<XcmHash, SendError> {
		match ticket {
			Some(msg) => {
				ROUTED_MESSAGE.with(|rm| *rm.borrow_mut() = Some(msg));
				Ok([0u8; 32])
			},
			None => Err(SendError::MissingArgument),
		}
	}
}

pub struct NotApplicableOrFailOnParachain2222XcmRouter;
impl SendXcm for NotApplicableOrFailOnParachain2222XcmRouter {
	type Ticket = Option<Xcm<()>>;

	fn validate(
		destination: &mut Option<MultiLocation>,
		message: &mut Option<Xcm<()>>,
	) -> SendResult<Self::Ticket> {
		log::info!(
			target: super::LOG_TARGET,
			"[NotApplicableOrFailOnParachain2222XcmRouter]: destination: {:?}, message: {:?}",
			destination,
			message
		);
		if matches!(
			destination,
			Some(MultiLocation { interior: X1(Parachain(Self::UNROUTABLE_PARA_ID)), parents: 1 })
		) {
			Err(SendError::Transport("Simulate what ever error"))
		} else {
			Err(SendError::NotApplicable)
		}
	}

	fn deliver(ticket: Self::Ticket) -> Result<XcmHash, SendError> {
		unimplemented!("We should not come here, ticket: {:?}", ticket)
	}
}

impl NotApplicableOrFailOnParachain2222XcmRouter {
	const UNROUTABLE_PARA_ID: u32 = 2222;
}

pub type XcmRouter = (NotApplicableOrFailOnParachain2222XcmRouter, ThreadLocalXcmRouter);

/// Bridge router, which wraps and sends xcm to BridgeHub to be delivered to the different GlobalConsensus
pub type TestBridgeXcmSender = UnpaidRemoteExporter<BridgeTransfer, XcmRouter, UniversalLocation>;

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

pub type LocationToAccountId = (
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
	// Different global consensus parachain sovereign account.
	// (Used for over-bridge transfers and reserve processing)
	GlobalConsensusParachainConvertsFor<UniversalLocation, AccountId>,
);

/// Means for transacting the native currency on this chain.
pub type CurrencyTransactor = CurrencyAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<RelayLocation>,
	// Convert an XCM MultiLocation into a local account id:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports of `Balances`.
	(),
>;

/// Benchmarks helper.
#[cfg(feature = "runtime-benchmarks")]
pub struct TestBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl BenchmarkHelper<RuntimeOrigin> for TestBenchmarkHelper {
	fn bridge_config() -> (NetworkId, BridgeConfig) {
		test_bridge_config()
	}

	fn prepare_asset_transfer() -> (RuntimeOrigin, VersionedMultiAssets, VersionedMultiLocation) {
		let assets_count = MaxAssetsLimit::get();

		// sender account must have enough funds
		let sender_account = account(1);
		let total_deposit = ExistentialDeposit::get() * (1 + assets_count as u64);
		let _ = Balances::deposit_creating(&sender_account, total_deposit);

		// finally - prepare assets and destination
		let assets = VersionedMultiAssets::V3(
			std::iter::repeat(MultiAsset {
				fun: Fungible(ExistentialDeposit::get().into()),
				id: Concrete(RelayLocation::get()),
			})
			.take(assets_count as usize)
			.collect::<Vec<_>>()
			.into(),
		);
		let destination = VersionedMultiLocation::V3(MultiLocation::new(
			2,
			X3(GlobalConsensus(Wococo), Parachain(1000), consensus_account(Wococo, 2)),
		));

		(RuntimeOrigin::signed(sender_account), assets, destination)
	}
}

parameter_types! {
	pub const TrapCode: u64 = 12345;
	pub const MaxAssetsLimit: u8 = 1;
}

impl Config for TestRuntime {
	type RuntimeEvent = RuntimeEvent;
	type UniversalLocation = UniversalLocation;
	type WeightInfo = ();
	type AdminOrigin = EnsureRoot<AccountId>;
	type AllowReserveAssetTransferOrigin = AsEnsureOriginWithArg<EnsureRoot<AccountId>>;
	type UniversalAliasesLimit = ConstU32<2>;
	type ReserveLocationsLimit = ConstU32<2>;
	type AssetsPerReserveLocationLimit = ConstU32<2>;
	type AssetTransactor = CurrencyTransactor;
	type AssetTransferKindResolver = ConfiguredConcreteAssetTransferKindResolver<Self>;
	type BridgeXcmSender = TestBridgeXcmSender;
	type AssetTransferOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type MaxAssetsLimit = MaxAssetsLimit;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = TestBenchmarkHelper;
}

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
	sp_tracing::try_init_simple();
	let t = frame_system::GenesisConfig::default().build_storage::<TestRuntime>().unwrap();

	// with 0 block_number events dont work
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		frame_system::Pallet::<TestRuntime>::set_block_number(1u32.into());
	});

	ext
}

fn account(account: u8) -> AccountId32 {
	AccountId32::new([account; 32])
}

fn consensus_account(network: NetworkId, account: u8) -> Junction {
	xcm::prelude::AccountId32 { network: Some(network), id: AccountId32::new([account; 32]).into() }
}

#[test]
fn test_ensure_reachable_remote_destination() {
	new_test_ext().execute_with(|| {
		// insert exporter config + allowed target location
		let bridged_network = BridgedNetwork::get();
		let bridge_location = MultiLocation::new(1, X1(Parachain(1013)));
		assert_ok!(BridgeTransfer::add_exporter_config(
			RuntimeOrigin::root(),
			bridged_network,
			Box::new(bridge_location.clone().into_versioned()),
			None,
		));
		let target_location: MultiLocation =
			MultiLocation::new(2, X2(GlobalConsensus(bridged_network), Parachain(1000)));
		let target_location_fee: MultiAsset = (MultiLocation::parent(), 1_000_000).into();
		assert_ok!(BridgeTransfer::update_bridged_target_location(
			RuntimeOrigin::root(),
			bridged_network,
			Box::new(target_location.clone().into_versioned()),
			Some(Box::new(target_location_fee.clone().into())),
		));

		// v2 not supported
		assert_eq!(
			BridgeTransfer::ensure_reachable_remote_destination(VersionedMultiLocation::V2(
				xcm::v2::MultiLocation::default()
			)),
			Err(Error::<TestRuntime>::UnsupportedDestination)
		);

		// v3 - "parent: 0" wrong
		assert_eq!(
			BridgeTransfer::ensure_reachable_remote_destination(VersionedMultiLocation::V3(
				MultiLocation::new(0, X2(GlobalConsensus(bridged_network), Parachain(1000)))
			)),
			Err(Error::<TestRuntime>::UnsupportedDestination)
		);
		// v3 - "parent: 1" wrong
		assert_eq!(
			BridgeTransfer::ensure_reachable_remote_destination(VersionedMultiLocation::V3(
				MultiLocation::new(1, X2(GlobalConsensus(bridged_network), Parachain(1000)))
			)),
			Err(Error::<TestRuntime>::UnsupportedDestination)
		);

		// v3 - Rococo is not supported
		assert_eq!(
			BridgeTransfer::ensure_reachable_remote_destination(VersionedMultiLocation::V3(
				MultiLocation::new(2, X2(GlobalConsensus(Rococo), Parachain(1000)))
			)),
			Err(Error::<TestRuntime>::UnsupportedDestination)
		);

		// v3 - remote_destination is not allowed
		assert_eq!(
			BridgeTransfer::ensure_reachable_remote_destination(VersionedMultiLocation::V3(
				MultiLocation::new(2, X2(GlobalConsensus(bridged_network), Parachain(1234)))
			)),
			Err(Error::<TestRuntime>::UnsupportedDestination)
		);

		// v3 - ok (allowed)
		assert_ok!(
			BridgeTransfer::ensure_reachable_remote_destination(VersionedMultiLocation::V3(
				MultiLocation::new(
					2,
					X3(
						GlobalConsensus(bridged_network),
						Parachain(1000),
						consensus_account(bridged_network, 35)
					)
				),
			)),
			ReachableDestination {
				bridge: MaybePaidLocation { location: bridge_location, maybe_fee: None },
				target: MaybePaidLocation {
					location: MultiLocation::new(
						2,
						X2(GlobalConsensus(bridged_network), Parachain(1000))
					),
					maybe_fee: Some(target_location_fee),
				},
				target_asset_filter: None,
				target_destination: MultiLocation::new(
					2,
					X3(
						GlobalConsensus(bridged_network),
						Parachain(1000),
						consensus_account(bridged_network, 35)
					)
				),
			}
		);
	})
}

#[test]
fn test_transfer_asset_via_bridge_for_currency_works() {
	new_test_ext().execute_with(|| {
		// initialize some Balances for user_account
		let user_account = account(1);
		let user_account_init_balance = 1000_u64;
		let _ = Balances::deposit_creating(&user_account, user_account_init_balance);
		let user_free_balance = Balances::free_balance(&user_account);
		let balance_to_transfer = 15_u64;
		assert!((user_free_balance - balance_to_transfer) >= ExistentialDeposit::get());
		// because, sovereign account needs to have ED otherwise reserve fails
		assert!(balance_to_transfer >= ExistentialDeposit::get());

		// insert bridge config
		let bridged_network = BridgedNetwork::get();
		let bridge_location: MultiLocation = (Parent, Parachain(1013)).into();
		assert_ok!(BridgeTransfer::add_exporter_config(
			RuntimeOrigin::root(),
			bridged_network,
			Box::new(bridge_location.into_versioned()),
			None,
		));
		// insert allowed reserve asset and allow all
		let target_location: MultiLocation =
			MultiLocation::new(2, X2(GlobalConsensus(bridged_network), Parachain(1000)));
		assert_ok!(BridgeTransfer::update_bridged_target_location(
			RuntimeOrigin::root(),
			bridged_network,
			Box::new(target_location.clone().into_versioned()),
			None,
		));
		assert_ok!(BridgeTransfer::allow_reserve_asset_transfer_for(
			RuntimeOrigin::root(),
			bridged_network,
			Box::new(target_location.clone().into_versioned()),
			AssetFilter::All,
		));

		// checks before
		assert!(ROUTED_MESSAGE.with(|r| r.borrow().is_none()));
		assert_eq!(Balances::free_balance(&user_account), user_account_init_balance);
		let reserve_account = LocationToAccountId::convert_location(&target_location)
			.expect("converted target_location as accountId");
		assert_eq!(Balances::free_balance(&reserve_account), 0);

		// trigger transfer_asset_via_bridge - should trigger new ROUTED_MESSAGE
		let asset = MultiAsset {
			fun: Fungible(balance_to_transfer.into()),
			id: Concrete(RelayLocation::get()),
		};
		let assets = Box::new(VersionedMultiAssets::from(MultiAssets::from(asset)));

		// destination is account from different consensus
		let destination = Box::new(VersionedMultiLocation::from(MultiLocation::new(
			2,
			X3(
				GlobalConsensus(bridged_network),
				Parachain(1000),
				consensus_account(bridged_network, 2),
			),
		)));

		// trigger asset transfer
		assert_ok!(BridgeTransfer::transfer_asset_via_bridge(
			RuntimeOrigin::signed(account(1)),
			assets,
			destination,
		));

		// check user account decressed
		assert_eq!(
			Balances::free_balance(&user_account),
			user_account_init_balance - balance_to_transfer
		);
		// check reserve account increased
		assert_eq!(Balances::free_balance(&reserve_account), 15);

		// check events
		let events = System::events();
		assert!(!events.is_empty());

		// check reserve asset deposited event
		assert!(System::events().iter().any(|r| matches!(
			r.event,
			RuntimeEvent::BridgeTransfer(Event::ReserveAssetsDeposited { .. })
		)));
		assert!(System::events().iter().any(|r| matches!(
			r.event,
			RuntimeEvent::BridgeTransfer(Event::TransferInitiated { .. })
		)));

		// check fired XCM ExportMessage to bridge-hub
		let fired_xcm =
			ROUTED_MESSAGE.with(|r| r.take().expect("xcm::ExportMessage should be here"));

		if let Some(ExportMessage { xcm, .. }) = fired_xcm.0.iter().find(|instr| {
			matches!(
				instr,
				ExportMessage {
					network,
					destination: X1(Parachain(1000)),
					..
				} if network == &bridged_network
			)
		}) {
			assert!(xcm.0.iter().any(|instr| matches!(instr, UnpaidExecution { .. })));
			assert!(xcm.0.iter().any(|instr| matches!(instr, ReserveAssetDeposited(..))));
			assert!(xcm.0.iter().any(|instr| matches!(instr, DepositAsset { .. })));
			assert!(xcm.0.iter().any(|instr| matches!(instr, SetTopic { .. })));
		} else {
			assert!(false, "Does not contains [`ExportMessage`], fired_xcm: {:?}", fired_xcm);
		}
	});
}

#[test]
fn test_transfer_asset_via_bridge_in_case_of_error_transactional_works() {
	new_test_ext().execute_with(|| {
		// initialize some Balances for user_account
		let user_account = account(1);
		let user_account_init_balance = 1000_u64;
		let _ = Balances::deposit_creating(&user_account, user_account_init_balance);
		let user_free_balance = Balances::free_balance(&user_account);
		let balance_to_transfer = 15_u64;
		assert!((user_free_balance - balance_to_transfer) >= ExistentialDeposit::get());
		// because, sovereign account needs to have ED otherwise reserve fails
		assert!(balance_to_transfer >= ExistentialDeposit::get());

		// insert bridge config (with unroutable bridge_location - 2222)
		let bridged_network = BridgedNetwork::get();
		assert_ok!(BridgeTransfer::add_exporter_config(
			RuntimeOrigin::root(),
			bridged_network,
			Box::new(
				MultiLocation::new(
					1,
					Parachain(NotApplicableOrFailOnParachain2222XcmRouter::UNROUTABLE_PARA_ID)
				)
				.into_versioned()
			),
			None,
		));
		let target_location =
			MultiLocation::new(2, X2(GlobalConsensus(bridged_network), Parachain(1000)));
		assert_ok!(BridgeTransfer::update_bridged_target_location(
			RuntimeOrigin::root(),
			bridged_network,
			Box::new(target_location.clone().into_versioned()),
			None,
		));
		assert_ok!(BridgeTransfer::allow_reserve_asset_transfer_for(
			RuntimeOrigin::root(),
			bridged_network,
			Box::new(target_location.into_versioned()),
			AssetFilter::All,
		));

		// checks before
		assert!(ROUTED_MESSAGE.with(|r| r.borrow().is_none()));
		let user_balance_before = Balances::free_balance(&user_account);
		assert_eq!(user_balance_before, user_account_init_balance);
		let reserve_account = LocationToAccountId::convert_location(&target_location)
			.expect("converted target_location as accountId");
		let reserve_account_before = Balances::free_balance(&reserve_account);
		assert_eq!(reserve_account_before, 0);

		// trigger transfer_asset_via_bridge - should trigger new ROUTED_MESSAGE
		let asset = MultiAsset {
			fun: Fungible(balance_to_transfer.into()),
			id: Concrete(RelayLocation::get()),
		};
		let assets = Box::new(VersionedMultiAssets::from(MultiAssets::from(asset)));

		// destination is account from different consensus
		let destination = Box::new(VersionedMultiLocation::from(MultiLocation::new(
			2,
			X3(
				GlobalConsensus(bridged_network),
				Parachain(1000),
				consensus_account(bridged_network, 2),
			),
		)));

		// reset events
		System::reset_events();

		// trigger asset transfer
		assert_noop!(
			BridgeTransfer::transfer_asset_via_bridge(
				RuntimeOrigin::signed(account(1)),
				assets,
				destination
			),
			DispatchError::Module(ModuleError {
				index: 52,
				error: [9, 0, 0, 0],
				message: Some("BridgeCallError")
			})
		);

		// checks after
		// balances are untouched
		assert_eq!(Balances::free_balance(&user_account), user_balance_before);
		assert_eq!(Balances::free_balance(&reserve_account), reserve_account_before);
		// no xcm messages fired
		assert!(ROUTED_MESSAGE.with(|r| r.borrow().is_none()));
		// check events (no events because of rollback)
		assert!(System::events().is_empty());
	});
}

#[test]
fn allowed_exporters_management_works() {
	let bridged_network = BridgedNetwork::get();
	let bridge_location = MultiLocation::new(1, X1(Parachain(1013)));
	let dummy_xcm = Xcm(vec![]);
	let dummy_remote_interior_multilocation = X1(Parachain(1234));

	new_test_ext().execute_with(|| {
		assert_eq!(AllowedExporters::<TestRuntime>::iter().count(), 0);

		// should fail - just root is allowed
		assert_noop!(
			BridgeTransfer::add_exporter_config(
				RuntimeOrigin::signed(account(1)),
				bridged_network,
				Box::new(bridge_location.clone().into_versioned()),
				None
			),
			DispatchError::BadOrigin
		);

		// should fail - we expect local bridge
		assert_noop!(
			BridgeTransfer::add_exporter_config(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(MultiLocation::new(2, X1(Parachain(1234))).into_versioned()),
				None
			),
			DispatchError::Module(ModuleError {
				index: 52,
				error: [0, 0, 0, 0],
				message: Some("InvalidConfiguration")
			})
		);
		assert_eq!(AllowedExporters::<TestRuntime>::iter().count(), 0);
		assert_eq!(
			BridgeTransfer::exporter_for(
				&bridged_network,
				&dummy_remote_interior_multilocation,
				&dummy_xcm
			),
			None
		);

		// add with root
		assert_ok!(BridgeTransfer::add_exporter_config(
			RuntimeOrigin::root(),
			bridged_network,
			Box::new(bridge_location.clone().into_versioned()),
			None
		));
		assert_eq!(AllowedExporters::<TestRuntime>::iter().count(), 1);
		assert_eq!(
			AllowedExporters::<TestRuntime>::get(bridged_network),
			Some(BridgeConfig {
				bridge_location: VersionedMultiLocation::from(bridge_location),
				bridge_location_fee: None,
				allowed_target_locations: Default::default(),
			})
		);
		assert_eq!(AllowedExporters::<TestRuntime>::get(&RelayNetwork::get()), None);
		assert_eq!(
			BridgeTransfer::exporter_for(
				&bridged_network,
				&dummy_remote_interior_multilocation,
				&dummy_xcm
			),
			Some((bridge_location.clone(), None))
		);
		assert_eq!(
			BridgeTransfer::exporter_for(
				&RelayNetwork::get(),
				&dummy_remote_interior_multilocation,
				&dummy_xcm
			),
			None
		);

		// update fee
		// remove
		assert_ok!(BridgeTransfer::update_exporter_config(
			RuntimeOrigin::root(),
			bridged_network,
			Some(VersionedMultiAsset::V3((Parent, 200u128).into()).into()),
		));
		assert_eq!(AllowedExporters::<TestRuntime>::iter().count(), 1);
		assert_eq!(
			AllowedExporters::<TestRuntime>::get(bridged_network),
			Some(BridgeConfig {
				bridge_location: VersionedMultiLocation::from(bridge_location),
				bridge_location_fee: Some((Parent, 200u128).into()),
				allowed_target_locations: Default::default(),
			})
		);
		assert_eq!(
			BridgeTransfer::exporter_for(
				&bridged_network,
				&dummy_remote_interior_multilocation,
				&dummy_xcm
			),
			Some((bridge_location, Some((Parent, 200u128).into())))
		);

		// remove
		assert_ok!(BridgeTransfer::remove_exporter_config(RuntimeOrigin::root(), bridged_network,));
		assert_eq!(AllowedExporters::<TestRuntime>::get(bridged_network), None);
		assert_eq!(AllowedExporters::<TestRuntime>::iter().count(), 0);
	})
}

#[test]
fn allowed_universal_aliases_management_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(AllowedUniversalAliases::<TestRuntime>::iter().count(), 0);

		let location1 = MultiLocation::new(1, X1(Parachain(1014)));
		let junction1 = GlobalConsensus(ByGenesis([1; 32]));
		let junction2 = GlobalConsensus(ByGenesis([2; 32]));

		// should fail - just root is allowed
		assert_noop!(
			BridgeTransfer::add_universal_alias(
				RuntimeOrigin::signed(account(1)),
				Box::new(VersionedMultiLocation::V3(location1.clone())),
				junction1.clone(),
			),
			DispatchError::BadOrigin
		);
		assert_eq!(AllowedUniversalAliases::<TestRuntime>::iter().count(), 0);
		assert!(!AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction1)));
		assert!(!AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction2)));

		// add ok
		assert_ok!(BridgeTransfer::add_universal_alias(
			RuntimeOrigin::root(),
			Box::new(VersionedMultiLocation::V3(location1.clone())),
			junction1.clone(),
		));
		assert_ok!(BridgeTransfer::add_universal_alias(
			RuntimeOrigin::root(),
			Box::new(VersionedMultiLocation::V3(location1.clone())),
			junction2.clone(),
		));
		assert!(AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction1)));
		assert!(AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction2)));
		assert_eq!(AllowedUniversalAliases::<TestRuntime>::iter().count(), 1);

		// remove ok
		assert_ok!(BridgeTransfer::remove_universal_alias(
			RuntimeOrigin::root(),
			Box::new(VersionedMultiLocation::V3(location1.clone())),
			vec![junction1.clone()],
		));
		assert!(!AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction1)));
		assert!(AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction2)));
		assert_eq!(AllowedUniversalAliases::<TestRuntime>::iter().count(), 1);

		assert_ok!(BridgeTransfer::remove_universal_alias(
			RuntimeOrigin::root(),
			Box::new(VersionedMultiLocation::V3(location1.clone())),
			vec![junction2.clone()],
		));
		assert!(!AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction1)));
		assert!(!AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction2)));
		assert_eq!(AllowedUniversalAliases::<TestRuntime>::iter().count(), 0);
	})
}

#[test]
fn allowed_reserve_locations_management_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(0, AllowedReserveLocations::<TestRuntime>::iter_values().count());

		let location1 = MultiLocation::new(1, X1(Parachain(1014)));
		let location1_as_latest = VersionedMultiLocation::from(location1.clone());
		let location1_as_latest_as_key =
			LatestVersionedMultiLocation::try_from(&location1_as_latest).expect("ok");
		let location2 =
			MultiLocation::new(2, X2(GlobalConsensus(ByGenesis([1; 32])), Parachain(1014)));
		let location2_as_latest = VersionedMultiLocation::from(location2.clone());
		let location2_as_key =
			LatestVersionedMultiLocation::try_from(&location2_as_latest).expect("ok");

		let asset_location = MultiLocation::parent();
		let asset: MultiAsset = (asset_location, 200u128).into();

		let asset_filter_for_asset_by_multilocation = MultiLocationFilter {
			equals_any: BoundedVec::truncate_from(vec![asset_location.into_versioned()]),
			starts_with_any: Default::default(),
		};
		let asset_filter_for_asset =
			AssetFilter::ByMultiLocation(asset_filter_for_asset_by_multilocation.clone());
		let asset_filter_for_other_by_multilocation = MultiLocationFilter {
			equals_any: BoundedVec::truncate_from(vec![
				MultiLocation::new(3, Here).into_versioned()
			]),
			starts_with_any: Default::default(),
		};
		let asset_filter_for_other =
			AssetFilter::ByMultiLocation(asset_filter_for_other_by_multilocation.clone());

		// should fail - just root is allowed
		assert_noop!(
			BridgeTransfer::add_reserve_location(
				RuntimeOrigin::signed(account(1)),
				Box::new(location1_as_latest.clone()),
				asset_filter_for_asset.clone(),
			),
			DispatchError::BadOrigin
		);
		assert!(AllowedReserveLocations::<TestRuntime>::get(&location1_as_latest_as_key).is_none());
		assert!(AllowedReserveLocations::<TestRuntime>::get(&location2_as_key).is_none());
		assert!(!IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
			&asset, &location1
		));
		assert!(!IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
			&asset, &location2
		));
		assert_eq!(
			ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(&asset, &location1),
			AssetTransferKind::Unsupported
		);
		assert_eq!(
			ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(&asset, &location2),
			AssetTransferKind::Unsupported
		);

		// add ok
		assert_ok!(BridgeTransfer::add_reserve_location(
			RuntimeOrigin::root(),
			Box::new(VersionedMultiLocation::V3(location1.clone())),
			asset_filter_for_asset.clone()
		));
		assert_ok!(BridgeTransfer::add_reserve_location(
			RuntimeOrigin::root(),
			Box::new(VersionedMultiLocation::V3(location2.clone())),
			asset_filter_for_other.clone()
		));
		assert_eq!(2, AllowedReserveLocations::<TestRuntime>::iter_values().count());
		assert!(IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
			&asset, &location1
		));
		assert!(!IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
			&asset, &location2
		));
		assert_eq!(
			ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(&asset, &location1),
			AssetTransferKind::WithdrawReserve
		);
		assert_eq!(
			ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(&asset, &location2),
			AssetTransferKind::Unsupported
		);

		assert_ok!(BridgeTransfer::add_reserve_location(
			RuntimeOrigin::root(),
			Box::new(location2_as_latest.clone()),
			asset_filter_for_asset.clone()
		));
		assert!(IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
			&asset, &location2
		));
		assert_eq!(
			ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(&asset, &location2),
			AssetTransferKind::WithdrawReserve
		);

		// test remove
		assert_noop!(
			BridgeTransfer::remove_reserve_location(
				RuntimeOrigin::root(),
				Box::new(location1_as_latest.clone()),
				Some(asset_filter_for_other_by_multilocation.clone())
			),
			DispatchError::Module(ModuleError {
				index: 52,
				error: [0, 0, 0, 0],
				message: Some("UnavailableConfiguration")
			})
		);

		assert_ok!(BridgeTransfer::remove_reserve_location(
			RuntimeOrigin::root(),
			Box::new(location1_as_latest.clone()),
			Some(asset_filter_for_asset_by_multilocation.clone())
		));
		assert_eq!(1, AllowedReserveLocations::<TestRuntime>::iter_values().count());
		assert!(!IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
			&asset, &location1
		));
		assert!(IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
			&asset, &location2
		));
		assert_eq!(
			ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(&asset, &location1),
			AssetTransferKind::Unsupported
		);
		assert_eq!(
			ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(&asset, &location2),
			AssetTransferKind::WithdrawReserve
		);

		assert_ok!(BridgeTransfer::remove_reserve_location(
			RuntimeOrigin::root(),
			Box::new(location2_as_latest.clone()),
			Some(asset_filter_for_other_by_multilocation.clone())
		));
		assert!(!IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
			&asset, &location1
		));
		assert!(IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
			&asset, &location2
		));
		assert_eq!(
			ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(&asset, &location1),
			AssetTransferKind::Unsupported
		);
		assert_eq!(
			ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(&asset, &location2),
			AssetTransferKind::WithdrawReserve
		);

		assert_ok!(BridgeTransfer::remove_reserve_location(
			RuntimeOrigin::root(),
			Box::new(location2_as_latest),
			Some(asset_filter_for_asset_by_multilocation)
		));
		assert!(!IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
			&asset, &location1
		));
		assert!(!IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
			&asset, &location2
		));
		assert_eq!(
			ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(&asset, &location1),
			AssetTransferKind::Unsupported
		);
		assert_eq!(
			ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(&asset, &location2),
			AssetTransferKind::Unsupported
		);
	})
}

#[test]
fn allowed_bridged_target_location_management_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(0, AllowedExporters::<TestRuntime>::iter_values().count());

		let bridged_network = BridgedNetwork::get();
		let bridge_location: MultiLocation = (Parent, Parachain(1013)).into();
		let target_location: MultiLocation =
			MultiLocation::new(2, X2(GlobalConsensus(bridged_network), Parachain(1000)));

		// should fail - we need BridgeConfig first
		assert_noop!(
			BridgeTransfer::allow_reserve_asset_transfer_for(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(target_location.clone().into_versioned()),
				AssetFilter::All,
			),
			DispatchError::Module(ModuleError {
				index: 52,
				error: [2, 0, 0, 0],
				message: Some("UnavailableConfiguration")
			})
		);

		// add bridge config
		assert_ok!(BridgeTransfer::add_exporter_config(
			RuntimeOrigin::root(),
			bridged_network,
			Box::new(bridge_location.into_versioned()),
			None,
		));

		// should fail - we need also target_location first
		assert_noop!(
			BridgeTransfer::allow_reserve_asset_transfer_for(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(target_location.clone().into_versioned()),
				AssetFilter::All,
			),
			DispatchError::Module(ModuleError {
				index: 52,
				error: [1, 0, 0, 0],
				message: Some("InvalidBridgeConfiguration")
			})
		);

		// insert allowed target location
		assert_ok!(BridgeTransfer::update_bridged_target_location(
			RuntimeOrigin::root(),
			bridged_network,
			Box::new(target_location.clone().into_versioned()),
			None,
		));

		let asset1_location = MultiLocation::new(2, X2(Parachain(1235), Parachain(5678)));
		let asset1 = MultiAsset::from((asset1_location, 1000));
		assert_eq!(
			ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(
				&asset1,
				&target_location
			),
			AssetTransferKind::Unsupported
		);

		// now should pass - add one start_with pattern
		assert_ok!(BridgeTransfer::allow_reserve_asset_transfer_for(
			RuntimeOrigin::root(),
			bridged_network,
			Box::new(target_location.clone().into_versioned()),
			AssetFilter::ByMultiLocation(MultiLocationFilter {
				equals_any: Default::default(),
				starts_with_any: BoundedVec::truncate_from(vec![MultiLocation::new(
					2,
					X1(Parachain(2223))
				)
				.into_versioned()]),
			}),
		));

		// not allowed yet
		assert_eq!(
			ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(
				&asset1,
				&target_location
			),
			AssetTransferKind::Unsupported
		);

		// now should pass - add another start_with pattern
		assert_ok!(BridgeTransfer::allow_reserve_asset_transfer_for(
			RuntimeOrigin::root(),
			bridged_network,
			Box::new(target_location.clone().into_versioned()),
			AssetFilter::ByMultiLocation(MultiLocationFilter {
				equals_any: Default::default(),
				starts_with_any: BoundedVec::truncate_from(vec![MultiLocation::new(
					2,
					X1(Parachain(1235))
				)
				.into_versioned()]),
			}),
		));

		// ok
		assert_eq!(
			ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(
				&asset1,
				&target_location
			),
			AssetTransferKind::ReserveBased
		);
	})
}
