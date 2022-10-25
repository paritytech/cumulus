use cumulus_relay_chain_interface::RelayChainResult;
use jsonrpsee::{
	core::{
		client::{Client as JsonRpcClient, ClientT, Subscription, SubscriptionClientT},
		Error as JsonRpseeError,
	},
	types::ParamsSer,
	ws_client::WsClientBuilder,
};
use url::Url;

pub struct PooledClient {
	ws_clients: Vec<(Url, JsonRpcClient)>,
	active_index: usize,
}

impl PooledClient {
	pub async fn new(url: Url) -> RelayChainResult<Self> {
		tracing::info!(target: "pooled-client", "Instantiating pooled websocket client");
		let ws_client = WsClientBuilder::default().build(url.as_str()).await?;
		Ok(Self { ws_clients: vec![(url, ws_client)], active_index: 0 })
	}

	pub async fn new_from_urls(urls: Vec<Url>) -> RelayChainResult<Self> {
		tracing::info!(target: "pooled-client", "Instantiating pooled websocket client");
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
		Ok(Self { ws_clients: clients, active_index: 0 })
	}

	fn active_client(&self) -> &JsonRpcClient {
		&self.ws_clients.get(self.active_index).unwrap().1
	}
}

#[async_trait::async_trait]
impl ClientT for PooledClient {
	async fn notification<'a>(
		&self,
		method: &'a str,
		params: Option<jsonrpsee::types::ParamsSer<'a>>,
	) -> Result<(), jsonrpsee::core::Error> {
		self.active_client().notification(method, params).await
	}

	async fn request<'a, R>(
		&self,
		method: &'a str,
		params: Option<jsonrpsee::types::ParamsSer<'a>>,
	) -> Result<R, jsonrpsee::core::Error>
	where
		R: sp_runtime::DeserializeOwned,
	{
		self.active_client().request(method, params).await
	}

	async fn batch_request<'a, R>(
		&self,
		batch: Vec<(&'a str, Option<jsonrpsee::types::ParamsSer<'a>>)>,
	) -> Result<Vec<R>, jsonrpsee::core::Error>
	where
		R: sp_runtime::DeserializeOwned + Default + Clone,
	{
		self.active_client().batch_request(batch).await
	}
}

#[async_trait::async_trait]
impl SubscriptionClientT for PooledClient {
	async fn subscribe<'a, Notif>(
		&self,
		subscribe_method: &'a str,
		params: Option<ParamsSer<'a>>,
		unsubscribe_method: &'a str,
	) -> Result<Subscription<Notif>, JsonRpseeError>
	where
		Notif: sp_runtime::DeserializeOwned,
	{
		self.active_client()
			.subscribe(subscribe_method, params, unsubscribe_method)
			.await
	}

	async fn subscribe_to_method<'a, Notif>(
		&self,
		method: &'a str,
	) -> Result<Subscription<Notif>, JsonRpseeError>
	where
		Notif: sp_runtime::DeserializeOwned,
	{
		self.active_client().subscribe_to_method(method).await
	}
}
