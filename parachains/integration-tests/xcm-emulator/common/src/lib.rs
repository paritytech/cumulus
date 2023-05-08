pub mod constants;

use frame_support::{
	sp_io, sp_tracing,
	parameter_types
};
use xcm::prelude::*;
use xcm_emulator::{decl_test_networks, decl_test_parachains, decl_test_relay_chains, TestExt, RelayChain, Parachain};
use xcm_executor::traits::Convert;
use statemint_runtime::constants::currency::DOLLARS;
pub use constants::{polkadot, kusama, statemint, statemine, penpal, accounts::{ALICE, BOB}};
pub use sp_core::{Get, sr25519, storage::Storage};
pub use parachains_common::{BlockNumber, AccountId, Balance, AuraId, StatemintAuraId};

decl_test_relay_chains! {
	pub struct Polkadot {
		Runtime = polkadot_runtime::Runtime,
		RuntimeOrigin = polkadot_runtime::RuntimeOrigin,
		RuntimeEvent = polkadot_runtime::RuntimeEvent,
		XcmConfig = polkadot_runtime::xcm_config::XcmConfig,
		System = polkadot_runtime::System,
		XcmPallet = polkadot_runtime::XcmPallet,
		Balances = polkadot_runtime::Balances,
		SovereignAccountOf = polkadot_runtime::xcm_config::SovereignAccountOf,
		genesis = polkadot::genesis(),
		on_init = (),
	},
	pub struct Kusama {
		Runtime = kusama_runtime::Runtime,
		RuntimeOrigin = kusama_runtime::RuntimeOrigin,
		RuntimeEvent = kusama_runtime::RuntimeEvent,
		XcmConfig = kusama_runtime::xcm_config::XcmConfig,
		System = kusama_runtime::System,
		XcmPallet = kusama_runtime::XcmPallet,
		Balances = kusama_runtime::Balances,
		SovereignAccountOf = kusama_runtime::xcm_config::SovereignAccountOf,
		genesis = kusama::genesis(),
		on_init = (),
	}
}

decl_test_parachains! {
	// Polkadot
	pub struct Statemint {
		Runtime = statemint_runtime::Runtime,
		RuntimeOrigin = statemint_runtime::RuntimeOrigin,
		RuntimeEvent = statemint_runtime::RuntimeEvent,
		XcmpMessageHandler = statemint_runtime::XcmpQueue,
		DmpMessageHandler = statemint_runtime::DmpQueue,
		System = statemint_runtime::System,
		ParachainSystem = statemint_runtime::ParachainSystem,
		ParachainInfo = statemint_runtime::ParachainInfo,
		XcmPallet = statemint_runtime::PolkadotXcm,
		Balances = statemint_runtime::Balances,
		LocationToAccountId = statemint_runtime::xcm_config::LocationToAccountId,
		genesis = statemint::genesis(),
		on_init = (),
	},
	pub struct PenpalPolkadot {
		Runtime = penpal_runtime::Runtime,
		RuntimeOrigin = penpal_runtime::RuntimeOrigin,
		RuntimeEvent = penpal_runtime::RuntimeEvent,
		XcmpMessageHandler = penpal_runtime::XcmpQueue,
		DmpMessageHandler = penpal_runtime::DmpQueue,
		System = penpal_runtime::System,
		ParachainSystem = penpal_runtime::ParachainSystem,
		ParachainInfo = penpal_runtime::ParachainInfo,
		XcmPallet = penpal_runtime::PolkadotXcm,
		Balances = penpal_runtime::Balances,
		LocationToAccountId = penpal_runtime::xcm_config::LocationToAccountId,
		genesis = penpal::genesis(penpal::PARA_ID),
		on_init = (),
	},
	// Kusama
	pub struct Statemine {
		Runtime = statemine_runtime::Runtime,
		RuntimeOrigin = statemine_runtime::RuntimeOrigin,
		RuntimeEvent = statemine_runtime::RuntimeEvent,
		XcmpMessageHandler = statemine_runtime::XcmpQueue,
		DmpMessageHandler = statemine_runtime::DmpQueue,
		System = statemine_runtime::System,
		ParachainSystem = statemine_runtime::ParachainSystem,
		ParachainInfo = statemine_runtime::ParachainInfo,
		XcmPallet = statemine_runtime::PolkadotXcm,
		Balances = statemine_runtime::Balances,
		LocationToAccountId = statemine_runtime::xcm_config::LocationToAccountId,
		genesis = statemine::genesis(),
		on_init = (),
	},
	pub struct PenpalKusama {
		Runtime = penpal_runtime::Runtime,
		RuntimeOrigin = penpal_runtime::RuntimeOrigin,
		RuntimeEvent = penpal_runtime::RuntimeEvent,
		XcmpMessageHandler = penpal_runtime::XcmpQueue,
		DmpMessageHandler = penpal_runtime::DmpQueue,
		System = penpal_runtime::System,
		ParachainSystem = penpal_runtime::ParachainSystem,
		ParachainInfo = penpal_runtime::ParachainInfo,
		XcmPallet = penpal_runtime::PolkadotXcm,
		Balances = penpal_runtime::Balances,
		LocationToAccountId = penpal_runtime::xcm_config::LocationToAccountId,
		genesis = penpal::genesis(penpal::PARA_ID),
		on_init = (),
	}
}

decl_test_networks! {
	pub struct PolkadotMockNet {
		relay_chain = Polkadot,
		parachains = vec![
			Statemint,
			PenpalPolkadot,
		],
	},
	pub struct KusamaMockNet {
		relay_chain = Kusama,
		parachains = vec![
			Statemine,
			PenpalKusama,
		],
	}
}

parameter_types! {
	// Polkadot
	pub PolkadotSender: AccountId = Polkadot::account_id_of(ALICE);
	pub PolkadotReceiver: AccountId = Polkadot::account_id_of(BOB);
	// Kusama
	pub KusamaSender: AccountId = Kusama::account_id_of(ALICE);
	pub KusamaReceiver: AccountId = Kusama::account_id_of(BOB);
	// Statemint
	pub StatemintSender: AccountId = Statemint::account_id_of(ALICE);
	pub StatemintReceiver: AccountId = Statemint::account_id_of(BOB);
	// Statemine
	pub StatemineSender: AccountId = Statemine::account_id_of(ALICE);
	pub StatemineReceiver: AccountId = Statemine::account_id_of(BOB);
	// Penpal Polkadot
	pub PenpalPolkadotSender: AccountId = PenpalPolkadot::account_id_of(ALICE);
	pub PenpalPolkadotReceiver: AccountId = PenpalPolkadot::account_id_of(BOB);
	// Penpal Kusama
	pub PenpalKusamaSender: AccountId = PenpalKusama::account_id_of(ALICE);
	pub PenpalKusamaReceiver: AccountId = PenpalKusama::account_id_of(BOB);

	pub StatemintLocation: MultiLocation = (Ancestor(0), Parachain(1000)).into();
}


pub const INITIAL_BALANCE: u128 = 1000 * DOLLARS;

pub fn parent_account_id() -> parachains_common::AccountId {
	let location = (Parent,);
	statemint_runtime::xcm_config::LocationToAccountId::convert(location.into()).unwrap()
}

pub fn child_account_id(para: u32) -> polkadot_core_primitives::AccountId {
	let location = (Parachain(para),);
	polkadot_runtime::xcm_config::SovereignAccountOf::convert(location.into()).unwrap()
}

pub mod helpers {
	use super::*;
	// pub fn account_id_of(seed: &str) -> AccountId {
	// 	get_account_id_from_seed::<sr25519::Public>(seed)
	// }

	// pub fn fund_accounts(accounts: (AccountId, Balance)) {

	// }
}
