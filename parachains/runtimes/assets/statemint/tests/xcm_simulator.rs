use frame_support::{assert_ok, dispatch::RawOrigin::Root, sp_io, sp_tracing};
use polkadot_runtime_constants::DOLLARS;
use sp_core::parameter_types;
use xcm::prelude::*;
use xcm_executor::traits::Convert;
use xcm_simulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain, TestExt};

pub const ALICE: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([0u8; 32]);
pub const INITIAL_BALANCE: u128 = 1000 * DOLLARS;

decl_test_parachain! {
	pub struct Statemint {
		Runtime = statemint_runtime::Runtime,
		XcmpMessageHandler = statemint_runtime::XcmpQueue,
		DmpMessageHandler = statemint_runtime::DmpQueue,
		new_ext = statemint_ext(),
	}
}

decl_test_parachain! {
	pub struct Penpal {
		Runtime = penpal_runtime::Runtime,
		XcmpMessageHandler = penpal_runtime::XcmpQueue,
		DmpMessageHandler = penpal_runtime::DmpQueue,
		new_ext = penpal_ext(),
	}
}

decl_test_relay_chain! {
	pub struct Polkadot {
		Runtime = polkadot_runtime::Runtime,
		XcmConfig = polkadot_runtime::xcm_config::XcmConfig,
		new_ext = relay_ext(),
	}
}

decl_test_network! {
	pub struct MockNet {
		relay_chain = Polkadot,
		parachains = vec![
			(1000, Statemint),
			(2000, Penpal),
		],
	}
}

// Define Statemint TestExternalities.
pub fn statemint_ext() -> sp_io::TestExternalities {
	use statemint_runtime::{Runtime, System};

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

// Define Polkadot TestExternalities.
pub fn relay_ext() -> sp_io::TestExternalities {
	use polkadot_runtime::{Runtime, RuntimeOrigin, System};

	let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

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
	statemint_runtime::xcm_config::LocationToAccountId::convert(location.into()).unwrap()
}

pub fn child_account_id(para: u32) -> polkadot_core_primitives::AccountId {
	let location = (Parachain(para),);
	polkadot_runtime::xcm_config::SovereignAccountOf::convert(location.into()).unwrap()
}

pub type RelayChainPalletXcm = pallet_xcm::Pallet<polkadot_runtime::Runtime>;
pub type StatemintPalletXcm = pallet_xcm::Pallet<statemint_runtime::Runtime>;
pub type PenpalPalletXcm = pallet_xcm::Pallet<penpal_runtime::Runtime>;

parameter_types! {
	pub StatemintLocation: MultiLocation = (Ancestor(0), Parachain(1000)).into();
}

#[test]
fn force_xcm_version() {
	let xcm_version = 3;
	Polkadot::execute_with(|| {
		use polkadot_runtime::{RuntimeEvent, System};

		let statemint_location: MultiLocation = (Ancestor(0), Parachain(1000)).into();
		let penpal_location: MultiLocation = (Ancestor(0), Parachain(2000)).into();

		// Check that we can force xcm version for Statemint and Penpal from Polkadot.
		for location in [statemint_location, penpal_location] {
			assert_ok!(RelayChainPalletXcm::force_xcm_version(
				polkadot_runtime::RuntimeOrigin::root(),
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

	// Penpal forces Polkadot xcm version.
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
	use crate::{Polkadot, RelayChainPalletXcm, ALICE};
	use frame_support::assert_ok;
	use parachains_common::Balance;
	use polkadot_runtime::{RuntimeEvent, RuntimeOrigin, System};
	use xcm::{
		latest::{Ancestor, MultiLocation},
		prelude::{AccountId32, Here, Parachain, WeightLimit},
		v3::Outcome,
		VersionedMultiAssets, VersionedMultiLocation,
	};
	use xcm_simulator::TestExt;

	fn get_balances() -> (Balance, Balance) {
		let relay_balance = polkadot_runtime::System::account(ALICE);
		let assets_para_balance = statemint_runtime::System::account(ALICE);

		(relay_balance.data.free, assets_para_balance.data.free)
	}

	#[test]
	fn teleport_native_assets_relay_to_assets_para() {
		let ap_dest: VersionedMultiLocation = (Ancestor(0), Parachain(1000)).into();
		println!("{:?}", ap_dest);
		let amount = 1000;
		let assets: VersionedMultiAssets = (Here, amount).into();
		println!("{:?}", assets);

		Polkadot::execute_with(|| {
			let (relay_balance, ap_balance) = get_balances();

			assert_ok!(RelayChainPalletXcm::limited_teleport_assets(
				RuntimeOrigin::signed(ALICE),
				Box::new(ap_dest),
				Box::new(AccountId32 { network: None, id: ALICE.into() }.into()),
				Box::new(assets),
				0,
				WeightLimit::Unlimited,
			));
			System::events().iter().for_each(|ev| println!("{:?}", ev));
			assert!(System::events().iter().any(|r| matches!(
				r.event,
				RuntimeEvent::XcmPallet(pallet_xcm::Event::Attempted(Outcome::Complete { .. }))
			)));
			assert!(statemint_runtime::System::events().iter().any(|r| matches!(
				r.event,
				statemint_runtime::RuntimeEvent::DmpQueue(
					cumulus_pallet_dmp_queue::Event::ExecutedDownward { .. }
				)
			)));
		});
	}
}
