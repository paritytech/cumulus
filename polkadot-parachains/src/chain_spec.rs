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
use parachain_runtime::{AccountId, AuraId, BalanceType, CeremonyPhaseType, Demurrage};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, GenericChainSpec};
use serde::{Deserialize, Serialize};

pub use crate::chain_spec_helpers::{
	public_from_ss58, rococo_properties, EncointerKeys, GenesisKeys, RelayChain, WellKnownKeys,
};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type EncointerChainSpec = GenericChainSpec<parachain_runtime::GenesisConfig, Extensions>;

/// Specialized `ChainSpec` for the launch parachain runtime.
pub type LaunchChainSpec = GenericChainSpec<launch_runtime::GenesisConfig, Extensions>;

pub const ENDOWED_FUNDING: u128 = 1 << 60;

/// Configure `endowed_accounts` with initial balance of `ENDOWED_FUNDING`.
pub fn allocate_endowance(endowed_accounts: Vec<AccountId>) -> Vec<(AccountId, u128)> {
	endowed_accounts.into_iter().map(|k| (k, ENDOWED_FUNDING)).collect()
}

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = 2;

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

/// Chain-spec for the encointer runtime
pub fn encointer_spec(
	id: ParaId,
	genesis_keys: GenesisKeys,
	relay_chain: RelayChain,
) -> EncointerChainSpec {
	let (council, endowed, authorities) = match genesis_keys {
		GenesisKeys::Encointer =>
			(EncointerKeys::council(), [].to_vec(), EncointerKeys::authorities()),
		GenesisKeys::EncointerWithCouncilEndowed =>
			(EncointerKeys::council(), EncointerKeys::council(), EncointerKeys::authorities()),
		GenesisKeys::WellKnown =>
			(WellKnownKeys::council(), WellKnownKeys::endowed(), WellKnownKeys::authorities()),
	};

	chain_spec(
		"Encointer Network",
		move || {
			encointer_genesis(
				council.clone(),
				authorities.clone(),
				allocate_endowance(endowed.clone()),
				id,
			)
		},
		relay_chain.chain_type(),
		id,
		&relay_chain,
	)
}

/// Chain-spec for the launch runtime
pub fn launch_spec(
	id: ParaId,
	genesis_keys: GenesisKeys,
	relay_chain: RelayChain,
) -> LaunchChainSpec {
	let (council, endowed, authorities) = match genesis_keys {
		GenesisKeys::Encointer =>
			(EncointerKeys::council(), [].to_vec(), EncointerKeys::authorities()),
		GenesisKeys::EncointerWithCouncilEndowed =>
			(EncointerKeys::council(), EncointerKeys::council(), EncointerKeys::authorities()),
		GenesisKeys::WellKnown =>
			(WellKnownKeys::council(), WellKnownKeys::endowed(), WellKnownKeys::authorities()),
	};

	chain_spec(
		"Encointer Launch",
		move || {
			launch_genesis(
				council.clone(),
				authorities.clone(),
				allocate_endowance(endowed.clone()),
				id,
			)
		},
		relay_chain.chain_type(),
		id,
		&relay_chain,
	)
}

/// decorates the given `testnet_constructor` with metadata.
///
/// Intended to remove redundant code when defining encointer-launch-runtime and
/// encointer-parachain-runtime chain-specs.
fn chain_spec<F: Fn() -> GenesisConfig + 'static + Send + Sync, GenesisConfig>(
	chain_name: &str,
	testnet_constructor: F,
	chain_type: ChainType,
	para_id: ParaId,
	relay_chain: &RelayChain,
) -> GenericChainSpec<GenesisConfig, Extensions> {
	GenericChainSpec::<GenesisConfig, Extensions>::from_genesis(
		chain_name,
		&format!("encointer-{}", relay_chain.to_string()),
		chain_type,
		testnet_constructor,
		Vec::new(),
		// telemetry endpoints
		None,
		// protocol id
		None,
		// properties
		Some(relay_chain.properties()),
		Extensions { relay_chain: relay_chain.to_string(), para_id: para_id.into() },
	)
}

pub fn sybil_dummy_spec(id: ParaId, relay_chain: RelayChain) -> EncointerChainSpec {
	let (council, endowed, authorities) =
		(WellKnownKeys::council(), WellKnownKeys::endowed(), WellKnownKeys::authorities());

	EncointerChainSpec::from_genesis(
		"Sybil Dummy",
		"sybil-dummy-rococo-v1",
		relay_chain.chain_type(),
		move || {
			encointer_genesis(
				council.clone(),
				authorities.clone(),
				allocate_endowance(endowed.clone()),
				id,
			)
		},
		Vec::new(),
		// telemetry endpoints
		None,
		// protocol id
		None,
		// properties
		Some(
			serde_json::from_str(
				r#"{
			"ss58Format": 42,
			"tokenDecimals": 12,
			"tokenSymbol": "DUM"
		  }"#,
			)
			.unwrap(),
		),
		Extensions { relay_chain: relay_chain.to_string(), para_id: id.into() },
	)
}

fn encointer_genesis(
	encointer_council: Vec<AccountId>,
	initial_authorities: Vec<AuraId>,
	endowance_allocation: Vec<(AccountId, u128)>,
	id: ParaId,
) -> parachain_runtime::GenesisConfig {
	let root_key = encointer_council.clone().get(0).unwrap().clone(); //TODO fix this hack
	parachain_runtime::GenesisConfig {
		system: parachain_runtime::SystemConfig {
			code: parachain_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		parachain_system: Default::default(),
		balances: parachain_runtime::BalancesConfig { balances: endowance_allocation },
		parachain_info: parachain_runtime::ParachainInfoConfig { parachain_id: id },
		aura: parachain_runtime::AuraConfig { authorities: initial_authorities },
		aura_ext: Default::default(),
		polkadot_xcm: parachain_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
		treasury: Default::default(),
		collective: Default::default(),
		membership: parachain_runtime::MembershipConfig {
			members: encointer_council,
			phantom: Default::default(),
		},
		encointer_scheduler: parachain_runtime::EncointerSchedulerConfig {
			current_phase: CeremonyPhaseType::REGISTERING,
			current_ceremony_index: 1,
			ceremony_master: root_key.clone(),
			phase_durations: vec![
				(CeremonyPhaseType::REGISTERING, 600_000),
				(CeremonyPhaseType::ASSIGNING, 600_000),
				(CeremonyPhaseType::ATTESTING, 600_000),
			],
		},
		encointer_ceremonies: parachain_runtime::EncointerCeremoniesConfig {
			ceremony_reward: BalanceType::from_num(1),
			time_tolerance: 600_000,   // +-10min
			location_tolerance: 1_000, // [m]
		},
		encointer_communities: parachain_runtime::EncointerCommunitiesConfig {
			community_master: root_key,
		},
		encointer_balances: parachain_runtime::EncointerBalancesConfig {
			demurrage_per_block_default: Demurrage::from_bits(
				0x0000000000000000000001E3F0A8A973_i128,
			),
		},
	}
}

fn launch_genesis(
	encointer_council: Vec<AccountId>,
	initial_authorities: Vec<AuraId>,
	endowance_allocation: Vec<(AccountId, u128)>,
	id: ParaId,
) -> launch_runtime::GenesisConfig {
	launch_runtime::GenesisConfig {
		system: launch_runtime::SystemConfig {
			code: launch_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		parachain_system: Default::default(),
		balances: launch_runtime::BalancesConfig { balances: endowance_allocation },
		parachain_info: launch_runtime::ParachainInfoConfig { parachain_id: id },
		aura: launch_runtime::AuraConfig { authorities: initial_authorities },
		aura_ext: Default::default(),
		polkadot_xcm: launch_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
		treasury: Default::default(),
		collective: Default::default(),
		membership: launch_runtime::MembershipConfig {
			members: encointer_council,
			phantom: Default::default(),
		},
	}
}

/// hard-coded launch-runtime config for rococo
pub fn launch_rococo() -> Result<LaunchChainSpec, String> {
	LaunchChainSpec::from_json_bytes(&include_bytes!("../res/launch-rococo.json")[..])
}

/// hard-coded launch-runtime config for kusama
pub fn launch_kusama() -> Result<LaunchChainSpec, String> {
	LaunchChainSpec::from_json_bytes(&include_bytes!("../res/launch-kusama.json")[..])
}

/// hard-coded launch-runtime config for westend
pub fn launch_westend() -> Result<LaunchChainSpec, String> {
	LaunchChainSpec::from_json_bytes(&include_bytes!("../res/launch-westend.json")[..])
}

/// hard-coded encointer-runtime config for rococo
pub fn encointer_rococo() -> Result<EncointerChainSpec, String> {
	EncointerChainSpec::from_json_bytes(&include_bytes!("../res/encointer-rococo.json")[..])
}

/// hard-coded encointer-runtime config for kusama
pub fn encointer_kusama() -> Result<EncointerChainSpec, String> {
	EncointerChainSpec::from_json_bytes(&include_bytes!("../res/encointer-kusama.json")[..])
}

/// hard-coded encointer-runtime config for westend
pub fn encointer_westend() -> Result<EncointerChainSpec, String> {
	EncointerChainSpec::from_json_bytes(&include_bytes!("../res/encointer-westend.json")[..])
}
