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

use clap::Parser;
use cumulus_primitives_core::ParaId;
use cumulus_test_service::Keyring::*;
use sc_network::config::MultiaddrWithPeerId;
use std::{path::PathBuf, sync::Arc};
use url::Url;

fn validate_relay_chain_url(arg: &str) -> Result<(), String> {
	let url = Url::parse(arg).map_err(|e| e.to_string())?;

	if url.scheme() == "ws" {
		Ok(())
	} else {
		Err(format!(
			"'{}' URL scheme not supported. Only websocket RPC is currently supported",
			url.scheme()
		))
	}
}

#[derive(Debug, Parser)]
pub struct RunCmd {
	/// The cumulus RunCmd inherents from sc_cli's
	#[clap(flatten)]
	base: sc_cli::RunCmd,

	/// Run node as collator.
	///
	/// Note that this is the same as running with `--validator`.
	#[clap(long, conflicts_with = "validator")]
	collator: bool,

	/// EXPERIMENTAL: Specify an URL to a relay chain full node to communicate with.
	#[clap(
		long,
		parse(try_from_str),
		validator = validate_relay_chain_url,
		conflicts_with_all = &["alice", "bob", "charlie", "dave", "eve", "ferdie", "one", "two"])
	]
	relay_chain_rpc_url: Option<Url>,

	#[clap(short, long)]
	use_null_consensus: bool,

	/// Do not announce blocks
	#[clap(short, long)]
	disable_block_announcements: bool,

	#[clap(long, default_value_t = 2000)]
	parachain_id: u32,

	#[clap(long)]
	relay_chain_bootnodes: Vec<MultiaddrWithPeerId>,

	#[clap(long)]
	relay_chain_port: Option<u16>,

	#[clap(long)]
	relay_chain_spec: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), sc_service::Error> {
	let args = RunCmd::parse();

	let mut builder = sc_cli::LoggerBuilder::new("");
	builder.with_colors(true);
	let _ = builder.init();

	let tokio_handle = tokio::runtime::Handle::current();
	let para_id = ParaId::from(args.parachain_id);

	let keyring = args.base.get_keyring();
	let mut parachain_node_builder = cumulus_test_service::TestNodeBuilder::new(
		para_id,
		tokio_handle.clone(),
		keyring.unwrap_or(Alice),
	)
	.no_memory_address()
	.connect_to_relay_chain_node_addresses(args.relay_chain_bootnodes)
	.with_bootnodes(args.base.network_params.bootnodes)
	.relay_chain_no_memory_address();

	if let Some(path) = args.relay_chain_spec {
		parachain_node_builder = parachain_node_builder.use_relay_chain_spec(path);
	}

	if args.base.network_params.node_key_params.node_key.is_some() {
		let node_key = args
			.base
			.network_params
			.node_key_params
			.node_key(&PathBuf::new())
			.expect("Invalid node key");
		parachain_node_builder = parachain_node_builder.use_node_key_config(node_key);
	}

	if let Some(port) = args.base.network_params.port {
		parachain_node_builder = parachain_node_builder.use_port(port);
	}

	if let Some(port) = args.relay_chain_port {
		parachain_node_builder = parachain_node_builder.use_relay_chain_port(port);
	}

	if args.disable_block_announcements {
		parachain_node_builder = parachain_node_builder.wrap_announce_block(|_| {
			// Never announce any block
			Arc::new(|_, _| {})
		});
	}

	if !args.base.network_params.reserved_nodes.is_empty() {
		parachain_node_builder = parachain_node_builder
			.connect_to_parachain_nodes_address(args.base.network_params.reserved_nodes)
	}

	if args.collator || args.base.validator {
		parachain_node_builder = parachain_node_builder.enable_collator();
	}

	if args.use_null_consensus {
		parachain_node_builder = parachain_node_builder.use_null_consensus();
	}

	if args.base.network_params.reserved_only {
		parachain_node_builder =
			parachain_node_builder.exclusively_connect_to_registered_parachain_nodes();
	}

	if let Some(url) = args.relay_chain_rpc_url {
		parachain_node_builder = parachain_node_builder.use_external_relay_chain_node_at_url(url);
	}

	let mut node = parachain_node_builder.build().await;

	node.task_manager.future().await
}
