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
use cumulus_primitives_core::relay_chain::{
	Block as RelayBlock, BlockNumber as RelayNumber, Hash as RelayHash, Header as RelayHeader,
};
use futures::{
	channel::mpsc::{self, Sender},
	prelude::*,
	stream::FuturesUnordered,
	task::Poll,
};
use jsonrpsee::core::{
	client::{ClientT, TransportReceiverT, TransportSenderT},
	Error,
};
use polkadot_service::generic::SignedBlock;
use sc_rpc_api::chain::ChainApiClient;
use sc_service::SpawnTaskHandle;
use smoldot::libp2p::{multiaddr::ProtocolRef, websocket, Multiaddr};
use smoldot_light::{
	platform::{ConnectError, PlatformConnection, PlatformSubstreamDirection, ReadBuffer},
	ChainId, Client, ClientConfig, JsonRpcResponses,
};
use std::{
	collections::VecDeque,
	io::IoSlice,
	net::{IpAddr, SocketAddr},
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
		std::time::UNIX_EPOCH.elapsed().expect("Time should be after unix epoch.")
	}

	fn now() -> Self::Instant {
		std::time::Instant::now()
	}

	fn sleep(duration: Duration) -> Self::Delay {
		tokio::time::sleep(duration).boxed()
	}

	fn sleep_until(when: Self::Instant) -> Self::Delay {
		tokio::time::sleep_until(when.into()).boxed()
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

	// Ask the client to connect to a chain.
	let smoldot_light::AddChainSuccess { chain_id, json_rpc_responses } = client
		.add_chain(smoldot_light::AddChainConfig {
			specification: chain_spec,
			disable_json_rpc: false,
			potential_relay_chains: core::iter::empty(),
			database_content: "",
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

fn handle_notification(
	maybe_header: Option<Result<RelayHeader, Error>>,
	senders: &mut Vec<Sender<RelayHeader>>,
) -> bool {
	match maybe_header {
		Some(Ok(header)) => distribute_header(header, senders),
		None => {
			tracing::error!(target: LOG_TARGET, "Subscription closed.");
			return true
		},
		Some(Err(error)) => {
			tracing::error!(target: LOG_TARGET, ?error, "Error in RPC subscription.");
			return true
		},
	};
	return false
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
			imported_header_listeners: Default::default(),
			finalized_header_listeners: Default::default(),
			best_header_listeners: Default::default(),
			smoldot_client,
			json_rpc_responses,
			chain_id,
		};
		(worker, tx)
	}

	pub async fn run(mut self) {
		tokio::time::sleep(Duration::from_secs(20)).await;

		let some_sender =
			SimpleStringSender { inner: self.smoldot_client, chain_id: self.chain_id };
		let some_receiver = SimpleStringReceiver { inner: self.json_rpc_responses };
		let smoldot_client = Arc::new(
			jsonrpsee::core::client::ClientBuilder::default()
				.build_with_tokio(some_sender, some_receiver),
		);

		let mut pending_requests = FuturesUnordered::new();

		let Ok(mut new_head_subscription) = <jsonrpsee::core::client::Client as ChainApiClient<
			RelayNumber,
			RelayHash,
			RelayHeader,
			SignedBlock<RelayBlock>,
		>>::subscribe_new_heads(&smoldot_client)
		.await else {
			tracing::error!(
				target: LOG_TARGET,
				"Unable to initialize new heads subscription"
			);
			return;
		};

		let Ok(mut finalized_head_subscription) = <jsonrpsee::core::client::Client as ChainApiClient<
			RelayNumber,
			RelayHash,
			RelayHeader,
			SignedBlock<RelayBlock>,
		>>::subscribe_finalized_heads(&smoldot_client)
		.await else {
			tracing::error!(
				target: LOG_TARGET,
				"Unable to initialize finalized heads subscription"
			);
			return;
		};

		let Ok(mut all_head_subscription) = <jsonrpsee::core::client::Client as ChainApiClient<
			RelayNumber,
			RelayHash,
			RelayHeader,
			SignedBlock<RelayBlock>,
		>>::subscribe_all_heads(&smoldot_client).await else {
			tracing::error!(
				target: LOG_TARGET,
				"Unable to initialize all heads subscription"
			);
			return
		};

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
						let closure_client = smoldot_client.clone();
						tracing::info!(
							target: LOG_TARGET,
							len = pending_requests.len(),
							"Inserting request: {method:?}"
						);
						pending_requests.push(async move {
							let response = closure_client.request(&method, params).await;
								tracing::info!(
									target: LOG_TARGET,
									method,
									?response,
									"Response"
								);
							if let Err(err) = response_sender.send(response) {
								tracing::debug!(
									target: LOG_TARGET,
									?err,
									"Recipient no longer interested in request result"
								);
							};
						});
					},
					None => {
						tracing::error!(target: LOG_TARGET, "RPC client receiver closed. Stopping RPC Worker.");
						return;
					}
				},
				_ = pending_requests.next(), if !pending_requests.is_empty() => {},
				import_event = all_head_subscription.next() => {
					if handle_notification(import_event, &mut self.imported_header_listeners) {
						return
					}
				},
				best_header_event = new_head_subscription.next() => {
					if handle_notification(best_header_event, &mut self.best_header_listeners) {
						return
					}
				}
				finalized_event = finalized_head_subscription.next() => {
					if handle_notification(finalized_event, &mut self.finalized_header_listeners) {
						return
					}
				}
			}
		}
	}
}

#[derive(thiserror::Error, Debug)]
enum LightClientError {
	#[error("Error occured while calling inserting smoldot request: {0}")]
	SmoldotError(String),
	#[error("Nothing returned from json_rpc_responses")]
	EmptyReturn,
}

struct SimpleStringSender {
	inner: smoldot_light::Client<TokioPlatform, ()>,
	chain_id: ChainId,
}

impl TransportSenderT for SimpleStringSender {
	type Error = LightClientError;

	fn send<'life0, 'async_trait>(
		&'life0 mut self,
		msg: String,
	) -> core::pin::Pin<
		Box<
			dyn core::future::Future<Output = Result<(), Self::Error>>
				+ core::marker::Send
				+ 'async_trait,
		>,
	>
	where
		'life0: 'async_trait,
		Self: 'async_trait,
	{
		Box::pin(async {
			self.inner
				.json_rpc_request(msg, self.chain_id)
				.map_err(|e| LightClientError::SmoldotError(e.to_string()))
		})
	}
}

struct SimpleStringReceiver {
	inner: JsonRpcResponses,
}

impl TransportReceiverT for SimpleStringReceiver {
	type Error = LightClientError;

	fn receive<'life0, 'async_trait>(
		&'life0 mut self,
	) -> core::pin::Pin<
		Box<
			dyn core::future::Future<
					Output = Result<jsonrpsee::core::client::ReceivedMessage, Self::Error>,
				> + core::marker::Send
				+ 'async_trait,
		>,
	>
	where
		'life0: 'async_trait,
		Self: 'async_trait,
	{
		async {
			self.inner
				.next()
				.await
				.map(|message| jsonrpsee::core::client::ReceivedMessage::Text(message))
				.ok_or(LightClientError::EmptyReturn)
		}
		.boxed()
	}
}
