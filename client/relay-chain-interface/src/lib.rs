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
use parking_lot::RwLock;
use polkadot_client::{ClientHandle, ExecuteWithClient, FullBackend};
use sc_client_api::{blockchain::BlockStatus, Backend, BlockchainEvents, HeaderBackend};
use sp_api::{ApiError, BlockT, ProvideRuntimeApi};
use sp_core::sp_std::collections::btree_map::BTreeMap;

const LOG_TARGET: &str = "cumulus-collator";

pub trait RelayChainInterface<Block: BlockT> {
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
}

pub struct RelayChainDirect<Client> {
	pub polkadot_client: Arc<Client>,
	pub backend: Arc<FullBackend>,
}

impl<T> Clone for RelayChainDirect<T> {
	fn clone(&self) -> Self {
		Self { polkadot_client: self.polkadot_client.clone(), backend: self.backend.clone() }
	}
}

impl<Client, Block> RelayChainInterface<Block> for RelayChainDirect<Client>
where
	Client: ProvideRuntimeApi<PBlock> + BlockchainEvents<Block>,
	Client::Api: ParachainHost<PBlock>,
	Block: BlockT,
{
	fn retrieve_dmq_contents(
		&self,
		para_id: ParaId,
		relay_parent: PHash,
	) -> Option<Vec<InboundDownwardMessage>> {
		self.polkadot_client
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
		self.polkadot_client
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
		self.polkadot_client.runtime_api().persisted_validation_data(
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
		self.polkadot_client
			.runtime_api()
			.candidate_pending_availability(block_id, para_id)
	}

	fn session_index_for_child(&self, block_id: &BlockId) -> Result<SessionIndex, ApiError> {
		self.polkadot_client.runtime_api().session_index_for_child(block_id)
	}

	fn validators(&self, block_id: &BlockId) -> Result<Vec<ValidatorId>, ApiError> {
		self.polkadot_client.runtime_api().validators(block_id)
	}

	fn import_notification_stream(&self) -> sc_client_api::ImportNotifications<Block> {
		self.polkadot_client.import_notification_stream()
	}

	fn finality_notification_stream(&self) -> sc_client_api::FinalityNotifications<Block> {
		self.polkadot_client.finality_notification_stream()
	}

	fn storage_changes_notification_stream(
		&self,
		filter_keys: Option<&[sc_client_api::StorageKey]>,
		child_filter_keys: Option<
			&[(sc_client_api::StorageKey, Option<Vec<sc_client_api::StorageKey>>)],
		>,
	) -> sc_client_api::blockchain::Result<sc_client_api::StorageEventStream<Block::Hash>> {
		self.polkadot_client
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
}

pub struct RelayChainDirectBuilder {
	polkadot_client: polkadot_client::Client,
	backend: Arc<FullBackend>,
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
		Client: ProvideRuntimeApi<PBlock> + BlockchainEvents<PBlock> + 'static + Sync + Send,
		Client::Api: ParachainHost<PBlock>,
	{
		Arc::new(RelayChainDirect { polkadot_client: client, backend: self.backend })
	}
}

impl<Block: BlockT> RelayChainInterface<Block>
	for Arc<dyn RelayChainInterface<Block> + Sync + Send>
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
}

impl<T, Block> RelayChainInterface<Block> for Arc<T>
where
	T: RelayChainInterface<Block>,
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
}

pub fn build_relay_chain_direct(
	client: polkadot_client::Client,
	backend: Arc<FullBackend>,
) -> Arc<(dyn RelayChainInterface<PBlock> + Send + Sync + 'static)> {
	let relay_chain_builder = RelayChainDirectBuilder { polkadot_client: client, backend };
	relay_chain_builder.build()
}
