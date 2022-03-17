// Copyright 2017-2022 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

use lru::LruCache;
use polkadot_node_network_protocol::request_response::{
	v1::{AvailableDataFetchingRequest, CollationFetchingRequest},
	IncomingRequest, IncomingRequestReceiver,
};
use polkadot_node_subsystem_util::metrics::{prometheus::Registry, Metrics};
use polkadot_overseer::{
	BlockInfo, DummySubsystem, MetricsTrait, Overseer, OverseerHandle, OverseerMetrics,
	OverseerRuntimeClient, SpawnNamed, KNOWN_LEAVES_CACHE_SIZE,
};
use polkadot_primitives::v1::CollatorPair;
use polkadot_service::{
	overseer::{
		AvailabilityRecoverySubsystem, CollationGenerationSubsystem, CollatorProtocolSubsystem,
		NetworkBridgeSubsystem, ProtocolSide, RuntimeApiSubsystem,
	},
	AuthorityDiscoveryApi, Error, OverseerConnector,
};
use sc_authority_discovery::Service as AuthorityDiscoveryService;
use sp_blockchain::HeaderBackend;
use sp_consensus::{BlockOrigin, Error as ConsensusError, SelectChain};
use sp_runtime::{traits::NumberFor, Justifications};

use std::sync::Arc;

use cumulus_primitives_core::relay_chain::{v2::ParachainHost, Block, BlockId, Hash as PHash};

use polkadot_client::FullBackend;
use polkadot_service::{
	BabeApi, Configuration, ConstructRuntimeApi, FullClient, Handle, IdentifyVariant,
	NativeExecutionDispatch, OverseerGen, RuntimeApiCollection, TaskManager,
};

use sc_telemetry::TelemetryWorkerHandle;
use sp_api::{HeaderT, ProvideRuntimeApi};
use sp_runtime::traits::Block as BlockT;

mod blockchain_rpc_client;
use blockchain_rpc_client::BlockChainRPCClient;

pub struct CollatorOverseerGen;
impl CollatorOverseerGen {
	pub fn generate<'a, Spawner, RuntimeClient>(
		&self,
		connector: OverseerConnector,
		CollatorOverseerGenArgs {
			leaves,
			runtime_client,
			network_service,
			authority_discovery_service,
			collation_req_receiver,
			available_data_req_receiver,
			registry,
			spawner,
			collator_pair,
		}: CollatorOverseerGenArgs<'a, Spawner, RuntimeClient>,
	) -> Result<(Overseer<Spawner, Arc<RuntimeClient>>, OverseerHandle), Error>
	where
		RuntimeClient: 'static + OverseerRuntimeClient + Sync + Send,
		Spawner: 'static + SpawnNamed + Clone + Unpin,
	{
		let metrics = <OverseerMetrics as MetricsTrait>::register(registry)?;

		let builder = Overseer::builder()
			.availability_distribution(DummySubsystem)
			.availability_recovery(AvailabilityRecoverySubsystem::with_chunks_only(
				available_data_req_receiver,
				Metrics::register(registry)?,
			))
			.availability_store(DummySubsystem)
			.bitfield_distribution(DummySubsystem)
			.bitfield_signing(DummySubsystem)
			.candidate_backing(DummySubsystem)
			.candidate_validation(DummySubsystem)
			.pvf_checker(DummySubsystem)
			.chain_api(DummySubsystem)
			.collation_generation(CollationGenerationSubsystem::new(Metrics::register(registry)?))
			.collator_protocol({
				let side = ProtocolSide::Collator(
					network_service.local_peer_id().clone(),
					collator_pair,
					collation_req_receiver,
					Metrics::register(registry)?,
				);
				CollatorProtocolSubsystem::new(side)
			})
			.network_bridge(NetworkBridgeSubsystem::new(
				network_service.clone(),
				authority_discovery_service.clone(),
				Box::new(network_service.clone()),
				Metrics::register(registry)?,
			))
			.provisioner(DummySubsystem)
			.runtime_api(RuntimeApiSubsystem::new(
				runtime_client.clone(),
				Metrics::register(registry)?,
				spawner.clone(),
			))
			.statement_distribution(DummySubsystem)
			.approval_distribution(DummySubsystem)
			.approval_voting(DummySubsystem)
			.gossip_support(DummySubsystem)
			.dispute_coordinator(DummySubsystem)
			.dispute_distribution(DummySubsystem)
			.chain_selection(DummySubsystem)
			.leaves(Vec::from_iter(
				leaves
					.into_iter()
					.map(|BlockInfo { hash, parent_hash: _, number }| (hash, number)),
			))
			.activation_external_listeners(Default::default())
			.span_per_active_leaf(Default::default())
			.active_leaves(Default::default())
			.supports_parachains(runtime_client)
			.known_leaves(LruCache::new(KNOWN_LEAVES_CACHE_SIZE))
			.metrics(metrics)
			.spawner(spawner);

		builder.build_with_connector(connector).map_err(|e| e.into())
	}
}

/// Arguments passed for overseer construction.
pub struct CollatorOverseerGenArgs<'a, Spawner, RuntimeClient>
where
	RuntimeClient: 'static + polkadot_overseer::OverseerRuntimeClient + Sync + Send,
	Spawner: 'static + SpawnNamed + Clone + Unpin,
{
	/// Set of initial relay chain leaves to track.
	pub leaves: Vec<BlockInfo>,
	/// Runtime client generic, providing the `ProvieRuntimeApi` trait besides others.
	pub runtime_client: Arc<RuntimeClient>,
	/// Underlying network service implementation.
	pub network_service: Arc<sc_network::NetworkService<Block, PHash>>,
	/// Underlying authority discovery service.
	pub authority_discovery_service: AuthorityDiscoveryService,
	pub collation_req_receiver: IncomingRequestReceiver<CollationFetchingRequest>,
	pub available_data_req_receiver: IncomingRequestReceiver<AvailableDataFetchingRequest>,
	/// Prometheus registry, commonly used for production systems, less so for test.
	pub registry: Option<&'a Registry>,
	/// Task spawner to be used throughout the overseer and the APIs it provides.
	pub spawner: Spawner,
	/// Determines the behavior of the collator.
	pub collator_pair: CollatorPair,
}

pub struct NewCollator {
	pub task_manager: TaskManager,
	pub overseer_handle: Option<Handle>,
	pub network: Arc<sc_network::NetworkService<Block, <Block as BlockT>::Hash>>,
}

/// TODO This is just copied from polkadot_service
/// Returns the active leaves the overseer should start with.
async fn active_leaves<Client>(
	select_chain: &impl SelectChain<Block>,
	client: &Client,
) -> Result<Vec<BlockInfo>, Error>
where
	Client: HeaderBackend<Block>,
{
	let best_block = select_chain.best_chain().await?;

	let mut leaves = select_chain
		.leaves()
		.await
		.unwrap_or_default()
		.into_iter()
		.filter_map(|hash| {
			let number = client.number(hash).ok()??;

			// Only consider leaves that are in maximum an uncle of the best block.
			if number < best_block.number().saturating_sub(1) {
				return None
			} else if hash == best_block.hash() {
				return None
			};

			let parent_hash = client.header(BlockId::Hash(hash)).ok()??.parent_hash;

			Some(BlockInfo { hash, parent_hash, number })
		})
		.collect::<Vec<_>>();

	// Sort by block number and get the maximum number of leaves
	leaves.sort_by_key(|b| b.number);

	leaves.push(BlockInfo {
		hash: best_block.hash(),
		parent_hash: *best_block.parent_hash(),
		number: *best_block.number(),
	});

	// TODO Move the 4 into a constant
	Ok(leaves.into_iter().rev().take(4).collect())
}

#[derive(Clone)]
struct RPCSelectChain {
	rpc_client: Arc<BlockChainRPCClient>,
}

impl RPCSelectChain {
	pub fn new(rpc_client: Arc<BlockChainRPCClient>) -> Self {
		RPCSelectChain { rpc_client }
	}
}

#[async_trait::async_trait]
impl SelectChain<Block> for RPCSelectChain {
	async fn leaves(&self) -> Result<Vec<<Block as BlockT>::Hash>, ConsensusError> {
		todo!("SelectChain::leaves")
	}

	async fn best_chain(&self) -> Result<<Block as BlockT>::Header, ConsensusError> {
		todo!("SelectChain::best_chain")
	}

	/// Get the best descendent of `target_hash` that we should attempt to
	/// finalize next, if any. It is valid to return the given `target_hash`
	/// itself if no better descendent exists.
	async fn finality_target(
		&self,
		target_hash: <Block as BlockT>::Hash,
		_maybe_max_number: Option<NumberFor<Block>>,
	) -> Result<<Block as BlockT>::Hash, ConsensusError> {
		// TODO We may use the default implementation for this
		todo!("SelectChain::finality_target")
	}
}

pub struct FakeImportQueue {}

impl sc_service::ImportQueue<Block> for FakeImportQueue {
	/// Import bunch of blocks.
	fn import_blocks(
		&mut self,
		origin: BlockOrigin,
		blocks: Vec<sc_consensus::IncomingBlock<Block>>,
	) {
		todo!("ImportQueue::import_blocks")
	}
	/// Import block justifications.
	fn import_justifications(
		&mut self,
		who: libp2p::PeerId,
		hash: PHash,
		number: NumberFor<Block>,
		justifications: Justifications,
	) {
		todo!("ImportQueue::import_blocks")
	}
	/// Polls for actions to perform on the network.
	///
	/// This method should behave in a way similar to `Future::poll`. It can register the current
	/// task and notify later when more actions are ready to be polled. To continue the comparison,
	/// it is as if this method always returned `Poll::Pending`.
	fn poll_actions(
		&mut self,
		cx: &mut futures::task::Context,
		link: &mut dyn sc_consensus::import_queue::Link<Block>,
	) {
		todo!("ImportQueue::import_blocks")
	}
}

pub fn new_mini<RuntimeApi, ExecutorDispatch, OverseerGenerator>(
	mut config: Configuration,
	collator_pair: CollatorPair,
	jaeger_agent: Option<std::net::SocketAddr>,
	telemetry_worker_handle: Option<TelemetryWorkerHandle>,
	overseer_enable_anyways: bool,
	relay_chain_rpc_client: Arc<BlockChainRPCClient>,
) -> Result<NewCollator, Error>
where
	RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi, ExecutorDispatch>>
		+ Send
		+ Sync
		+ 'static,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	ExecutorDispatch: NativeExecutionDispatch + 'static,
	OverseerGenerator: OverseerGen,
{
	// use sc_network::config::IncomingRequest;

	let role = config.role.clone();

	// let basics = polkadot_service::new_partial_basics::<RuntimeApi, ExecutorDispatch>(
	// 	&mut config,
	// 	jaeger_agent,
	// 	telemetry_worker_handle,
	// )?;

	let task_manager = {
		let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
		TaskManager::new(config.tokio_handle.clone(), registry)?
	};

	let prometheus_registry = config.prometheus_registry().cloned();

	let overseer_connector = OverseerConnector::default();
	let overseer_handle = Handle::new(overseer_connector.handle());

	let chain_spec = config.chain_spec.cloned_box();

	let requires_overseer_for_chain_sel = false;

	let disputes_enabled = chain_spec.is_rococo() ||
		chain_spec.is_kusama() ||
		chain_spec.is_westend() ||
		chain_spec.is_versi() ||
		chain_spec.is_wococo();

	// TODO Shortcut, we ignore the chain-selection-subsystem for now
	// let select_chain = if requires_overseer_for_chain_sel {
	// 	let metrics = Metrics::register(prometheus_registry.as_ref())?;

	// 	SelectRelayChain::new_disputes_aware(
	// 		basics.backend.clone(),
	// 		overseer_handle.clone(),
	// 		metrics,
	// 		disputes_enabled,
	// 	)
	// } else {
	// 	SelectRelayChain::new_longest_chain(basics.backend.clone())
	// };

	let select_chain = RPCSelectChain::new(relay_chain_rpc_client.clone());

	let auth_disc_publish_non_global_ips = config.network.allow_non_globals_in_dht;
	{
		use polkadot_network_bridge::{peer_sets_info, IsAuthority};
		let is_authority = if role.is_authority() { IsAuthority::Yes } else { IsAuthority::No };
		config.network.extra_sets.extend(peer_sets_info(is_authority));
	}

	let (collation_req_receiver, cfg) = IncomingRequest::get_config_receiver();
	config.network.request_response_protocols.push(cfg);
	let (available_data_req_receiver, cfg) = IncomingRequest::get_config_receiver();
	config.network.request_response_protocols.push(cfg);

	let import_queue = FakeImportQueue {};
	// let transaction_pool = Arc::new(sc_network::config::EmptyTransactionPool);
	let (network, _, network_starter) =
		sc_service::build_collator_network(sc_service::BuildCollatorNetworkParams {
			config: &config,
			client: relay_chain_rpc_client.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
		})?;

	let overseer_client = relay_chain_rpc_client.clone();
	let spawner = task_manager.spawn_handle();
	// Cannot use the `RelayChainSelection`, since that'd require a setup _and running_ overseer
	// which we are about to setup.
	let active_leaves =
		futures::executor::block_on(active_leaves(&select_chain, &*relay_chain_rpc_client))?;

	let authority_discovery_service = {
		use futures::StreamExt;
		use sc_network::Event;

		let authority_discovery_role = sc_authority_discovery::Role::Discover;
		let dht_event_stream =
			network.event_stream("authority-discovery").filter_map(|e| async move {
				match e {
					Event::Dht(e) => Some(e),
					_ => None,
				}
			});
		let (worker, service) = sc_authority_discovery::new_worker_and_service_with_config(
			sc_authority_discovery::WorkerConfig {
				publish_non_global_ips: auth_disc_publish_non_global_ips,
				..Default::default()
			},
			relay_chain_rpc_client.clone(),
			network.clone(),
			Box::pin(dht_event_stream),
			authority_discovery_role,
			prometheus_registry.clone(),
		);

		task_manager.spawn_handle().spawn(
			"authority-discovery-worker",
			Some("authority-discovery"),
			Box::pin(worker.run()),
		);
		Some(service)
	};

	let overseer_handle = if let Some(authority_discovery_service) = authority_discovery_service {
		let overseer_gen = CollatorOverseerGen;
		let (overseer, overseer_handle) = overseer_gen
			.generate::<sc_service::SpawnTaskHandle, BlockChainRPCClient>(
				overseer_connector,
				CollatorOverseerGenArgs {
					leaves: active_leaves,
					runtime_client: overseer_client.clone(),
					network_service: network.clone(),
					authority_discovery_service,
					collation_req_receiver,
					available_data_req_receiver,
					registry: prometheus_registry.as_ref(),
					spawner,
					collator_pair,
				},
			)
			.map_err(|e| {
				tracing::error!("Failed to init overseer: {}", e);
				e
			})?;
		let handle = Handle::new(overseer_handle.clone());

		{
			let handle = handle.clone();
			task_manager.spawn_essential_handle().spawn_blocking(
				"overseer",
				None,
				Box::pin(async move {
					use futures::{pin_mut, select, FutureExt};

					let forward = polkadot_overseer::forward_events(overseer_client, handle);

					let forward = forward.fuse();
					let overseer_fut = overseer.run().fuse();

					pin_mut!(overseer_fut);
					pin_mut!(forward);

					select! {
						_ = forward => (),
						_ = overseer_fut => (),
						complete => (),
					}
				}),
			);
		}
		Some(handle)
	} else {
		assert!(
			!requires_overseer_for_chain_sel,
			"Precondition congruence (false) is guaranteed by manual checking. qed"
		);
		None
	};

	network_starter.start_network();

	Ok(NewCollator { task_manager, overseer_handle, network })
}
