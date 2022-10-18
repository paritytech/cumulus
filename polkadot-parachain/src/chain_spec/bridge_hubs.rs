// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Cumulus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

use crate::chain_spec::{get_account_id_from_seed, get_collator_keys_from_seed};
use cumulus_primitives_core::ParaId;
use sc_chain_spec::{ChainSpec, ChainType};
use sc_cli::RuntimeVersion;
use sp_core::sr25519;
use std::{path::PathBuf, str::FromStr};

/// Collects all supported BridgeHub configurations
#[derive(Debug, PartialEq)]
pub enum BridgeHubRuntimeType {
	Rococo { default_config: bool },
	RococoLocal,
	Wococo { default_config: bool },
	WococoLocal,
}

impl FromStr for BridgeHubRuntimeType {
	type Err = String;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		match value {
			rococo::BRIDGE_HUB_ROCOCO => Ok(BridgeHubRuntimeType::Rococo { default_config: false }),
			rococo::BRIDGE_HUB_ROCOCO_DEFAULT =>
				Ok(BridgeHubRuntimeType::Rococo { default_config: true }),
			rococo::BRIDGE_HUB_ROCOCO_LOCAL => Ok(BridgeHubRuntimeType::RococoLocal),
			wococo::BRIDGE_HUB_WOCOCO => Ok(BridgeHubRuntimeType::Wococo { default_config: false }),
			wococo::BRIDGE_HUB_WOCOCO_DEFAULT =>
				Ok(BridgeHubRuntimeType::Wococo { default_config: true }),
			wococo::BRIDGE_HUB_WOCOCO_LOCAL => Ok(BridgeHubRuntimeType::WococoLocal),
			_ => Err(format!("Value '{}' is not configured yet", value)),
		}
	}
}

impl BridgeHubRuntimeType {
	pub const ID_PREFIX: &'static str = "bridge-hub";

	pub fn chain_spec_from_json_file(&self, path: PathBuf) -> Result<Box<dyn ChainSpec>, String> {
		Ok(Box::new(match self {
			BridgeHubRuntimeType::Rococo { .. } =>
				rococo::BridgeHubChainSpec::from_json_file(path)?,
			BridgeHubRuntimeType::RococoLocal => rococo::BridgeHubChainSpec::from_json_file(path)?,
			BridgeHubRuntimeType::Wococo { .. } =>
				wococo::BridgeHubChainSpec::from_json_file(path)?,
			BridgeHubRuntimeType::WococoLocal => wococo::BridgeHubChainSpec::from_json_file(path)?,
		}))
	}

	pub fn load_config(&self) -> Result<Box<dyn ChainSpec>, String> {
		match self {
			BridgeHubRuntimeType::Rococo { default_config } =>
				if *default_config {
					Ok(Box::new(rococo::default_config(
						rococo::BRIDGE_HUB_ROCOCO,
						"Rococo BridgeHub",
						ChainType::Live,
						"rococo",
						ParaId::new(1013),
						None,
						None,
					)))
				} else {
					Ok(Box::new(rococo::BridgeHubChainSpec::from_json_bytes(
						&include_bytes!("../../../parachains/chain-specs/bridge-hub-rococo.json")[..],
					)?))
				},
			BridgeHubRuntimeType::RococoLocal => Ok(Box::new(rococo::default_config(
				rococo::BRIDGE_HUB_ROCOCO_LOCAL,
				"Rococo BridgeHub Local",
				ChainType::Local,
				"rococo-local",
				ParaId::new(1013),
				Some("Alice".to_string()),
				Some("Bob".to_string()),
			))),
			BridgeHubRuntimeType::Wococo { default_config } =>
				if *default_config {
					Ok(Box::new(wococo::default_config(
						wococo::BRIDGE_HUB_WOCOCO,
						"Wococo BridgeHub",
						ChainType::Live,
						"wococo",
						ParaId::new(1013),
						None,
						None,
					)))
				} else {
					Ok(Box::new(rococo::BridgeHubChainSpec::from_json_bytes(
						&include_bytes!("../../../parachains/chain-specs/bridge-hub-wococo.json")[..],
					)?))
				},
			BridgeHubRuntimeType::WococoLocal => Ok(Box::new(wococo::default_config(
				wococo::BRIDGE_HUB_WOCOCO_LOCAL,
				"Wococo BridgeHub Local",
				ChainType::Local,
				"wococo-local",
				ParaId::new(1013),
				Some("Alice".to_string()),
				Some("Bob".to_string()),
			))),
		}
	}

	pub fn runtime_version(&self) -> &'static RuntimeVersion {
		match self {
			BridgeHubRuntimeType::Rococo { .. } |
			BridgeHubRuntimeType::Wococo { .. } |
			BridgeHubRuntimeType::RococoLocal |
			BridgeHubRuntimeType::WococoLocal => {
				// this is intentional, for Rococo/Wococo we just want to have one runtime, which is configured for both sides
				&bridge_hub_rococo_runtime::VERSION
			},
		}
	}
}

/// Check if 'id' satisfy BridgeHub-like format
fn ensure_id(id: &str) -> Result<&str, String> {
	if id.starts_with(BridgeHubRuntimeType::ID_PREFIX) {
		Ok(id)
	} else {
		Err(format!(
			"Invalid 'id' attribute ({}), should start with prefix: {}",
			id,
			BridgeHubRuntimeType::ID_PREFIX
		))
	}
}

/// Sub-module for Rococo setup
pub mod rococo {
	use super::{get_account_id_from_seed, get_collator_keys_from_seed, sr25519, ParaId};
	use crate::chain_spec::{Extensions, SAFE_XCM_VERSION};
	use parachains_common::{AccountId, AuraId};
	use sc_chain_spec::ChainType;

	pub(crate) const BRIDGE_HUB_ROCOCO: &str = "bridge-hub-rococo";
	pub(crate) const BRIDGE_HUB_ROCOCO_DEFAULT: &str = "bridge-hub-rococo-default";
	pub(crate) const BRIDGE_HUB_ROCOCO_LOCAL: &str = "bridge-hub-rococo-local";

	/// Specialized `ChainSpec` for the normal parachain runtime.
	pub type BridgeHubChainSpec =
		sc_service::GenericChainSpec<bridge_hub_rococo_runtime::GenesisConfig, Extensions>;

	pub type RuntimeApi = bridge_hub_rococo_runtime::RuntimeApi;

	pub fn default_config(
		id: &str,
		chain_name: &str,
		chain_type: ChainType,
		relay_chain: &str,
		para_id: ParaId,
		root_key_seed: Option<String>,
		bridges_pallet_owner_seed: Option<String>,
	) -> BridgeHubChainSpec {
		let properties = sc_chain_spec::Properties::new();
		// TODO: check
		// properties.insert("ss58Format".into(), 2.into());
		// properties.insert("tokenSymbol".into(), "ROC".into());
		// properties.insert("tokenDecimals".into(), 12.into());

		BridgeHubChainSpec::from_genesis(
			// Name
			chain_name,
			// ID
			super::ensure_id(id).expect("invalid id"),
			chain_type,
			move || {
				genesis(
					// initial collators.
					vec![
						(
							get_account_id_from_seed::<sr25519::Public>("Alice"),
							get_collator_keys_from_seed::<AuraId>("Alice"),
						),
						(
							get_account_id_from_seed::<sr25519::Public>("Bob"),
							get_collator_keys_from_seed::<AuraId>("Bob"),
						),
					],
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
					],
					para_id,
					root_key_seed
						.as_ref()
						.map(|seed| get_account_id_from_seed::<sr25519::Public>(&seed)),
					bridges_pallet_owner_seed
						.as_ref()
						.map(|seed| get_account_id_from_seed::<sr25519::Public>(&seed)),
				)
			},
			Vec::new(),
			None,
			None,
			None,
			Some(properties),
			Extensions { relay_chain: relay_chain.to_string(), para_id: para_id.into() },
		)
	}

	fn genesis(
		invulnerables: Vec<(AccountId, AuraId)>,
		endowed_accounts: Vec<AccountId>,
		id: ParaId,
		root_key: Option<AccountId>,
		bridges_pallet_owner: Option<AccountId>,
	) -> bridge_hub_rococo_runtime::GenesisConfig {
		bridge_hub_rococo_runtime::GenesisConfig {
			system: bridge_hub_rococo_runtime::SystemConfig {
				code: bridge_hub_rococo_runtime::WASM_BINARY
					.expect("WASM binary was not build, please build it!")
					.to_vec(),
			},
			balances: bridge_hub_rococo_runtime::BalancesConfig {
				balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
			},
			parachain_info: bridge_hub_rococo_runtime::ParachainInfoConfig { parachain_id: id },
			collator_selection: bridge_hub_rococo_runtime::CollatorSelectionConfig {
				invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
				// TODO: check
				candidacy_bond: 10,
				..Default::default()
			},
			session: bridge_hub_rococo_runtime::SessionConfig {
				keys: invulnerables
					.into_iter()
					.map(|(acc, aura)| {
						(
							acc.clone(),                                     // account id
							acc,                                             // validator id
							bridge_hub_rococo_runtime::SessionKeys { aura }, // session keys
						)
					})
					.collect(),
			},
			aura: Default::default(),
			aura_ext: Default::default(),
			parachain_system: Default::default(),
			polkadot_xcm: bridge_hub_rococo_runtime::PolkadotXcmConfig {
				safe_xcm_version: Some(SAFE_XCM_VERSION),
			},
			// TODO: when go live, check it: https://github.com/paritytech/parity-bridges-common/issues/1551
			sudo: bridge_hub_rococo_runtime::SudoConfig { key: root_key },
			bridge_wococo_grandpa: bridge_hub_rococo_runtime::BridgeWococoGrandpaConfig {
				owner: bridges_pallet_owner.clone(),
				..Default::default()
			},
			bridge_rococo_grandpa: bridge_hub_rococo_runtime::BridgeRococoGrandpaConfig {
				owner: bridges_pallet_owner,
				..Default::default()
			},
		}
	}
}

/// Sub-module for Wococo setup (reuses stuff from Rococo)
pub mod wococo {
	use super::ParaId;
	use crate::chain_spec::bridge_hubs::rococo;
	use sc_chain_spec::ChainType;

	pub(crate) const BRIDGE_HUB_WOCOCO: &str = "bridge-hub-wococo";
	pub(crate) const BRIDGE_HUB_WOCOCO_DEFAULT: &str = "bridge-hub-wococo-default";
	pub(crate) const BRIDGE_HUB_WOCOCO_LOCAL: &str = "bridge-hub-wococo-local";

	pub type BridgeHubChainSpec = rococo::BridgeHubChainSpec;
	pub type RuntimeApi = rococo::RuntimeApi;

	pub fn default_config(
		id: &str,
		chain_name: &str,
		chain_type: ChainType,
		relay_chain: &str,
		para_id: ParaId,
		root_key_seed: Option<String>,
		bridges_pallet_owner_seed: Option<String>,
	) -> BridgeHubChainSpec {
		rococo::default_config(
			id,
			chain_name,
			chain_type,
			relay_chain,
			para_id,
			root_key_seed,
			bridges_pallet_owner_seed,
		)
	}
}
