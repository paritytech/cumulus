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

use std::pin::Pin;

use async_trait::async_trait;
use core::time::Duration;
use cumulus_primitives_core::{
	relay_chain::{
		v1::{CommittedCandidateReceipt, OccupiedCoreAssumption, SessionIndex, ValidatorId},
		Block as PBlock, BlockId, Hash as PHash, Header as PHeader, InboundHrmpMessage,
	},
	InboundDownwardMessage, ParaId, PersistedValidationData,
};
use cumulus_relay_chain_interface::{RelayChainError, RelayChainInterface, RelayChainResult};
use futures::{FutureExt, Stream, StreamExt};
use jsonrpsee::{
	core::client::{Client as JsonRPCClient, ClientT, Subscription, SubscriptionClientT},
	rpc_params,
	types::ParamsSer,
	ws_client::WsClientBuilder,
};
use parity_scale_codec::{Decode, Encode};
use polkadot_service::Handle;
use sc_client_api::{blockchain::BlockStatus, StorageData, StorageProof};
use sc_rpc_api::{state::ReadProof, system::Health};
use sp_core::sp_std::collections::btree_map::BTreeMap;
use sp_runtime::{generic::SignedBlock, DeserializeOwned};
use sp_state_machine::StorageValue;
use sp_storage::StorageKey;
use std::sync::Arc;

pub use url::Url;

const LOG_TARGET: &str = "relay-chain-network";
const TIMEOUT_IN_SECONDS: u64 = 6;

#[derive(Clone)]
struct RelayChainRPCClient {
	ws_client: Arc<JsonRPCClient>,
}

/// Client that calls RPC endpoints and deserializes call results
impl RelayChainRPCClient {
	/// Call a runtime function via rpc
	async fn call_remote_runtime_function(
		&self,
		method_name: &str,
		block_id: &BlockId,
		payload: Option<Vec<u8>>,
	) -> RelayChainResult<sp_core::Bytes> {
		let payload_bytes = payload.map_or(sp_core::Bytes(Vec::new()), sp_core::Bytes);
		let params = rpc_params! {
			method_name,
			payload_bytes,
			block_id
		};
		self.request("state_call", params).await
	}

	async fn subscribe<'a, R>(
		&self,
		sub_name: &'a str,
		unsub_name: &'a str,
		params: Option<ParamsSer<'a>>,
	) -> RelayChainResult<Subscription<R>>
	where
		R: DeserializeOwned,
	{
		tracing::trace!(target: LOG_TARGET, "Subscribing to stream: {}", sub_name);
		self.ws_client
			.subscribe::<R>(sub_name, params, unsub_name)
			.await
			.map_err(|err| RelayChainError::NetworkError(sub_name.to_string(), err))
	}

	async fn request<'a, R>(
		&self,
		method: &'a str,
		params: Option<ParamsSer<'a>>,
	) -> Result<R, RelayChainError>
	where
		R: DeserializeOwned,
	{
		tracing::trace!(target: LOG_TARGET, "Calling rpc endpoint: {}", method);
		self.ws_client
			.request(method, params)
			.await
			.map_err(|err| RelayChainError::NetworkError(method.to_string(), err))
	}

	async fn system_health(&self) -> Result<Health, RelayChainError> {
		self.request("system_health", None).await
	}

	async fn state_get_read_proof(
		&self,
		storage_keys: Vec<StorageKey>,
		at: Option<PHash>,
	) -> Result<ReadProof<PHash>, RelayChainError> {
		let params = rpc_params!(storage_keys, at);
		self.request("state_getReadProof", params).await
	}

	async fn state_get_storage(
		&self,
		storage_key: StorageKey,
		at: Option<PHash>,
	) -> Result<Option<StorageData>, RelayChainError> {
		let params = rpc_params!(storage_key, at);
		self.request("state_getStorage", params).await
	}

	async fn chain_get_block(
		&self,
		at: Option<PHash>,
	) -> Result<Option<SignedBlock<PBlock>>, RelayChainError> {
		let params = rpc_params!(at);
		self.request("chain_getBlock", params).await
	}

	async fn chain_get_head(&self) -> Result<PHash, RelayChainError> {
		self.request("chain_getHead", None).await
	}

	async fn chain_get_header(
		&self,
		hash: Option<PHash>,
	) -> Result<Option<PHeader>, RelayChainError> {
		let params = rpc_params!(hash);
		self.request("chain_getHeader", params).await
	}

	async fn parachain_host_candidate_pending_availability(
		&self,
		at: &BlockId,
		para_id: ParaId,
	) -> Result<Option<CommittedCandidateReceipt>, RelayChainError> {
		let response_bytes = self
			.call_remote_runtime_function(
				"ParachainHost_candidate_pending_availability",
				at,
				Some(para_id.encode()),
			)
			.await?;

		Ok(Option::<CommittedCandidateReceipt<PHash>>::decode(&mut &*response_bytes.0)
			.expect("should deserialize"))
	}

	async fn parachain_host_session_index_for_child(
		&self,
		at: &BlockId,
	) -> Result<SessionIndex, RelayChainError> {
		let response_bytes = self
			.call_remote_runtime_function("ParachainHost_session_index_for_child", at, None)
			.await?;

		Ok(SessionIndex::decode(&mut &*response_bytes.0).expect("should deserialize"))
	}

	async fn parachain_host_validators(
		&self,
		at: &BlockId,
	) -> Result<Vec<ValidatorId>, RelayChainError> {
		let response_bytes =
			self.call_remote_runtime_function("ParachainHost_validators", at, None).await?;

		Ok(Vec::<ValidatorId>::decode(&mut &*response_bytes.0).expect("should deserialize"))
	}

	async fn parachain_host_persisted_validation_data(
		&self,
		block_id: &BlockId,
		para_id: ParaId,
		occupied_core_assumption: OccupiedCoreAssumption,
	) -> Result<Option<PersistedValidationData>, RelayChainError> {
		let response_bytes = self
			.call_remote_runtime_function(
				"ParachainHost_persisted_validation_data",
				block_id,
				Some((para_id, occupied_core_assumption).encode()),
			)
			.await?;

		Ok(Option::<PersistedValidationData>::decode(&mut &*response_bytes.0)
			.expect("should deserialize"))
	}

	async fn parachain_host_inbound_hrmp_channels_contents(
		&self,
		para_id: ParaId,
		at: &BlockId,
	) -> Result<BTreeMap<ParaId, Vec<InboundHrmpMessage>>, RelayChainError> {
		let response_bytes = self
			.call_remote_runtime_function(
				"ParachainHost_inbound_hrmp_channels_contents",
				&at,
				Some(para_id.encode()),
			)
			.await?;

		Ok(BTreeMap::<ParaId, Vec<InboundHrmpMessage>>::decode(&mut &*response_bytes.0)
			.expect("should deserialize"))
	}

	async fn parachain_host_dmq_contents(
		&self,
		para_id: ParaId,
		at: &BlockId,
	) -> Result<Vec<InboundDownwardMessage>, RelayChainError> {
		let response_bytes = self
			.call_remote_runtime_function("ParachainHost_dmq_contents", &at, Some(para_id.encode()))
			.await?;

		Ok(Vec::<InboundDownwardMessage>::decode(&mut &*response_bytes.0)
			.expect("should deserialize"))
	}

	async fn subscribe_all_heads(&self) -> Result<Subscription<PHeader>, RelayChainError> {
		self.subscribe::<PHeader>("chain_subscribeAllHeads", "chain_unsubscribeAllHeads", None)
			.await
	}

	async fn subscribe_new_best_heads(&self) -> Result<Subscription<PHeader>, RelayChainError> {
		self.subscribe::<PHeader>("chain_subscribeNewHeads", "chain_unsubscribeNewHeads", None)
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
}

/// RelayChainNetwork is used to interact with a full node that is running locally
/// in the same process.
#[derive(Clone)]
pub struct RelayChainNetwork {
	rpc_client: RelayChainRPCClient,
}

impl RelayChainNetwork {
	pub async fn new(url: Url) -> Self {
		tracing::trace!(target: LOG_TARGET, "Initializing RPC Client. Relay Chain Url: {}", url);
		let ws_client = WsClientBuilder::default()
			.build(url.as_str())
			.await
			.expect("Should be able to initialize websocket client.");

		Self { rpc_client: RelayChainRPCClient { ws_client: Arc::new(ws_client) } }
	}
}

#[async_trait]
impl RelayChainInterface for RelayChainNetwork {
	async fn retrieve_dmq_contents(
		&self,
		para_id: ParaId,
		relay_parent: PHash,
	) -> RelayChainResult<Vec<InboundDownwardMessage>> {
		let block_id = BlockId::hash(relay_parent);
		let response = self.rpc_client.parachain_host_dmq_contents(para_id, &block_id).await;

		response
	}

	async fn retrieve_all_inbound_hrmp_channel_contents(
		&self,
		para_id: ParaId,
		relay_parent: PHash,
	) -> RelayChainResult<BTreeMap<ParaId, Vec<InboundHrmpMessage>>> {
		let block_id = BlockId::hash(relay_parent);
		let response = self
			.rpc_client
			.parachain_host_inbound_hrmp_channels_contents(para_id, &block_id)
			.await;

		response
	}

	async fn persisted_validation_data(
		&self,
		block_id: &BlockId,
		para_id: ParaId,
		occupied_core_assumption: OccupiedCoreAssumption,
	) -> RelayChainResult<Option<PersistedValidationData>> {
		let response = self
			.rpc_client
			.parachain_host_persisted_validation_data(block_id, para_id, occupied_core_assumption)
			.await;

		response
	}

	async fn candidate_pending_availability(
		&self,
		block_id: &BlockId,
		para_id: ParaId,
	) -> RelayChainResult<Option<CommittedCandidateReceipt>> {
		let response = self
			.rpc_client
			.parachain_host_candidate_pending_availability(block_id, para_id)
			.await;

		response
	}

	async fn session_index_for_child(&self, block_id: &BlockId) -> RelayChainResult<SessionIndex> {
		let response = self.rpc_client.parachain_host_session_index_for_child(block_id).await;

		response
	}

	async fn validators(&self, block_id: &BlockId) -> RelayChainResult<Vec<ValidatorId>> {
		let response = self.rpc_client.parachain_host_validators(block_id).await;

		response
	}

	async fn import_notification_stream(
		&self,
	) -> RelayChainResult<Pin<Box<dyn Stream<Item = PHeader> + Send>>> {
		let imported_headers_stream = self.rpc_client.subscribe_all_heads().await?;
		Ok(Box::pin(imported_headers_stream.filter_map(|item| async move {
			item.map_err(|err| {
				tracing::error!(target: LOG_TARGET, "Error occured in stream: {}", err)
			})
			.ok()
		})))
	}

	async fn finality_notification_stream(
		&self,
	) -> RelayChainResult<Pin<Box<dyn Stream<Item = PHeader> + Send>>> {
		let imported_headers_stream = self
			.rpc_client
			.subscribe_finalized_heads()
			.await
			.expect("Should be able to subscribe");

		Ok(Box::pin(imported_headers_stream.filter_map(|item| async move {
			item.map_err(|err| {
				tracing::error!(target: LOG_TARGET, "Error occured in stream: {}", err)
			})
			.ok()
		})))
	}

	async fn best_block_hash(&self) -> RelayChainResult<PHash> {
		let response = self.rpc_client.chain_get_head().await;
		response
	}

	async fn block_status(&self, block_id: BlockId) -> RelayChainResult<BlockStatus> {
		let hash = match block_id {
			sp_api::BlockId::Hash(hash) => hash,
			sp_api::BlockId::Number(_) => todo!(),
		};

		let response = self.rpc_client.chain_get_block(Some(hash.clone())).await;
		match response {
			Ok(Some(_)) => Ok(BlockStatus::InChain),
			_ => Ok(BlockStatus::Unknown),
		}
	}

	async fn is_major_syncing(&self) -> RelayChainResult<bool> {
		self.rpc_client.system_health().await.map(|h| h.is_syncing)
	}

	fn overseer_handle(&self) -> RelayChainResult<Option<Handle>> {
		todo!("overseer_handle");
	}

	async fn get_storage_by_key(
		&self,
		block_id: &BlockId,
		key: &[u8],
	) -> RelayChainResult<Option<StorageValue>> {
		let storage_key = StorageKey(key.to_vec());
		let hash = match block_id {
			sp_api::BlockId::Hash(hash) => hash,
			sp_api::BlockId::Number(_) => todo!(),
		};

		let response = self.rpc_client.state_get_storage(storage_key, Some(*hash)).await;
		response.map(|v| v.map(|sv| sv.0))
	}

	async fn prove_read(
		&self,
		block_id: &BlockId,
		relevant_keys: &Vec<Vec<u8>>,
	) -> RelayChainResult<StorageProof> {
		let cloned = relevant_keys.clone();
		let storage_keys: Vec<StorageKey> = cloned.into_iter().map(StorageKey).collect();

		let hash = match block_id {
			sp_api::BlockId::Hash(hash) => hash,
			sp_api::BlockId::Number(_) => todo!(),
		};

		let result = self.rpc_client.state_get_read_proof(storage_keys, Some(*hash)).await;

		result.map(|read_proof| {
			let bytes: Vec<Vec<u8>> =
				read_proof.proof.into_iter().map(|bytes| (*bytes).to_vec()).collect();

			StorageProof::new(bytes.to_vec())
		})
	}

	/// Wait for a given relay chain block
	///
	/// The hash of the block to wait for is passed. We wait for the block to arrive or return after a timeout.
	///
	/// Implementation:
	/// 1. Register a listener to all new blocks.
	/// 2. Check if the block is already in chain. If yes, succeed early.
	/// 3. Wait for the block to be imported via subscription.
	/// 4. If timeout is reached, we return an error.
	async fn wait_for_block(&self, wait_for_hash: PHash) -> RelayChainResult<()> {
		let mut head_stream = self.rpc_client.subscribe_all_heads().await?;

		let block_header = self.rpc_client.chain_get_header(Some(wait_for_hash)).await;
		if block_header.ok().is_some() {
			return Ok(())
		}

		let mut timeout = futures_timer::Delay::new(Duration::from_secs(TIMEOUT_IN_SECONDS)).fuse();

		loop {
			futures::select! {
				_ = timeout => return Err(RelayChainError::WaitTimeout(wait_for_hash)),
				evt = head_stream.next().fuse() => match evt {
					Some(Ok(evt)) if evt.hash() == wait_for_hash => return Ok(()),
					// Not the event we waited on.
					Some(_) => continue,
					None => return Err(RelayChainError::ImportListenerClosed(wait_for_hash)),
				}
			}
		}
	}

	async fn new_best_notification_stream(
		&self,
	) -> RelayChainResult<Pin<Box<dyn Stream<Item = PHeader> + Send>>> {
		let imported_headers_stream = self
			.rpc_client
			.subscribe_new_best_heads()
			.await
			.expect("Should be able to subscribe");

		Ok(Box::pin(imported_headers_stream.filter_map(|item| async move {
			item.map_err(|err| {
				tracing::error!(target: LOG_TARGET, "Error occured in stream: {}", err)
			})
			.ok()
		})))
	}
}
