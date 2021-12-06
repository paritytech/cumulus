use std::{sync::Arc, time::Duration};

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
use parking_lot::RwLock;
use polkadot_client::{ClientHandle, ExecuteWithClient, FullBackend};
use polkadot_service::{
	AuxStore, BabeApi, CollatorPair, Configuration, Handle, NewFull, Role, TaskManager,
};
use sc_client_api::{
	blockchain::BlockStatus, Backend, BlockchainEvents, HeaderBackend, UsageProvider,
};
use sc_telemetry::TelemetryWorkerHandle;
use sp_api::{ApiError, BlockT, ProvideRuntimeApi};
use sp_consensus::SyncOracle;
use sp_core::{sp_std::collections::btree_map::BTreeMap, Pair};
use std::sync::Mutex;

const LOG_TARGET: &str = "relay-chain-interface";

pub trait RelayChainInterface<Block: BlockT> {
	fn get_state_at(
		&self,
		block_id: BlockId,
	) -> Result<<FullBackend as sc_client_api::Backend<PBlock>>::State, sp_blockchain::Error>;

	fn get_import_lock(&self) -> &RwLock<()>;

	fn validators(&self, block_id: &BlockId) -> Result<Vec<ValidatorId>, ApiError>;

	fn block_status(&self, block_id: BlockId) -> Result<BlockStatus, sp_blockchain::Error>;

	fn best_block_hash(&self) -> PHash;
	/// Returns the whole contents of the downward message queue for the parachain we are collating
	/// for.
	///
	/// Returns `None` in case of an error.
	fn retrieve_dmq_contents(
		&self,
		para_id: ParaId,
		relay_parent: PHash,
	) -> Option<Vec<InboundDownwardMessage>>;

	/// Returns channels contents for each inbound HRMP channel addressed to the parachain we are
	/// collating for.
	///
	/// Empty channels are also included.
	fn retrieve_all_inbound_hrmp_channel_contents(
		&self,
		para_id: ParaId,
		relay_parent: PHash,
	) -> Option<BTreeMap<ParaId, Vec<InboundHrmpMessage>>>;

	fn persisted_validation_data(
		&self,
		block_id: &BlockId,
		para_id: ParaId,
		_: OccupiedCoreAssumption,
	) -> Result<Option<PersistedValidationData>, ApiError>;

	fn candidate_pending_availability(
		&self,
		block_id: &BlockId,
		para_id: ParaId,
	) -> Result<Option<CommittedCandidateReceipt>, ApiError>;

	fn session_index_for_child(&self, block_id: &BlockId) -> Result<SessionIndex, ApiError>;

	fn import_notification_stream(&self) -> sc_client_api::ImportNotifications<Block>;
	fn finality_notification_stream(&self) -> sc_client_api::FinalityNotifications<Block>;
	fn storage_changes_notification_stream(
		&self,
		filter_keys: Option<&[sc_client_api::StorageKey]>,
		child_filter_keys: Option<
			&[(sc_client_api::StorageKey, Option<Vec<sc_client_api::StorageKey>>)],
		>,
	) -> sc_client_api::blockchain::Result<sc_client_api::StorageEventStream<Block::Hash>>;

	fn is_major_syncing(&self) -> bool;

	fn slot_duration(&self) -> Result<Duration, sp_blockchain::Error>;

	fn overseer_handle(&self) -> Option<Handle>;
}

pub struct RelayChainDirect<Client> {
	pub full_client: Arc<Client>,
	pub backend: Arc<FullBackend>,
	pub network: Arc<Mutex<Box<dyn SyncOracle + Send + Sync>>>,
	pub overseer_handle: Option<Handle>,
}

impl<T> Clone for RelayChainDirect<T> {
	fn clone(&self) -> Self {
		Self {
			full_client: self.full_client.clone(),
			backend: self.backend.clone(),
			network: self.network.clone(),
			overseer_handle: self.overseer_handle.clone(),
		}
	}
}

impl<Client, Block> RelayChainInterface<Block> for RelayChainDirect<Client>
where
	Client: ProvideRuntimeApi<PBlock> + BlockchainEvents<Block> + AuxStore + UsageProvider<PBlock>,
	Client::Api: ParachainHost<PBlock> + BabeApi<PBlock>,
	Block: BlockT,
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

	fn import_notification_stream(&self) -> sc_client_api::ImportNotifications<Block> {
		self.full_client.import_notification_stream()
	}

	fn finality_notification_stream(&self) -> sc_client_api::FinalityNotifications<Block> {
		self.full_client.finality_notification_stream()
	}

	fn storage_changes_notification_stream(
		&self,
		filter_keys: Option<&[sc_client_api::StorageKey]>,
		child_filter_keys: Option<
			&[(sc_client_api::StorageKey, Option<Vec<sc_client_api::StorageKey>>)],
		>,
	) -> sc_client_api::blockchain::Result<sc_client_api::StorageEventStream<Block::Hash>> {
		self.full_client
			.storage_changes_notification_stream(filter_keys, child_filter_keys)
	}

	fn best_block_hash(&self) -> PHash {
		self.backend.blockchain().info().best_hash
	}

	fn block_status(&self, block_id: BlockId) -> Result<BlockStatus, sp_blockchain::Error> {
		self.backend.blockchain().status(block_id)
	}

	fn get_import_lock(&self) -> &RwLock<()> {
		self.backend.get_import_lock()
	}

	fn get_state_at(
		&self,
		block_id: BlockId,
	) -> Result<<FullBackend as sc_client_api::Backend<PBlock>>::State, sp_blockchain::Error> {
		self.backend.state_at(block_id)
	}

	fn is_major_syncing(&self) -> bool {
		let mut network = self.network.lock().unwrap();
		network.is_major_syncing()
	}

	fn slot_duration(&self) -> Result<Duration, sp_blockchain::Error> {
		Ok(sc_consensus_babe::Config::get_or_compute(&*self.full_client)?.slot_duration())
	}

	fn overseer_handle(&self) -> Option<Handle> {
		self.overseer_handle.clone()
	}
}

/// Builder for a concrete relay chain interface, creatd from a full node. Builds
/// a [`RelayChainDirect`] to access relay chain data necessary for parachain operation.
///
/// The builder takes a [`polkadot_client::Client`]
/// that wraps a concrete instance. By using [`polkadot_client::ExecuteWithClient`]
/// the builder gets access to this concrete instance and instantiates a RelayChainDirect with it.
pub struct RelayChainDirectBuilder {
	polkadot_client: polkadot_client::Client,
	backend: Arc<FullBackend>,
	network: Arc<Mutex<Box<dyn SyncOracle + Send + Sync>>>,
	overseer_handle: Option<Handle>,
}

impl RelayChainDirectBuilder {
	pub fn build(self) -> Arc<dyn RelayChainInterface<PBlock> + Sync + Send> {
		self.polkadot_client.clone().execute_with(self)
	}
}

impl ExecuteWithClient for RelayChainDirectBuilder {
	type Output = Arc<dyn RelayChainInterface<PBlock> + Sync + Send>;

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
		Arc::new(RelayChainDirect {
			full_client: client,
			backend: self.backend,
			network: self.network,
			overseer_handle: self.overseer_handle,
		})
	}
}

impl<T, Block> RelayChainInterface<Block> for Arc<T>
where
	T: RelayChainInterface<Block> + ?Sized,
	Block: BlockT,
{
	fn retrieve_dmq_contents(
		&self,
		para_id: ParaId,
		relay_parent: PHash,
	) -> Option<Vec<InboundDownwardMessage>> {
		(**self).retrieve_dmq_contents(para_id, relay_parent)
	}

	fn retrieve_all_inbound_hrmp_channel_contents(
		&self,
		para_id: ParaId,
		relay_parent: PHash,
	) -> Option<BTreeMap<ParaId, Vec<InboundHrmpMessage>>> {
		(**self).retrieve_all_inbound_hrmp_channel_contents(para_id, relay_parent)
	}

	fn persisted_validation_data(
		&self,
		block_id: &BlockId,
		para_id: ParaId,
		occupied_core_assumption: OccupiedCoreAssumption,
	) -> Result<Option<PersistedValidationData>, ApiError> {
		(**self).persisted_validation_data(block_id, para_id, occupied_core_assumption)
	}

	fn candidate_pending_availability(
		&self,
		block_id: &BlockId,
		para_id: ParaId,
	) -> Result<Option<CommittedCandidateReceipt>, ApiError> {
		(**self).candidate_pending_availability(block_id, para_id)
	}

	fn session_index_for_child(&self, block_id: &BlockId) -> Result<SessionIndex, ApiError> {
		(**self).session_index_for_child(block_id)
	}

	fn validators(&self, block_id: &BlockId) -> Result<Vec<ValidatorId>, ApiError> {
		(**self).validators(block_id)
	}

	fn import_notification_stream(&self) -> sc_client_api::ImportNotifications<Block> {
		(**self).import_notification_stream()
	}

	fn finality_notification_stream(&self) -> sc_client_api::FinalityNotifications<Block> {
		(**self).finality_notification_stream()
	}

	fn storage_changes_notification_stream(
		&self,
		filter_keys: Option<&[sc_client_api::StorageKey]>,
		child_filter_keys: Option<
			&[(sc_client_api::StorageKey, Option<Vec<sc_client_api::StorageKey>>)],
		>,
	) -> sc_client_api::blockchain::Result<sc_client_api::StorageEventStream<Block::Hash>> {
		(**self).storage_changes_notification_stream(filter_keys, child_filter_keys)
	}

	fn best_block_hash(&self) -> PHash {
		(**self).best_block_hash()
	}

	fn block_status(&self, block_id: BlockId) -> Result<BlockStatus, sp_blockchain::Error> {
		(**self).block_status(block_id)
	}

	fn get_import_lock(&self) -> &RwLock<()> {
		(**self).get_import_lock()
	}

	fn get_state_at(
		&self,
		block_id: BlockId,
	) -> Result<<FullBackend as sc_client_api::Backend<PBlock>>::State, sp_blockchain::Error> {
		(**self).get_state_at(block_id)
	}

	fn is_major_syncing(&self) -> bool {
		(**self).is_major_syncing()
	}

	fn slot_duration(&self) -> Result<Duration, sp_blockchain::Error> {
		(**self).slot_duration()
	}

	fn overseer_handle(&self) -> Option<Handle> {
		(**self).overseer_handle()
	}
}

/// Build the Polkadot full node using the given `config`.
#[sc_tracing::logging::prefix_logs_with("Relaychain")]
pub fn build_polkadot_full_node(
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

pub fn build_relay_chain_interface(
	polkadot_config: Configuration,
	telemetry_worker_handle: Option<TelemetryWorkerHandle>,
	task_manager: &mut TaskManager,
) -> Result<
	(Arc<(dyn RelayChainInterface<PBlock> + Send + Sync + 'static)>, CollatorPair),
	polkadot_service::Error,
> {
	let (full_node, collator_key) =
		build_polkadot_full_node(polkadot_config, telemetry_worker_handle).map_err(
			|e| match e {
				polkadot_service::Error::Sub(x) => x,
				s => format!("{}", s).into(),
			},
		)?;

	let sync_oracle: Box<dyn SyncOracle + Send + Sync> = Box::new(full_node.network.clone());
	let network = Arc::new(Mutex::new(sync_oracle));
	let relay_chain_interface_builder = RelayChainDirectBuilder {
		polkadot_client: full_node.client.clone(),
		backend: full_node.backend.clone(),
		network,
		overseer_handle: full_node.overseer_handle.clone(),
	};
	task_manager.add_child(full_node.task_manager);

	Ok((relay_chain_interface_builder.build(), collator_key))
}
