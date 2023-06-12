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

use crate::{
	features::{
		ConcreteAssetTransferKindResolver,
		IsAllowedReserveBasedTransferForConcreteAssetToBridgedLocation,
	},
	Config, Error, Event,
};
use frame_support::{
	assert_noop, assert_ok,
	dispatch::DispatchError,
	parameter_types, sp_io, sp_tracing,
	traits::{ConstU32, Currency},
};
use pallet_bridge_transfer_primitives::{
	AssetFilter, BridgeConfig, BridgesConfig, BridgesConfigAdapter, BridgesConfigBuilder,
	MaybePaidLocation, ReachableDestination,
};
use polkadot_parachain::primitives::Sibling;
use sp_runtime::{
	testing::{Header, H256},
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32, ModuleError,
};
use sp_version::RuntimeVersion;
use xcm::prelude::*;
use xcm_builder::{
	AccountId32Aliases, CurrencyAdapter, EnsureXcmOrigin, GlobalConsensusParachainConvertsFor,
	IsConcrete, SiblingParachainConvertsVia, SignedToAccountId32, UnpaidRemoteExporter,
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
	pub BridgeLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(1013)));
	pub TargetLocation: MultiLocation =  MultiLocation::new(2, X2(GlobalConsensus(BridgedNetwork::get()), Parachain(1000)));
	// pub TargetLocationFee: Option<MultiAsset> = Some((MultiLocation::parent(), 1_000_000).into());
	pub TargetLocationFee: Option<MultiAsset> = None;

	pub const RelayNetwork: NetworkId = NetworkId::ByGenesis([9; 32]);
	pub UniversalLocation: InteriorMultiLocation = X2(GlobalConsensus(RelayNetwork::get()), Parachain(1000));
	// Relay chain currency/balance location (e.g. KsmLocation, DotLocation, ..)
	pub const RelayLocation: MultiLocation = MultiLocation::parent();

	pub Bridges: BridgesConfig = BridgesConfigBuilder::default()
		.add_or_panic(
			BridgedNetwork::get(),
			BridgeConfig::new(
				MaybePaidLocation {
					location: BridgeLocation::get(),
					maybe_fee: None,
				}
			).add_target_location(
				MaybePaidLocation {
					location: TargetLocation::get(),
					maybe_fee: TargetLocationFee::get(),
				},
				Some(AssetFilter::All)
			)
		)
		.build();

}

std::thread_local! {
	static ROUTED_MESSAGE: std::cell::RefCell<Option<Xcm<()>>> = std::cell::RefCell::new(None);
	static NOT_APPLICABLE_AS_SOME_OR_FAIL_ROUTER_SWITCH: std::cell::RefCell<Option<()>> = std::cell::RefCell::new(Some(()));
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

pub struct NotApplicableOrFailRouter;
impl SendXcm for NotApplicableOrFailRouter {
	type Ticket = Option<Xcm<()>>;

	fn validate(
		destination: &mut Option<MultiLocation>,
		message: &mut Option<Xcm<()>>,
	) -> SendResult<Self::Ticket> {
		log::info!(
			target: super::LOG_TARGET,
			"[NotApplicableOrFailRouter]: destination: {:?}, message: {:?}",
			destination,
			message
		);

		let wanna_fail =
			NOT_APPLICABLE_AS_SOME_OR_FAIL_ROUTER_SWITCH.with(|s| s.borrow().is_none());
		if wanna_fail {
			Err(SendError::Transport("Simulate what ever error"))
		} else {
			Err(SendError::NotApplicable)
		}
	}

	fn deliver(ticket: Self::Ticket) -> Result<XcmHash, SendError> {
		unimplemented!("We should not come here, ticket: {:?}", ticket)
	}
}

pub type XcmRouter = (NotApplicableOrFailRouter, ThreadLocalXcmRouter);

/// Bridge router, which wraps and sends xcm to BridgeHub to be delivered to the different GlobalConsensus
pub type TestBridgeXcmSender =
	UnpaidRemoteExporter<BridgesConfigAdapter<Bridges>, XcmRouter, UniversalLocation>;

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

parameter_types! {
	// na constantu
	pub const AssetsLimit: u8 = 1;
}

impl Config for TestRuntime {
	type RuntimeEvent = RuntimeEvent;
	type UniversalLocation = UniversalLocation;
	type WeightInfo = ();
	type AssetTransactor = CurrencyTransactor;
	type AssetTransferKindResolver = ConcreteAssetTransferKindResolver<
		frame_support::traits::Nothing,
		IsAllowedReserveBasedTransferForConcreteAssetToBridgedLocation<UniversalLocation, Bridges>,
	>;
	type AssetTransferOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type AssetsLimit = AssetsLimit;
	type BridgedDestinationValidator = BridgesConfigAdapter<Bridges>;
	type BridgeXcmSender = TestBridgeXcmSender;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
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
		let bridged_network = BridgedNetwork::get();

		// v3 - "parent: 0" wrong
		assert_eq!(
			BridgeTransfer::ensure_reachable_remote_destination(MultiLocation::new(
				0,
				X2(GlobalConsensus(bridged_network), Parachain(1000))
			)),
			Err(Error::<TestRuntime>::UnsupportedDestination)
		);
		// v3 - "parent: 1" wrong
		assert_eq!(
			BridgeTransfer::ensure_reachable_remote_destination(MultiLocation::new(
				1,
				X2(GlobalConsensus(bridged_network), Parachain(1000))
			)),
			Err(Error::<TestRuntime>::UnsupportedDestination)
		);

		// v3 - Rococo is not supported
		assert_eq!(
			BridgeTransfer::ensure_reachable_remote_destination(MultiLocation::new(
				2,
				X2(GlobalConsensus(Rococo), Parachain(1000))
			)),
			Err(Error::<TestRuntime>::UnsupportedDestination)
		);

		// v3 - remote_destination is not allowed
		assert_eq!(
			BridgeTransfer::ensure_reachable_remote_destination(MultiLocation::new(
				2,
				X2(GlobalConsensus(bridged_network), Parachain(1234))
			)),
			Err(Error::<TestRuntime>::UnsupportedDestination)
		);

		// v3 - ok (allowed)
		assert_ok!(
			BridgeTransfer::ensure_reachable_remote_destination(MultiLocation::new(
				2,
				X3(
					GlobalConsensus(bridged_network),
					Parachain(1000),
					consensus_account(bridged_network, 35)
				)
			),),
			ReachableDestination {
				bridge: MaybePaidLocation { location: BridgeLocation::get(), maybe_fee: None },
				target: MaybePaidLocation {
					location: MultiLocation::new(
						2,
						X2(GlobalConsensus(bridged_network), Parachain(1000))
					),
					maybe_fee: TargetLocationFee::get(),
				},
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

		let bridged_network = BridgedNetwork::get();
		let target_location = TargetLocation::get();

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
			println!("{:?}", xcm.0);
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

		let bridged_network = BridgedNetwork::get();
		let target_location = TargetLocation::get();

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

		// Simulate XcmRouter failure
		NOT_APPLICABLE_AS_SOME_OR_FAIL_ROUTER_SWITCH.with(|s| *s.borrow_mut() = None);

		// trigger asset transfer
		assert_noop!(
			BridgeTransfer::transfer_asset_via_bridge(
				RuntimeOrigin::signed(account(1)),
				assets,
				destination
			),
			DispatchError::Module(ModuleError {
				index: 52,
				error: [6, 0, 0, 0],
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
