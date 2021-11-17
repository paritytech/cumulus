use std::sync::Arc;

use cumulus_primitives_core::{
	relay_chain::{
		v1::{CommittedCandidateReceipt, OccupiedCoreAssumption, ParachainHost},
		Block as PBlock, BlockId, Hash as PHash, InboundHrmpMessage,
	},
	InboundDownwardMessage, ParaId, PersistedValidationData,
};
use sp_core::sp_std::collections::btree_map::BTreeMap;
use polkadot_client::{ClientHandle, ExecuteWithClient};
use sp_api::{ApiError, ProvideRuntimeApi};

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

pub struct RelayChainDirect<Client> {
	pub polkadot_client: Arc<Client>,
}


impl <Client>RelayChainInterface for RelayChainDirect<Client>
where
	Client: ProvideRuntimeApi<PBlock>,
	Client::Api: ParachainHost<PBlock>,
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
}

pub struct RelayChainDirectBuilder {
	polkadot_client: polkadot_client::Client,
}

impl RelayChainDirectBuilder {
	pub fn build(self) -> Arc<dyn RelayChainInterface + Sync + Send> {
		self.polkadot_client.clone().execute_with(self)
	}
}

impl ExecuteWithClient for RelayChainDirectBuilder {
	type Output = Arc<dyn RelayChainInterface + Sync + Send>;

	fn execute_with_client<Client, Api, Backend>(self, client: Arc<Client>) -> Self::Output
	where
		Client: ProvideRuntimeApi<PBlock> + 'static + Sync + Send,
		Client::Api: ParachainHost<PBlock>,
	{
		Arc::new(RelayChainDirect { polkadot_client: client })
	}
}

impl RelayChainInterface for Arc<dyn RelayChainInterface + Sync + Send> {
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
}

impl <Client>RelayChainInterface for Arc<RelayChainDirect<Client>>
	where
		Client: ProvideRuntimeApi<PBlock> + 'static + Sync + Send,
	Client::Api: ParachainHost<PBlock>, {
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
}

pub fn build_relay_chain_direct(client: polkadot_client::Client) -> Arc<(dyn RelayChainInterface + Send + Sync + 'static)> {
	let relay_chain_builder = RelayChainDirectBuilder { polkadot_client: client };
	relay_chain_builder.build()
}
