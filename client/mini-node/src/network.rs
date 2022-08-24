use futures::FutureExt;
use polkadot_core_primitives::Hash;
use polkadot_service::BlockT;
use sc_consensus::ImportQueue;
use sc_network::{NetworkService, SyncState};
use sc_network_common::header_backend::NetworkHeaderBackend;
use sc_network_common::sync::SyncStatus;
use sc_network_light::light_client_requests;
use sc_network_sync::{block_request_handler, state_request_handler};
use sc_service::{error::Error, Configuration, NetworkStarter, SpawnTaskHandle};

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
	TCl: NetworkHeaderBackend<TBl> + 'static,
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

	let chain_sync = DummyChainSync {};

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
	C: NetworkHeaderBackend<B>,
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

struct DummyChainSync {}

impl<B: BlockT> sc_network_common::sync::ChainSync<B> for DummyChainSync {
	fn peer_info(&self, who: &libp2p::PeerId) -> Option<sc_network_common::sync::PeerInfo<B>> {
		todo!()
	}

	fn status(&self) -> sc_network_common::sync::SyncStatus<B> {
		SyncStatus {
			state: SyncState::Idle,
			best_seen_block: None,
			num_peers: 0,
			queued_blocks: 0,
			state_sync: None,
			warp_sync: None,
		}
	}

	fn num_sync_requests(&self) -> usize {
		0
	}

	fn num_downloaded_blocks(&self) -> usize {
		0
	}

	fn num_peers(&self) -> usize {
		0
	}

	fn new_peer(
		&mut self,
		who: libp2p::PeerId,
		best_hash: <B as BlockT>::Hash,
		best_number: polkadot_service::NumberFor<B>,
	) -> Result<
		Option<sc_network_common::sync::message::BlockRequest<B>>,
		sc_network_common::sync::BadPeer,
	> {
		Ok(None)
	}

	fn update_chain_info(
		&mut self,
		best_hash: &<B as BlockT>::Hash,
		best_number: polkadot_service::NumberFor<B>,
	) {
	}

	fn request_justification(
		&mut self,
		hash: &<B as BlockT>::Hash,
		number: polkadot_service::NumberFor<B>,
	) {
	}

	fn clear_justification_requests(&mut self) {}

	fn set_sync_fork_request(
		&mut self,
		peers: Vec<libp2p::PeerId>,
		hash: &<B as BlockT>::Hash,
		number: polkadot_service::NumberFor<B>,
	) {
	}

	fn justification_requests(
		&mut self,
	) -> Box<
		dyn Iterator<Item = (libp2p::PeerId, sc_network_common::sync::message::BlockRequest<B>)>
			+ '_,
	> {
		Box::new(Vec::new().into_iter()) as Box<_>
	}

	fn block_requests(
		&mut self,
	) -> Box<
		dyn Iterator<Item = (&libp2p::PeerId, sc_network_common::sync::message::BlockRequest<B>)>
			+ '_,
	> {
		Box::new(Vec::new().into_iter()) as Box<_>
	}

	fn state_request(
		&mut self,
	) -> Option<(libp2p::PeerId, sc_network_common::sync::OpaqueStateRequest)> {
		None
	}

	fn warp_sync_request(
		&mut self,
	) -> Option<(libp2p::PeerId, sc_network_common::sync::warp::WarpProofRequest<B>)> {
		None
	}

	fn on_block_data(
		&mut self,
		who: &libp2p::PeerId,
		request: Option<sc_network_common::sync::message::BlockRequest<B>>,
		response: sc_network_common::sync::message::BlockResponse<B>,
	) -> Result<sc_network_common::sync::OnBlockData<B>, sc_network_common::sync::BadPeer> {
		todo!()
	}

	fn on_state_data(
		&mut self,
		who: &libp2p::PeerId,
		response: sc_network_common::sync::OpaqueStateResponse,
	) -> Result<sc_network_common::sync::OnStateData<B>, sc_network_common::sync::BadPeer> {
		todo!()
	}

	fn on_warp_sync_data(
		&mut self,
		who: &libp2p::PeerId,
		response: sc_network_common::sync::warp::EncodedProof,
	) -> Result<(), sc_network_common::sync::BadPeer> {
		todo!()
	}

	fn on_block_justification(
		&mut self,
		who: libp2p::PeerId,
		response: sc_network_common::sync::message::BlockResponse<B>,
	) -> Result<sc_network_common::sync::OnBlockJustification<B>, sc_network_common::sync::BadPeer>
	{
		todo!()
	}

	fn on_blocks_processed(
		&mut self,
		imported: usize,
		count: usize,
		results: Vec<(
			Result<
				sc_consensus::BlockImportStatus<polkadot_service::NumberFor<B>>,
				sc_consensus::BlockImportError,
			>,
			<B as BlockT>::Hash,
		)>,
	) -> Box<
		dyn Iterator<
			Item = Result<
				(libp2p::PeerId, sc_network_common::sync::message::BlockRequest<B>),
				sc_network_common::sync::BadPeer,
			>,
		>,
	> {
		Box::new(Vec::new().into_iter()) as Box<_>
	}

	fn on_justification_import(
		&mut self,
		hash: <B as BlockT>::Hash,
		number: polkadot_service::NumberFor<B>,
		success: bool,
	) {
	}

	fn on_block_finalized(
		&mut self,
		hash: &<B as BlockT>::Hash,
		number: polkadot_service::NumberFor<B>,
	) {
	}

	fn push_block_announce_validation(
		&mut self,
		who: libp2p::PeerId,
		hash: <B as BlockT>::Hash,
		announce: sc_network_common::sync::message::BlockAnnounce<<B as BlockT>::Header>,
		is_best: bool,
	) {
	}

	fn poll_block_announce_validation(
		&mut self,
		cx: &mut std::task::Context,
	) -> std::task::Poll<sc_network_common::sync::PollBlockAnnounceValidation<<B as BlockT>::Header>>
	{
		std::task::Poll::Pending
	}

	fn peer_disconnected(
		&mut self,
		who: &libp2p::PeerId,
	) -> Option<sc_network_common::sync::OnBlockData<B>> {
		todo!()
	}

	fn metrics(&self) -> sc_network_common::sync::Metrics {
		todo!()
	}

	fn create_opaque_block_request(
		&self,
		request: &sc_network_common::sync::message::BlockRequest<B>,
	) -> sc_network_common::sync::OpaqueBlockRequest {
		todo!()
	}

	fn encode_block_request(
		&self,
		request: &sc_network_common::sync::OpaqueBlockRequest,
	) -> Result<Vec<u8>, String> {
		todo!()
	}

	fn decode_block_response(
		&self,
		response: &[u8],
	) -> Result<sc_network_common::sync::OpaqueBlockResponse, String> {
		todo!()
	}

	fn block_response_into_blocks(
		&self,
		request: &sc_network_common::sync::message::BlockRequest<B>,
		response: sc_network_common::sync::OpaqueBlockResponse,
	) -> Result<Vec<sc_network_common::sync::message::BlockData<B>>, String> {
		todo!()
	}

	fn encode_state_request(
		&self,
		request: &sc_network_common::sync::OpaqueStateRequest,
	) -> Result<Vec<u8>, String> {
		todo!()
	}

	fn decode_state_response(
		&self,
		response: &[u8],
	) -> Result<sc_network_common::sync::OpaqueStateResponse, String> {
		todo!()
	}
}
