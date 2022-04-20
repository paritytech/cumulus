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

use clap::{Parser, Subcommand};
use cumulus_client_service::genesis::generate_genesis_block;
use cumulus_primitives_core::{relay_chain::v2::CollatorPair, ParaId};
use polkadot_service::{
	chain_spec::Extensions, runtime_traits::AccountIdConversion, ChainSpec, PrometheusConfig,
};
use sc_cli::{
	CliConfiguration, DefaultConfigurationValues, ImportParams, KeystoreParams, NetworkParams,
	Result as CliResult, RuntimeVersion, SharedParams, SubstrateCli,
};
use sc_service::BasePath;
use sp_core::{hexdisplay::HexDisplay, Encode, Pair};
use sp_runtime::traits::Block;
use std::{io::Write, net::SocketAddr, path::PathBuf};

#[derive(Subcommand, Debug)]
pub enum Commands {
	ExportGenesisWasm {
		#[clap(parse(from_os_str))]
		output: Option<PathBuf>,

		/// Write output in binary. Default is to write in hex.
		#[clap(short, long)]
		raw: bool,
	},
	ExportGenesisState {
		#[clap(parse(from_os_str))]
		output: Option<PathBuf>,

		/// Write output in binary. Default is to write in hex.
		#[clap(short, long)]
		raw: bool,
	},
}

#[derive(Debug, Parser)]
#[clap(
	propagate_version = true,
	args_conflicts_with_subcommands = true,
	subcommand_negates_reqs = true
)]
pub struct TestCollatorCli {
	#[clap(subcommand)]
	pub subcommand: Option<Commands>,

	#[clap(flatten)]
	pub run: cumulus_client_cli::RunCmd,

	#[clap(default_value_t = 2000u32)]
	pub parachain_id: u32,

	/// Relay chain arguments
	#[clap(raw = true, conflicts_with = "relay-chain-rpc-url")]
	pub relaychain_args: Vec<String>,
}

#[derive(Debug)]
pub struct RelayChainCli {
	/// The actual relay chain cli object.
	pub base: polkadot_cli::RunCmd,

	/// Optional chain id that should be passed to the relay chain.
	pub chain_id: Option<String>,

	/// The base path that should be used by the relay chain.
	pub base_path: Option<PathBuf>,
}

impl RelayChainCli {
	/// Parse the relay chain CLI parameters using the para chain `Configuration`.
	pub fn new<'a>(
		para_config: &sc_service::Configuration,
		relay_chain_args: impl Iterator<Item = &'a String>,
	) -> Self {
		let base_path = para_config.base_path.as_ref().map(|x| x.path().join("polkadot"));
		Self { base_path, chain_id: None, base: polkadot_cli::RunCmd::parse_from(relay_chain_args) }
	}
}

impl CliConfiguration<Self> for RelayChainCli {
	fn shared_params(&self) -> &SharedParams {
		self.base.base.shared_params()
	}

	fn import_params(&self) -> Option<&ImportParams> {
		self.base.base.import_params()
	}

	fn network_params(&self) -> Option<&NetworkParams> {
		self.base.base.network_params()
	}

	fn keystore_params(&self) -> Option<&KeystoreParams> {
		self.base.base.keystore_params()
	}

	fn base_path(&self) -> CliResult<Option<BasePath>> {
		Ok(self
			.shared_params()
			.base_path()
			.or_else(|| self.base_path.clone().map(Into::into)))
	}

	fn rpc_http(&self, default_listen_port: u16) -> CliResult<Option<SocketAddr>> {
		self.base.base.rpc_http(default_listen_port)
	}

	fn rpc_ipc(&self) -> CliResult<Option<String>> {
		self.base.base.rpc_ipc()
	}

	fn rpc_ws(&self, default_listen_port: u16) -> CliResult<Option<SocketAddr>> {
		self.base.base.rpc_ws(default_listen_port)
	}

	fn prometheus_config(
		&self,
		default_listen_port: u16,
		chain_spec: &Box<dyn ChainSpec>,
	) -> CliResult<Option<PrometheusConfig>> {
		self.base.base.prometheus_config(default_listen_port, chain_spec)
	}

	fn init<F>(
		&self,
		_support_url: &String,
		_impl_version: &String,
		_logger_hook: F,
		_config: &sc_service::Configuration,
	) -> CliResult<()>
	where
		F: FnOnce(&mut sc_cli::LoggerBuilder, &sc_service::Configuration),
	{
		unreachable!("PolkadotCli is never initialized; qed");
	}

	fn chain_id(&self, is_dev: bool) -> CliResult<String> {
		let chain_id = self.base.base.chain_id(is_dev)?;

		Ok(if chain_id.is_empty() { self.chain_id.clone().unwrap_or_default() } else { chain_id })
	}

	fn role(&self, is_dev: bool) -> CliResult<sc_service::Role> {
		self.base.base.role(is_dev)
	}

	fn transaction_pool(&self) -> CliResult<sc_service::config::TransactionPoolOptions> {
		self.base.base.transaction_pool()
	}

	fn state_cache_child_ratio(&self) -> CliResult<Option<usize>> {
		self.base.base.state_cache_child_ratio()
	}

	fn rpc_methods(&self) -> CliResult<sc_service::config::RpcMethods> {
		self.base.base.rpc_methods()
	}

	fn rpc_ws_max_connections(&self) -> CliResult<Option<usize>> {
		self.base.base.rpc_ws_max_connections()
	}

	fn rpc_cors(&self, is_dev: bool) -> CliResult<Option<Vec<String>>> {
		self.base.base.rpc_cors(is_dev)
	}

	fn default_heap_pages(&self) -> CliResult<Option<u64>> {
		self.base.base.default_heap_pages()
	}

	fn force_authoring(&self) -> CliResult<bool> {
		self.base.base.force_authoring()
	}

	fn disable_grandpa(&self) -> CliResult<bool> {
		self.base.base.disable_grandpa()
	}

	fn max_runtime_instances(&self) -> CliResult<Option<usize>> {
		self.base.base.max_runtime_instances()
	}

	fn announce_block(&self) -> CliResult<bool> {
		self.base.base.announce_block()
	}

	fn telemetry_endpoints(
		&self,
		chain_spec: &Box<dyn ChainSpec>,
	) -> CliResult<Option<sc_telemetry::TelemetryEndpoints>> {
		self.base.base.telemetry_endpoints(chain_spec)
	}

	fn node_name(&self) -> CliResult<String> {
		self.base.base.node_name()
	}
}

impl DefaultConfigurationValues for RelayChainCli {
	fn p2p_listen_port() -> u16 {
		30334
	}

	fn rpc_ws_listen_port() -> u16 {
		9945
	}

	fn rpc_http_listen_port() -> u16 {
		9934
	}

	fn prometheus_listen_port() -> u16 {
		9616
	}
}

impl SubstrateCli for TestCollatorCli {
	fn impl_name() -> String {
		"Polkadot collator".into()
	}

	fn impl_version() -> String {
		String::new()
	}

	fn description() -> String {
		format!(
			"Polkadot collator\n\nThe command-line arguments provided first will be \
		passed to the parachain node, while the arguments provided after -- will be passed \
		to the relaychain node.\n\n\
		{} [parachain-args] -- [relaychain-args]",
			Self::executable_name()
		)
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/paritytech/cumulus/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2017
	}

	fn load_spec(&self, _: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		Ok(Box::new(cumulus_test_service::get_chain_spec(ParaId::from(self.parachain_id)))
			as Box<_>)
	}

	fn native_runtime_version(_: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		&cumulus_test_service::runtime::VERSION
	}
}

impl SubstrateCli for RelayChainCli {
	fn impl_name() -> String {
		"Polkadot collator".into()
	}

	fn impl_version() -> String {
		String::new()
	}

	fn description() -> String {
		format!(
			"Polkadot collator\n\nThe command-line arguments provided first will be \
		passed to the parachain node, while the arguments provided after -- will be passed \
		to the relay chain node.\n\n\
		{} [parachain-args] -- [relay_chain-args]",
			Self::executable_name()
		)
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/paritytech/cumulus/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2017
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		<polkadot_cli::Cli as SubstrateCli>::from_iter(
			[RelayChainCli::executable_name().to_string()].iter(),
		)
		.load_spec(id)
	}

	fn native_runtime_version(chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		polkadot_cli::Cli::native_runtime_version(chain_spec)
	}
}

fn extract_genesis_wasm(chain_spec: &Box<dyn sc_service::ChainSpec>) -> Result<Vec<u8>, String> {
	let mut storage = chain_spec.build_storage()?;

	storage
		.top
		.remove(sp_core::storage::well_known_keys::CODE)
		.ok_or_else(|| "Could not find wasm file in genesis state!".into())
}

fn main() -> Result<(), sc_cli::Error> {
	let cli = TestCollatorCli::parse();

	match &cli.subcommand {
		Some(Commands::ExportGenesisState { output, raw }) => {
			let mut builder = sc_cli::LoggerBuilder::new("");
			builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
			let _ = builder.init();

			let parachain_id = ParaId::from(2000u32);
			let spec = Box::new(cumulus_test_service::get_chain_spec(parachain_id)) as Box<_>;
			let state_version = cumulus_test_service::runtime::VERSION.state_version();

			let block: parachains_common::Block = generate_genesis_block(&spec, state_version)?;
			let raw_header = block.header().encode();
			let output_buf = if *raw {
				raw_header
			} else {
				format!("0x{:?}", HexDisplay::from(&block.header().encode())).into_bytes()
			};

			if let Some(output) = &output {
				std::fs::write(output, output_buf)?;
			} else {
				std::io::stdout().write_all(&output_buf)?;
			}

			Ok(())
		},
		Some(Commands::ExportGenesisWasm { output, raw }) => {
			let mut builder = sc_cli::LoggerBuilder::new("");
			builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
			let _ = builder.init();

			let parachain_id = ParaId::from(2000u32);
			let spec = Box::new(cumulus_test_service::get_chain_spec(parachain_id)) as Box<_>;
			let raw_wasm_blob = extract_genesis_wasm(&spec)?;
			let output_buf = if *raw {
				raw_wasm_blob
			} else {
				format!("0x{:?}", HexDisplay::from(&raw_wasm_blob)).into_bytes()
			};

			if let Some(output) = &output {
				std::fs::write(output, output_buf)?;
			} else {
				std::io::stdout().write_all(&output_buf)?;
			}

			Ok(())
		},
		Some(_) => {
			panic!("Not supported");
		},
		None => {
			let mut builder = sc_cli::LoggerBuilder::new("");
			builder.with_colors(true);
			let _ = builder.init();

			let collator_options = cli.run.collator_options();
			let tokio_runtime = sc_cli::build_runtime()?;
			let tokio_handle = tokio_runtime.handle();
			let mut config = cli
				.run
				.normalize()
				.create_configuration(&cli, tokio_handle.clone())
				.expect("Should be able to generate config");

			let parachain_id = ParaId::from(2000u32);
			config.chain_spec =
				Box::new(cumulus_test_service::get_chain_spec(parachain_id)) as Box<_>;

			let polkadot_cli = RelayChainCli::new(
				&config,
				[RelayChainCli::executable_name().to_string()]
					.iter()
					.chain(cli.relaychain_args.iter()),
			);

			let parachain_account =
				AccountIdConversion::<polkadot_primitives::v2::AccountId>::into_account(
					&parachain_id,
				);

			let state_version =
				RelayChainCli::native_runtime_version(&config.chain_spec).state_version();

			let block: parachains_common::Block =
				generate_genesis_block(&config.chain_spec, state_version)
					.map_err(|e| format!("{:?}", e))?;
			let genesis_state = format!("0x{:?}", HexDisplay::from(&block.header().encode()));

			let tokio_handle = config.tokio_handle.clone();
			let polkadot_config =
				SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, tokio_handle)
					.map_err(|err| format!("Relay chain argument error: {}", err))?;

			tracing::info!("Parachain id: {:?}", parachain_id);
			tracing::info!("Parachain Account: {}", parachain_account);
			tracing::info!("Parachain genesis state: {}", genesis_state);
			tracing::info!(
				"Is collating: {}",
				if config.role.is_authority() { "yes" } else { "no" }
			);

			let collator_key = Some(CollatorPair::generate().0);

			let (mut task_manager, client, network, rpc_handlers, transaction_pool) = tokio_runtime
				.block_on(cumulus_test_service::start_node_impl(
					config,
					collator_key,
					polkadot_config,
					parachain_id,
					None,
					|_| Ok(Default::default()),
					cumulus_test_service::Consensus::RelayChain,
					collator_options,
				))
				.expect("could not create Cumulus test service");

			tokio_runtime.block_on(task_manager.future());
			Ok(())
		},
	}
}
