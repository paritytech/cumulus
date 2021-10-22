use cumulus_primitives_core::{
	relay_chain::{
		v1::{CommittedCandidateReceipt, OccupiedCoreAssumption, ParachainHost},
		BlakeTwo256, Block as PBlock, BlockId, Hash as PHash, InboundHrmpMessage,
	},
	InboundDownwardMessage, ParaId, PersistedValidationData,
};
use polkadot_client::{AbstractClient, ClientHandle, ExecuteWithClient, RuntimeApiCollection};
use sc_client_api::{
	BlockchainEvents, FinalityNotifications, ImportNotifications, StorageEventStream, StorageKey,
};
use sp_api::{ApiError, ApiExt, ProvideRuntimeApi};
use sp_core::sp_std::{collections::btree_map::BTreeMap, sync::Arc};
use std::marker::PhantomData;

const LOG_TARGET: &str = "cumulus-collator";

pub trait RelayChainInterface {
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
}

#[derive(Clone)]
pub struct RelayChainDirect<T> {
	pub polkadot_client: T,
}

/// Special structure to run [`ParachainInherentData::create_at`] with a [`Client`].
struct DmqContentsWithClient {
	relay_parent: PHash,
	para_id: ParaId,
}

impl ExecuteWithClient for DmqContentsWithClient {
	type Output = Option<Vec<InboundDownwardMessage>>;

	fn execute_with_client<Client, Api, Backend>(self, client: Arc<Client>) -> Self::Output
	where
		Client: ProvideRuntimeApi<PBlock>,
		Client::Api: ParachainHost<PBlock>,
	{
		let my_client = &*client;
		my_client
			.runtime_api()
			.dmq_contents_with_context(
				&BlockId::hash(self.relay_parent),
				sp_core::ExecutionContext::Importing,
				self.para_id,
			)
			.map_err(|e| {
				tracing::error!(
					target: LOG_TARGET,
					relay_parent = ?self.relay_parent,
					error = ?e,
					"An error occured during requesting the downward messages.",
				);
			})
			.ok()
	}
}

struct InboundHrmpMessageWithClient {
	relay_parent: PHash,
	para_id: ParaId,
}

impl ExecuteWithClient for InboundHrmpMessageWithClient {
	type Output = Option<BTreeMap<ParaId, Vec<InboundHrmpMessage>>>;

	fn execute_with_client<Client, Api, Backend>(self, client: Arc<Client>) -> Self::Output
	where
		Client: ProvideRuntimeApi<PBlock>,
		Client::Api: ParachainHost<PBlock>,
	{
		let my_client = &*client;
		my_client
			.runtime_api()
			.inbound_hrmp_channels_contents_with_context(
				&BlockId::hash(self.relay_parent),
				sp_core::ExecutionContext::Importing,
				self.para_id,
			)
			.map_err(|e| {
				tracing::error!(
					target: LOG_TARGET,
					relay_parent = ?self.relay_parent,
					error = ?e,
					"An error occured during requesting the inbound HRMP messages.",
				);
			})
			.ok()
	}
}
struct CandidatePendingAvailabilityWithClient {
	block_id: BlockId,
	para_id: ParaId,
}

impl ExecuteWithClient for CandidatePendingAvailabilityWithClient {
	type Output = Result<Option<CommittedCandidateReceipt>, ApiError>;

	fn execute_with_client<Client, Api, Backend>(self, client: Arc<Client>) -> Self::Output
	where
		Client: ProvideRuntimeApi<PBlock>,
		Client::Api: ParachainHost<PBlock>,
	{
		client
			.runtime_api()
			.candidate_pending_availability(&self.block_id, self.para_id)
	}
}

struct PersistedValidationDataWithClient {
	block_id: BlockId,
	para_id: ParaId,
	occupied_core_assumption: OccupiedCoreAssumption,
}

impl ExecuteWithClient for PersistedValidationDataWithClient {
	type Output = Result<Option<PersistedValidationData>, ApiError>;

	fn execute_with_client<Client, Api, Backend>(self, client: Arc<Client>) -> Self::Output
	where
		Client: ProvideRuntimeApi<PBlock>,
		Client::Api: ParachainHost<PBlock>,
	{
		client.runtime_api().persisted_validation_data(
			&self.block_id,
			self.para_id,
			self.occupied_core_assumption,
		)
	}
}

impl RelayChainInterface for RelayChainDirect<polkadot_client::Client> {
	fn retrieve_dmq_contents(
		&self,
		para_id: ParaId,
		relay_parent: PHash,
	) -> Option<Vec<InboundDownwardMessage>> {
		self.polkadot_client
			.execute_with(DmqContentsWithClient { para_id, relay_parent })
	}

	fn retrieve_all_inbound_hrmp_channel_contents(
		&self,
		para_id: ParaId,
		relay_parent: PHash,
	) -> Option<BTreeMap<ParaId, Vec<InboundHrmpMessage>>> {
		self.polkadot_client
			.execute_with(InboundHrmpMessageWithClient { para_id, relay_parent })
	}

	fn persisted_validation_data(
		&self,
		block_id: &BlockId,
		para_id: ParaId,
		occupied_core_assumption: OccupiedCoreAssumption,
	) -> Result<Option<PersistedValidationData>, ApiError> {
		self.polkadot_client.execute_with(PersistedValidationDataWithClient {
			block_id: block_id.clone(),
			para_id,
			occupied_core_assumption,
		})
	}

	fn candidate_pending_availability(
		&self,
		block_id: &BlockId,
		para_id: ParaId,
	) -> Result<Option<CommittedCandidateReceipt>, ApiError> {
		self.polkadot_client.execute_with(CandidatePendingAvailabilityWithClient {
			block_id: block_id.clone(),
			para_id,
		})
	}
}
