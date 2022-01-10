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

use std::sync::Arc;

use async_trait::async_trait;
use cumulus_primitives_core::{
	relay_chain::{
		v1::{CommittedCandidateReceipt, OccupiedCoreAssumption, SessionIndex, ValidatorId},
		v2::ParachainHost,
		Block as PBlock, BlockId, Hash as PHash, InboundHrmpMessage,
	},
	InboundDownwardMessage, ParaId, PersistedValidationData,
};
use cumulus_relay_chain_interface::RelayChainInterface;
use parity_scale_codec::{Decode, Encode};

use jsonrpsee::{
	http_client::{HttpClient, HttpClientBuilder},
	rpc_params,
	types::traits::Client,
};
use parking_lot::Mutex;
use polkadot_client::{ClientHandle, ExecuteWithClient, FullBackend};
use polkadot_service::{
	AuxStore, BabeApi, CollatorPair, Configuration, Handle, NewFull, Role, TaskManager,
};
use sc_client_api::{
	blockchain::BlockStatus, Backend, BlockchainEvents, HeaderBackend, StorageData, StorageProof,
	UsageProvider,
};
use sc_telemetry::TelemetryWorkerHandle;
use sp_api::{ApiError, ProvideRuntimeApi};
use sp_consensus::SyncOracle;
use sp_core::{sp_std::collections::btree_map::BTreeMap, Pair};
use sp_runtime::generic::SignedBlock;
use sp_state_machine::{Backend as StateBackend, StorageValue};
const LOG_TARGET: &str = "relay-chain-local";

/// RelayChainNetwork is used to interact with a full node that is running locally
/// in the same process.
pub struct RelayChainNetwork<Client> {
	full_client: Arc<Client>,
	backend: Arc<FullBackend>,
	sync_oracle: Arc<Mutex<Box<dyn SyncOracle + Send + Sync>>>,
	overseer_handle: Option<Handle>,
	http_client: HttpClient,
}

impl<Client> RelayChainNetwork<Client> {
	pub fn new(
		full_client: Arc<Client>,
		backend: Arc<FullBackend>,
		sync_oracle: Arc<Mutex<Box<dyn SyncOracle + Send + Sync>>>,
		overseer_handle: Option<Handle>,
	) -> Self {
		let url = "http://localhost:9933";
		let http_client = HttpClientBuilder::default().build(url).expect("yo");
		Self { full_client, backend, sync_oracle, overseer_handle, http_client }
	}
}

impl<T> Clone for RelayChainNetwork<T> {
	fn clone(&self) -> Self {
		Self {
			http_client: self.http_client.clone(),
			full_client: self.full_client.clone(),
			backend: self.backend.clone(),
			sync_oracle: self.sync_oracle.clone(),
			overseer_handle: self.overseer_handle.clone(),
		}
	}
}

impl<Client> RelayChainNetwork<Client> {
	async fn call_remote_runtime_function(
		&self,
		method_name: &str,
		block_id: &BlockId,
		payload: Option<Vec<u8>>,
	) -> sp_core::Bytes {
		let payload_bytes =
			payload.map_or(sp_core::Bytes(Vec::new()), |pl| sp_core::Bytes(pl.encode().to_vec()));
		let params = rpc_params! {
			method_name,
			payload_bytes,
			block_id
		};
		let response: Result<sp_core::Bytes, _> =
			self.http_client.request("state_call", params).await;
		response.expect("Should not explode")
	}
}
#[async_trait]
impl<Client> RelayChainInterface for RelayChainNetwork<Client>
where
	Client: ProvideRuntimeApi<PBlock>
		+ BlockchainEvents<PBlock>
		+ AuxStore
		+ UsageProvider<PBlock>
		+ Sync
		+ Send,
	Client::Api: ParachainHost<PBlock> + BabeApi<PBlock>,
{
	async fn retrieve_dmq_contents(
		&self,
		para_id: ParaId,
		relay_parent: PHash,
	) -> Option<Vec<InboundDownwardMessage>> {
		let block_id = BlockId::hash(relay_parent);
		let bytes = self
			.call_remote_runtime_function(
				"ParachainHost_dmq_contents",
				&block_id,
				Some(para_id.encode()),
			)
			.await;

		let decoded = Option::<Vec<InboundDownwardMessage>>::decode(&mut &*bytes.0);

		decoded.unwrap()
	}

	async fn retrieve_all_inbound_hrmp_channel_contents(
		&self,
		para_id: ParaId,
		relay_parent: PHash,
	) -> Option<BTreeMap<ParaId, Vec<InboundHrmpMessage>>> {
		let block_id = BlockId::hash(relay_parent);
		let bytes = self
			.call_remote_runtime_function(
				"ParachainHost_inbound_hrmp_channels_contents",
				&block_id,
				Some(para_id.encode()),
			)
			.await;

		let decoded = Option::<BTreeMap<ParaId, Vec<InboundHrmpMessage>>>::decode(&mut &*bytes.0);

		decoded.unwrap()
	}

	async fn persisted_validation_data(
		&self,
		block_id: &BlockId,
		para_id: ParaId,
		occupied_core_assumption: OccupiedCoreAssumption,
	) -> Result<Option<PersistedValidationData>, ApiError> {
		let bytes = self
			.call_remote_runtime_function(
				"ParachainHost_persisted_validation_data",
				block_id,
				Some((para_id, occupied_core_assumption).encode()),
			)
			.await;

		let decoded = Option::<PersistedValidationData>::decode(&mut &*bytes.0);

		Ok(decoded.unwrap())
	}

	async fn candidate_pending_availability(
		&self,
		block_id: &BlockId,
		para_id: ParaId,
	) -> Result<Option<CommittedCandidateReceipt>, ApiError> {
		let bytes = self
			.call_remote_runtime_function(
				"ParachainHost_candidate_pending_availability",
				block_id,
				Some(para_id.encode()),
			)
			.await;

		let decoded = Option::<CommittedCandidateReceipt<PHash>>::decode(&mut &*bytes.0);

		Ok(decoded.unwrap())
	}

	async fn session_index_for_child(&self, block_id: &BlockId) -> Result<SessionIndex, ApiError> {
		let bytes = self
			.call_remote_runtime_function("ParachainHost_session_index_for_child", block_id, None)
			.await;

		let decoded = SessionIndex::decode(&mut &*bytes.0);

		Ok(decoded.unwrap())
	}

	async fn validators(&self, block_id: &BlockId) -> Result<Vec<ValidatorId>, ApiError> {
		let bytes = self
			.call_remote_runtime_function("ParachainHost_validators", block_id, None)
			.await;

		let decoded = Vec::<ValidatorId>::decode(&mut &*bytes.0);

		Ok(decoded.unwrap())
	}

	fn import_notification_stream(&self) -> sc_client_api::ImportNotifications<PBlock> {
		todo!("import_notification_stream");
	}

	fn finality_notification_stream(&self) -> sc_client_api::FinalityNotifications<PBlock> {
		todo!("finality_notification_stream");
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
		let response: Option<PHash> =
			self.http_client.request("chain_getHead", None).await.expect("yo again");
		tracing::info!(target: LOG_TARGET, response = ?response);
		response.unwrap()
	}

	async fn block_status(&self, block_id: BlockId) -> Result<BlockStatus, sp_blockchain::Error> {
		let hash = match block_id {
			sp_api::BlockId::Hash(hash) => hash,
			sp_api::BlockId::Number(_) => todo!(),
		};

		let params = rpc_params!(hash);
		let response: Option<SignedBlock<PBlock>> =
			self.http_client.request("chain_getBlock", params).await.expect("yo again");
		tracing::info!(target: LOG_TARGET, response = ?response);
		match response {
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
		let storage_key = sp_storage::StorageKey(key.to_vec());
		let hash = match block_id {
			sp_api::BlockId::Hash(hash) => hash,
			sp_api::BlockId::Number(_) => todo!(),
		};
		let params = rpc_params!(storage_key, hash);
		let response: Option<StorageData> =
			self.http_client.request("state_getStorage", params).await.expect("yo again");
		tracing::info!(target: LOG_TARGET, response = ?response);
		Ok(response.map(|v| v.0))
	}

	fn prove_read(
		&self,
		block_id: &BlockId,
		relevant_keys: &Vec<Vec<u8>>,
	) -> Result<Option<StorageProof>, Box<dyn sp_state_machine::Error>> {
		todo!("wait_for_block_not_implemented");
	}

	async fn wait_for_block(
		&self,
		hash: PHash,
	) -> Result<(), cumulus_relay_chain_interface::WaitError> {
		todo!("wait_for_block_not_implemented");
	}
}

/// Builder for a concrete relay chain interface, creatd from a full node. Builds
/// a [`RelayChainNetwork`] to access relay chain data necessary for parachain operation.
///
/// The builder takes a [`polkadot_client::Client`]
/// that wraps a concrete instance. By using [`polkadot_client::ExecuteWithClient`]
/// the builder gets access to this concrete instance and instantiates a RelayChainNetwork with it.
struct RelayChainNetworkBuilder {
	polkadot_client: polkadot_client::Client,
	backend: Arc<FullBackend>,
	sync_oracle: Arc<Mutex<Box<dyn SyncOracle + Send + Sync>>>,
	overseer_handle: Option<Handle>,
}

impl RelayChainNetworkBuilder {
	pub fn build(self) -> Arc<dyn RelayChainInterface> {
		self.polkadot_client.clone().execute_with(self)
	}
}

impl ExecuteWithClient for RelayChainNetworkBuilder {
	type Output = Arc<dyn RelayChainInterface>;

	fn execute_with_client<Client, Api, Backend>(self, client: Arc<Client>) -> Self::Output
	where
		Client: ProvideRuntimeApi<PBlock>
			+ BlockchainEvents<PBlock>
			+ AuxStore
			+ UsageProvider<PBlock>
			+ 'static
			+ Sync
			+ Send,
		Client::Api: ParachainHost<PBlock> + BabeApi<PBlock>,
	{
		Arc::new(RelayChainNetwork::new(
			client,
			self.backend,
			self.sync_oracle,
			self.overseer_handle,
		))
	}
}

/// Build the Polkadot full node using the given `config`.
#[sc_tracing::logging::prefix_logs_with("Relaychain")]
fn build_polkadot_full_node(
	config: Configuration,
	telemetry_worker_handle: Option<TelemetryWorkerHandle>,
) -> Result<(NewFull<polkadot_client::Client>, CollatorPair), polkadot_service::Error> {
	let is_light = matches!(config.role, Role::Light);
	if is_light {
		Err(polkadot_service::Error::Sub("Light client not supported.".into()))
	} else {
		let collator_key = CollatorPair::generate().0;

		let relay_chain_full_node = polkadot_service::build_full(
			config,
			polkadot_service::IsCollator::Yes(collator_key.clone()),
			None,
			true,
			None,
			telemetry_worker_handle,
			polkadot_service::RealOverseerGen,
		)?;

		Ok((relay_chain_full_node, collator_key))
	}
}

/// Builds a relay chain interface by constructing a full relay chain node
pub fn build_relay_chain_network(
	polkadot_config: Configuration,
	telemetry_worker_handle: Option<TelemetryWorkerHandle>,
	task_manager: &mut TaskManager,
) -> Result<(Arc<(dyn RelayChainInterface + 'static)>, CollatorPair), polkadot_service::Error> {
	let (full_node, collator_key) =
		build_polkadot_full_node(polkadot_config, telemetry_worker_handle).map_err(
			|e| match e {
				polkadot_service::Error::Sub(x) => x,
				s => format!("{}", s).into(),
			},
		)?;

	let sync_oracle: Box<dyn SyncOracle + Send + Sync> = Box::new(full_node.network.clone());
	let sync_oracle = Arc::new(Mutex::new(sync_oracle));
	let relay_chain_interface_builder = RelayChainNetworkBuilder {
		polkadot_client: full_node.client.clone(),
		backend: full_node.backend.clone(),
		sync_oracle,
		overseer_handle: full_node.overseer_handle.clone(),
	};
	task_manager.add_child(full_node.task_manager);

	Ok((relay_chain_interface_builder.build(), collator_key))
}
