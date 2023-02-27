// Copyright 2023 Parity Technologies (UK) Ltd.
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

use core::time::Duration;
use cumulus_primitives_core::relay_chain::Header as RelayHeader;
use cumulus_relay_chain_interface::{RelayChainError, RelayChainInterface, RelayChainResult};
use futures::{
	channel::{
		mpsc::{self, Sender},
		oneshot::Sender as OneshotSender,
	},
	prelude::*,
	task::Poll,
};
use jsonrpsee::{
	core::{error::Error as JsonRpseeError, params::ArrayParams, traits::ToRpcParams},
	rpc_params,
	types::{
		response::SubscriptionResponse, Id, NotificationSer, RequestSer, SubscriptionId,
		TwoPointZero,
	},
};
use polkadot_service::{CollatorPair, Configuration, TaskManager};
use sc_service::SpawnTaskHandle;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value as JsonValue};
use smoldot::libp2p::{multiaddr::ProtocolRef, websocket, Multiaddr};
use smoldot_light::{
	platform::{ConnectError, PlatformConnection, PlatformSubstreamDirection, ReadBuffer},
	ChainId, Client, ClientConfig, JsonRpcResponses,
};
use std::{
	collections::{HashMap, VecDeque},
	io::IoSlice,
	net::{IpAddr, SocketAddr},
	path::PathBuf,
	pin::Pin,
	sync::Arc,
};
use tokio::{
	net::TcpStream,
	sync::mpsc::{channel as tokio_channel, Receiver, Sender as TokioSender},
};
use tokio_util::compat::TokioAsyncReadCompatExt;

use crate::reconnecting_ws_client::RpcDispatcherMessage;

const LOG_TARGET: &str = "rpc-light-client-worker";

pub struct TokioPlatform;

/// Implementation detail of [`AsyncStdTcpWebSocket`].
pub struct TokioStream {
	shared: Arc<StreamShared>,
	read_data_rx: Arc<parking_lot::Mutex<stream::Peekable<mpsc::Receiver<Vec<u8>>>>>,
	read_buffer: Option<Vec<u8>>,
}

struct StreamShared {
	guarded: parking_lot::Mutex<StreamSharedGuarded>,
	write_queue_pushed: event_listener::Event,
}

struct StreamSharedGuarded {
	write_queue: VecDeque<u8>,
}
impl smoldot_light::platform::Platform for TokioPlatform {
	type Delay = future::BoxFuture<'static, ()>;
	type Yield = future::Ready<()>;
	type Instant = std::time::Instant;
	type Connection = std::convert::Infallible;
	type Stream = TokioStream;
	type ConnectFuture = future::BoxFuture<
		'static,
		Result<PlatformConnection<Self::Stream, Self::Connection>, ConnectError>,
	>;
	type StreamDataFuture<'a> = future::BoxFuture<'a, ()>;
	type NextSubstreamFuture<'a> =
		future::Pending<Option<(Self::Stream, PlatformSubstreamDirection)>>;

	fn now_from_unix_epoch() -> Duration {
		// Intentionally panic if the time is configured earlier than the UNIX EPOCH.
		std::time::UNIX_EPOCH.elapsed().unwrap()
	}

	fn now() -> Self::Instant {
		std::time::Instant::now()
	}

	fn sleep(duration: Duration) -> Self::Delay {
		tokio::time::sleep(duration).boxed()
	}

	fn sleep_until(when: Self::Instant) -> Self::Delay {
		let duration = when.saturating_duration_since(std::time::Instant::now());
		Self::sleep(duration)
	}

	fn yield_after_cpu_intensive() -> Self::Yield {
		// No-op.
		future::ready(())
	}

	fn connect(multiaddr: &str) -> Self::ConnectFuture {
		// We simply copy the address to own it. We could be more zero-cost here, but doing so
		// would considerably complicate the implementation.
		let multiaddr = multiaddr.to_owned();

		Box::pin(async move {
			let addr = multiaddr.parse::<Multiaddr>().map_err(|_| ConnectError {
				is_bad_addr: true,
				message: format!("Failed to parse address"),
			})?;

			let mut iter = addr.iter().fuse();
			let proto1 = iter.next().ok_or(ConnectError {
				is_bad_addr: true,
				message: format!("Unknown protocols combination"),
			})?;
			let proto2 = iter.next().ok_or(ConnectError {
				is_bad_addr: true,
				message: format!("Unknown protocols combination"),
			})?;
			let proto3 = iter.next();

			if iter.next().is_some() {
				return Err(ConnectError {
					is_bad_addr: true,
					message: format!("Unknown protocols combination"),
				})
			}

			// TODO: doesn't support WebSocket secure connections

			// Ensure ahead of time that the multiaddress is supported.
			let (addr, host_if_websocket) = match (&proto1, &proto2, &proto3) {
				(ProtocolRef::Ip4(ip), ProtocolRef::Tcp(port), None) =>
					(either::Left(SocketAddr::new(IpAddr::V4((*ip).into()), *port)), None),
				(ProtocolRef::Ip6(ip), ProtocolRef::Tcp(port), None) =>
					(either::Left(SocketAddr::new(IpAddr::V6((*ip).into()), *port)), None),
				(ProtocolRef::Ip4(ip), ProtocolRef::Tcp(port), Some(ProtocolRef::Ws)) => {
					let addr = SocketAddr::new(IpAddr::V4((*ip).into()), *port);
					(either::Left(addr), Some(addr.to_string()))
				},
				(ProtocolRef::Ip6(ip), ProtocolRef::Tcp(port), Some(ProtocolRef::Ws)) => {
					let addr = SocketAddr::new(IpAddr::V6((*ip).into()), *port);
					(either::Left(addr), Some(addr.to_string()))
				},

				// TODO: we don't care about the differences between Dns, Dns4, and Dns6
				(
					ProtocolRef::Dns(addr) | ProtocolRef::Dns4(addr) | ProtocolRef::Dns6(addr),
					ProtocolRef::Tcp(port),
					None,
				) => (either::Right((addr.to_string(), *port)), None),
				(
					ProtocolRef::Dns(addr) | ProtocolRef::Dns4(addr) | ProtocolRef::Dns6(addr),
					ProtocolRef::Tcp(port),
					Some(ProtocolRef::Ws),
				) => (either::Right((addr.to_string(), *port)), Some(format!("{}:{}", addr, *port))),

				_ =>
					return Err(ConnectError {
						is_bad_addr: true,
						message: format!("Unknown protocols combination"),
					}),
			};

			let tcp_socket = match addr {
				either::Left(socket_addr) => TcpStream::connect(socket_addr).await,
				either::Right((dns, port)) => TcpStream::connect((&dns[..], port)).await,
			};

			if let Ok(tcp_socket) = &tcp_socket {
				let _ = tcp_socket.set_nodelay(true);
			}

			let mut socket = match (tcp_socket, host_if_websocket) {
				(Ok(tcp_socket), Some(host)) => future::Either::Right(
					websocket::websocket_client_handshake(websocket::Config {
						tcp_socket: tcp_socket.compat(),
						host: &host,
						url: "/",
					})
					.await
					.map_err(|err| ConnectError {
						message: format!("Failed to negotiate WebSocket: {}", err),
						is_bad_addr: false,
					})?,
				),
				(Ok(tcp_socket), None) => future::Either::Left(tcp_socket.compat()),
				(Err(err), _) =>
					return Err(ConnectError {
						is_bad_addr: false,
						message: format!("Failed to reach peer: {}", err),
					}),
			};

			let shared = Arc::new(StreamShared {
				guarded: parking_lot::Mutex::new(StreamSharedGuarded {
					write_queue: VecDeque::with_capacity(1024),
				}),
				write_queue_pushed: event_listener::Event::new(),
			});
			let shared_clone = shared.clone();

			let (mut read_data_tx, read_data_rx) = mpsc::channel(2);
			let mut read_buffer = vec![0; 4096];
			let mut write_queue_pushed_listener = shared.write_queue_pushed.listen();

			// TODO: this whole code is a mess, but the Platform trait must be modified to fix it
			// TODO: spawning a task per connection is necessary because the Platform trait isn't suitable for better strategies
			tokio::spawn(future::poll_fn(move |cx| {
				let mut lock = shared.guarded.lock();

				loop {
					match Pin::new(&mut read_data_tx).poll_ready(cx) {
						Poll::Ready(Ok(())) => {
							match Pin::new(&mut socket).poll_read(cx, &mut read_buffer) {
								Poll::Pending => break,
								Poll::Ready(result) => {
									match result {
										Ok(0) | Err(_) => return Poll::Ready(()), // End the task
										Ok(bytes) => {
											let _ = read_data_tx
												.try_send(read_buffer[..bytes].to_vec());
										},
									}
								},
							}
						},
						Poll::Ready(Err(_)) => return Poll::Ready(()), // End the task
						Poll::Pending => break,
					}
				}

				loop {
					if lock.write_queue.is_empty() {
						if let Poll::Ready(Err(_)) = Pin::new(&mut socket).poll_flush(cx) {
							// End the task
							return Poll::Ready(())
						}

						break
					} else {
						let write_queue_slices = lock.write_queue.as_slices();
						if let Poll::Ready(result) = Pin::new(&mut socket).poll_write_vectored(
							cx,
							&[
								IoSlice::new(write_queue_slices.0),
								IoSlice::new(write_queue_slices.1),
							],
						) {
							match result {
								Ok(bytes) =>
									for _ in 0..bytes {
										lock.write_queue.pop_front();
									},
								Err(_) => return Poll::Ready(()), // End the task
							}
						} else {
							break
						}
					}
				}

				loop {
					if let Poll::Ready(()) = Pin::new(&mut write_queue_pushed_listener).poll(cx) {
						write_queue_pushed_listener = shared.write_queue_pushed.listen();
					} else {
						break
					}
				}

				Poll::Pending
			}));

			Ok(PlatformConnection::SingleStreamMultistreamSelectNoiseYamux(TokioStream {
				shared: shared_clone,
				read_data_rx: Arc::new(parking_lot::Mutex::new(read_data_rx.peekable())),
				read_buffer: Some(Vec::with_capacity(4096)),
			}))
		})
	}

	fn open_out_substream(c: &mut Self::Connection) {
		// This function can only be called with so-called "multi-stream" connections. We never
		// open such connection.
		match *c {}
	}

	fn next_substream(c: &'_ mut Self::Connection) -> Self::NextSubstreamFuture<'_> {
		// This function can only be called with so-called "multi-stream" connections. We never
		// open such connection.
		match *c {}
	}

	fn wait_more_data(stream: &'_ mut Self::Stream) -> Self::StreamDataFuture<'_> {
		if stream.read_buffer.as_ref().map_or(true, |b| !b.is_empty()) {
			return Box::pin(future::ready(()))
		}

		let read_data_rx = stream.read_data_rx.clone();
		Box::pin(future::poll_fn(move |cx| {
			let mut lock = read_data_rx.lock();
			Pin::new(&mut *lock).poll_peek(cx).map(|_| ())
		}))
	}

	fn read_buffer(stream: &mut Self::Stream) -> ReadBuffer {
		if stream.read_buffer.is_none() {
			// TODO: the implementation doesn't let us differentiate between Closed and Reset
			return ReadBuffer::Reset
		}

		let mut lock = stream.read_data_rx.lock();
		while let Some(buf) = lock.next().now_or_never() {
			match buf {
				Some(b) => stream.read_buffer.as_mut().unwrap().extend(b),
				None => {
					stream.read_buffer = None;
					return ReadBuffer::Reset
				},
			}
		}

		ReadBuffer::Open(stream.read_buffer.as_ref().unwrap())
	}

	fn advance_read_cursor(stream: &mut Self::Stream, bytes: usize) {
		if let Some(read_buffer) = &mut stream.read_buffer {
			// TODO: meh for copying
			*read_buffer = read_buffer[bytes..].to_vec();
		}
	}

	fn send(stream: &mut Self::Stream, data: &[u8]) {
		let mut lock = stream.shared.guarded.lock();
		lock.write_queue.reserve(data.len());
		lock.write_queue.extend(data.iter().copied());
		stream.shared.write_queue_pushed.notify(usize::max_value());
	}
}

pub async fn get_light_client(
	spawner: SpawnTaskHandle,
	chain_spec: &str,
) -> (smoldot_light::Client<TokioPlatform, ()>, ChainId, JsonRpcResponses) {
	let config = ClientConfig {
		tasks_spawner: Box::new(move |_, task| {
			spawner.spawn("cumulus-relay-chain-light-client-task", None, task);
		}),
		system_name: env!("CARGO_PKG_NAME").to_string(),
		system_version: env!("CARGO_PKG_VERSION").to_string(),
	};
	let mut client: Client<TokioPlatform, ()> = Client::new(config);

	tracing::info!(target: "skunert", "Initializing light client");
	// Ask the client to connect to a chain.
	let smoldot_light::AddChainSuccess { chain_id, json_rpc_responses } = client
		.add_chain(smoldot_light::AddChainConfig {
			// The most important field of the configuration is the chain specification. This is a
			// JSON document containing all the information necessary for the client to connect to said
			// chain.
			specification: chain_spec,

			// If `true`, the chain will not be able to handle JSON-RPC requests. This can be used
			// to save up some resources.
			disable_json_rpc: false,

			// This field is necessary only if adding a parachain.
			potential_relay_chains: core::iter::empty(),

			// After a chain has been added, it is possible to extract a "database" (in the form of a
			// simple string). This database can later be passed back the next time the same chain is
			// added again.
			// A database with an invalid format is simply ignored by the client.
			// In this example, we don't use this feature, and as such we simply pass an empty string,
			// which is intentionally an invalid database content.
			database_content: "",

			// The client gives the possibility to insert an opaque "user data" alongside each chain.
			// This avoids having to create a separate `HashMap<ChainId, ...>` in parallel of the
			// client.
			// In this example, this feature isn't used. The chain simply has `()`.
			user_data: (),
		})
		.unwrap();

	(client, chain_id, json_rpc_responses.expect("JSON RPC is not disabled; qed"))
}

pub struct LightClientRpcWorker {
	client_receiver: Receiver<RpcDispatcherMessage>,
	imported_header_listeners: Vec<Sender<RelayHeader>>,
	finalized_header_listeners: Vec<Sender<RelayHeader>>,
	best_header_listeners: Vec<Sender<RelayHeader>>,
	smoldot_client: smoldot_light::Client<TokioPlatform, ()>,
	json_rpc_responses: JsonRpcResponses,
	chain_id: ChainId,
}

fn submit_request(
	client: &mut smoldot_light::Client<TokioPlatform, ()>,
	chain_id: ChainId,
	method: String,
	params: ArrayParams,
	id: u64,
) {
	let request = serde_json::to_string(&RequestSer::owned(
		jsonrpsee::types::Id::Number(id),
		method,
		params.to_rpc_params().expect("this should work"),
	))
	.expect("Serialization failed");
	tracing::info!(target: LOG_TARGET, "Transforming request into string: {request}");
	client.json_rpc_request(request, chain_id).unwrap();
}

enum Subscriptions {
	AllHead,
	NewHead,
	FinalizedHead,
}

fn distribute_header(header: RelayHeader, senders: &mut Vec<Sender<RelayHeader>>) {
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
}

fn handle_notification_response(
	subscription: SubscriptionResponse<RelayHeader>,
	subscription_lookup: &mut HashMap<String, Subscriptions>,
	all_head: &mut Vec<Sender<RelayHeader>>,
	new_head: &mut Vec<Sender<RelayHeader>>,
	finalized_head: &mut Vec<Sender<RelayHeader>>,
) {
	tracing::info!(target: LOG_TARGET, size = ?subscription_lookup.len(), "Subscription lookup size");
	if let SubscriptionId::Str(id) = subscription.params.subscription {
		let id = id.to_string();
		tracing::info!(target: LOG_TARGET, id, "Looking for");
		match subscription_lookup.get(&id) {
			Some(Subscriptions::AllHead) => {
				tracing::info!(target: LOG_TARGET, what = ?subscription.params.result, "received all notification");
				distribute_header(subscription.params.result, all_head)
			},
			Some(Subscriptions::FinalizedHead) => {
				tracing::info!(target: LOG_TARGET, what = ?subscription.params.result, "received finalized notification");
				distribute_header(subscription.params.result, finalized_head)
			},
			Some(Subscriptions::NewHead) => {
				tracing::info!(target: LOG_TARGET, what = ?subscription.params.result, "received new notification");
				distribute_header(subscription.params.result, new_head)
			},
			None => {
				tracing::error!(target: LOG_TARGET, ?id, "received notification with unknown id");
			},
		}
	}
}

fn handle_single_response(
	response: jsonrpsee::types::Response<JsonValue>,
	lookup: &mut HashMap<u64, SubscriptionType>,
	subscription_lookup: &mut HashMap<String, Subscriptions>,
) {
	let id = match response.id {
		jsonrpsee::types::params::Id::Number(num) => num,
		_ => return,
	};

	match lookup.remove(&id) {
		Some(SubscriptionType::Request(response_sender)) => {
			tracing::info!(target: LOG_TARGET, "found response sender for request with id: {id}");
			response_sender.send(Ok(response.result)).expect("Should send response");
		},
		Some(SubscriptionType::PendingAllHeadSubscription) => {
			let sub_id = response.result.as_str().expect("id hsould be a string").to_string();
			tracing::info!(target: LOG_TARGET, sub_id, "allHead stream initialized");
			subscription_lookup.insert(sub_id, Subscriptions::AllHead);
		},
		Some(SubscriptionType::PendingFinalizedHeadSubscription) => {
			let sub_id = response.result.as_str().expect("id hsould be a string").to_string();
			tracing::info!(target: LOG_TARGET, sub_id, "finalizedHead stream initialized");
			subscription_lookup.insert(sub_id, Subscriptions::FinalizedHead);
		},
		Some(SubscriptionType::PendingNewHeadSubscription) => {
			let sub_id = response.result.as_str().expect("id hsould be a string").to_string();
			tracing::info!(target: LOG_TARGET, sub_id, "newHead stream initialized");
			subscription_lookup.insert(sub_id, Subscriptions::NewHead);
		},
		None => {},
	}
}

fn handle_response(
	response: String,
	lookup: &mut HashMap<u64, SubscriptionType>,
	subscription_lookup: &mut HashMap<String, Subscriptions>,
	all_head: &mut Vec<Sender<RelayHeader>>,
	new_head: &mut Vec<Sender<RelayHeader>>,
	finalized_head: &mut Vec<Sender<RelayHeader>>,
) -> Result<(), ()> {
	tracing::info!(target: LOG_TARGET, "Got response {response:?}");
	if let Ok(response) = from_str::<jsonrpsee::types::Response<JsonValue>>(&response) {
		handle_single_response(response, lookup, subscription_lookup);
	} else if let Ok(subscription) = from_str::<SubscriptionResponse<RelayHeader>>(&response) {
		handle_notification_response(
			subscription,
			subscription_lookup,
			all_head,
			new_head,
			finalized_head,
		);
	} else {
		tracing::error!(target: LOG_TARGET, "Received unexpected response");
	};
	Err(())
}

enum SubscriptionType {
	Request(OneshotSender<Result<JsonValue, JsonRpseeError>>),
	PendingNewHeadSubscription,
	PendingAllHeadSubscription,
	PendingFinalizedHeadSubscription,
}

impl LightClientRpcWorker {
	pub fn new(
		smoldot_client: smoldot_light::Client<TokioPlatform, ()>,
		json_rpc_responses: JsonRpcResponses,
		chain_id: ChainId,
	) -> (LightClientRpcWorker, TokioSender<RpcDispatcherMessage>) {
		let (tx, rx) = tokio_channel(100);
		let worker = LightClientRpcWorker {
			client_receiver: rx,
			imported_header_listeners: Vec::new(),
			finalized_header_listeners: Vec::new(),
			best_header_listeners: Vec::new(),
			smoldot_client,
			json_rpc_responses,
			chain_id,
		};
		(worker, tx)
	}

	#[sc_tracing::logging::prefix_logs_with("Light Client")]
	pub async fn run(mut self) {
		let mut counter: u64 = 0;
		let mut id_to_request_sender_mapping = HashMap::new();
		let mut id_to_subscription_mapping = HashMap::new();
		{
			tracing::info!(target: LOG_TARGET, "Subscribing to all heads stream.");
			submit_request(
				&mut self.smoldot_client,
				self.chain_id,
				"chain_subscribeAllHeads".to_string(),
				rpc_params![],
				counter,
			);
			id_to_request_sender_mapping
				.insert(counter, SubscriptionType::PendingAllHeadSubscription);
			counter += 1;
		}
		{
			tracing::info!(target: LOG_TARGET, "Subscribing to new heads stream.");
			submit_request(
				&mut self.smoldot_client,
				self.chain_id,
				"chain_subscribeNewHeads".to_string(),
				rpc_params![],
				counter,
			);
			id_to_request_sender_mapping
				.insert(counter, SubscriptionType::PendingNewHeadSubscription);
			counter += 1;
		}
		{
			tracing::info!(target: LOG_TARGET, "Subscribing to finalized heads stream.");
			submit_request(
				&mut self.smoldot_client,
				self.chain_id,
				"chain_subscribeFinalizedHeads".to_string(),
				rpc_params![],
				counter,
			);
			id_to_request_sender_mapping
				.insert(counter, SubscriptionType::PendingFinalizedHeadSubscription);
			counter += 1;
		}

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
						id_to_request_sender_mapping.insert(counter, SubscriptionType::Request(response_sender));
						submit_request(&mut self.smoldot_client, self.chain_id, method, params, counter);
						counter += 1;
					},
					None => {
						tracing::error!(target: LOG_TARGET, "RPC client receiver closed. Stopping RPC Worker.");
						return;
					}
				},
				response = self.json_rpc_responses.next() => {
					if let Some(response) = response {
						handle_response(response, &mut id_to_request_sender_mapping, &mut id_to_subscription_mapping, &mut self.imported_header_listeners, &mut self.best_header_listeners, &mut self.finalized_header_listeners);
					}
				}
			}
		}
	}
}

#[derive(Deserialize)]
#[serde(untagged)]
enum LightClientRPCElement<'a> {
	#[serde(borrow)]
	NotificationRequest(jsonrpsee::types::Request<'a>),
	#[serde(borrow)]
	Response(jsonrpsee::types::Response<'a, JsonValue>),
}
