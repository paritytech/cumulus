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

use polkadot_service::ParaId;
use sc_chain_spec::ChainSpec;
use sc_cli::RuntimeVersion;
use std::{path::PathBuf, str::FromStr};

/// Collects all supported BridgeHub configurations
pub enum BridgeHubRuntimeType {
	RococoLocal,
}

impl FromStr for BridgeHubRuntimeType {
	type Err = String;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		match value {
			rococo::BRIDGE_HUB_ROCOCO_LOCAL => Ok(BridgeHubRuntimeType::RococoLocal),
			_ => Err(format!("Value '{}' is not configured yet", value)),
		}
	}
}

impl BridgeHubRuntimeType {
	pub const ID_PREFIX: &'static str = "bridge-hub";

	pub fn chain_spec_from_json_file(&self, path: PathBuf) -> Result<Box<dyn ChainSpec>, String> {
		Ok(Box::new(match self {
			BridgeHubRuntimeType::RococoLocal => rococo::BridgeHubChainSpec::from_json_file(path)?,
		}))
	}

	pub fn load_config(&self) -> Box<dyn ChainSpec> {
		Box::new(match self {
			BridgeHubRuntimeType::RococoLocal =>
				rococo::local_config("rococo-local", ParaId::new(1013)),
		})
	}

	pub fn runtime_version(&self) -> &'static RuntimeVersion {
		match self {
			BridgeHubRuntimeType::RococoLocal => &bridge_hub_rococo_runtime::VERSION,
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

pub mod rococo {
	use crate::chain_spec::{
		get_account_id_from_seed, get_collator_keys_from_seed, Extensions, SAFE_XCM_VERSION,
	};
	use bridge_hub_rococo_runtime::{AccountId, AuraId};
	use cumulus_primitives_core::ParaId;
	use sc_chain_spec::ChainType;
	use sp_core::sr25519;

	pub const BRIDGE_HUB_ROCOCO_LOCAL: &str = "bridge-hub-rococo-local";

	/// Specialized `ChainSpec` for the normal parachain runtime.
	pub type BridgeHubChainSpec =
		sc_service::GenericChainSpec<bridge_hub_rococo_runtime::GenesisConfig, Extensions>;

	pub fn local_config(relay_chain: &str, para_id: ParaId) -> BridgeHubChainSpec {
		let properties = sc_chain_spec::Properties::new();
		// TODO: check
		// properties.insert("ss58Format".into(), 2.into());
		// properties.insert("tokenSymbol".into(), "ROC".into());
		// properties.insert("tokenDecimals".into(), 12.into());

		BridgeHubChainSpec::from_genesis(
			// Name
			"Rococo BrideHub Local",
			// ID
			super::ensure_id(BRIDGE_HUB_ROCOCO_LOCAL).expect("invalid id"),
			ChainType::Local,
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
		}
	}
}
