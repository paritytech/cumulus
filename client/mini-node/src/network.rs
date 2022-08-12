use futures::FutureExt;
use polkadot_core_primitives::Hash;
use polkadot_service::{BlockT, HeaderMetadata};
use sc_client_api::{BlockBackend, HeaderBackend, ProofProvider};
use sc_consensus::ImportQueue;
use sc_network::{config::SyncMode, NetworkService, NetworkStateInfo};
use sc_network_common::service::NetworkEventStream;
use sc_network_light::light_client_requests;
use sc_network_sync::{block_request_handler, state_request_handler, ChainSync};
use sc_service::{error::Error, Configuration, NetworkStarter, SpawnTaskHandle};
use sp_consensus::block_validation::DefaultBlockAnnounceValidator;
use std::sync::Arc;

pub struct BuildCollatorNetworkParams<'a, TImpQu, TCl> {
	/// The service configuration.
	pub config: &'a Configuration,
	/// A shared client returned by `new_full_parts`.
	pub client: Arc<TCl>,
	/// A handle for spawning tasks.
	pub spawn_handle: SpawnTaskHandle,
	/// An import queue.
	pub import_queue: TImpQu,

	pub genesis_hash: Hash,
}

/// Build the network service, the network status sinks and an RPC sender.
pub fn build_collator_network<TBl, TImpQu, TCl>(
	params: BuildCollatorNetworkParams<TImpQu, TCl>,
) -> Result<(Arc<NetworkService<TBl, <TBl as BlockT>::Hash>>, NetworkStarter), Error>
where
	TBl: BlockT,
	TCl: HeaderMetadata<TBl, Error = sp_blockchain::Error>
		+ HeaderBackend<TBl>
		+ BlockBackend<TBl>
		+ ProofProvider<TBl>
		+ 'static,
	TImpQu: ImportQueue<TBl> + 'static,
{
	let BuildCollatorNetworkParams { config, client, spawn_handle, import_queue, genesis_hash } =
		params;

	let transaction_pool_adapter = Arc::new(sc_network::config::EmptyTransactionPool {});

	let protocol_id = config.protocol_id();

	let block_request_protocol_config =
		block_request_handler::generate_protocol_config(&protocol_id, genesis_hash, None);

	let state_request_protocol_config =
		state_request_handler::generate_protocol_config(&protocol_id, genesis_hash, None);

	let light_client_request_protocol_config =
		light_client_requests::generate_protocol_config(&protocol_id, genesis_hash, None);

	let block_announce_validator = Box::new(DefaultBlockAnnounceValidator);

	let chain_sync = ChainSync::new(
		match config.network.sync_mode {
			SyncMode::Full => sc_network_common::sync::SyncMode::Full,
			SyncMode::Fast { skip_proofs, storage_chain_mode } => {
				sc_network_common::sync::SyncMode::LightState { skip_proofs, storage_chain_mode }
			},
			SyncMode::Warp => sc_network_common::sync::SyncMode::Warp,
		},
		client.clone(),
		block_announce_validator,
		config.network.max_parallel_downloads,
		None,
	)?;

	let network_params = sc_network::config::Params {
		role: config.role.clone(),
		executor: {
			let spawn_handle = Clone::clone(&spawn_handle);
			Some(Box::new(move |fut| {
				spawn_handle.spawn("libp2p-node", Some("networking"), fut);
			}))
		},
		transactions_handler_executor: {
			let spawn_handle = Clone::clone(&spawn_handle);
			Box::new(move |fut| {
				spawn_handle.spawn("network-transactions-handler", Some("networking"), fut);
			})
		},
		fork_id: None,
		bitswap: None,
		chain_sync: Box::new(chain_sync),
		network_config: config.network.clone(),
		chain: client.clone(),
		transaction_pool: transaction_pool_adapter as _,
		import_queue: Box::new(import_queue),
		protocol_id,
		metrics_registry: config.prometheus_config.as_ref().map(|config| config.registry.clone()),
		block_request_protocol_config,
		state_request_protocol_config,
		warp_sync_protocol_config: None,
		light_client_request_protocol_config,
	};

	let network_mut = sc_network::NetworkWorker::new(network_params)?;
	let network = network_mut.service().clone();

	let future = build_network_collator_future(network_mut);

	// TODO: [skunert] Remove this comment
	// TODO: Normally, one is supposed to pass a list of notifications protocols supported by the
	// node through the `NetworkConfiguration` struct. But because this function doesn't know in
	// advance which components, such as GrandPa or Polkadot, will be plugged on top of the
	// service, it is unfortunately not possible to do so without some deep refactoring. To bypass
	// this problem, the `NetworkService` provides a `register_notifications_protocol` method that
	// can be called even after the network has been initialized. However, we want to avoid the
	// situation where `register_notifications_protocol` is called *after* the network actually
	// connects to other peers. For this reason, we delay the process of the network future until
	// the user calls `NetworkStarter::start_network`.
	//
	// This entire hack should eventually be removed in favour of passing the list of protocols
	// through the configuration.
	//
	// See also https://github.com/paritytech/substrate/issues/6827
	let (network_start_tx, network_start_rx) = futures_channel::oneshot::channel();

	// The network worker is responsible for gathering all network messages and processing
	// them. This is quite a heavy task, and at the time of the writing of this comment it
	// frequently happens that this future takes several seconds or in some situations
	// even more than a minute until it has processed its entire queue. This is clearly an
	// issue, and ideally we would like to fix the network future to take as little time as
	// possible, but we also take the extra harm-prevention measure to execute the networking
	// future using `spawn_blocking`.
	spawn_handle.spawn_blocking("network-worker", Some("networking"), async move {
		if network_start_rx.await.is_err() {
			tracing::warn!(
				"The NetworkStart returned as part of `build_network` has been silently dropped"
			);
			// This `return` might seem unnecessary, but we don't want to make it look like
			// everything is working as normal even though the user is clearly misusing the API.
			return;
		}

		future.await
	});

	Ok((network, NetworkStarter::new(network_start_tx)))
}

/// Builds a never-ending future that continuously polls the network.
///
/// The `status_sink` contain a list of senders to send a periodic network status to.
async fn build_network_collator_future<
	B: BlockT,
	H: sc_network::ExHashT,
	C: HeaderBackend<B> + HeaderMetadata<B, Error = sp_blockchain::Error>,
>(
	mut network: sc_network::NetworkWorker<B, H, C>,
) {
	loop {
		futures::select! {
			// Answer incoming RPC requests.

			// The network worker has done something. Nothing special to do, but could be
			// used in the future to perform actions in response of things that happened on
			// the network.
			_ = (&mut network).fuse() => {}
		}
	}
}
