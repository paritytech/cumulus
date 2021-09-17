// Copyright (C) 2018-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use canvas_runtime::{AccountId, AuraId, Signature, EXISTENTIAL_DEPOSIT};
use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<canvas_runtime::GenesisConfig, Extensions>;

/// Helper function to generate a crypto pair from seed
pub fn get_pair_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
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

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_pair_from_seed::<AuraId>(seed)
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_pair_from_seed::<TPublic>(seed)).into_account()
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn canvas_session_keys(keys: AuraId) -> canvas_runtime::SessionKeys {
	canvas_runtime::SessionKeys { aura: keys }
}

pub fn development_config(id: ParaId) -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "ROC".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			canvas_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed("Bob"),
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
				id,
			)
		},
		vec![],
		None,
		None,
		None,
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: id.into(),
		},
	)
}

pub fn local_testnet_config(id: ParaId) -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "ROC".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::from_genesis(
		// Name
		"Canvas",
		// ID
		"canvas_local",
		ChainType::Local,
		move || {
			canvas_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed("Bob"),
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
				id,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		Some("canvas-local"),
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: id.into(),
		},
	)
}

pub fn rococo_testnet_config(id: ParaId) -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "ROC".into());
	properties.insert("tokenDecimals".into(), 12.into());

	ChainSpec::from_genesis(
		// Name
		"Canvas",
		// ID
		"canvas_rococo",
		ChainType::Live,
		move || {
			canvas_genesis(
				// initial collators.
				vec![
					(
						hex!["4ad2d854d1bc5bbfdf93b689cb1aa3986684126c2b16d86924019de6798b0971"]
							.into(),
						hex!["4ad2d854d1bc5bbfdf93b689cb1aa3986684126c2b16d86924019de6798b0971"]
							.unchecked_into(),
					),
					(
						hex!["526d7d3d5357d20e4cd75d1a452bd5b4903caf160f55d6bd19807efbdf165319"]
							.into(),
						hex!["526d7d3d5357d20e4cd75d1a452bd5b4903caf160f55d6bd19807efbdf165319"]
							.unchecked_into(),
					),
					(
						hex!["bc09354c12c054c8f6b3da208485eacec4ac648bad348895273b37bab5a0937c"]
							.into(),
						hex!["bc09354c12c054c8f6b3da208485eacec4ac648bad348895273b37bab5a0937c"]
							.unchecked_into(),
					),
				],
				// Warning: The configuration for a production chain should not contain
				// any endowed accounts here, otherwise it'll be minting extra native tokens
				// from the relay chain on the parachain.
				vec![
					// NOTE: Remove endowed accounts if deployed on other relay chains.
					// Endowed accounts
					hex!["baa78c7154c7f82d6d377177e20bcab65d327eca0086513f9964f5a0f6bdad56"].into(),
					// AccountId of an account which `ink-waterfall` uses for automated testing
					hex!["0e47e2344d523c3cc5c34394b0d58b9a4200e813a038e6c5a6163cc07d70b069"].into(),
				],
				id,
			)
		},
		// Bootnodes
		vec![
			"/ip4/34.90.191.237/tcp/30333/p2p/12D3KooWKg3Rpxcr9oJ8n6khoxpGKWztCZydtUZk2cojHqnfLrpj"
				.parse()
				.expect("MultiaddrWithPeerId"),
			"/ip4/35.204.68.28/tcp/30333/p2p/12D3KooWPEXYrz8tHU3nDtPoPw4V7ou5dzMEWSTuUj7vaWiYVAVh"
				.parse()
				.expect("MultiaddrWithPeerId"),
			"/ip4/34.90.139.15/tcp/30333/p2p/12D3KooWEVU8AFNary4nP4qEnEcwJaRuy59Wefekzdu9pKbnVEhk"
				.parse()
				.expect("MultiaddrWithPeerId"),
			"/ip4/35.204.99.97/tcp/30333/p2p/12D3KooWP6pV3ZmcXzGDjv8ZMgA6nZxfAKDxSz4VNiLx6vVCQgJX"
				.parse()
				.expect("MultiaddrWithPeerId"),
		],
		// Telemetry
		None,
		// Protocol ID
		Some("canvas-rococo"),
		// Properties
		Some(properties),
		// Extensions
		Extensions { relay_chain: "rococo".into(), para_id: id.into() },
	)
}

fn canvas_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> canvas_runtime::GenesisConfig {
	canvas_runtime::GenesisConfig {
		system: canvas_runtime::SystemConfig {
			code: canvas_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: canvas_runtime::BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		parachain_info: canvas_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: canvas_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
			..Default::default()
		},
		session: canvas_runtime::SessionConfig {
			keys: invulnerables
				.iter()
				.cloned()
				.map(|(acc, aura)| {
					(
						acc.clone(),               // account id
						acc.clone(),               // validator id
						canvas_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
	}
}
