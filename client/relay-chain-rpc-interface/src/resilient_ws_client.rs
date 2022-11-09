use cumulus_primitives_core::relay_chain::Header as PHeader;
use cumulus_relay_chain_interface::{RelayChainError, RelayChainResult};
use futures::{
	channel::mpsc::{Receiver, Sender},
	channel::oneshot::Sender as OneshotSender,
	stream::FuturesUnordered,
	StreamExt,
};
use jsonrpsee::{
	core::{
		client::{Client as JsonRpcClient, ClientT, SubscriptionClientT},
		Error as JsonRpseeError, JsonValue,
	},
	ws_client::WsClientBuilder,
};
use polkadot_service::TaskManager;
use tokio::sync::mpsc::{
	channel as tokio_channel, Receiver as TokioReceiver, Sender as TokioSender,
};
use url::Url;

const NOTIFICATION_CHANNEL_SIZE_LIMIT: usize = 20;
const LOG_TARGET: &str = "pooled-rpc-client";

pub struct PooledClient {
	/// Channel to communicate with the RPC worker
	to_worker_channel: TokioSender<RpcDispatcherMessage>,
}

impl PooledClient {
	pub async fn new(urls: Vec<Url>, task_manager: &mut TaskManager) -> RelayChainResult<Self> {
		tracing::info!(target: "pooled-client", "Instantiating pooled websocket client");

		let (worker, sender) = RpcStreamWorker::new(urls).await;

		task_manager
			.spawn_essential_handle()
			.spawn("relay-chain-rpc-worker", None, worker.run());

		Ok(Self { to_worker_channel: sender })
	}
}

impl PooledClient {}

impl PooledClient {
	pub async fn request<R>(
		&self,
		method: &str,
		params: Option<Vec<JsonValue>>,
	) -> Result<R, jsonrpsee::core::Error>
	where
		R: sp_runtime::DeserializeOwned,
	{
		let (tx, rx) = futures::channel::oneshot::channel();
		let message = RpcDispatcherMessage::Request(method.to_string(), params, tx);
		self.to_worker_channel.send(message).await.expect("worker sending fucked up");
		let value = rx.await.expect("fucked up during deserialization")?;
		serde_json::from_value(value).map_err(JsonRpseeError::ParseError)
	}
	/// Get a stream of new best relay chain headers
	pub fn get_best_heads_stream(&self) -> Result<Receiver<PHeader>, RelayChainError> {
		let (tx, rx) = futures::channel::mpsc::channel::<PHeader>(NOTIFICATION_CHANNEL_SIZE_LIMIT);
		self.send_register_message_to_worker(RpcDispatcherMessage::RegisterBestHeadListener(tx))?;
		Ok(rx)
	}

	/// Get a stream of finalized relay chain headers
	pub fn get_finalized_heads_stream(&self) -> Result<Receiver<PHeader>, RelayChainError> {
		let (tx, rx) = futures::channel::mpsc::channel::<PHeader>(NOTIFICATION_CHANNEL_SIZE_LIMIT);
		self.send_register_message_to_worker(RpcDispatcherMessage::RegisterFinalizationListener(
			tx,
		))?;
		Ok(rx)
	}

	/// Get a stream of all imported relay chain headers
	pub fn get_imported_heads_stream(&self) -> Result<Receiver<PHeader>, RelayChainError> {
		let (tx, rx) = futures::channel::mpsc::channel::<PHeader>(NOTIFICATION_CHANNEL_SIZE_LIMIT);
		self.send_register_message_to_worker(RpcDispatcherMessage::RegisterImportListener(tx))?;
		Ok(rx)
	}

	fn send_register_message_to_worker(
		&self,
		message: RpcDispatcherMessage,
	) -> Result<(), RelayChainError> {
		self.to_worker_channel
			.try_send(message)
			.map_err(|e| RelayChainError::WorkerCommunicationError(e.to_string()))
	}
}

/// Worker messages to register new notification listeners
#[derive(Debug)]
pub enum RpcDispatcherMessage {
	RegisterBestHeadListener(Sender<PHeader>),
	RegisterImportListener(Sender<PHeader>),
	RegisterFinalizationListener(Sender<PHeader>),
	Request(String, Option<Vec<JsonValue>>, OneshotSender<Result<JsonValue, JsonRpseeError>>),
}

/// Worker that should be used in combination with [`RelayChainRpcClient`]. Must be polled to distribute header notifications to listeners.
struct RpcStreamWorker {
	ws_clients: Vec<(Url, JsonRpcClient)>,
	// Communication channel with the RPC client
	client_receiver: TokioReceiver<RpcDispatcherMessage>,

	// Senders to distribute incoming header notifications to
	imported_header_listeners: Vec<Sender<PHeader>>,
	finalized_header_listeners: Vec<Sender<PHeader>>,
	best_header_listeners: Vec<Sender<PHeader>>,
}

fn handle_event_distribution(
	event: Option<Result<PHeader, JsonRpseeError>>,
	senders: &mut Vec<Sender<PHeader>>,
) -> Result<(), String> {
	match event {
		Some(Ok(header)) => {
			senders.retain_mut(|e| {
				match e.try_send(header.clone()) {
					// Receiver has been dropped, remove Sender from list.
					Err(error) if error.is_disconnected() => false,
					// Channel is full. This should not happen.
					// TODO: Improve error handling here
					// https://github.com/paritytech/cumulus/issues/1482
					Err(error) => {
						tracing::error!(target: LOG_TARGET, ?error, "Event distribution channel has reached its limit. This can lead to missed notifications.");
						true
					},
					_ => true,
				}
			});
			Ok(())
		},
		None => Err("RPC Subscription closed.".to_string()),
		Some(Err(err)) => Err(format!("Error in RPC subscription: {}", err)),
	}
}

impl RpcStreamWorker {
	/// Create new worker. Returns the worker and a channel to register new listeners.
	async fn new(urls: Vec<Url>) -> (RpcStreamWorker, TokioSender<RpcDispatcherMessage>) {
		let clients: Vec<(Url, Result<JsonRpcClient, JsonRpseeError>)> =
			futures::future::join_all(urls.into_iter().map(|url| async move {
				(url.clone(), WsClientBuilder::default().build(url.as_str()).await)
			}))
			.await;
		let clients = clients.into_iter().filter_map(|element| {
			match element.1 {
				Ok(client) => Some((element.0, client)),
				_ => {
					tracing::warn!(target: "pooled-client", url = ?element.0, "Unable to connect to provided relay chain.");
					None}
			}
		}).collect();

		let (tx, rx) = tokio_channel(100);
		let worker = RpcStreamWorker {
			ws_clients: clients,
			client_receiver: rx,
			imported_header_listeners: Vec::new(),
			finalized_header_listeners: Vec::new(),
			best_header_listeners: Vec::new(),
		};
		(worker, tx)
	}

	/// Run this worker to drive notification streams.
	/// The worker does two things:
	/// 1. Listen for `NotificationRegisterMessage` and register new listeners for the notification streams
	/// 2. Distribute incoming import, best head and finalization notifications to registered listeners.
	///    If an error occurs during sending, the receiver has been closed and we remove the sender from the list.
	pub async fn run(mut self) {
		let ws_client = &self.ws_clients.first().unwrap().1;
		let mut pending_requests = FuturesUnordered::new();

		let mut import_sub = ws_client
			.subscribe::<PHeader>("chain_subscribeAllHeads", None, "chain_unsubscribeAllHeads")
			.await
			.expect("should not fail")
			.fuse();

		let mut best_head_sub = ws_client
			.subscribe::<PHeader>(
				"chain_subscribeNewHeads",
				None,
				"chain_unsubscribeFinalizedHeads",
			)
			.await
			.expect("should not fail");
		let mut finalized_sub = ws_client
			.subscribe::<PHeader>(
				"chain_subscribeFinalizedHeads",
				None,
				"chain_unsubscribeFinalizedHeads",
			)
			.await
			.expect("should not fail")
			.fuse();

		loop {
			tokio::select! {
				evt = self.client_receiver.recv() => match evt {
					Some(RpcDispatcherMessage::RegisterBestHeadListener(tx)) => {
						self.best_header_listeners.push(tx);
					},
					Some(RpcDispatcherMessage::RegisterImportListener(tx)) => {
						self.imported_header_listeners.push(tx)
					},
					Some(RpcDispatcherMessage::RegisterFinalizationListener(tx)) => {
						self.finalized_header_listeners.push(tx)
					},
					Some(RpcDispatcherMessage::Request(method, params, response_sender)) => {
						pending_requests.push(async move {
							let resp = ws_client.request(&method, params.map(|p| p.into())).await;
							if let Err(err) = response_sender.send(resp) {
								tracing::error!(target: LOG_TARGET, ?err, "Recipient no longer interested in request result");
							}
						});
					},
					None => {
						tracing::error!(target: LOG_TARGET, "RPC client receiver closed. Stopping RPC Worker.");
						return;
					}
				},
				_ = pending_requests.next() => {},
				import_event = import_sub.next() => {
					if let Err(err) = handle_event_distribution(import_event, &mut self.imported_header_listeners) {
						tracing::error!(target: LOG_TARGET, err, "Encountered error while processing imported header notification. Stopping RPC Worker.");
						return;
					}
				},
				best_header_event = best_head_sub.next() => {
					if let Err(err) = handle_event_distribution(best_header_event, &mut self.best_header_listeners) {
						tracing::error!(target: LOG_TARGET, err, "Encountered error while processing best header notification. Stopping RPC Worker.");
						return;
					}
				}
				finalized_event = finalized_sub.next() => {
					if let Err(err) = handle_event_distribution(finalized_event, &mut self.finalized_header_listeners) {
						tracing::error!(target: LOG_TARGET, err, "Encountered error while processing finalized header notification. Stopping RPC Worker.");
						return;
					}
				}
			}
		}
	}
}
