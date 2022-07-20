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
	channel::mpsc::{UnboundedReceiver, UnboundedSender},
	FutureExt, StreamExt,
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

	to_worker_channel: Option<UnboundedSender<RPCMessages>>,
}

#[derive(Clone, Debug)]
pub enum RPCMessages {
	NewHeadListener(UnboundedSender<PHeader>),
	NewImportListener(UnboundedSender<PHeader>),
	NewFinalizedListener(UnboundedSender<PHeader>),
}

pub struct RPCWorker {
	client_receiver: UnboundedReceiver<RPCMessages>,
	import_listener: Vec<UnboundedSender<PHeader>>,
	finalized_listener: Vec<UnboundedSender<PHeader>>,
	head_listener: Vec<UnboundedSender<PHeader>>,
	import_sub: Subscription<PHeader>,
	finalized_sub: Subscription<PHeader>,
	best_sub: Subscription<PHeader>,
}

pub async fn create_worker_client(url: Url) -> RelayChainResult<(RPCWorker, RelayChainRPCClient)> {
	let (mut client, import, head, finalized) = RelayChainRPCClient::new_with_streams(url).await?;
	let (worker, sender) = RPCWorker::new(import, head, finalized);
	client.set_worker_channel(sender);
	Ok((worker, client))
}

impl RPCWorker {
	pub fn new(
		import_sub: Subscription<PHeader>,
		best_sub: Subscription<PHeader>,
		finalized_sub: Subscription<PHeader>,
	) -> (RPCWorker, UnboundedSender<RPCMessages>) {
		let (tx, rx) = futures::channel::mpsc::unbounded();
		let worker = RPCWorker {
			client_receiver: rx,
			import_listener: Vec::new(),
			finalized_listener: Vec::new(),
			head_listener: Vec::new(),
			import_sub,
			best_sub,
			finalized_sub,
		};
		(worker, tx)
	}

	pub async fn run(mut self) {
		loop {
			futures::select! {
				evt = self.client_receiver.next().fuse() => match evt {
					Some(RPCMessages::NewHeadListener(tx)) => { self.head_listener.push(tx) },
					Some(RPCMessages::NewImportListener(tx)) => { self.import_listener.push(tx) },
					Some(RPCMessages::NewFinalizedListener(tx)) => {self.finalized_listener.push(tx)  },
					None => {}
				},
				import_evt = self.import_sub.next().fuse() => {
					match import_evt {
						Some(Ok(header)) => self.import_listener.iter().for_each(|e| {
							tracing::info!(target:LOG_TARGET, header = header.number, "Sending header to import listener.");
							if let Err(err) = e.unbounded_send(header.clone()) {
								tracing::error!(target:LOG_TARGET, ?err, "Unable to send to import stream.");
								return;
							};
						}),
						_ => {
							tracing::error!(target:LOG_TARGET, "Received some non-ok value from import stream.");
							return;
						}
					};
				},
				header_evt = self.best_sub.next().fuse() => {
					match header_evt {
						Some(Ok(header)) => self.head_listener.iter().for_each(|e| {
							tracing::info!(target:LOG_TARGET, header = header.number, "Sending header to best head listener.");
							if let Err(err) = e.unbounded_send(header.clone()) {
								tracing::error!(target:LOG_TARGET, ?err,  "Unable to send to best-block stream.");
								return;
							};
						}),
						_ => {
							tracing::error!(target:LOG_TARGET, "Received some non-ok value from best-block stream.");
							return;
						}
					};
				},
				finalized_evt = self.finalized_sub.next().fuse() => {
					match finalized_evt {
						Some(Ok(header)) => self.finalized_listener.iter().for_each(|e| {
							tracing::info!(target:LOG_TARGET, header = header.number, "Sending header to finalized head listener.");
							if let Err(err) = e.unbounded_send(header.clone()) {
								tracing::error!(target:LOG_TARGET, ?err, "Unable to send to best-block stream.");
								return;
							};
						}),
						_ => {
							tracing::error!(target:LOG_TARGET, "Received some non-ok value from finalized stream.");
							return;
						}
					};
				},
			}
		}
	}
}

impl RelayChainRPCClient {
	fn set_worker_channel(&mut self, sender: UnboundedSender<RPCMessages>) {
		self.to_worker_channel = Some(sender);
	}

	pub async fn new_with_streams(
		url: Url,
	) -> RelayChainResult<(Self, Subscription<PHeader>, Subscription<PHeader>, Subscription<PHeader>)>
	{
		tracing::info!(target: LOG_TARGET, url = %url.to_string(), "Initializing RPC Client");
		let ws_client = WsClientBuilder::default().build(url.as_str()).await?;

		let client = RelayChainRPCClient {
			to_worker_channel: None,
			ws_client: Arc::new(ws_client),
			retry_strategy: ExponentialBackoff::default(),
		};
		let head_stream = client.subscribe_new_best_heads().await?;
		let finalized_stream = client.subscribe_finalized_heads().await?;
		let imported_stream = client.subscribe_imported_heads().await?;
		Ok((client, imported_stream, head_stream, finalized_stream))
	}

	pub async fn new(url: Url) -> RelayChainResult<Self> {
		tracing::info!(target: LOG_TARGET, url = %url.to_string(), "Initializing RPC Client");
		let ws_client = WsClientBuilder::default().build(url.as_str()).await?;

		Ok(RelayChainRPCClient {
			to_worker_channel: None,
			ws_client: Arc::new(ws_client),
			retry_strategy: ExponentialBackoff::default(),
		})
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

	pub async fn get_imported_heads_stream(
		&self,
	) -> Result<UnboundedReceiver<PHeader>, RelayChainError> {
		let (tx, rx) = futures::channel::mpsc::unbounded::<PHeader>();
		if let Some(channel) = &self.to_worker_channel {
			tracing::info!(target: LOG_TARGET, "Registering 'NewImportListener'");
			channel
				.unbounded_send(RPCMessages::NewImportListener(tx))
				.map_err(|e| RelayChainError::WorkerCommunicationError(e.to_string()))?;
		}
		Ok(rx)
	}

	pub async fn get_best_heads_stream(
		&self,
	) -> Result<UnboundedReceiver<PHeader>, RelayChainError> {
		let (tx, rx) = futures::channel::mpsc::unbounded::<PHeader>();
		if let Some(channel) = &self.to_worker_channel {
			tracing::info!(target: LOG_TARGET, "Registering 'NewHeadListener'");
			channel
				.unbounded_send(RPCMessages::NewHeadListener(tx))
				.map_err(|e| RelayChainError::WorkerCommunicationError(e.to_string()))?;
		}
		Ok(rx)
	}

	pub async fn get_finalized_heads_stream(
		&self,
	) -> Result<UnboundedReceiver<PHeader>, RelayChainError> {
		let (tx, rx) = futures::channel::mpsc::unbounded::<PHeader>();
		if let Some(channel) = &self.to_worker_channel {
			tracing::info!(target: LOG_TARGET, "Registering 'NewFinalizedListener'");
			channel
				.unbounded_send(RPCMessages::NewFinalizedListener(tx))
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
