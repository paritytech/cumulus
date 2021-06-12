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

use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use rococo_parachain_runtime::{AccountId, AuraId, Signature};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, GenericChainSpec};
use serde::{Deserialize, Serialize};
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<rococo_parachain_runtime::GenesisConfig, Extensions>;

/// Specialized `ChainSpec` for the shell parachain runtime.
pub type ShellChainSpec = sc_service::GenericChainSpec<shell_runtime::GenesisConfig, Extensions>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn get_chain_spec(id: ParaId) -> ChainSpec {
	ChainSpec::from_genesis(
		"Local Testnet",
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![
					get_from_seed::<AuraId>("Alice"),
					get_from_seed::<AuraId>("Bob"),
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
				id,
			)
		},
		vec![],
		None,
		None,
		None,
		Extensions {
			relay_chain: "westend-dev".into(),
			para_id: id.into(),
		},
	)
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum GenesisKeys {
	/// Use integriTEE root as key and endow it.
	IntegriteeRoot,
	/// Use Keys from the keyring as root and endow them
	WellKnown,
}

impl GenesisKeys {
	pub fn root(&self) -> AccountId {
		match self {
			GenesisKeys::IntegriteeRoot => hex!["7a7ff92b215258d2441e041425693e2f0c73da4a813db166d7c4018db8d16153"].into(),
			GenesisKeys::WellKnown => get_account_id_from_seed::<sr25519::Public>("Alice")
		}
	}

	pub fn endowed_accounts(&self) -> Vec<AccountId> {
		match self {
			GenesisKeys::IntegriteeRoot => vec![hex!["7a7ff92b215258d2441e041425693e2f0c73da4a813db166d7c4018db8d16153"].into()],
			GenesisKeys::WellKnown => vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
			]
		}
	}

	pub fn initial_authorities(&self) -> Vec<AuraId> {
		vec![
			get_from_seed::<AuraId>("Alice"),
			get_from_seed::<AuraId>("Bob"),
		]
	}
}

pub fn get_shell_chain_spec(id: ParaId, genesis_keys: GenesisKeys) -> ShellChainSpec {
	let chain_type = match genesis_keys {
		GenesisKeys::IntegriteeRoot => ChainType::Live,
		GenesisKeys::WellKnown => ChainType::Local
	};

	integritee_genesis(
		"shell-polkadot-v0.9.4",
		move || shell_testnet_genesis(id, genesis_keys),
		chain_type,
		id,
	)
}

pub fn staging_test_net(id: ParaId) -> ChainSpec {
	ChainSpec::from_genesis(
		"Staging Testnet",
		"staging_testnet",
		ChainType::Live,
		move || {
			testnet_genesis(
				hex!["9ed7705e3c7da027ba0583a22a3212042f7e715d3c168ba14f1424e2bc111d00"].into(),
				vec![
					// $secret//one
					hex!["aad9fa2249f87a210a0f93400b7f90e47b810c6d65caa0ca3f5af982904c2a33"]
						.unchecked_into(),
					// $secret//two
					hex!["d47753f0cca9dd8da00c70e82ec4fc5501a69c49a5952a643d18802837c88212"]
						.unchecked_into(),
				],
				vec![
					hex!["9ed7705e3c7da027ba0583a22a3212042f7e715d3c168ba14f1424e2bc111d00"].into(),
				],
				id,
			)
		},
		Vec::new(),
		None,
		None,
		None,
		Extensions {
			relay_chain: "westend-dev".into(),
			para_id: id.into(),
		},
	)
}

pub fn integritee_spec(id: ParaId, genesis_keys: GenesisKeys) -> ChainSpec {
	let chain_type = match genesis_keys {
		GenesisKeys::IntegriteeRoot => ChainType::Live,
		GenesisKeys::WellKnown => ChainType::Local
	};

	integritee_genesis(
		"integritee-polkadot-v0.9.4",
		move || {
			testnet_genesis(
				genesis_keys.root(),
				// todo: What do I actually need to put as initial authorities??
				genesis_keys.initial_authorities(),
				genesis_keys.endowed_accounts(),
				id,
			)
		}, chain_type, id)
}

fn integritee_genesis<F: Fn() -> GenesisConfig + 'static + Send + Sync, GenesisConfig>(
	chain_id: &str,
	testnet_constructor: F,
	chain_type: ChainType,
	para_id: ParaId,
) -> GenericChainSpec<GenesisConfig, Extensions> {
	GenericChainSpec::<GenesisConfig, Extensions>::from_genesis(
		"IntegriTEE PC1",
		chain_id,
		chain_type,
		testnet_constructor,
		Vec::new(),
		// telemetry endpoints
		None,
		// protocol id
		Some("integritee-polkadot-v0.9.4"),
		// properties
		Some(serde_json::from_str(
			r#"{
				"ss58Format": 42,
				"tokenDecimals": 12,
				"tokenSymbol": "ITY"
				}"#).unwrap()),
		Extensions {
			relay_chain: "rococo".into(),
			para_id: para_id.into(),
		},
	)
}


fn testnet_genesis(
	root_key: AccountId,
	initial_authorities: Vec<AuraId>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> rococo_parachain_runtime::GenesisConfig {
	rococo_parachain_runtime::GenesisConfig {
		frame_system: rococo_parachain_runtime::SystemConfig {
			code: rococo_parachain_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: rococo_parachain_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1 << 60))
				.collect(),
		},
		pallet_sudo: rococo_parachain_runtime::SudoConfig { key: root_key },
		parachain_info: rococo_parachain_runtime::ParachainInfoConfig { parachain_id: id },
		pallet_aura: rococo_parachain_runtime::AuraConfig {
			authorities: initial_authorities,
		},
		cumulus_pallet_aura_ext: Default::default(),
		cumulus_pallet_parachain_system: Default::default(),
	}
}

fn shell_testnet_genesis(parachain_id: ParaId, genesis_keys: GenesisKeys) -> shell_runtime::GenesisConfig {
	shell_runtime::GenesisConfig {
		frame_system: shell_runtime::SystemConfig {
			code: shell_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: shell_runtime::BalancesConfig {
			balances: genesis_keys.endowed_accounts()
				.iter()
				.cloned()
				.map(|k| (k, 1 << 60))
				.collect(),
		},
		pallet_sudo: shell_runtime::SudoConfig { key: genesis_keys.root() },
		parachain_info: shell_runtime::ParachainInfoConfig { parachain_id },
		cumulus_pallet_parachain_system: Default::default(),
	}
}
