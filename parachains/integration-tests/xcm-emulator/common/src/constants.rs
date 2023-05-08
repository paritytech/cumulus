pub use polkadot_runtime_parachains::configuration::HostConfiguration;
use polkadot_primitives::{AccountPublic, AssignmentId, ValidatorId};
pub use parachains_common::{BlockNumber, AccountId, Balance, AuraId, StatemintAuraId};
use cumulus_primitives_core::ParaId;
pub use cumulus_test_service::{get_account_id_from_seed, get_from_seed};
pub use xcm;
use grandpa::AuthorityId as GrandpaId;
use sp_core::{crypto::UncheckedInto, sr25519, storage::Storage};
use sp_runtime::{Perbill, BuildStorage};
use sp_consensus_babe::AuthorityId as BabeId;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use polkadot_service::chain_spec::get_authority_keys_from_seed_no_beefy;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;

pub mod accounts {
	use super::*;
	pub const ALICE: &str = "Alice";
	pub const BOB: &str = "Bob";
	pub const CHARLIE: &str = "Charlie";
	pub const DAVE: &str = "Dave";
	pub const EVE: &str = "Eve";
	pub const FERDIE: &str = "Ferdeir";
	pub const ALICE_STASH: &str = "Alice//stash";
	pub const BOB_STASH: &str = "Bob//stash";
	pub const CHARLIE_STASH: &str = "Charlie//stash";
	pub const DAVE_STASH: &str = "Dave//stash";
	pub const EVE_STASH: &str = "Eve//stash";
	pub const FERDIE_STASH: &str = "Ferdie//stash";

	pub fn init_balances() -> Vec<AccountId> {
		vec![
			get_account_id_from_seed::<sr25519::Public>(ALICE),
			get_account_id_from_seed::<sr25519::Public>(BOB),
			get_account_id_from_seed::<sr25519::Public>(CHARLIE),
			get_account_id_from_seed::<sr25519::Public>(DAVE),
			get_account_id_from_seed::<sr25519::Public>(EVE),
			get_account_id_from_seed::<sr25519::Public>(FERDIE),
			get_account_id_from_seed::<sr25519::Public>(ALICE_STASH),
			get_account_id_from_seed::<sr25519::Public>(BOB_STASH),
			get_account_id_from_seed::<sr25519::Public>(CHARLIE_STASH),
			get_account_id_from_seed::<sr25519::Public>(DAVE_STASH),
			get_account_id_from_seed::<sr25519::Public>(EVE_STASH),
			get_account_id_from_seed::<sr25519::Public>(FERDIE_STASH),
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

pub mod validators {
	use super::*;

	pub fn initial_authorities() -> Vec<(
		AccountId,
		AccountId,
		BabeId,
		GrandpaId,
		ImOnlineId,
		ValidatorId,
		AssignmentId,
		AuthorityDiscoveryId,
	)> {
		vec![get_authority_keys_from_seed_no_beefy("Alice")]
	}
}

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

// Polkadot
pub mod polkadot {
	use super::*;
	pub const ED: Balance = polkadot_runtime_constants::currency::EXISTENTIAL_DEPOSIT;
	const STASH: u128 = 100 * polkadot_runtime_constants::currency::UNITS;

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

	fn session_keys(
		babe: BabeId,
		grandpa: GrandpaId,
		im_online: ImOnlineId,
		para_validator: ValidatorId,
		para_assignment: AssignmentId,
		authority_discovery: AuthorityDiscoveryId,
	) -> polkadot_runtime::SessionKeys {
		polkadot_runtime::SessionKeys {
			babe,
			grandpa,
			im_online,
			para_validator,
			para_assignment,
			authority_discovery,
		}
	}

	pub fn genesis() -> Storage {
		let genesis_config = polkadot_runtime::GenesisConfig {
			system: polkadot_runtime::SystemConfig { code: polkadot_runtime::WASM_BINARY.unwrap().to_vec() },
			balances: polkadot_runtime::BalancesConfig {
				balances: accounts::init_balances().iter().cloned().map(|k| (k, ED * 4096)).collect(),
			},
			indices: polkadot_runtime::IndicesConfig { indices: vec![] },
			session: polkadot_runtime::SessionConfig {
				keys: validators::initial_authorities()
					.iter()
					.map(|x| {
						(
							x.0.clone(),
							x.0.clone(),
							polkadot::session_keys(
								x.2.clone(),
								x.3.clone(),
								x.4.clone(),
								x.5.clone(),
								x.6.clone(),
								x.7.clone(),
							),
						)
					})
					.collect::<Vec<_>>(),
			},
			staking: polkadot_runtime::StakingConfig {
				validator_count: validators::initial_authorities().len() as u32,
				minimum_validator_count: 1,
				stakers: validators::initial_authorities()
					.iter()
					.map(|x| (x.0.clone(), x.1.clone(), STASH, polkadot_runtime::StakerStatus::Validator))
					.collect(),
				invulnerables: validators::initial_authorities().iter().map(|x| x.0.clone()).collect(),
				force_era: pallet_staking::Forcing::ForceNone,
				slash_reward_fraction: Perbill::from_percent(10),
				..Default::default()
			},
			phragmen_election: Default::default(),
			democracy: Default::default(),
			council: polkadot_runtime::CouncilConfig { members: vec![], phantom: Default::default() },
			technical_committee: polkadot_runtime::TechnicalCommitteeConfig {
				members: vec![],
				phantom: Default::default(),
			},
			technical_membership: Default::default(),
			babe: polkadot_runtime::BabeConfig {
				authorities: Default::default(),
				epoch_config: Some(polkadot_runtime::BABE_GENESIS_EPOCH_CONFIG),
			},
			grandpa: Default::default(),
			im_online: Default::default(),
			authority_discovery: polkadot_runtime::AuthorityDiscoveryConfig { keys: vec![] },
			claims: polkadot_runtime::ClaimsConfig { claims: vec![], vesting: vec![] },
			vesting: polkadot_runtime::VestingConfig { vesting: vec![] },
			treasury: Default::default(),
			hrmp: Default::default(),
			configuration: polkadot_runtime::ConfigurationConfig {
				config: get_host_config(),
			},
			paras: Default::default(),
			xcm_pallet: Default::default(),
			nomination_pools: Default::default(),
		};

		genesis_config.build_storage().unwrap()
	}
}

// Kusama
pub mod kusama {
	use super::*;
	pub const ED: Balance = kusama_runtime_constants::currency::EXISTENTIAL_DEPOSIT;
	const STASH: u128 = 100 * kusama_runtime_constants::currency::UNITS;

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

	fn session_keys(
		babe: BabeId,
		grandpa: GrandpaId,
		im_online: ImOnlineId,
		para_validator: ValidatorId,
		para_assignment: AssignmentId,
		authority_discovery: AuthorityDiscoveryId,
	) -> kusama_runtime::SessionKeys {
		kusama_runtime::SessionKeys {
			babe,
			grandpa,
			im_online,
			para_validator,
			para_assignment,
			authority_discovery,
		}
	}

	pub fn genesis() -> Storage {
		let genesis_config = kusama_runtime::GenesisConfig {
			system: kusama_runtime::SystemConfig { code: kusama_runtime::WASM_BINARY.unwrap().to_vec() },
			balances: kusama_runtime::BalancesConfig {
				balances: accounts::init_balances().iter().cloned().map(|k| (k, ED * 4096)).collect(),
			},
			indices: kusama_runtime::IndicesConfig { indices: vec![] },
			session: kusama_runtime::SessionConfig {
				keys: validators::initial_authorities()
					.iter()
					.map(|x| {
						(
							x.0.clone(),
							x.0.clone(),
							kusama::session_keys(
								x.2.clone(),
								x.3.clone(),
								x.4.clone(),
								x.5.clone(),
								x.6.clone(),
								x.7.clone(),
							),
						)
					})
					.collect::<Vec<_>>(),
			},
			staking: kusama_runtime::StakingConfig {
				minimum_validator_count: 1,
				validator_count: validators::initial_authorities().len() as u32,
				stakers: validators::initial_authorities()
					.iter()
					.map(|x| (x.0.clone(), x.1.clone(), STASH, kusama_runtime::StakerStatus::Validator))
					.collect(),
				invulnerables: validators::initial_authorities().iter().map(|x| x.0.clone()).collect(),
				force_era: pallet_staking::Forcing::NotForcing,
				slash_reward_fraction: Perbill::from_percent(10),
				..Default::default()
			},
			babe: kusama_runtime::BabeConfig {
				authorities: Default::default(),
				epoch_config: Some(kusama_runtime::BABE_GENESIS_EPOCH_CONFIG),
			},
			grandpa: Default::default(),
			im_online: Default::default(),
			authority_discovery: kusama_runtime::AuthorityDiscoveryConfig { keys: vec![] },
			claims: kusama_runtime::ClaimsConfig { claims: vec![], vesting: vec![] },
			vesting: kusama_runtime::VestingConfig { vesting: vec![] },
			treasury: Default::default(),
			hrmp: Default::default(),
			configuration: kusama_runtime::ConfigurationConfig {
				config: get_host_config(),
			},
			paras: Default::default(),
			xcm_pallet: Default::default(),
			nomination_pools: Default::default(),
			nis_counterpart_balances: Default::default(),
		};

		genesis_config.build_storage().unwrap()
	}
}

// Statemint
pub mod statemint {
	use super::*;
	pub const PARA_ID: u32 = 1000;
	pub const ED: Balance = statemint_runtime::constants::currency::EXISTENTIAL_DEPOSIT;

	pub fn genesis() -> Storage {
		let genesis_config = statemint_runtime::GenesisConfig {
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
		};

		genesis_config.build_storage().unwrap()
	}
}

// Statemint
pub mod statemine {
	use super::*;
	pub const PARA_ID: u32 = 1000;
	pub const ED: Balance = statemine_runtime::constants::currency::EXISTENTIAL_DEPOSIT;

	pub fn genesis() -> Storage {
		let genesis_config = statemine_runtime::GenesisConfig {
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
		};

		genesis_config.build_storage().unwrap()
	}
}

// Penpal
pub mod penpal {
	use super::*;
	pub const PARA_ID: u32 = 2000;
	pub const ED: Balance = penpal_runtime::EXISTENTIAL_DEPOSIT;

	pub fn genesis(para_id: u32) -> Storage {
		let genesis_config = penpal_runtime::GenesisConfig {
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
		};

		genesis_config.build_storage().unwrap()
	}
}
