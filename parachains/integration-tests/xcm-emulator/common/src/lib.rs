pub mod constants;

use frame_support::{
	assert_ok,
	instances::Instance1,
	pallet_prelude::Hooks,
	sp_io, sp_tracing,
	traits::{fungibles::Inspect, GenesisBuild},
};
use xcm::prelude::*;
use xcm_emulator::{decl_test_networks, decl_test_parachains, decl_test_relay_chains, TestExt, Network, Relay, Parachain};
use polkadot_runtime_parachains::configuration::HostConfiguration;
use xcm_executor::traits::Convert;
use statemint_runtime::constants::currency::DOLLARS;
pub use constants::{polkadot, kusama, statemint, statemine, penpal, accounts::{ALICE, BOB}};
use sp_runtime::BuildStorage;
pub use sp_core::{Get, storage::Storage};
use parachain_info::pallet::Pallet;

decl_test_relay_chains! {
	pub struct Polkadot {
		Runtime = polkadot_runtime::Runtime,
		XcmConfig = polkadot_runtime::xcm_config::XcmConfig,
		System = polkadot_runtime::System,
		genesis = polkadot_storage(),
		on_init = (),
	},
	pub struct Kusama {
		Runtime = kusama_runtime::Runtime,
		XcmConfig = kusama_runtime::xcm_config::XcmConfig,
		System = kusama_runtime::System,
		genesis = kusama::genesis(),
		on_init = (),
	}
}

decl_test_parachains! {
	// Polkadot
	pub struct Statemint {
		Runtime = statemint_runtime::Runtime,
		RuntimeOrigin = statemint_runtime::RuntimeOrigin,
		XcmpMessageHandler = statemint_runtime::XcmpQueue,
		DmpMessageHandler = statemint_runtime::DmpQueue,
		System = statemint_runtime::System,
		genesis = statemint_storage(),
		on_init = (),
		para_id = statemint_runtime::ParachainInfo::get(),
	},
	pub struct PenpalPolkadot {
		Runtime = penpal_runtime::Runtime,
		RuntimeOrigin = penpal_runtime::RuntimeOrigin,
		XcmpMessageHandler = penpal_runtime::XcmpQueue,
		DmpMessageHandler = penpal_runtime::DmpQueue,
		System = penpal_runtime::System,
		genesis = penpal::genesis(penpal::PARA_ID),
		on_init = (),
		para_id = penpal_runtime::ParachainInfo::get(),
	},
	// Kusama
	pub struct Statemine {
		Runtime = statemine_runtime::Runtime,
		RuntimeOrigin = statemine_runtime::RuntimeOrigin,
		XcmpMessageHandler = statemine_runtime::XcmpQueue,
		DmpMessageHandler = statemine_runtime::DmpQueue,
		System = statemine_runtime::System,
		genesis = statemine::genesis(),
		on_init = (),
		para_id = statemine_runtime::ParachainInfo::get(),
	},
	pub struct PenpalKusama {
		Runtime = penpal_runtime::Runtime,
		RuntimeOrigin = penpal_runtime::RuntimeOrigin,
		XcmpMessageHandler = penpal_runtime::XcmpQueue,
		DmpMessageHandler = penpal_runtime::DmpQueue,
		System = penpal_runtime::System,
		genesis = penpal::genesis(penpal::PARA_ID),
		on_init = (),
		para_id = penpal_runtime::ParachainInfo::get(),
	}
}

decl_test_networks! {
	pub struct PolkadotMockNet {
		relay_chain = Polkadot,
		parachains = vec![
			(1000, Statemint),
			(2000, PenpalPolkadot),
		],
	},
	pub struct KusamaMockNet {
		relay_chain = Kusama,
		parachains = vec![
			(1000, Statemine),
			(2000, PenpalKusama),
		],
	}
}

// pub fn on_init() {
// 	polkadot_runtime::System::set_block_number(1);
// }


pub const INITIAL_BALANCE: u128 = 1000 * DOLLARS;

pub fn parent_account_id() -> parachains_common::AccountId {
	let location = (Parent,);
	statemint_runtime::xcm_config::LocationToAccountId::convert(location.into()).unwrap()
}

pub fn child_account_id(para: u32) -> polkadot_core_primitives::AccountId {
	let location = (Parachain(para),);
	polkadot_runtime::xcm_config::SovereignAccountOf::convert(location.into()).unwrap()
}

pub fn relay_ext() -> sp_io::TestExternalities {
	use polkadot_runtime::{Runtime, RuntimeOrigin, System};

	// <XcmConfig::XcmSender as xcm_executor::Config>::XcmSender = RelayChainXcmRouter;
	// <Runtime as pallet_xcm::Config>::XcmRouter = RelayChainXcmRouter;

	let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

	polkadot_runtime_parachains::configuration::GenesisConfig::<Runtime> {
		config: polkadot::get_host_config(),
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

pub fn polkadot_storage() -> Storage {
	use polkadot_runtime::{Runtime};
	let mut t = polkadot::genesis();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![
			(ALICE, INITIAL_BALANCE),
			(child_account_id(1000), INITIAL_BALANCE),
			(child_account_id(2000), INITIAL_BALANCE),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	t
}

pub fn statemint_storage() -> Storage {
	use statemint_runtime::{Runtime};
	let mut t = statemint::genesis();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(ALICE, INITIAL_BALANCE), (parent_account_id(), INITIAL_BALANCE)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	t
}
// Define Statemint TestExternalities.
pub fn statemint_ext() -> sp_io::TestExternalities {
	use statemint_runtime::{Runtime, System};

	let mut t = statemint::genesis().build_storage().unwrap();

	// let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

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

// mod accounts {
// 	pub const ALICE: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([0u8; 32]);
// 	pub const BOB: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([1u8; 32]);
// }
