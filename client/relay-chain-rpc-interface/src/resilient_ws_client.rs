use std::{sync::Arc, time::Instant};

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
		client::{Client as JsonRpcClient, ClientT, Subscription, SubscriptionClientT},
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
	ws_urls: Vec<Url>,
	// Communication channel with the RPC client
	client_receiver: TokioReceiver<RpcDispatcherMessage>,
	self_sender: TokioSender<RpcDispatcherMessage>,

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
			tracing::info!(
				target: LOG_TARGET,
				"Received block via RPC: #{} ({})",
				header.number,
				header.hash()
			);
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
		None => {
			// TODO skunert We should replace the stream at this point
			tracing::error!(target: LOG_TARGET, "The subscription was closed");
			return Err("Subscription closed".to_string());
		},
		Some(Err(err)) => Err(format!("Error in RPC subscription: {}", err)),
	}
}

#[derive(Clone, Debug)]
struct ClientManager {
	urls: Vec<Url>,
	active_client: (usize, Arc<JsonRpcClient>),
}

struct RelayChainSubscriptions {
	import_subscription: Subscription<PHeader>,
	finalized_subscription: Subscription<PHeader>,
	best_subscription: Subscription<PHeader>,
}

async fn find_next_client(
	urls: &Vec<Url>,
	starting_position: usize,
) -> Result<(usize, Arc<JsonRpcClient>), ()> {
	for (counter, url) in urls.iter().cycle().skip(starting_position).take(urls.len()).enumerate() {
		let index = (starting_position + counter) % urls.len();
		tracing::info!(target: LOG_TARGET, index, ?url, "Connecting to RPC node",);
		if let Ok(ws_client) = WsClientBuilder::default().build(url.clone()).await {
			tracing::info!(target: LOG_TARGET, ?url, "Successfully switched.");
			return Ok((index, Arc::new(ws_client)));
		};
	}
	Err(())
}

async fn create_request(
	client: Arc<JsonRpcClient>,
	method: String,
	params: Option<Vec<JsonValue>>,
	response_sender: OneshotSender<Result<JsonValue, JsonRpseeError>>,
) -> Result<(), RpcDispatcherMessage> {
	let resp = client.request(&method, params.clone().map(|p| p.into())).await;

	if let Err(JsonRpseeError::RestartNeeded(_)) = resp {
		return Err(RpcDispatcherMessage::Request(method, params, response_sender));
	}

	if let Err(err) = response_sender.send(resp) {
		tracing::error!(
			target: LOG_TARGET,
			?err,
			"Recipient no longer interested in request result"
		);
	}
	return Ok(());
}

impl ClientManager {
	pub async fn new(urls: Vec<Url>) -> Result<Self, ()> {
		if urls.is_empty() {
			return Err(());
		}
		let active_client = find_next_client(&urls, 0).await?;
		Ok(Self { urls, active_client })
	}

	pub async fn connect_to_new_rpc_server(&mut self) -> Result<Arc<JsonRpcClient>, ()> {
		tracing::info!(
			target: LOG_TARGET,
			is_connected = self.active_client.1.is_connected(),
			"Trying to find new RPC server."
		);
		let new_active = find_next_client(&self.urls, self.active_client.0 + 1).await?;
		self.active_client = new_active;
		Ok(self.active_client.1.clone())
	}

	pub fn get_client(&mut self) -> Arc<JsonRpcClient> {
		return self.active_client.1.clone();
	}
}

impl RpcStreamWorker {
	/// Create new worker. Returns the worker and a channel to register new listeners.
	async fn new(urls: Vec<Url>) -> (RpcStreamWorker, TokioSender<RpcDispatcherMessage>) {
		let (tx, rx) = tokio_channel(100);
		let worker = RpcStreamWorker {
			ws_urls: urls,
			client_receiver: rx,
			self_sender: tx.clone(),
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
		let Ok(mut client_manager) = ClientManager::new(self.ws_urls.clone()).await else {
			tracing::error!(target: LOG_TARGET, "No valid RPC url found. Stopping RPC worker.");
			return;
		};

		let active_client = client_manager.get_client();
		let mut pending_requests = FuturesUnordered::new();

		let Ok(mut subscriptions) = get_subscriptions(&active_client).await else {
			return;
		};

		let mut reset_streams = false;
		loop {
			if reset_streams {
				let Ok(active_client) = client_manager.connect_to_new_rpc_server().await else {
					tracing::error!(
						target: LOG_TARGET,
						"Unable to find valid external RPC server, shutting down."
					);
					return;
				};

				if let Ok(new_subscriptions) = get_subscriptions(&active_client).await {
					subscriptions = new_subscriptions;
				} else {
					return;
				};

				reset_streams = false;
			};

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
						let client = client_manager.get_client();
						pending_requests.push(create_request(client, method, params, response_sender));
					},
					None => {
						tracing::error!(target: LOG_TARGET, "RPC client receiver closed. Stopping RPC Worker.");
						return;
					}
				},
				should_retry = pending_requests.next() => {
					if let Some(Err(req)) = should_retry {
						reset_streams = true;
						tracing::info!(target: LOG_TARGET, ?req, "Retrying request");
						self.self_sender.send(req).await.expect("This should work");
					}
				},
				import_event = subscriptions.import_subscription.next() => {
					if let Err(err) = handle_event_distribution(import_event, &mut self.imported_header_listeners) {
						tracing::error!(target: LOG_TARGET, err, "Encountered error while processing imported header notification.");
						reset_streams = true;
					}
				},
				best_header_event = subscriptions.best_subscription.next() => {
					if let Err(err) = handle_event_distribution(best_header_event, &mut self.best_header_listeners) {
						tracing::error!(target: LOG_TARGET, err, "Encountered error while processing best header notification.");
						reset_streams = true;
					}
				}
				finalized_event = subscriptions.finalized_subscription.next() => {
					if let Err(err) = handle_event_distribution(finalized_event, &mut self.finalized_header_listeners) {
						tracing::error!(target: LOG_TARGET, err, "Encountered error while processing finalized header notification.");
						reset_streams = true;
					}
				}
			}
		}
	}
}

async fn get_imported_header_subscription(
	subscription_client: &Arc<JsonRpcClient>,
) -> Result<Subscription<PHeader>, JsonRpseeError> {
	subscription_client
		.subscribe::<PHeader>("chain_subscribeAllHeads", None, "chain_unsubscribeAllHeads")
		.await
		.map_err(|e| {
			tracing::error!(target: LOG_TARGET, ?e, "Unable to open subscription.");
			e
		})
}

async fn get_best_head_subscription(
	subscription_client: &Arc<JsonRpcClient>,
) -> Result<Subscription<PHeader>, JsonRpseeError> {
	subscription_client
		.subscribe::<PHeader>("chain_subscribeNewHeads", None, "chain_unsubscribeNewHeads")
		.await
		.map_err(|e| {
			tracing::error!(target: LOG_TARGET, ?e, "Unable to open subscription.");
			e
		})
}

async fn get_finalized_head_subscription(
	subscription_client: &Arc<JsonRpcClient>,
) -> Result<Subscription<PHeader>, JsonRpseeError> {
	subscription_client
		.subscribe::<PHeader>(
			"chain_subscribeFinalizedHeads",
			None,
			"chain_unsubscribeFinalizedHeads",
		)
		.await
		.map_err(|e| {
			tracing::error!(target: LOG_TARGET, ?e, "Unable to open subscription.");
			e
		})
}

async fn get_subscriptions(
	active_client: &Arc<JsonRpcClient>,
) -> Result<RelayChainSubscriptions, JsonRpseeError> {
	let import_subscription = get_imported_header_subscription(&active_client).await?;
	let best_subscription = get_best_head_subscription(&active_client).await?;
	let finalized_subscription = get_finalized_head_subscription(&active_client).await?;
	Ok(RelayChainSubscriptions { import_subscription, best_subscription, finalized_subscription })
}
