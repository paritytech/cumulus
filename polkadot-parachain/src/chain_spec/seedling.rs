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

use crate::chain_spec::{get_account_id_from_seed, Extensions};
use cumulus_primitives_core::ParaId;
use parachains_common::{AccountId, AuraId};
use sc_service::ChainType;
use sp_core::{sr25519, Public, Pair};
use seedling_runtime::AuraConfig;
use sp_runtime::traits::{IdentifyAccount, Verify};

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> AuraId {
	get_from_seed::<AuraId>(s)
}

/// Specialized `ChainSpec` for the seedling parachain runtime.
pub type SeedlingChainSpec =
	sc_service::GenericChainSpec<seedling_runtime::GenesisConfig, Extensions>;

pub fn get_seedling_chain_spec() -> SeedlingChainSpec {
	SeedlingChainSpec::from_genesis(
		"Seedling Local Testnet",
		"seedling_local_testnet",
		ChainType::Local,
		move || {
			seedling_testnet_genesis(
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Parachain Id
				2000.into(),
			)
		},
		Vec::new(),
		None,
		None,
		None,
		None,
		Extensions { relay_chain: "westend".into(), para_id: 2000 },
	)
}

fn seedling_testnet_genesis(
	initial_authorities: Vec<AuraId>,
	root_key: AccountId,
	parachain_id: ParaId,
) -> seedling_runtime::GenesisConfig {
	seedling_runtime::GenesisConfig {
		system: seedling_runtime::SystemConfig {
			code: seedling_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		aura: AuraConfig {
			authorities: initial_authorities.iter().map(|x| x.clone()).collect(),
		},
		aura_ext: Default::default(),
		sudo: seedling_runtime::SudoConfig { key: Some(root_key) },
		parachain_info: seedling_runtime::ParachainInfoConfig { parachain_id },
		parachain_system: Default::default(),
	}
}
