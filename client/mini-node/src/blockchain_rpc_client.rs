use std::pin::Pin;

use cumulus_relay_chain_interface::RelayChainError;
use cumulus_relay_chain_rpc_interface::RelayChainRpcClient;
use futures::{executor::block_on, Future, Stream, StreamExt};
use polkadot_core_primitives::{Block, Hash, Header};
use polkadot_overseer::RuntimeApiSubsystemClient;
use polkadot_service::HeaderBackend;
use sc_authority_discovery::AuthorityDiscoveryWrapper;
use sc_client_api::{BlockBackend, ProofProvider};
use sp_api::{ApiError, RuntimeApiInfo};
use sp_blockchain::HeaderMetadata;

const LOG_TARGET: &'static str = "blockchain-rpc-client";

#[derive(Clone)]
pub struct BlockChainRpcClient {
	rpc_client: RelayChainRpcClient,
}

impl BlockChainRpcClient {
	pub async fn new(rpc_client: RelayChainRpcClient) -> Self {
		Self { rpc_client }
	}

	pub async fn chain_get_header(
		&self,
		hash: Option<Hash>,
	) -> Result<Option<Header>, RelayChainError> {
		self.rpc_client.chain_get_header(hash).await
	}

	pub async fn block_get_hash(
		&self,
		number: Option<polkadot_service::BlockNumber>,
	) -> Result<Option<Hash>, RelayChainError> {
		self.rpc_client.chain_get_block_hash(number).await
	}
}

#[async_trait::async_trait]
impl RuntimeApiSubsystemClient for BlockChainRpcClient {
	async fn validators(
		&self,
		at: Hash,
	) -> Result<Vec<polkadot_primitives::v2::ValidatorId>, sp_api::ApiError> {
		let result = self.rpc_client.parachain_host_validators(at).await?;
		Ok(result)
	}

	async fn validator_groups(
		&self,
		at: Hash,
	) -> Result<
		(
			Vec<Vec<polkadot_primitives::v2::ValidatorIndex>>,
			polkadot_primitives::v2::GroupRotationInfo<polkadot_core_primitives::BlockNumber>,
		),
		sp_api::ApiError,
	> {
		let result = self.rpc_client.parachain_host_validator_groups(at).await?;
		Ok(result)
	}

	async fn availability_cores(
		&self,
		at: Hash,
	) -> Result<
		Vec<polkadot_primitives::v2::CoreState<Hash, polkadot_core_primitives::BlockNumber>>,
		sp_api::ApiError,
	> {
		let result = self.rpc_client.parachain_host_availability_cores(at).await?;
		Ok(result)
	}

	async fn persisted_validation_data(
		&self,
		at: Hash,
		para_id: cumulus_primitives_core::ParaId,
		assumption: polkadot_primitives::v2::OccupiedCoreAssumption,
	) -> Result<
		Option<
			cumulus_primitives_core::PersistedValidationData<
				Hash,
				polkadot_core_primitives::BlockNumber,
			>,
		>,
		sp_api::ApiError,
	> {
		let result = self
			.rpc_client
			.parachain_host_persisted_validation_data(at, para_id, assumption)
			.await?;
		Ok(result)
	}

	async fn assumed_validation_data(
		&self,
		at: Hash,
		para_id: cumulus_primitives_core::ParaId,
		expected_persisted_validation_data_hash: Hash,
	) -> Result<
		Option<(
			cumulus_primitives_core::PersistedValidationData<
				Hash,
				polkadot_core_primitives::BlockNumber,
			>,
			polkadot_primitives::v2::ValidationCodeHash,
		)>,
		sp_api::ApiError,
	> {
		let result = self
			.rpc_client
			.parachain_host_assumed_validation_data(
				at,
				para_id,
				expected_persisted_validation_data_hash,
			)
			.await?;
		Ok(result)
	}

	async fn check_validation_outputs(
		&self,
		at: Hash,
		para_id: cumulus_primitives_core::ParaId,
		outputs: polkadot_primitives::v2::CandidateCommitments,
	) -> Result<bool, sp_api::ApiError> {
		let result = self
			.rpc_client
			.parachain_host_check_validation_outputs(at, para_id, outputs)
			.await?;
		Ok(result)
	}

	async fn session_index_for_child(
		&self,
		at: Hash,
	) -> Result<polkadot_primitives::v2::SessionIndex, sp_api::ApiError> {
		let result = self.rpc_client.parachain_host_session_index_for_child(at).await?;
		Ok(result)
	}

	async fn validation_code(
		&self,
		at: Hash,
		para_id: cumulus_primitives_core::ParaId,
		assumption: polkadot_primitives::v2::OccupiedCoreAssumption,
	) -> Result<Option<polkadot_primitives::v2::ValidationCode>, sp_api::ApiError> {
		let result =
			self.rpc_client.parachain_host_validation_code(at, para_id, assumption).await?;
		Ok(result)
	}

	async fn candidate_pending_availability(
		&self,
		at: Hash,
		para_id: cumulus_primitives_core::ParaId,
	) -> Result<Option<polkadot_primitives::v2::CommittedCandidateReceipt<Hash>>, sp_api::ApiError>
	{
		let result = self
			.rpc_client
			.parachain_host_candidate_pending_availability(at, para_id)
			.await?;
		Ok(result)
	}

	async fn candidate_events(
		&self,
		at: Hash,
	) -> Result<Vec<polkadot_primitives::v2::CandidateEvent<Hash>>, sp_api::ApiError> {
		let result = self.rpc_client.parachain_host_candidate_events(at).await?;
		Ok(result)
	}

	async fn dmq_contents(
		&self,
		at: Hash,
		recipient: cumulus_primitives_core::ParaId,
	) -> Result<
		Vec<cumulus_primitives_core::InboundDownwardMessage<polkadot_core_primitives::BlockNumber>>,
		sp_api::ApiError,
	> {
		let result = self.rpc_client.parachain_host_dmq_contents(recipient, at).await?;
		Ok(result)
	}

	async fn inbound_hrmp_channels_contents(
		&self,
		at: Hash,
		recipient: cumulus_primitives_core::ParaId,
	) -> Result<
		std::collections::BTreeMap<
			cumulus_primitives_core::ParaId,
			Vec<
				polkadot_core_primitives::InboundHrmpMessage<polkadot_core_primitives::BlockNumber>,
			>,
		>,
		sp_api::ApiError,
	> {
		let result = self
			.rpc_client
			.parachain_host_inbound_hrmp_channels_contents(recipient, at)
			.await?;
		Ok(result)
	}

	async fn validation_code_by_hash(
		&self,
		at: Hash,
		validation_code_hash: polkadot_primitives::v2::ValidationCodeHash,
	) -> Result<Option<polkadot_primitives::v2::ValidationCode>, sp_api::ApiError> {
		let result = self
			.rpc_client
			.parachain_host_validation_code_by_hash(at, validation_code_hash)
			.await?;
		Ok(result)
	}

	async fn on_chain_votes(
		&self,
		at: Hash,
	) -> Result<Option<polkadot_primitives::v2::ScrapedOnChainVotes<Hash>>, sp_api::ApiError> {
		let result = self.rpc_client.parachain_host_on_chain_votes(at).await?;
		Ok(result)
	}

	async fn session_info(
		&self,
		at: Hash,
		index: polkadot_primitives::v2::SessionIndex,
	) -> Result<Option<polkadot_primitives::v2::SessionInfo>, sp_api::ApiError> {
		let result = self.rpc_client.parachain_host_session_info(at, index).await?;
		Ok(result)
	}

	async fn session_info_before_version_2(
		&self,
		at: Hash,
		index: polkadot_primitives::v2::SessionIndex,
	) -> Result<Option<polkadot_primitives::v2::OldV1SessionInfo>, sp_api::ApiError> {
		let result =
			self.rpc_client.parachain_host_session_info_before_version_2(at, index).await?;
		Ok(result)
	}

	async fn submit_pvf_check_statement(
		&self,
		at: Hash,
		stmt: polkadot_primitives::v2::PvfCheckStatement,
		signature: polkadot_primitives::v2::ValidatorSignature,
	) -> Result<(), sp_api::ApiError> {
		let result = self
			.rpc_client
			.parachain_host_submit_pvf_check_statement(at, stmt, signature)
			.await?;
		Ok(result)
	}

	async fn pvfs_require_precheck(
		&self,
		at: Hash,
	) -> Result<Vec<polkadot_primitives::v2::ValidationCodeHash>, sp_api::ApiError> {
		let result = self.rpc_client.parachain_host_pvfs_require_precheck(at).await?;
		Ok(result)
	}

	async fn validation_code_hash(
		&self,
		at: Hash,
		para_id: cumulus_primitives_core::ParaId,
		assumption: polkadot_primitives::v2::OccupiedCoreAssumption,
	) -> Result<Option<polkadot_primitives::v2::ValidationCodeHash>, sp_api::ApiError> {
		let result = self
			.rpc_client
			.parachain_host_validation_code_hash(at, para_id, assumption)
			.await?;
		Ok(result)
	}

	async fn current_epoch(&self, at: Hash) -> Result<sp_consensus_babe::Epoch, sp_api::ApiError> {
		let result = self.rpc_client.babe_api_current_epoch(at).await?;
		Ok(result)
	}

	async fn authorities(
		&self,
		at: Hash,
	) -> std::result::Result<Vec<polkadot_primitives::v2::AuthorityDiscoveryId>, sp_api::ApiError> {
		let result = self.rpc_client.authority_discovery_authorities(at).await?;
		Ok(result)
	}

	async fn api_version_parachain_host(&self, at: Hash) -> Result<Option<u32>, sp_api::ApiError> {
		let api_id = <dyn polkadot_primitives::runtime_api::ParachainHost<Block>>::ID;
		let result = self.rpc_client.runtime_version(at).await.map(|v| v.api_version(&api_id))?;
		Ok(result)
	}

	async fn staging_get_disputes(
		&self,
		at: Hash,
	) -> Result<
		Vec<(
			polkadot_primitives::v2::SessionIndex,
			polkadot_primitives::v2::CandidateHash,
			polkadot_primitives::v2::DisputeState<polkadot_primitives::v2::BlockNumber>,
		)>,
		ApiError,
	> {
		let result = self.rpc_client.parachain_host_staging_get_disputes(at).await?;
		Ok(result)
	}
}

#[async_trait::async_trait]
impl AuthorityDiscoveryWrapper<Block> for BlockChainRpcClient {
	async fn authorities(
		&self,
		at: Hash,
	) -> std::result::Result<Vec<polkadot_primitives::v2::AuthorityDiscoveryId>, sp_api::ApiError> {
		let result = self.rpc_client.authority_discovery_authorities(at).await?;
		Ok(result)
	}
}

impl BlockChainRpcClient {
	pub async fn import_notification_stream_async(
		&self,
	) -> Pin<Box<dyn Stream<Item = Header> + Send>> {
		self.rpc_client
			.get_imported_heads_stream()
			.await
			.expect("subscribe_all_heads")
			.boxed()
	}

	pub async fn finality_notification_stream_async(
		&self,
	) -> Pin<Box<dyn Stream<Item = Header> + Send>> {
		self.rpc_client
			.get_finalized_heads_stream()
			.await
			.expect("imported_headers_stream")
			.boxed()
	}
}

fn block_local<T>(fut: impl Future<Output = T>) -> T {
	let tokio_handle = tokio::runtime::Handle::current();
	tokio::task::block_in_place(|| tokio_handle.block_on(fut))
}

impl HeaderBackend<Block> for BlockChainRpcClient {
	fn header(
		&self,
		id: sp_api::BlockId<Block>,
	) -> sp_blockchain::Result<Option<<Block as polkadot_service::BlockT>::Header>> {
		if let sp_api::BlockId::Hash(hash) = id {
			block_on(self.rpc_client.chain_get_header(Some(hash)))
				.map_err(|err| sp_blockchain::Error::Backend(err.to_string()))
		} else {
			unimplemented!("header with number not supported")
		}
	}

	fn info(&self) -> sp_blockchain::Info<Block> {
		tracing::debug!(target: LOG_TARGET, "BlockBackend::block_status");

		let best_header = block_local(self.rpc_client.chain_get_header(None))
			.expect("get_header")
			.unwrap();
		tracing::debug!(
			target: LOG_TARGET,
			"BlockBackend::block_status - succeeded to get header parent: {:?}, number: {:?}",
			best_header.parent_hash,
			best_header.number
		);
		let genesis_hash = block_local(self.rpc_client.chain_get_head(Some(0))).expect("get_head");
		let finalized_head =
			block_local(self.rpc_client.chain_get_finalized_head()).expect("get_head");
		let finalized_header = block_local(self.rpc_client.chain_get_header(Some(finalized_head)))
			.expect("get_head")
			.unwrap();
		sp_blockchain::Info {
			best_hash: best_header.hash(),
			best_number: best_header.number,
			genesis_hash,
			finalized_hash: finalized_head,
			finalized_number: finalized_header.number,
			finalized_state: None,
			number_leaves: 1,
			block_gap: None,
		}
	}

	fn status(
		&self,
		_id: sp_api::BlockId<Block>,
	) -> sp_blockchain::Result<sp_blockchain::BlockStatus> {
		unimplemented!()
	}

	fn number(
		&self,
		hash: <Block as polkadot_service::BlockT>::Hash,
	) -> sp_blockchain::Result<
		Option<<<Block as polkadot_service::BlockT>::Header as polkadot_service::HeaderT>::Number>,
	> {
		tracing::debug!(target: LOG_TARGET, "BlockBackend::block_status");
		block_local(self.rpc_client.chain_get_header(Some(hash)))
			.map_err(|err| sp_blockchain::Error::Backend(err.to_string()))
			.map(|maybe_header| maybe_header.map(|header| header.number))
	}

	fn hash(
		&self,
		number: polkadot_service::NumberFor<Block>,
	) -> sp_blockchain::Result<Option<<Block as polkadot_service::BlockT>::Hash>> {
		block_local(self.rpc_client.chain_get_block_hash(number.into()))
			.map_err(|err| sp_blockchain::Error::Backend(err.to_string()))
	}
}
