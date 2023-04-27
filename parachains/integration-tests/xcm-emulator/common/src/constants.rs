pub use polkadot_runtime_parachains::configuration::HostConfiguration;
pub use parachains_common::{BlockNumber, AccountId, Balance, AuraId, StatemintAuraId};
use cumulus_primitives_core::ParaId;
pub use cumulus_test_service::{get_account_id_from_seed, get_from_seed};
pub use xcm;
use sp_core::{crypto::UncheckedInto, sr25519};

pub mod accounts {
	use super::*;
	pub const ALICE: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([0u8; 32]);
	pub const BOB: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([1u8; 32]);

	pub fn init_balances() -> Vec<AccountId> {
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
			get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
			get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
		]
	}
}

pub mod collators {
	use super::*;

	pub fn invulnerables_statemint() -> Vec<(AccountId, StatemintAuraId)> {
		vec![
			(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_from_seed::<StatemintAuraId>("Alice"),
			),
			(
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_from_seed::<StatemintAuraId>("Bob"),
			),
		]
	}

	pub fn invulnerables() -> Vec<(AccountId, AuraId)> {
		vec![
			(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_from_seed::<AuraId>("Alice"),
			),
			(
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_from_seed::<AuraId>("Bob"),
			),
		]
	}
}

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

// Polkadot
pub mod polkadot {
	use super::*;

	pub fn get_host_config() -> HostConfiguration<BlockNumber> {
		HostConfiguration {
			max_upward_queue_count: 10,
			max_upward_queue_size: 51200,
			max_upward_message_size: 51200,
			max_upward_message_num_per_candidate: 10,
			max_downward_message_size: 51200,
			..Default::default()
		}
	}
}

// Kusama
pub mod kusama {
	use super::*;

	pub fn get_host_config() -> HostConfiguration<BlockNumber> {
		HostConfiguration {
			max_upward_queue_count: 10,
			max_upward_queue_size: 51200,
			max_upward_message_size: 51200,
			max_upward_message_num_per_candidate: 10,
			max_downward_message_size: 51200,
			..Default::default()
		}
	}
}

// Statemint
pub mod statemint {
	use super::*;
	pub const PARA_ID: u32 = 1000;
	pub const ED: Balance = statemint_runtime::constants::currency::EXISTENTIAL_DEPOSIT;

	pub fn genesis() -> statemint_runtime::GenesisConfig {
		statemint_runtime::GenesisConfig {
			system: statemint_runtime::SystemConfig {
				code: statemint_runtime::WASM_BINARY
					.expect("WASM binary was not build, please build it!")
					.to_vec(),
			},
			balances: statemint_runtime::BalancesConfig {
				balances: accounts::init_balances().iter().cloned().map(|k| (k, ED * 4096)).collect(),
			},
			parachain_info: statemint_runtime::ParachainInfoConfig { parachain_id: PARA_ID.into() },
			collator_selection: statemint_runtime::CollatorSelectionConfig {
				invulnerables: collators::invulnerables_statemint().iter().cloned().map(|(acc, _)| acc).collect(),
				candidacy_bond: ED * 16,
				..Default::default()
			},
			session: statemint_runtime::SessionConfig {
				keys: collators::invulnerables_statemint()
					.into_iter()
					.map(|(acc, aura)| {
						(
							acc.clone(),                  // account id
							acc,                          // validator id
							statemint_runtime::SessionKeys { aura }, // session keys
						)
					})
					.collect(),
			},
			aura: Default::default(),
			aura_ext: Default::default(),
			parachain_system: Default::default(),
			polkadot_xcm: statemint_runtime::PolkadotXcmConfig {
				safe_xcm_version: Some(SAFE_XCM_VERSION),
			},
		}
	}
}

// Statemint
pub mod statemine {
	use super::*;
	pub const PARA_ID: u32 = 1000;
	pub const ED: Balance = statemine_runtime::constants::currency::EXISTENTIAL_DEPOSIT;

	pub fn genesis() -> statemine_runtime::GenesisConfig {
		statemine_runtime::GenesisConfig {
			system: statemine_runtime::SystemConfig {
				code: statemine_runtime::WASM_BINARY
					.expect("WASM binary was not build, please build it!")
					.to_vec(),
			},
			balances: statemine_runtime::BalancesConfig {
				balances: accounts::init_balances().iter().cloned().map(|k| (k, ED * 4096)).collect(),
			},
			parachain_info: statemine_runtime::ParachainInfoConfig { parachain_id: PARA_ID.into() },
			collator_selection: statemine_runtime::CollatorSelectionConfig {
				invulnerables: collators::invulnerables().iter().cloned().map(|(acc, _)| acc).collect(),
				candidacy_bond: ED * 16,
				..Default::default()
			},
			session: statemine_runtime::SessionConfig {
				keys: collators::invulnerables()
					.into_iter()
					.map(|(acc, aura)| {
						(
							acc.clone(),                  // account id
							acc,                          // validator id
							statemine_runtime::SessionKeys { aura }, // session keys
						)
					})
					.collect(),
			},
			aura: Default::default(),
			aura_ext: Default::default(),
			parachain_system: Default::default(),
			polkadot_xcm: statemine_runtime::PolkadotXcmConfig {
				safe_xcm_version: Some(SAFE_XCM_VERSION),
			},
		}
	}
}

// Penpal
pub mod penpal {
	use super::*;
	pub const PARA_ID: u32 = 2000;
	pub const ED: Balance = penpal_runtime::EXISTENTIAL_DEPOSIT;

	pub fn genesis(para_id: u32) -> penpal_runtime::GenesisConfig {
		penpal_runtime::GenesisConfig {
			system: penpal_runtime::SystemConfig {
				code: penpal_runtime::WASM_BINARY
					.expect("WASM binary was not build, please build it!")
					.to_vec(),
			},
			balances: penpal_runtime::BalancesConfig {
				balances: accounts::init_balances().iter().cloned().map(|k| (k, ED * 4096)).collect(),
			},
			parachain_info: penpal_runtime::ParachainInfoConfig { parachain_id: para_id.into() },
			collator_selection: penpal_runtime::CollatorSelectionConfig {
				invulnerables: collators::invulnerables().iter().cloned().map(|(acc, _)| acc).collect(),
				candidacy_bond: ED * 16,
				..Default::default()
			},
			session: penpal_runtime::SessionConfig {
				keys: collators::invulnerables()
					.into_iter()
					.map(|(acc, aura)| {
						(
							acc.clone(),                  // account id
							acc,                          // validator id
							penpal_runtime::SessionKeys { aura }, // session keys
						)
					})
					.collect(),
			},
			aura: Default::default(),
			aura_ext: Default::default(),
			parachain_system: Default::default(),
			polkadot_xcm: penpal_runtime::PolkadotXcmConfig {
				safe_xcm_version: Some(SAFE_XCM_VERSION),
			},
			sudo: penpal_runtime::SudoConfig {
				key: Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
			},
		}
	}
}
