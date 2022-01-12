// Copyright 2021 Parity Technologies (UK) Ltd.
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
use cumulus_primitives_core::{
	relay_chain::{
		v1::{CommittedCandidateReceipt, OccupiedCoreAssumption, SessionIndex, ValidatorId},
		Block as PBlock, BlockId, Hash as PHash, Header as PHeader, InboundHrmpMessage,
	},
	InboundDownwardMessage, ParaId, PersistedValidationData,
};
use cumulus_relay_chain_interface::RelayChainInterface;
use futures::{Stream, StreamExt, TryStreamExt};
use parity_scale_codec::{Decode, Encode};

use jsonrpsee::{
	core::{
		client::{Client as JsonRPCClient, ClientT, Subscription, SubscriptionClientT},
		Error as JsonRPSeeError,
	},
	rpc_params,
	types::ParamsSer,
	ws_client::WsClientBuilder,
};
use polkadot_service::Handle;
use sc_client_api::{blockchain::BlockStatus, BlockImportNotification, StorageData, StorageProof};
use sc_rpc_api::state::ReadProof;
use sp_api::ApiError;
use sp_core::sp_std::collections::btree_map::BTreeMap;
use sp_runtime::{generic::SignedBlock, DeserializeOwned};
use sp_state_machine::StorageValue;
use sp_storage::StorageKey;
use std::sync::Arc;

pub use url::Url;

const LOG_TARGET: &str = "relay_chain_network";

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
	) -> Result<sp_core::Bytes, JsonRPSeeError> {
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
	) -> Result<Subscription<R>, JsonRPSeeError>
	where
		R: DeserializeOwned,
	{
		tracing::trace!(target: LOG_TARGET, "Subscribing to stream: {}", sub_name);
		self.ws_client.subscribe::<R>(sub_name, params, unsub_name).await
	}

	async fn request<'a, R>(
		&self,
		method: &'a str,
		params: Option<ParamsSer<'a>>,
	) -> Result<R, JsonRPSeeError>
	where
		R: DeserializeOwned,
	{
		tracing::trace!(target: LOG_TARGET, "Calling rpc endpoint: {}", method);
		self.ws_client.request(method, params).await
	}

	async fn state_get_read_proof(
		&self,
		storage_keys: Vec<StorageKey>,
		at: Option<PHash>,
	) -> Result<Option<ReadProof<PHash>>, JsonRPSeeError> {
		let params = rpc_params!(storage_keys, at);
		self.request("state_getReadProof", params).await
	}

	async fn state_get_storage(
		&self,
		storage_key: StorageKey,
		at: Option<PHash>,
	) -> Result<Option<StorageData>, JsonRPSeeError> {
		let params = rpc_params!(storage_key, at);
		self.request("state_getStorage", params).await
	}

	async fn chain_get_block(
		&self,
		at: Option<PHash>,
	) -> Result<Option<SignedBlock<PBlock>>, JsonRPSeeError> {
		let params = rpc_params!(at);
		self.request("chain_getBlock", params).await
	}

	async fn chain_get_head(&self) -> Result<PHash, JsonRPSeeError> {
		self.request("chain_getHead", None).await
	}

	async fn parachain_host_candidate_pending_availability(
		&self,
		at: &BlockId,
		para_id: ParaId,
	) -> Result<Option<CommittedCandidateReceipt>, JsonRPSeeError> {
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
	) -> Result<SessionIndex, JsonRPSeeError> {
		let response_bytes = self
			.call_remote_runtime_function("ParachainHost_session_index_for_child", at, None)
			.await?;

		Ok(SessionIndex::decode(&mut &*response_bytes.0).expect("should deserialize"))
	}

	async fn parachain_host_validators(
		&self,
		at: &BlockId,
	) -> Result<Vec<ValidatorId>, JsonRPSeeError> {
		let response_bytes =
			self.call_remote_runtime_function("ParachainHost_validators", at, None).await?;

		Ok(Vec::<ValidatorId>::decode(&mut &*response_bytes.0).expect("should deserialize"))
	}

	async fn parachain_host_persisted_validation_data(
		&self,
		block_id: &BlockId,
		para_id: ParaId,
		occupied_core_assumption: OccupiedCoreAssumption,
	) -> Result<Option<PersistedValidationData>, JsonRPSeeError> {
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
	) -> Result<Option<BTreeMap<ParaId, Vec<InboundHrmpMessage>>>, JsonRPSeeError> {
		let response_bytes = self
			.call_remote_runtime_function(
				"ParachainHost_inbound_hrmp_channels_contents",
				&at,
				Some(para_id.encode()),
			)
			.await?;

		Ok(Option::<BTreeMap<ParaId, Vec<InboundHrmpMessage>>>::decode(&mut &*response_bytes.0)
			.expect("should deserialize"))
	}

	async fn parachain_host_dmq_contents(
		&self,
		para_id: ParaId,
		at: &BlockId,
	) -> Result<Option<Vec<InboundDownwardMessage>>, JsonRPSeeError> {
		let response_bytes = self
			.call_remote_runtime_function("ParachainHost_dmq_contents", &at, Some(para_id.encode()))
			.await?;

		Ok(Option::<Vec<InboundDownwardMessage>>::decode(&mut &*response_bytes.0)
			.expect("should deserialize"))
	}

	async fn subscribe_all_heads(&self) -> Result<Subscription<PHeader>, JsonRPSeeError> {
		self.subscribe::<PHeader>("chain_subscribeAllHeads", "chain_unsubscribeAllHeads", None)
			.await
	}

	async fn subscribe_new_best_heads(&self) -> Result<Subscription<PHeader>, JsonRPSeeError> {
		self.subscribe::<PHeader>("chain_subscribeNewHeads", "chain_unsubscribeNewHeads", None)
			.await
	}

	async fn subscribe_finalized_heads(&self) -> Result<Subscription<PHeader>, JsonRPSeeError> {
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
	) -> Option<Vec<InboundDownwardMessage>> {
		let block_id = BlockId::hash(relay_parent);
		let response = self.rpc_client.parachain_host_dmq_contents(para_id, &block_id).await;

		response.expect("nope")
	}

	async fn retrieve_all_inbound_hrmp_channel_contents(
		&self,
		para_id: ParaId,
		relay_parent: PHash,
	) -> Option<BTreeMap<ParaId, Vec<InboundHrmpMessage>>> {
		let block_id = BlockId::hash(relay_parent);
		let response = self
			.rpc_client
			.parachain_host_inbound_hrmp_channels_contents(para_id, &block_id)
			.await;

		response.expect("nope")
	}

	async fn persisted_validation_data(
		&self,
		block_id: &BlockId,
		para_id: ParaId,
		occupied_core_assumption: OccupiedCoreAssumption,
	) -> Result<Option<PersistedValidationData>, ApiError> {
		let response = self
			.rpc_client
			.parachain_host_persisted_validation_data(block_id, para_id, occupied_core_assumption)
			.await;

		Ok(response.expect("nope"))
	}

	async fn candidate_pending_availability(
		&self,
		block_id: &BlockId,
		para_id: ParaId,
	) -> Result<Option<CommittedCandidateReceipt>, ApiError> {
		let response = self
			.rpc_client
			.parachain_host_candidate_pending_availability(block_id, para_id)
			.await;

		Ok(response.expect("nope"))
	}

	async fn session_index_for_child(&self, block_id: &BlockId) -> Result<SessionIndex, ApiError> {
		let response = self.rpc_client.parachain_host_session_index_for_child(block_id).await;

		Ok(response.expect("nope"))
	}

	async fn validators(&self, block_id: &BlockId) -> Result<Vec<ValidatorId>, ApiError> {
		let response = self.rpc_client.parachain_host_validators(block_id).await;

		Ok(response.expect("nope"))
	}

	async fn import_notification_stream(&self) -> Pin<Box<dyn Stream<Item = PHeader> + Send>> {
		let imported_headers_stream = self
			.rpc_client
			.subscribe_all_heads()
			.await
			.expect("Should be able to subscribe");

		Box::pin(imported_headers_stream.filter_map(|item| async move {
			item.map_err(|err| {
				tracing::error!(target: LOG_TARGET, "Error occured in stream: {}", err)
			})
			.ok()
		}))
	}

	async fn finality_notification_stream(&self) -> Pin<Box<dyn Stream<Item = PHeader> + Send>> {
		let imported_headers_stream = self
			.rpc_client
			.subscribe_finalized_heads()
			.await
			.expect("Should be able to subscribe");

		Box::pin(imported_headers_stream.filter_map(|item| async move {
			item.map_err(|err| {
				tracing::error!(target: LOG_TARGET, "Error occured in stream: {}", err)
			})
			.ok()
		}))
	}

	fn storage_changes_notification_stream(
		&self,
		filter_keys: Option<&[sc_client_api::StorageKey]>,
		child_filter_keys: Option<
			&[(sc_client_api::StorageKey, Option<Vec<sc_client_api::StorageKey>>)],
		>,
	) -> sc_client_api::blockchain::Result<sc_client_api::StorageEventStream<PHash>> {
		todo!("storage_changes_notification_stream");
	}

	async fn best_block_hash(&self) -> PHash {
		let response = self.rpc_client.chain_get_head().await;
		response.expect("nope")
	}

	async fn block_status(&self, block_id: BlockId) -> Result<BlockStatus, sp_blockchain::Error> {
		let hash = match block_id {
			sp_api::BlockId::Hash(hash) => hash,
			sp_api::BlockId::Number(_) => todo!(),
		};

		let response = self.rpc_client.chain_get_block(Some(hash.clone())).await;
		match response.expect("nope") {
			Some(_) => Ok(BlockStatus::InChain),
			None => Ok(BlockStatus::Unknown),
		}
	}

	fn is_major_syncing(&self) -> bool {
		todo!("major_syncing");
	}

	fn overseer_handle(&self) -> Option<Handle> {
		todo!("overseer_handle");
	}

	async fn get_storage_by_key(
		&self,
		block_id: &BlockId,
		key: &[u8],
	) -> Result<Option<StorageValue>, sp_blockchain::Error> {
		let storage_key = StorageKey(key.to_vec());
		let hash = match block_id {
			sp_api::BlockId::Hash(hash) => hash,
			sp_api::BlockId::Number(_) => todo!(),
		};

		let response = self.rpc_client.state_get_storage(storage_key, Some(*hash)).await;
		Ok(response.expect("nope").map(|v| v.0))
	}

	async fn prove_read(
		&self,
		block_id: &BlockId,
		relevant_keys: &Vec<Vec<u8>>,
	) -> Result<Option<StorageProof>, Box<dyn sp_state_machine::Error>> {
		let cloned = relevant_keys.clone();
		let storage_keys: Vec<StorageKey> = cloned.into_iter().map(StorageKey).collect();

		let hash = match block_id {
			sp_api::BlockId::Hash(hash) => hash,
			sp_api::BlockId::Number(_) => todo!(),
		};

		let result = self.rpc_client.state_get_read_proof(storage_keys, Some(*hash)).await;

		Ok(result.expect("nope").map(|read_proof| {
			let bytes: Vec<Vec<u8>> =
				read_proof.proof.into_iter().map(|bytes| (*bytes).to_vec()).collect();

			StorageProof::new(bytes.to_vec())
		}))
	}

	async fn wait_for_block(
		&self,
		hash: PHash,
	) -> Result<(), cumulus_relay_chain_interface::WaitError> {
		todo!("wait_for_block_not_implemented");
	}

	async fn new_best_notification_stream(&self) -> Pin<Box<dyn Stream<Item = PHeader> + Send>> {
		let imported_headers_stream = self
			.rpc_client
			.subscribe_new_best_heads()
			.await
			.expect("Should be able to subscribe");

		Box::pin(imported_headers_stream.filter_map(|item| async move {
			item.map_err(|err| {
				tracing::error!(target: LOG_TARGET, "Error occured in stream: {}", err)
			})
			.ok()
		}))
	}
}
