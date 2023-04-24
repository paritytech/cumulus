use frame_support::{
	assert_ok,
	instances::Instance1,
	pallet_prelude::Hooks,
	sp_io, sp_tracing,
	traits::{fungibles::Inspect, GenesisBuild},
};

use codec::Encode;
use polkadot_runtime_parachains::configuration::HostConfiguration;
use sp_core::parameter_types;
use statemine_runtime::constants::currency::GRAND;
use xcm::prelude::*;
use xcm_emulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain, TestExt};

use xcm_executor::traits::Convert;

use parachains_common::Balance;
use polkadot_core_primitives::InboundDownwardMessage;
use polkadot_parachain::primitives::DmpMessageHandler;
use sp_weights::Weight;
use xcm::{
	latest::{Ancestor, MultiLocation},
	prelude::{AccountId32, Here, Parachain},
	v3::Outcome,
	VersionedMultiAssets,
};

pub const ALICE: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([0u8; 32]);
pub const BOB: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([1u8; 32]);
pub const INITIAL_BALANCE: u128 = 1000 * GRAND;

decl_test_parachain! {
	pub struct Statemine {
		Runtime = statemine_runtime::Runtime,
		RuntimeOrigin = statemine_runtime::RuntimeOrigin,
		XcmpMessageHandler = statemine_runtime::XcmpQueue,
		DmpMessageHandler = statemine_runtime::DmpQueue,
		new_ext = statemine_ext(),
	}
}

decl_test_parachain! {
	pub struct Penpal {
		Runtime = penpal_runtime::Runtime,
		RuntimeOrigin = penpal_runtime::RuntimeOrigin,
		XcmpMessageHandler = penpal_runtime::XcmpQueue,
		DmpMessageHandler = penpal_runtime::DmpQueue,
		new_ext = penpal_ext(),
	}
}

decl_test_relay_chain! {
	pub struct Relay {
		Runtime = kusama_runtime::Runtime,
		XcmConfig = kusama_runtime::xcm_config::XcmConfig,
		new_ext = relay_ext(),
	}
}

decl_test_network! {
	pub struct MockNet {
		relay_chain = Relay,
		parachains = vec![
			(1000, Statemine),
			(2000, Penpal),
		],
	}
}

// Define Statemine TestExternalities.
pub fn statemine_ext() -> sp_io::TestExternalities {
	use statemine_runtime::{Runtime, System};

	let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(ALICE, INITIAL_BALANCE), (parent_account_id(), INITIAL_BALANCE)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		sp_tracing::try_init_simple();
		System::set_block_number(1);
	});
	ext
}

// Define Penpal TestExternalities.
pub fn penpal_ext() -> sp_io::TestExternalities {
	use penpal_runtime::{Runtime, System};

	let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(ALICE, INITIAL_BALANCE), (parent_account_id(), INITIAL_BALANCE)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		sp_tracing::try_init_simple();
		System::set_block_number(1);
	});
	ext
}

// Define Kusama TestExternalities.
pub fn relay_ext() -> sp_io::TestExternalities {
	use kusama_runtime::{Runtime, RuntimeOrigin, System};

	// <XcmConfig::XcmSender as xcm_executor::Config>::XcmSender = RelayChainXcmRouter;
	// <Runtime as pallet_xcm::Config>::XcmRouter = RelayChainXcmRouter;

	let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

	polkadot_runtime_parachains::configuration::GenesisConfig::<Runtime> {
		config: HostConfiguration {
			max_upward_queue_count: 10,
			max_upward_queue_size: 51200,
			max_upward_message_size: 51200,
			max_upward_message_num_per_candidate: 10,
			max_downward_message_size: 51200,
			..Default::default()
		},
	}
	.assimilate_storage(&mut t)
	.unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![
			(ALICE, INITIAL_BALANCE),
			(child_account_id(1000), INITIAL_BALANCE),
			(child_account_id(2000), INITIAL_BALANCE),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
	});
	ext
}

pub fn parent_account_id() -> parachains_common::AccountId {
	let location = (Parent,);
	statemine_runtime::xcm_config::LocationToAccountId::convert(location.into()).unwrap()
}

pub fn child_account_id(para: u32) -> polkadot_core_primitives::AccountId {
	let location = (Parachain(para),);
	kusama_runtime::xcm_config::SovereignAccountOf::convert(location.into()).unwrap()
}

pub type RelayChainPalletXcm = pallet_xcm::Pallet<kusama_runtime::Runtime>;
pub type StateminePalletXcm = pallet_xcm::Pallet<statemine_runtime::Runtime>;
pub type PenpalPalletXcm = pallet_xcm::Pallet<penpal_runtime::Runtime>;

parameter_types! {
	pub StatemineLocation: MultiLocation = (Ancestor(0), Parachain(1000)).into();
}

#[test]
// NOTE: This needs to be run before every other test to ensure that chains can communicate with one
// another.
fn force_xcm_version() {
	let xcm_version = 3;
	Relay::execute_with(|| {
		use kusama_runtime::{RuntimeEvent, System};

		let statemine_location: MultiLocation = (Ancestor(0), Parachain(1000)).into();
		let penpal_location: MultiLocation = (Ancestor(0), Parachain(2000)).into();

		// Check that we can force xcm version for Statemine and Penpal from Kusama.
		for location in [statemine_location, penpal_location] {
			assert_ok!(RelayChainPalletXcm::force_xcm_version(
				kusama_runtime::RuntimeOrigin::root(),
				Box::new(location),
				xcm_version,
			));
			assert!(System::events().iter().any(|r| matches!(
				r.event,
				RuntimeEvent::XcmPallet(pallet_xcm::Event::SupportedVersionChanged {
					0: loc,
					1: ver,
				}) if loc == location && ver == xcm_version
			)));
		}
	});

	// Penpal forces Kusama xcm version.
	Penpal::execute_with(|| {
		use penpal_runtime::{RuntimeEvent, System};

		let location: MultiLocation = (Parent).into();

		assert_ok!(PenpalPalletXcm::force_xcm_version(
			penpal_runtime::RuntimeOrigin::root(),
			Box::new(location),
			xcm_version,
		));

		assert!(System::events().iter().any(|r| matches!(
			r.event,
			RuntimeEvent::PolkadotXcm(pallet_xcm::Event::SupportedVersionChanged {
				0: loc,
				1: ver,
			}) if loc == location && ver == xcm_version
		)));
	});
}

// Direct message passing tests.
mod dmp {
	use super::*;
	use xcm::latest::Error;
	use xcm_emulator::cumulus_pallet_dmp_queue;

	fn get_balances() -> (Balance, Balance) {
		let mut relay_balance = Default::default();
		Relay::execute_with(|| {
			relay_balance =
				kusama_runtime::System::account::<sp_runtime::AccountId32>(ALICE.into())
					.data
					.free;
		});
		let mut assets_para_balance = Default::default();

		Statemine::execute_with(|| {
			assets_para_balance =
				statemine_runtime::System::account::<sp_runtime::AccountId32>(ALICE.into())
					.data
					.free;
		});

		(relay_balance, assets_para_balance)
	}

	fn get_benf() -> Junction {
		AccountId32 { network: None, id: ALICE.into() }
	}

	const AP_DEST: (Ancestor, Junction) = (Ancestor(0), Parachain(1000));

	#[test]
	fn teleport_native_assets_relay_to_assets_para() {
		force_xcm_version();
		let amount = 1000_000_000;
		let assets: VersionedMultiAssets = (Here, amount).into();

		let mut messages: Vec<InboundDownwardMessage> = Vec::new();

		let (relay_balance, ap_balance) = get_balances();

		Relay::execute_with(|| {
			use kusama_runtime::{RuntimeEvent, RuntimeOrigin, System};

			assert_ok!(RelayChainPalletXcm::limited_teleport_assets(
				RuntimeOrigin::signed(ALICE.into()),
				Box::new(AP_DEST.into()),
				Box::new(get_benf().into()),
				Box::new(assets),
				0,
				WeightLimit::Unlimited,
			));
			assert!(System::events().iter().any(|r| matches!(
				r.event,
				RuntimeEvent::XcmPallet(pallet_xcm::Event::Attempted(Outcome::Complete { .. }))
			)));
		});

		Statemine::execute_with(|| {
			use statemine_runtime::{Runtime, RuntimeEvent, System};
			assert!(System::events().iter().any(|r| matches!(
				&r.event,
				RuntimeEvent::Balances(pallet_balances::Event::Deposit { who, .. })
				if *who == ALICE.into()
			)));
		});

		let (relay_balance_after, ap_balance_after) = get_balances();
		assert_eq!(relay_balance - amount, relay_balance_after);
		assert!(ap_balance_after > ap_balance);
	}

	#[test]
	fn transact_sudo_relay_to_assets_para_works() {
		force_xcm_version();

		Relay::execute_with(|| {
			use kusama_runtime::{RuntimeEvent, RuntimeOrigin, System};

			let call = statemine_runtime::RuntimeCall::Assets(pallet_assets::Call::<
				statemine_runtime::Runtime,
				Instance1,
			>::force_create {
				id: 1.into(),
				is_sufficient: true,
				min_balance: 1000,
				owner: ALICE.into(),
			});
			let xcm = Xcm(vec![
				UnpaidExecution { weight_limit: WeightLimit::Unlimited, check_origin: None },
				Transact {
					require_weight_at_most: Weight::from_parts(1000000000, 200000),
					origin_kind: OriginKind::Superuser,
					call: call.encode().into(),
				},
			]);
			assert_ok!(RelayChainPalletXcm::send(
				RuntimeOrigin::root(),
				Box::new(AP_DEST.into()),
				Box::new(VersionedXcm::from(xcm)),
			));
			assert!(System::events().iter().any(|r| matches!(
				r.event,
				RuntimeEvent::XcmPallet(pallet_xcm::Event::Sent { .. })
			)));
		});

		Statemine::execute_with(|| {
			assert!(statemine_runtime::Assets::asset_exists(1));
		});
	}

	#[test]
	fn reserved_transfer_native_relay_to_assets_para_fails() {
		force_xcm_version();
		let (relay_balance, ap_balance) = get_balances();
		let amount = 1000_000_000;
		let assets: VersionedMultiAssets = (Here, amount).into();

		Relay::execute_with(|| {
			use kusama_runtime::{RuntimeEvent, RuntimeOrigin, System};

			assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
				RuntimeOrigin::signed(ALICE.into()),
				Box::new(AP_DEST.into()),
				Box::new(get_benf().into()),
				Box::new(assets),
				0,
				WeightLimit::Unlimited,
			));

			assert!(System::events().iter().any(|r| matches!(
				r.event,
				RuntimeEvent::XcmPallet(pallet_xcm::Event::Attempted(Outcome::Complete { .. }))
			)));
		});

		Statemine::execute_with(|| {
			use statemine_runtime::{RuntimeEvent, System};

			assert!(System::events().iter().any(|r| matches!(
				r.event,
				RuntimeEvent::DmpQueue(cumulus_pallet_dmp_queue::Event::ExecutedDownward {
					outcome: Outcome::Incomplete(_, Error::UntrustedReserveLocation),
					..
				})
			)));
		});

		let (relay_balance_after, ap_balance_after) = get_balances();
		assert_eq!(relay_balance - amount, relay_balance_after);
		assert_eq!(ap_balance_after, ap_balance);
	}
}
