use cumulus_primitives_core::{
	relay_chain::{v1::ParachainHost, Block as PBlock, BlockId, Hash as PHash, InboundHrmpMessage},
	InboundDownwardMessage, ParaId,
};
use polkadot_client::{ClientHandle, ExecuteWithClient};
use sp_api::ProvideRuntimeApi;
use sp_core::sp_std::{collections::btree_map::BTreeMap, sync::Arc};

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
}
