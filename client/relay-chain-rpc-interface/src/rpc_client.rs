// Copyright 2022 Parity Technologies (UK) Ltd.
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

use backoff::{future::retry_notify, ExponentialBackoff};
use cumulus_primitives_core::{
	relay_chain::{
		v2::{CommittedCandidateReceipt, OccupiedCoreAssumption, SessionIndex, ValidatorId},
		Hash as PHash, Header as PHeader, InboundHrmpMessage,
	},
	InboundDownwardMessage, ParaId, PersistedValidationData,
};
use cumulus_relay_chain_interface::{RelayChainError, RelayChainResult};
use futures::{
	channel::mpsc::{Receiver, Sender},
	StreamExt,
};
use jsonrpsee::{
	core::{
		client::{Client as JsonRPCClient, ClientT, Subscription, SubscriptionClientT},
		Error as JsonRpseeError,
	},
	rpc_params,
	types::ParamsSer,
	ws_client::WsClientBuilder,
};
use parity_scale_codec::{Decode, Encode};
use sc_client_api::StorageData;
use sc_rpc_api::{state::ReadProof, system::Health};
use sc_utils::mpsc::{tracing_unbounded, TracingUnboundedReceiver, TracingUnboundedSender};
use sp_core::sp_std::collections::btree_map::BTreeMap;
use sp_runtime::DeserializeOwned;
use sp_storage::StorageKey;
use std::sync::Arc;

pub use url::Url;

const LOG_TARGET: &str = "relay-chain-rpc-client";

/// Client that maps RPC methods and deserializes results
#[derive(Clone)]
pub struct RelayChainRPCClient {
	/// Websocket client to make calls
	ws_client: Arc<JsonRPCClient>,

	/// Retry strategy that should be used for requests and subscriptions
	retry_strategy: ExponentialBackoff,

	to_worker_channel: Option<TracingUnboundedSender<NotificationRegisterMessage>>,
}

#[derive(Clone, Debug)]
pub enum NotificationRegisterMessage {
	RegisterBestHeadListener(Sender<PHeader>),
	RegisterImportListener(Sender<PHeader>),
	RegisterFinalizationListener(Sender<PHeader>),
}

/// Worker that should be used in combination with [`RelayChainRPCClient`]. Must be polled to distribute header notifications to listeners.
pub struct RPCStreamWorker {
	client_receiver: TracingUnboundedReceiver<NotificationRegisterMessage>,
	imported_heads_listeners: Vec<Sender<PHeader>>,
	finalized_heads_listeners: Vec<Sender<PHeader>>,
	best_heads_listeners: Vec<Sender<PHeader>>,
	import_sub: Subscription<PHeader>,
	finalized_sub: Subscription<PHeader>,
	best_sub: Subscription<PHeader>,
}

/// Entry point to create [`RelayChainRPCClient`] and [`RPCStreamWorker`];
pub async fn create_worker_client(
	url: Url,
) -> RelayChainResult<(RPCStreamWorker, RelayChainRPCClient)> {
	let mut client = RelayChainRPCClient::new(url).await?;
	let best_head_stream = client.subscribe_new_best_heads().await?;
	let finalized_head_stream = client.subscribe_finalized_heads().await?;
	let imported_head_stream = client.subscribe_imported_heads().await?;

	let (worker, sender) =
		RPCStreamWorker::new(imported_head_stream, best_head_stream, finalized_head_stream);
	client.set_worker_channel(sender);
	Ok((worker, client))
}

fn handle_event_distribution(
	event: Option<Result<PHeader, JsonRpseeError>>,
	senders: &mut Vec<Sender<PHeader>>,
) {
	match event {
		Some(Ok(header)) => senders.retain_mut(|e| {
			if let Err(err) = e.try_send(header.clone()) {
				tracing::debug!(
					target: LOG_TARGET,
					?err,
					header = header.number,
					"Unable to send, removing Sender from listeners."
				);
				false
			} else {
				true
			}
		}),
		err @ _ => {
			tracing::error!(target: LOG_TARGET, ?err, "Received error value from stream.");
			return
		},
	};
}

impl RPCStreamWorker {
	/// Create new worker
	fn new(
		import_sub: Subscription<PHeader>,
		best_sub: Subscription<PHeader>,
		finalized_sub: Subscription<PHeader>,
	) -> (RPCStreamWorker, TracingUnboundedSender<NotificationRegisterMessage>) {
		let (tx, rx) = tracing_unbounded("mpsc-cumulus-rpc-worker");
		let worker = RPCStreamWorker {
			client_receiver: rx,
			imported_heads_listeners: Vec::new(),
			finalized_heads_listeners: Vec::new(),
			best_heads_listeners: Vec::new(),
			import_sub,
			best_sub,
			finalized_sub,
		};
		(worker, tx)
	}

	/// Run this worker to drive notification streams.
	/// The worker does two things:
	/// 1. Listen for `NotificationRegisterMessage` and register new listeners for the notification streams
	/// 2. Distribute incoming import, best head and finalization notifications to registered listeners.
	///    If an error occurs during sending, the receiver has been closed and we remove the sender from the list.
	pub async fn run(mut self) {
		let mut import_sub = self.import_sub.fuse();
		let mut best_head_sub = self.best_sub.fuse();
		let mut finalized_sub = self.finalized_sub.fuse();
		loop {
			futures::select! {
				evt = self.client_receiver.next() => match evt {
					Some(NotificationRegisterMessage::RegisterBestHeadListener(tx)) => {
						self.best_heads_listeners.push(tx);
					},
					Some(NotificationRegisterMessage::RegisterImportListener(tx)) => {
						self.imported_heads_listeners.push(tx)
					},
					Some(NotificationRegisterMessage::RegisterFinalizationListener(tx)) => {
						self.finalized_heads_listeners.push(tx)
					},
					None => {}
				},
				import_evt = import_sub.next() =>
					handle_event_distribution(import_evt, &mut self.imported_heads_listeners),
				header_evt = best_head_sub.next() =>
					handle_event_distribution(header_evt, &mut self.best_heads_listeners),
				finalized_evt = finalized_sub.next() =>
					handle_event_distribution(finalized_evt, &mut self.finalized_heads_listeners),
			}
		}
	}
}

impl RelayChainRPCClient {
	/// Set channel to exchange messages with the rpc-worker
	fn set_worker_channel(&mut self, sender: TracingUnboundedSender<NotificationRegisterMessage>) {
		self.to_worker_channel = Some(sender);
	}

	/// Initialize new RPC Client and connect vie WebSocket to a given endpoint.
	async fn new(url: Url) -> RelayChainResult<Self> {
		tracing::info!(target: LOG_TARGET, url = %url.to_string(), "Initializing RPC Client");
		let ws_client = WsClientBuilder::default().build(url.as_str()).await?;

		let client = RelayChainRPCClient {
			to_worker_channel: None,
			ws_client: Arc::new(ws_client),
			retry_strategy: ExponentialBackoff::default(),
		};

		Ok(client)
	}

	/// Call a call to `state_call` rpc method.
	pub async fn call_remote_runtime_function<R: Decode>(
		&self,
		method_name: &str,
		hash: PHash,
		payload: Option<impl Encode>,
	) -> RelayChainResult<R> {
		let payload_bytes =
			payload.map_or(sp_core::Bytes(Vec::new()), |v| sp_core::Bytes(v.encode()));
		let params = rpc_params! {
			method_name,
			payload_bytes,
			hash
		};
		let res = self
			.request_tracing::<sp_core::Bytes, _>("state_call", params, |err| {
				tracing::trace!(
					target: LOG_TARGET,
					%method_name,
					%hash,
					error = %err,
					"Error during call to 'state_call'.",
				);
			})
			.await?;
		Decode::decode(&mut &*res.0).map_err(Into::into)
	}

	/// Subscribe to a notification stream via RPC
	pub async fn subscribe<'a, R>(
		&self,
		sub_name: &'a str,
		unsub_name: &'a str,
		params: Option<ParamsSer<'a>>,
	) -> RelayChainResult<Subscription<R>>
	where
		R: DeserializeOwned,
	{
		self.ws_client
			.subscribe::<R>(sub_name, params, unsub_name)
			.await
			.map_err(|err| RelayChainError::RPCCallError(sub_name.to_string(), err))
	}

	/// Perform RPC request
	async fn request<'a, R>(
		&self,
		method: &'a str,
		params: Option<ParamsSer<'a>>,
	) -> Result<R, RelayChainError>
	where
		R: DeserializeOwned + std::fmt::Debug,
	{
		self.request_tracing(
			method,
			params,
			|e| tracing::trace!(target:LOG_TARGET, error = %e, %method, "Unable to complete RPC request"),
		)
		.await
	}

	/// Perform RPC request
	async fn request_tracing<'a, R, OR>(
		&self,
		method: &'a str,
		params: Option<ParamsSer<'a>>,
		trace_error: OR,
	) -> Result<R, RelayChainError>
	where
		R: DeserializeOwned + std::fmt::Debug,
		OR: Fn(&jsonrpsee::core::Error),
	{
		retry_notify(
			self.retry_strategy.clone(),
			|| async {
				self.ws_client.request(method, params.clone()).await.map_err(|err| match err {
					JsonRpseeError::Transport(_) =>
						backoff::Error::Transient { err, retry_after: None },
					_ => backoff::Error::Permanent(err),
				})
			},
			|error, dur| tracing::trace!(target: LOG_TARGET, %error, ?dur, "Encountered transport error, retrying."),
		)
		.await
		.map_err(|err| {
			trace_error(&err);
			RelayChainError::RPCCallError(method.to_string(), err)})
	}

	pub async fn system_health(&self) -> Result<Health, RelayChainError> {
		self.request("system_health", None).await
	}

	pub async fn state_get_read_proof(
		&self,
		storage_keys: Vec<StorageKey>,
		at: Option<PHash>,
	) -> Result<ReadProof<PHash>, RelayChainError> {
		let params = rpc_params!(storage_keys, at);
		self.request("state_getReadProof", params).await
	}

	pub async fn state_get_storage(
		&self,
		storage_key: StorageKey,
		at: Option<PHash>,
	) -> Result<Option<StorageData>, RelayChainError> {
		let params = rpc_params!(storage_key, at);
		self.request("state_getStorage", params).await
	}

	pub async fn chain_get_head(&self) -> Result<PHash, RelayChainError> {
		self.request("chain_getHead", None).await
	}

	pub async fn chain_get_header(
		&self,
		hash: Option<PHash>,
	) -> Result<Option<PHeader>, RelayChainError> {
		let params = rpc_params!(hash);
		self.request("chain_getHeader", params).await
	}

	pub async fn parachain_host_candidate_pending_availability(
		&self,
		at: PHash,
		para_id: ParaId,
	) -> Result<Option<CommittedCandidateReceipt>, RelayChainError> {
		self.call_remote_runtime_function(
			"ParachainHost_candidate_pending_availability",
			at,
			Some(para_id),
		)
		.await
	}

	pub async fn parachain_host_session_index_for_child(
		&self,
		at: PHash,
	) -> Result<SessionIndex, RelayChainError> {
		self.call_remote_runtime_function("ParachainHost_session_index_for_child", at, None::<()>)
			.await
	}

	pub async fn parachain_host_validators(
		&self,
		at: PHash,
	) -> Result<Vec<ValidatorId>, RelayChainError> {
		self.call_remote_runtime_function("ParachainHost_validators", at, None::<()>)
			.await
	}

	pub async fn parachain_host_persisted_validation_data(
		&self,
		at: PHash,
		para_id: ParaId,
		occupied_core_assumption: OccupiedCoreAssumption,
	) -> Result<Option<PersistedValidationData>, RelayChainError> {
		self.call_remote_runtime_function(
			"ParachainHost_persisted_validation_data",
			at,
			Some((para_id, occupied_core_assumption)),
		)
		.await
	}

	pub async fn parachain_host_inbound_hrmp_channels_contents(
		&self,
		para_id: ParaId,
		at: PHash,
	) -> Result<BTreeMap<ParaId, Vec<InboundHrmpMessage>>, RelayChainError> {
		self.call_remote_runtime_function(
			"ParachainHost_inbound_hrmp_channels_contents",
			at,
			Some(para_id),
		)
		.await
	}

	pub async fn parachain_host_dmq_contents(
		&self,
		para_id: ParaId,
		at: PHash,
	) -> Result<Vec<InboundDownwardMessage>, RelayChainError> {
		self.call_remote_runtime_function("ParachainHost_dmq_contents", at, Some(para_id))
			.await
	}

	pub async fn get_imported_heads_stream(&self) -> Result<Receiver<PHeader>, RelayChainError> {
		let (tx, rx) = futures::channel::mpsc::channel::<PHeader>(128);
		if let Some(channel) = &self.to_worker_channel {
			channel
				.unbounded_send(NotificationRegisterMessage::RegisterImportListener(tx))
				.map_err(|e| RelayChainError::WorkerCommunicationError(e.to_string()))?;
		}
		Ok(rx)
	}

	pub async fn get_best_heads_stream(&self) -> Result<Receiver<PHeader>, RelayChainError> {
		let (tx, rx) = futures::channel::mpsc::channel::<PHeader>(128);
		if let Some(channel) = &self.to_worker_channel {
			channel
				.unbounded_send(NotificationRegisterMessage::RegisterBestHeadListener(tx))
				.map_err(|e| RelayChainError::WorkerCommunicationError(e.to_string()))?;
		}
		Ok(rx)
	}

	pub async fn get_finalized_heads_stream(&self) -> Result<Receiver<PHeader>, RelayChainError> {
		let (tx, rx) = futures::channel::mpsc::channel::<PHeader>(128);
		if let Some(channel) = &self.to_worker_channel {
			channel
				.unbounded_send(NotificationRegisterMessage::RegisterFinalizationListener(tx))
				.map_err(|e| RelayChainError::WorkerCommunicationError(e.to_string()))?;
		}
		Ok(rx)
	}

	async fn subscribe_imported_heads(&self) -> Result<Subscription<PHeader>, RelayChainError> {
		self.subscribe::<PHeader>("chain_subscribeAllHeads", "chain_unsubscribeAllHeads", None)
			.await
	}

	async fn subscribe_finalized_heads(&self) -> Result<Subscription<PHeader>, RelayChainError> {
		self.subscribe::<PHeader>(
			"chain_subscribeFinalizedHeads",
			"chain_unsubscribeFinalizedHeads",
			None,
		)
		.await
	}

	async fn subscribe_new_best_heads(&self) -> Result<Subscription<PHeader>, RelayChainError> {
		self.subscribe::<PHeader>("chain_subscribeNewHeads", "chain_unsubscribeNewHeads", None)
			.await
	}
}
