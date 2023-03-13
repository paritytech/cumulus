// Copyright 2019-2022 Parity Technologies (UK) Ltd.
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

use crate::chain_spec::Extensions;
use cumulus_primitives_core::ParaId;
use sc_service::ChainType;

/// Specialized `ChainSpec` for the Glutton parachain runtime.
pub type GluttonChainSpec =
	sc_service::GenericChainSpec<glutton_runtime::GenesisConfig, Extensions>;

pub fn glutton_development_config() -> GluttonChainSpec {
	GluttonChainSpec::from_genesis(
		// Name
		"Glutton Development",
		// ID
		"glutton_dev",
		ChainType::Local,
		move || glutton_genesis(2005.into()),
		Vec::new(),
		None,
		None,
		None,
		None,
		Extensions { relay_chain: "kusama-dev".into(), para_id: 2005 },
	)
}

pub fn glutton_local_config() -> GluttonChainSpec {
	GluttonChainSpec::from_genesis(
		// Name
		"Glutton Local",
		// ID
		"glutton_local",
		ChainType::Local,
		move || glutton_genesis(2005.into()),
		Vec::new(),
		None,
		None,
		None,
		None,
		Extensions { relay_chain: "rococo-local".into(), para_id: 2005 },
	)
}

pub fn glutton_config() -> GluttonChainSpec {
	GluttonChainSpec::from_genesis(
		// Name
		"Glutton",
		// ID
		"glutton",
		ChainType::Live,
		move || glutton_genesis(2005.into()),
		Vec::new(),
		None,
		None,
		None,
		None,
		Extensions { relay_chain: "kusama".into(), para_id: 2005 },
	)
}

fn glutton_genesis(parachain_id: ParaId) -> glutton_runtime::GenesisConfig {
	glutton_runtime::GenesisConfig {
		system: glutton_runtime::SystemConfig {
			code: glutton_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		parachain_info: glutton_runtime::ParachainInfoConfig { parachain_id },
		parachain_system: Default::default(),
	}
}
