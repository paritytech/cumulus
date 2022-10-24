use jsonrpsee::{
	core::{
		client::{Client as JsonRpcClient, ClientT, Subscription, SubscriptionClientT},
		Error as JsonRpseeError,
	},
	types::ParamsSer,
};

pub struct PooledClient {
	ws_client: JsonRpcClient,
}

impl PooledClient {
	pub fn new(ws_client: JsonRpcClient) -> Self {
		tracing::info!(target: "pooled-client", "Instantiating pooled websocket client");
		Self { ws_client }
	}
}

#[async_trait::async_trait]
impl ClientT for PooledClient {
	async fn notification<'a>(
		&self,
		method: &'a str,
		params: Option<jsonrpsee::types::ParamsSer<'a>>,
	) -> Result<(), jsonrpsee::core::Error> {
		self.ws_client.notification(method, params).await
	}

	async fn request<'a, R>(
		&self,
		method: &'a str,
		params: Option<jsonrpsee::types::ParamsSer<'a>>,
	) -> Result<R, jsonrpsee::core::Error>
	where
		R: sp_runtime::DeserializeOwned,
	{
		self.ws_client.request(method, params).await
	}

	async fn batch_request<'a, R>(
		&self,
		batch: Vec<(&'a str, Option<jsonrpsee::types::ParamsSer<'a>>)>,
	) -> Result<Vec<R>, jsonrpsee::core::Error>
	where
		R: sp_runtime::DeserializeOwned + Default + Clone,
	{
		self.ws_client.batch_request(batch).await
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
		self.ws_client.subscribe(subscribe_method, params, unsubscribe_method).await
	}

	async fn subscribe_to_method<'a, Notif>(
		&self,
		method: &'a str,
	) -> Result<Subscription<Notif>, JsonRpseeError>
	where
		Notif: sp_runtime::DeserializeOwned,
	{
		self.ws_client.subscribe_to_method(method).await
	}
}
