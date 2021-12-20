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

use cumulus_primitives_core::{
	relay_chain::{
		v1::{
			CommittedCandidateReceipt, OccupiedCoreAssumption, ParachainHost, SessionIndex,
			ValidatorId,
		},
		Block as PBlock, BlockId, Hash as PHash, InboundHrmpMessage,
	},
	InboundDownwardMessage, ParaId, PersistedValidationData,
};
use cumulus_relay_chain_interface::{BlockCheckResult, RelayChainInterface};
use polkadot_client::{ClientHandle, ExecuteWithClient, FullBackend};
use polkadot_service::{
	AuxStore, BabeApi, CollatorPair, Configuration, Handle, NewFull, Role, TaskManager,
};
use sc_client_api::{
	blockchain::BlockStatus, Backend, BlockchainEvents, HeaderBackend, StorageProof, UsageProvider,
};
use sc_telemetry::TelemetryWorkerHandle;
use sp_api::{ApiError, ProvideRuntimeApi};
use sp_consensus::SyncOracle;
use sp_core::{sp_std::collections::btree_map::BTreeMap, Pair};
use sp_state_machine::{Backend as StateBackend, StorageValue};
use std::sync::Mutex;
const LOG_TARGET: &str = "relay-chain-local";

/// RelayChainLocal is used to interact with a full node that is running locally
/// in the same process.
pub struct RelayChainLocal<Client> {
	full_client: Arc<Client>,
	backend: Arc<FullBackend>,
	sync_oracle: Arc<Mutex<Box<dyn SyncOracle + Send + Sync>>>,
	overseer_handle: Option<Handle>,
}

impl<Client> RelayChainLocal<Client> {
	pub fn new(
		full_client: Arc<Client>,
		backend: Arc<FullBackend>,
		sync_oracle: Arc<Mutex<Box<dyn SyncOracle + Send + Sync>>>,
		overseer_handle: Option<Handle>,
	) -> Self {
		Self { full_client, backend, sync_oracle, overseer_handle }
	}
}

impl<T> Clone for RelayChainLocal<T> {
	fn clone(&self) -> Self {
		Self {
			full_client: self.full_client.clone(),
			backend: self.backend.clone(),
			sync_oracle: self.sync_oracle.clone(),
			overseer_handle: self.overseer_handle.clone(),
		}
	}
}

impl<Client> RelayChainInterface for RelayChainLocal<Client>
where
	Client: ProvideRuntimeApi<PBlock>
		+ BlockchainEvents<PBlock>
		+ AuxStore
		+ UsageProvider<PBlock>
		+ Sync
		+ Send,
	Client::Api: ParachainHost<PBlock> + BabeApi<PBlock>,
{
	fn retrieve_dmq_contents(
		&self,
		para_id: ParaId,
		relay_parent: PHash,
	) -> Option<Vec<InboundDownwardMessage>> {
		self.full_client
			.runtime_api()
			.dmq_contents_with_context(
				&BlockId::hash(relay_parent),
				sp_core::ExecutionContext::Importing,
				para_id,
			)
			.map_err(|e| {
				tracing::error!(
					target: LOG_TARGET,
					relay_parent = ?relay_parent,
					error = ?e,
					"An error occured during requesting the downward messages.",
				);
			})
			.ok()
	}

	fn retrieve_all_inbound_hrmp_channel_contents(
		&self,
		para_id: ParaId,
		relay_parent: PHash,
	) -> Option<BTreeMap<ParaId, Vec<InboundHrmpMessage>>> {
		self.full_client
			.runtime_api()
			.inbound_hrmp_channels_contents_with_context(
				&BlockId::hash(relay_parent),
				sp_core::ExecutionContext::Importing,
				para_id,
			)
			.map_err(|e| {
				tracing::error!(
					target: LOG_TARGET,
					relay_parent = ?relay_parent,
					error = ?e,
					"An error occured during requesting the inbound HRMP messages.",
				);
			})
			.ok()
	}

	fn persisted_validation_data(
		&self,
		block_id: &BlockId,
		para_id: ParaId,
		occupied_core_assumption: OccupiedCoreAssumption,
	) -> Result<Option<PersistedValidationData>, ApiError> {
		self.full_client.runtime_api().persisted_validation_data(
			block_id,
			para_id,
			occupied_core_assumption,
		)
	}

	fn candidate_pending_availability(
		&self,
		block_id: &BlockId,
		para_id: ParaId,
	) -> Result<Option<CommittedCandidateReceipt>, ApiError> {
		self.full_client.runtime_api().candidate_pending_availability(block_id, para_id)
	}

	fn session_index_for_child(&self, block_id: &BlockId) -> Result<SessionIndex, ApiError> {
		self.full_client.runtime_api().session_index_for_child(block_id)
	}

	fn validators(&self, block_id: &BlockId) -> Result<Vec<ValidatorId>, ApiError> {
		self.full_client.runtime_api().validators(block_id)
	}

	fn import_notification_stream(&self) -> sc_client_api::ImportNotifications<PBlock> {
		self.full_client.import_notification_stream()
	}

	fn finality_notification_stream(&self) -> sc_client_api::FinalityNotifications<PBlock> {
		self.full_client.finality_notification_stream()
	}

	fn storage_changes_notification_stream(
		&self,
		filter_keys: Option<&[sc_client_api::StorageKey]>,
		child_filter_keys: Option<
			&[(sc_client_api::StorageKey, Option<Vec<sc_client_api::StorageKey>>)],
		>,
	) -> sc_client_api::blockchain::Result<sc_client_api::StorageEventStream<PHash>> {
		self.full_client
			.storage_changes_notification_stream(filter_keys, child_filter_keys)
	}

	fn best_block_hash(&self) -> PHash {
		self.backend.blockchain().info().best_hash
	}

	fn block_status(&self, block_id: BlockId) -> Result<BlockStatus, sp_blockchain::Error> {
		self.backend.blockchain().status(block_id)
	}

	fn is_major_syncing(&self) -> bool {
		let mut network = self.sync_oracle.lock().unwrap();
		network.is_major_syncing()
	}

	fn overseer_handle(&self) -> Option<Handle> {
		self.overseer_handle.clone()
	}

	fn get_storage_by_key(&self, block_id: &BlockId, key: &[u8]) -> Option<StorageValue> {
		let state = self
			.backend
			.state_at(*block_id)
			.map_err(|e| {
				tracing::error!(
					target: LOG_TARGET,
					relay_parent = ?block_id,
					error = ?e,
					"Cannot obtain the state of the relay chain.",
				)
			})
			.ok()?;
		state
			.storage(key)
			.map_err(|e| {
				tracing::error!(
					target: LOG_TARGET,
					error = ?e,
					"Cannot decode the hrmp ingress channel index.",
				)
			})
			.ok()?
	}

	fn prove_read(
		&self,
		block_id: &BlockId,
		relevant_keys: &Vec<Vec<u8>>,
	) -> Result<Option<StorageProof>, Box<dyn sp_state_machine::Error>> {
		let state_backend = self
			.backend
			.state_at(*block_id)
			.map_err(|e| {
				tracing::error!(
					target: LOG_TARGET,
					relay_parent = ?block_id,
					error = ?e,
					"Cannot obtain the state of the relay chain.",
				);
			})
			.ok();

		match state_backend {
			Some(state) => sp_state_machine::prove_read(state, relevant_keys)
				.map_err(|e| {
					tracing::error!(
						target: LOG_TARGET,
						relay_parent = ?block_id,
						error = ?e,
						"Failed to collect required relay chain state storage proof.",
					);
					e
				})
				.map(|v| Some(v)),
			None => Ok(None),
		}
	}

	fn check_block_in_chain(
		&self,
		block_id: BlockId,
	) -> Result<BlockCheckResult, sp_blockchain::Error> {
		let _lock = self.backend.get_import_lock();

		match self.backend.blockchain().status(block_id) {
			Ok(BlockStatus::InChain) => return Ok(BlockCheckResult::InChain),
			Err(err) => return Err(err),
			_ => {},
		}

		let listener = self.full_client.import_notification_stream();

		// Now it is safe to drop the lock, even when the block is now imported, it should show
		// up in our registered listener.
		drop(_lock);

		Ok(BlockCheckResult::NotFound(listener))
	}
}

/// Builder for a concrete relay chain interface, creatd from a full node. Builds
/// a [`RelayChainLocal`] to access relay chain data necessary for parachain operation.
///
/// The builder takes a [`polkadot_client::Client`]
/// that wraps a concrete instance. By using [`polkadot_client::ExecuteWithClient`]
/// the builder gets access to this concrete instance and instantiates a RelayChainLocal with it.
struct RelayChainLocalBuilder {
	polkadot_client: polkadot_client::Client,
	backend: Arc<FullBackend>,
	sync_oracle: Arc<Mutex<Box<dyn SyncOracle + Send + Sync>>>,
	overseer_handle: Option<Handle>,
}

impl RelayChainLocalBuilder {
	pub fn build(self) -> Arc<dyn RelayChainInterface> {
		self.polkadot_client.clone().execute_with(self)
	}
}

impl ExecuteWithClient for RelayChainLocalBuilder {
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
		Arc::new(RelayChainLocal::new(client, self.backend, self.sync_oracle, self.overseer_handle))
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
pub fn build_relay_chain_interface(
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
	let relay_chain_interface_builder = RelayChainLocalBuilder {
		polkadot_client: full_node.client.clone(),
		backend: full_node.backend.clone(),
		sync_oracle,
		overseer_handle: full_node.overseer_handle.clone(),
	};
	task_manager.add_child(full_node.task_manager);

	Ok((relay_chain_interface_builder.build(), collator_key))
}
