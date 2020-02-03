// Copyright 2019 Parity Technologies (UK) Ltd.
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

use crate::chain_spec;

use parachain_runtime::Block;

pub use sc_cli::{error::{self, Result}, VersionInfo};
use sc_client::genesis;
use sc_service::{Configuration, Roles as ServiceRoles};
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::{
	traits::{Block as BlockT, Hash as HashT, Header as HeaderT},
	BuildStorage,
};
use polkadot_service::ChainSpec as ChainSpecPolkadot;
use polkadot_cli::Cli as PolkadotCli;

use futures::{future::Map, FutureExt};

use codec::Encode;

use log::info;

use std::{path::PathBuf, sync::Arc};

use structopt::StructOpt;

/// Parse command line arguments into service configuration.
pub fn run(version: VersionInfo) -> error::Result<()> {
	/*
	type Config<T> = Configuration<(), T>;

	match parse_and_prepare::<SubCommands, NoCustom, _>(
		&version,
		"cumulus-test-parachain-collator",
		args_parachain,
	) {
		ParseAndPrepare::Run(cmd) => cmd.run(
			load_spec,
			exit,
			|exit, _cli_args, _custom_args, mut config: Config<_>| {
				info!("{}", version.name);
				info!("  version {}", config.full_version());
				info!("  by {}, 2019", version.author);
				info!("Chain specification: {}", config.chain_spec.name());
				info!("Node name: {}", config.name);
				info!("Roles: {:?}", config.roles);
				info!("Parachain id: {:?}", crate::PARA_ID);

				// TODO
				let key = Arc::new(sp_core::Pair::from_seed(&[10; 32]));

				// TODO
				config.network.listen_addresses = Vec::new();

				let mut polkadot_config = parse_and_prepare::<NoCustom, NoCustom, _>(
					&version,
					"cumulus-test-parachain-collator",
					args_relaychain,
				).into_configuration::<CustomConfigurationPolkadot, _, _, _>(
					load_spec_polkadot,
					config.in_chain_config_dir("polkadot"),
				)
				.map_err(|e| e.to_string())?
				.expect(
					"can only fail when this is a CustomCommand. Running parse_and_prepare with \
					NoCustom can never return a CustomCommand; therefore this will never fail; qed"
				);
				polkadot_config.network.boot_nodes = config.network.boot_nodes.clone();

				if let Some(ref config_dir) = polkadot_config.config_dir {
					polkadot_config.database = sc_service::config::DatabaseConfig::Path {
						cache_size: Default::default(),
						path: config_dir.join("db"),
					};

					polkadot_config.keystore = sc_service::config::KeystoreConfig::Path {
						path: config_dir.join("keystore"),
						password: None,
					};
				}

				match config.roles {
					ServiceRoles::LIGHT => unimplemented!("Light client not supported!"),
					_ => crate::service::run_collator(config, exit, key, polkadot_config),
				}
				.map_err(|e| format!("{:?}", e))
			},
		),
		ParseAndPrepare::BuildSpec(cmd) => cmd.run::<NoCustom, _, _, _>(load_spec),
		ParseAndPrepare::ExportBlocks(cmd) => cmd.run_with_builder(
			|config: Config<_>| Ok(new_full_start!(config).0),
			load_spec,
			exit,
		),
		ParseAndPrepare::ImportBlocks(cmd) => cmd.run_with_builder(
			|config: Config<_>| Ok(new_full_start!(config).0),
			load_spec,
			exit,
		),
		ParseAndPrepare::CheckBlock(cmd) => cmd.run_with_builder(
			|config: Config<_>| Ok(new_full_start!(config).0),
			load_spec,
			exit,
		),
		ParseAndPrepare::PurgeChain(cmd) => cmd.run(load_spec),
		ParseAndPrepare::RevertChain(cmd) => {
			cmd.run_with_builder(|config: Config<_>| Ok(new_full_start!(config).0), load_spec)
		}
		ParseAndPrepare::CustomCommand(SubCommands::ExportGenesisState(cmd)) => {
			export_genesis_state(cmd.output)
		}
	}?;

	Ok(())
}

fn load_spec(_: &str) -> std::result::Result<Option<chain_spec::ChainSpec>, String> {
	Ok(Some(chain_spec::get_chain_spec()))
}

fn load_spec_polkadot(_: &str) -> std::result::Result<Option<ChainSpecPolkadot>, String> {
	Some(polkadot_service::ChainSpec::from_json_bytes(
		&include_bytes!("../res/polkadot_chainspec.json")[..],
	)).transpose()
}

/// Export the genesis state of the parachain.
fn export_genesis_state(output: Option<PathBuf>) -> error::Result<()> {
	let storage = (&chain_spec::get_chain_spec()).build_storage()?;

	let child_roots = storage.children.iter().map(|(sk, child_content)| {
		let state_root = <<<Block as BlockT>::Header as HeaderT>::Hashing as HashT>::trie_root(
			child_content.data.clone().into_iter().collect(),
		);
		(sk.clone(), state_root.encode())
	});
	let state_root = <<<Block as BlockT>::Header as HeaderT>::Hashing as HashT>::trie_root(
		storage.top.clone().into_iter().chain(child_roots).collect(),
	);
	let block: Block = genesis::construct_genesis_block(state_root);

	let header_hex = format!("0x{:?}", HexDisplay::from(&block.header().encode()));

	if let Some(output) = output {
		std::fs::write(output, header_hex)?;
	} else {
		println!("{}", header_hex);
	}

	Ok(())
	*/
	todo!();
}
