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
use polkadot_node_network_protocol::{
	request_response::{
		v1::{AvailableDataFetchingRequest, CollationFetchingRequest},
		IncomingRequest, IncomingRequestReceiver, ReqProtocolNames,
	},
	PeerId,
};
use polkadot_node_subsystem_types::RuntimeApiSubsystemClient;
use polkadot_node_subsystem_util::metrics::{prometheus::Registry, Metrics};
use polkadot_overseer::{
	BlockInfo, DummySubsystem, MetricsTrait, Overseer, OverseerHandle, OverseerMetrics, SpawnGlue,
	KNOWN_LEAVES_CACHE_SIZE,
};
use polkadot_primitives::v2::CollatorPair;
use polkadot_service::overseer::NetworkBridgeMetrics;
use polkadot_service::{
	overseer::{
		AvailabilityRecoverySubsystem, CollationGenerationSubsystem, CollatorProtocolSubsystem,
		NetworkBridgeRxSubsystem, NetworkBridgeTxSubsystem, ProtocolSide, RuntimeApiSubsystem,
	},
	Error, OverseerConnector,
};
use sc_authority_discovery::Service as AuthorityDiscoveryService;
use sc_network::{Event, NetworkService, NetworkStateInfo};
use sc_network_common::{header_backend::NetworkHeaderBackend, service::NetworkEventStream};
use sp_consensus::BlockOrigin;
use sp_core::traits::SpawnNamed;
use sp_runtime::{traits::NumberFor, Justifications};

use std::sync::Arc;

use cumulus_primitives_core::relay_chain::{Block, Hash as PHash};

use polkadot_service::{Configuration, Handle, TaskManager};

use futures::{select, StreamExt};

use sp_runtime::traits::Block as BlockT;

mod blockchain_rpc_client;
mod network;
pub use blockchain_rpc_client::BlockChainRpcClient;

/// Arguments passed for overseer construction.
pub struct CollatorOverseerGenArgs<'a, Spawner, RuntimeClient>
where
	RuntimeClient: 'static + RuntimeApiSubsystemClient + Sync + Send,
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
	pub req_protocol_names: ReqProtocolNames,
}

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
			req_protocol_names,
		}: CollatorOverseerGenArgs<'a, Spawner, RuntimeClient>,
	) -> Result<(Overseer<SpawnGlue<Spawner>, Arc<RuntimeClient>>, OverseerHandle), Error>
	where
		RuntimeClient: 'static + RuntimeApiSubsystemClient + Sync + Send,
		Spawner: 'static + SpawnNamed + Clone + Unpin,
	{
		let metrics = <OverseerMetrics as MetricsTrait>::register(registry)?;

		let spawner = SpawnGlue(spawner);
		let network_bridge_metrics: NetworkBridgeMetrics = Metrics::register(registry)?;
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
			.network_bridge_rx(NetworkBridgeRxSubsystem::new(
				network_service.clone(),
				authority_discovery_service.clone(),
				Box::new(network_service.clone()),
				Metrics::register(registry)?,
			))
			.network_bridge_tx(NetworkBridgeTxSubsystem::new(
				network_service.clone(),
				authority_discovery_service.clone(),
				network_bridge_metrics.clone(),
				req_protocol_names,
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

pub struct NewCollator {
	pub task_manager: TaskManager,
	pub overseer_handle: Handle,
	pub network: Arc<sc_network::NetworkService<Block, <Block as BlockT>::Hash>>,
}

pub struct DummyImportQueue {}

impl sc_service::ImportQueue<Block> for DummyImportQueue {
	/// Import bunch of blocks.
	fn import_blocks(
		&mut self,
		_origin: BlockOrigin,
		_blocks: Vec<sc_consensus::IncomingBlock<Block>>,
	) {
	}
	/// Import block justifications.
	fn import_justifications(
		&mut self,
		_who: PeerId,
		_hash: PHash,
		_number: NumberFor<Block>,
		_justifications: Justifications,
	) {
	}

	/// Polls for actions to perform on the network.
	///
	/// This method should behave in a way similar to `Future::poll`. It can register the current
	/// task and notify later when more actions are ready to be polled. To continue the comparison,
	/// it is as if this method always returned `Poll::Pending`.
	fn poll_actions(
		&mut self,
		_cx: &mut futures::task::Context,
		_link: &mut dyn sc_consensus::import_queue::Link<Block>,
	) {
	}
}

fn build_authority_discovery_service<Block: BlockT>(
	task_manager: &TaskManager,
	client: Arc<BlockChainRpcClient>,
	config: &Configuration,
	network: Arc<NetworkService<Block, <Block as BlockT>::Hash>>,
	prometheus_registry: Option<Registry>,
) -> AuthorityDiscoveryService {
	let auth_disc_publish_non_global_ips = config.network.allow_non_globals_in_dht;
	let authority_discovery_role = sc_authority_discovery::Role::Discover;
	let dht_event_stream = network.event_stream("authority-discovery").filter_map(|e| async move {
		match e {
			Event::Dht(e) => Some(e),
			_ => None,
		}
	});
	let (worker, service) = sc_authority_discovery::new_worker_and_service_with_config(
		sc_authority_discovery::WorkerConfig {
			publish_non_global_ips: auth_disc_publish_non_global_ips,
			// Require that authority discovery records are signed.
			strict_record_validation: true,
			..Default::default()
		},
		client,
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
	service
}

#[sc_tracing::logging::prefix_logs_with("Relaychain")]
pub async fn new_mini(
	mut config: Configuration,
	collator_pair: CollatorPair,
	relay_chain_rpc_client: Arc<BlockChainRpcClient>,
) -> Result<NewCollator, Error> {
	let role = config.role.clone();

	let task_manager = {
		let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
		TaskManager::new(config.tokio_handle.clone(), registry)?
	};

	let prometheus_registry = config.prometheus_registry().cloned();

	let overseer_connector = OverseerConnector::default();

	{
		use polkadot_network_bridge::{peer_sets_info, IsAuthority};
		let is_authority = if role.is_authority() { IsAuthority::Yes } else { IsAuthority::No };
		config.network.extra_sets.extend(peer_sets_info(is_authority));
	}

	let import_queue = DummyImportQueue {};
	let genesis_hash = relay_chain_rpc_client
		.block_get_hash(Some(0))
		.await
		.expect("Crash here if no genesis is available")
		.unwrap_or_default();

	let request_protocol_names = ReqProtocolNames::new(genesis_hash, config.chain_spec.fork_id());
	let (collation_req_receiver, cfg) =
		IncomingRequest::get_config_receiver(&request_protocol_names);
	config.network.request_response_protocols.push(cfg);
	let (available_data_req_receiver, cfg) =
		IncomingRequest::get_config_receiver(&request_protocol_names);
	config.network.request_response_protocols.push(cfg);

	let (network, network_starter) =
		network::build_collator_network(network::BuildCollatorNetworkParams {
			config: &config,
			client: relay_chain_rpc_client.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			genesis_hash,
		})?;

	let active_leaves = Vec::new();

	let authority_discovery_service = build_authority_discovery_service(
		&task_manager,
		relay_chain_rpc_client.clone(),
		&config,
		network.clone(),
		prometheus_registry.clone(),
	);

	let overseer_gen = CollatorOverseerGen;
	let (overseer, overseer_handle) = overseer_gen
		.generate::<sc_service::SpawnTaskHandle, BlockChainRpcClient>(
			overseer_connector,
			CollatorOverseerGenArgs {
				leaves: active_leaves,
				runtime_client: relay_chain_rpc_client.clone(),
				network_service: network.clone(),
				authority_discovery_service,
				collation_req_receiver,
				available_data_req_receiver,
				registry: prometheus_registry.as_ref(),
				spawner: task_manager.spawn_handle(),
				collator_pair,
				req_protocol_names: ReqProtocolNames::new(
					genesis_hash,
					config.chain_spec.fork_id(),
				),
			},
		)
		.map_err(|e| {
			tracing::error!("Failed to init overseer: {}", e);
			e
		})?;

	let overseer_handle = Handle::new(overseer_handle.clone());

	{
		let handle = overseer_handle.clone();
		task_manager.spawn_essential_handle().spawn_blocking(
			"overseer",
			None,
			Box::pin(async move {
				use futures::{pin_mut, FutureExt};

				let forward = forward_collator_events(relay_chain_rpc_client, handle);

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

	network_starter.start_network();

	Ok(NewCollator { task_manager, overseer_handle, network })
}

/// Glues together the [`Overseer`] and `BlockchainEvents` by forwarding
/// import and finality notifications into the [`OverseerHandle`].
pub async fn forward_collator_events(client: Arc<BlockChainRpcClient>, mut handle: Handle) {
	let mut finality = client.finality_notification_stream_async().await.fuse();
	let mut imports = client.import_notification_stream_async().await.fuse();

	loop {
		select! {
			f = finality.next() => {
				match f {
					Some(header) => {
						handle.block_finalized(header.into()).await;
					}
					None => break,
				}
			},
			i = imports.next() => {
				match i {
					Some(header) => {
						handle.block_imported(header.into()).await;
					}
					None => break,
				}
			},
			complete => break,
		}
	}
}
