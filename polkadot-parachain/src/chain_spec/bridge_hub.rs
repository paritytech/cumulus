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

use crate::chain_spec::{
	get_account_id_from_seed, get_collator_keys_from_seed, Extensions, SAFE_XCM_VERSION,
};
use cumulus_primitives_core::ParaId;
// TODO: why this is rococo?
use rococo_parachain_runtime::{AccountId, AuraId};
use sc_service::ChainType;
use sp_core::sr25519;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type BridgeHubChainSpec =
	sc_service::GenericChainSpec<bridge_hub_runtime::GenesisConfig, Extensions>;

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn bridge_hub_session_keys(keys: AuraId) -> bridge_hub_runtime::SessionKeys {
	bridge_hub_runtime::SessionKeys { aura: keys }
}

pub fn bridge_hub_local_config() -> BridgeHubChainSpec {
	let mut properties = sc_chain_spec::Properties::new();
	// properties.insert("ss58Format".into(), 2.into());
	// properties.insert("tokenSymbol".into(), "KSM".into());
	properties.insert("tokenDecimals".into(), 12.into());

	BridgeHubChainSpec::from_genesis(
		// Name
		"Bride Hub Local",
		// ID
		"bridge-hub-local",
		ChainType::Local,
		move || {
			bridge_hub_genesis(
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
				1000.into(),
			)
		},
		Vec::new(),
		None,
		None,
		None,
		Some(properties),
		// TODO: really rococo-local?
		Extensions { relay_chain: "rococo-local".into(), para_id: 1000 },
	)
}

fn bridge_hub_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> bridge_hub_runtime::GenesisConfig {
	bridge_hub_runtime::GenesisConfig {
		system: bridge_hub_runtime::SystemConfig {
			code: bridge_hub_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		balances: bridge_hub_runtime::BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		parachain_info: bridge_hub_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: bridge_hub_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			// TODO: what is this?
			candidacy_bond: 10,
			..Default::default()
		},
		session: bridge_hub_runtime::SessionConfig {
			keys: invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                   // account id
						acc,                           // validator id
						bridge_hub_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		polkadot_xcm: bridge_hub_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
	}
}
