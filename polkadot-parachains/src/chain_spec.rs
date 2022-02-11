// Copyright 2021 Integritee AG and Supercomputing Systems AG
// This file is part of the "Integritee parachain" and is
// based on Cumulus from Parity Technologies (UK) Ltd.

// Integritee parachain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Integritee parachain.  If not, see <http://www.gnu.org/licenses/>.
#![allow(clippy::inconsistent_digit_grouping)]

use cumulus_primitives_core::ParaId;
use parachain_runtime::{AccountId, AuraId};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, GenericChainSpec};
use serde::{Deserialize, Serialize};
use sp_core::{crypto::Ss58Codec, sr25519, Public};
use sp_keyring::AccountKeyring::{Alice, Bob, Dave, Eve};
use std::str::FromStr;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type IntegriteeChainSpec =
	sc_service::GenericChainSpec<parachain_runtime::GenesisConfig, Extensions>;

/// Specialized `ChainSpec` for the shell parachain runtime.
pub type ShellChainSpec = sc_service::GenericChainSpec<shell_runtime::GenesisConfig, Extensions>;

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

pub fn public_from_ss58<TPublic: Public + FromStr>(ss58: &str) -> TPublic
where
	// what's up with this weird trait bound??
	<TPublic as FromStr>::Err: std::fmt::Debug,
{
	TPublic::from_ss58check(ss58).expect("supply valid ss58!")
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum GenesisKeys {
	/// Use integriTEE keys.
	Integritee,
	/// Use Keys from the keyring for a test setup
	WellKnown,
}

struct WellKnownKeys;

impl WellKnownKeys {
	fn root() -> AccountId {
		Alice.to_account_id()
	}

	fn endowed() -> Vec<AccountId> {
		vec![Alice.to_account_id(), Bob.to_account_id()]
	}

	fn authorities() -> Vec<AuraId> {
		vec![Dave.public().into(), Eve.public().into()]
	}
}

struct IntegriteeKeys;

impl IntegriteeKeys {
	fn root() -> AccountId {
		public_from_ss58::<sr25519::Public>("5EqGFRTN3m2kLpoaThANra5REs5C7B2rfLmmZv2nbJsxaTe1")
			.into()
	}
	fn authorities() -> Vec<AuraId> {
		vec![
			public_from_ss58::<sr25519::Public>("5GZJjbPPD9u6NDgK1ApYmbyGs7EBX4HeEz2y2CD38YJxjvQH")
				.into(),
			/*
			public_from_ss58::<sr25519::Public>("5CcSd1GZus6Jw7rP47LLqMMmtr2KeXCH6W11ZKk1LbCQ9dPY").into(),
			public_from_ss58::<sr25519::Public>("5FsECrDjBXrh5hXmN4PhQfNPbjYYwwW7edu2UQ8G5LR1JFuH").into(),
			public_from_ss58::<sr25519::Public>("5HBdSEnswkqm6eoHzzX5PCeKoC15CCy88vARrT8XMaRRuyaE").into(),
			public_from_ss58::<sr25519::Public>("5GGxVLYTXS7JZAwVzisdXbsugHSD6gtDb3AT3MVzih9jTLQT").into(),

			 */
		]
	}
}

pub fn shell_chain_spec(
	id: ParaId,
	genesis_keys: GenesisKeys,
	relay_chain: RelayChain,
) -> ShellChainSpec {
	let (root, endowed, authorities) = match genesis_keys {
		GenesisKeys::Integritee =>
			(IntegriteeKeys::root(), vec![IntegriteeKeys::root()], IntegriteeKeys::authorities()),
		GenesisKeys::WellKnown =>
			(WellKnownKeys::root(), WellKnownKeys::endowed(), WellKnownKeys::authorities()),
	};

	let chain_name = "Integritee Shell".to_string();

	chain_spec(
		&chain_name,
		move || shell_genesis_config(root.clone(), endowed.clone(), authorities.clone(), id),
		relay_chain.chain_type(),
		id,
		&relay_chain.to_string(),
	)
}

pub fn integritee_chain_spec(
	id: ParaId,
	genesis_keys: GenesisKeys,
	relay_chain: RelayChain,
) -> IntegriteeChainSpec {
	let (root, endowed, authorities) = match genesis_keys {
		GenesisKeys::Integritee =>
			(IntegriteeKeys::root(), vec![IntegriteeKeys::root()], IntegriteeKeys::authorities()),
		GenesisKeys::WellKnown =>
			(WellKnownKeys::root(), WellKnownKeys::endowed(), WellKnownKeys::authorities()),
	};

	let chain_name = "Integritee Network".to_string();

	chain_spec(
		&chain_name,
		move || integritee_genesis_config(root.clone(), endowed.clone(), authorities.clone(), id),
		relay_chain.chain_type(),
		id,
		&relay_chain.to_string(),
	)
}

fn chain_spec<F: Fn() -> GenesisConfig + 'static + Send + Sync, GenesisConfig>(
	chain_name: &str,
	testnet_constructor: F,
	chain_type: ChainType,
	para_id: ParaId,
	relay_chain: &str,
) -> GenericChainSpec<GenesisConfig, Extensions> {
	GenericChainSpec::<GenesisConfig, Extensions>::from_genesis(
		chain_name,
		&format!("integritee-{}", relay_chain),
		chain_type,
		testnet_constructor,
		Vec::new(),
		// telemetry endpoints
		None,
		// protocol id
		Some("teer"),
		// fork id
		None,
		// properties
		Some(
			serde_json::from_str(
				r#"{
				"ss58Format": 13,
				"tokenDecimals": 12,
				"tokenSymbol": "TEER"
				}"#,
			)
			.unwrap(),
		),
		Extensions { relay_chain: relay_chain.into(), para_id: para_id.into() },
	)
}

fn integritee_genesis_config(
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	initial_authorities: Vec<AuraId>,
	id: ParaId,
) -> parachain_runtime::GenesisConfig {
	parachain_runtime::GenesisConfig {
		system: parachain_runtime::SystemConfig {
			code: parachain_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		balances: parachain_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 10_000_000__000_000_000_000))
				.collect(),
		},
		sudo: parachain_runtime::SudoConfig { key: Some(root_key) },
		vesting: Default::default(),
		parachain_info: parachain_runtime::ParachainInfoConfig { parachain_id: id },
		aura: parachain_runtime::AuraConfig { authorities: initial_authorities },
		aura_ext: Default::default(),
		parachain_system: Default::default(),
	}
}

fn shell_genesis_config(
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	initial_authorities: Vec<AuraId>,
	parachain_id: ParaId,
) -> shell_runtime::GenesisConfig {
	shell_runtime::GenesisConfig {
		system: shell_runtime::SystemConfig {
			code: shell_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		balances: shell_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 10_000_000__000_000_000_000))
				.collect(),
		},
		sudo: shell_runtime::SudoConfig { key: Some(root_key) },
		vesting: Default::default(),
		parachain_info: shell_runtime::ParachainInfoConfig { parachain_id },
		parachain_system: Default::default(),
		aura: shell_runtime::AuraConfig { authorities: initial_authorities },
		aura_ext: Default::default(),
	}
}

pub enum RelayChain {
	RococoLocal,
	Kusama,
	KusamaLocal,
	PolkadotLocal,
	Rococo,
	Polkadot,
}

pub fn shell_westend_config() -> Result<ShellChainSpec, String> {
	ShellChainSpec::from_json_bytes(&include_bytes!("../res/shell-westend.json")[..])
}

pub fn integritee_westend_config() -> Result<IntegriteeChainSpec, String> {
	IntegriteeChainSpec::from_json_bytes(&include_bytes!("../res/integritee-westend.json")[..])
}

pub fn shell_kusama_config() -> Result<ShellChainSpec, String> {
	ShellChainSpec::from_json_bytes(&include_bytes!("../res/shell-kusama.json")[..])
}

pub fn integritee_kusama_config() -> Result<IntegriteeChainSpec, String> {
	IntegriteeChainSpec::from_json_bytes(&include_bytes!("../res/integritee-kusama.json")[..])
}

pub fn shell_rococo_config() -> Result<ShellChainSpec, String> {
	ShellChainSpec::from_json_bytes(&include_bytes!("../res/shell-rococo.json")[..])
}

pub fn integritee_rococo_config() -> Result<IntegriteeChainSpec, String> {
	IntegriteeChainSpec::from_json_bytes(&include_bytes!("../res/integritee-rococo.json")[..])
}

impl ToString for RelayChain {
	fn to_string(&self) -> String {
		match self {
			RelayChain::RococoLocal => "rococo-local".into(),
			RelayChain::Kusama => "kusama".into(),
			RelayChain::KusamaLocal => "kusama-local".into(),
			RelayChain::PolkadotLocal => "polkadot-local".into(),
			RelayChain::Rococo => "rococo".into(),
			RelayChain::Polkadot => "polkadot".into(),
		}
	}
}

impl RelayChain {
	fn chain_type(&self) -> ChainType {
		match self {
			RelayChain::RococoLocal => ChainType::Local,
			RelayChain::KusamaLocal => ChainType::Local,
			RelayChain::PolkadotLocal => ChainType::Local,
			RelayChain::Kusama => ChainType::Live,
			RelayChain::Rococo => ChainType::Live,
			RelayChain::Polkadot => ChainType::Live,
		}
	}
}
