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

use ansi_term::Color;
use cumulus_collator::{prepare_collator_config, CollatorBuilder};
use cumulus_network::DelayedBlockAnnounceValidator;
use futures::{future::ready, FutureExt};
use polkadot_primitives::v0::CollatorPair;
use sc_executor::native_executor_instance;
pub use sc_executor::NativeExecutor;
use sc_informant::OutputFormat;
use sc_service::{error::Error as ServiceError, Configuration, TaskManager};
use std::sync::Arc;
use parachain_runtime::{opaque::Block, RuntimeApi};
use sp_consensus::{import_queue::BasicQueue, BlockImport};

type FullClient = sc_service::TFullClient<Block, RuntimeApi, Executor>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

// Our native executor instance.
native_executor_instance!(
	pub Executor,
	parachain_runtime::api::dispatch,
	parachain_runtime::native_version,
);

/// Create the `PartialComponents` of the node service.
pub fn new_partial(config: &Configuration) -> Result<sc_service::PartialComponents<
	FullClient,
	FullBackend,
	FullSelectChain,
	BasicQueue<Block, <FullClient as BlockImport<Block>>::Transaction>,
	sc_transaction_pool::FullPool<Block, FullClient>,
	()
>, ServiceError> {
	let inherent_data_providers = sp_inherents::InherentDataProviders::new();

	let (client, backend, keystore, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, Executor>(&config)?;
	let client = Arc::new(client);

	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.prometheus_registry(),
		task_manager.spawn_handle(),
		client.clone(),
	);

	let import_queue = cumulus_consensus::import_queue::import_queue(
		client.clone(),
		client.clone(),
		inherent_data_providers.clone(),
		&task_manager.spawn_handle(),
		config.prometheus_registry(),
	)?;

	Ok(sc_service::PartialComponents {
		client,
		backend,
		task_manager,
		import_queue,
		keystore,
		select_chain,
		transaction_pool,
		inherent_data_providers,
		other: (),
	})
}

/// Run a collator node with the given parachain `Configuration` and relaychain `Configuration`
///
/// This function blocks until done.
pub fn run_collator(
	parachain_config: Configuration,
	key: Arc<CollatorPair>,
	mut polkadot_config: polkadot_collator::Configuration,
	id: polkadot_primitives::v0::Id,
) -> sc_service::error::Result<(TaskManager, Arc<FullClient>)> {
	let mut parachain_config = prepare_collator_config(parachain_config);

	parachain_config.informant_output_format = OutputFormat {
		enable_color: true,
		prefix: format!("[{}] ", Color::Yellow.bold().paint("Parachain")),
	};

	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore,
		transaction_pool,
		inherent_data_providers,
		..
	} = new_partial(&parachain_config)?;
	inherent_data_providers
		.register_provider(sp_timestamp::InherentDataProvider)
		.unwrap();

	let block_announce_validator = DelayedBlockAnnounceValidator::new();
	let block_announce_validator_clone = block_announce_validator.clone();

	let (network, network_status_sinks, system_rpc_tx) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &parachain_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			on_demand: None,
			block_announce_validator_builder: Some(Box::new(|_| Box::new(block_announce_validator_clone))),
			finality_proof_request_builder: None,
			finality_proof_provider: None,
		})?;

	if parachain_config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&parachain_config,
			backend.clone(),
			task_manager.spawn_handle(),
			client.clone(),
			network.clone(),
		);
	}

	let proposer_factory = sc_basic_authorship::ProposerFactory::new(
		client.clone(),
		transaction_pool.clone(),
		parachain_config.prometheus_registry(),
	);

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		network: network.clone(),
		client: client.clone(),
		keystore,
		task_manager: &mut task_manager,
		transaction_pool: transaction_pool.clone(),
		telemetry_connection_sinks: Default::default(),
		rpc_extensions_builder: Box::new(|_| ()),
		on_demand: None,
		remote_blockchain: None,
		backend,
		network_status_sinks,
		system_rpc_tx,
		config: parachain_config,
	})?;

	let block_import = client.clone();
	let client = client.clone();
	let announce_block = Arc::new(move |hash, data| network.announce_block(hash, data));
	let builder = CollatorBuilder::new(
		proposer_factory,
		inherent_data_providers,
		block_import,
		client.clone(),
		id,
		client.clone(),
		announce_block,
		block_announce_validator,
	);

	polkadot_config.informant_output_format = OutputFormat {
		enable_color: true,
		prefix: format!("[{}] ", Color::Blue.bold().paint("Relaychain")),
	};

	let (polkadot_future, polkadot_task_manager) =
		polkadot_collator::start_collator(builder, id, key, polkadot_config)?;

	// Make sure the polkadot task manager survives as long as the service.
	let polkadot_future = polkadot_future.then(move |_| {
		let _ = polkadot_task_manager;
		ready(())
	});

	task_manager
		.spawn_essential_handle()
		.spawn("polkadot", polkadot_future);

	Ok((task_manager, client))
}
