use std::pin::Pin;

use cumulus_relay_chain_interface::{RelayChainError, RelayChainResult};
use cumulus_relay_chain_rpc_interface::RelayChainRPCClient;
use futures::{executor::block_on, Future, Stream, StreamExt};
use polkadot_core_primitives::{Block, BlockId, Hash, Header};
use polkadot_node_network_protocol::Arc;
use polkadot_overseer::OverseerRuntimeClient;
use polkadot_service::{runtime_traits::BlockIdTo, HeaderBackend};
use sc_authority_discovery::AuthorityDiscoveryWrapper;
use sc_client_api::{BlockBackend, BlockchainEvents, ProofProvider};
use sp_api::ApiError;
use sp_blockchain::HeaderMetadata;
use tokio::runtime::Handle;
use url::Url;

const LOG_TARGET: &'static str = "blockchain-rpc-client";

#[derive(Clone)]
pub struct BlockChainRPCClient {
	rpc_client: RelayChainRPCClient,
}

impl BlockChainRPCClient {
	pub async fn new(url: Url) -> Self {
		Self { rpc_client: RelayChainRPCClient::new(url).await.expect("should not fail") }
	}

	pub async fn chain_get_header(
		&self,
		hash: Option<Hash>,
	) -> Result<Option<Header>, RelayChainError> {
		self.rpc_client.chain_get_header(hash).await
	}
}

#[async_trait::async_trait]
impl OverseerRuntimeClient for BlockChainRPCClient {
	async fn validators(
		&self,
		at: &polkadot_core_primitives::BlockId,
	) -> Result<Vec<polkadot_primitives::v2::ValidatorId>, sp_api::ApiError> {
		if let BlockId::Hash(hash) = at {
			self.rpc_client
				.parachain_host_validators(*hash)
				.await
				.map_err(|e| sp_api::ApiError::Application(Box::new(e) as Box<_>))
		} else {
			Err(sp_api::ApiError::Application(Box::new(RelayChainError::GenericError(
				"Only hash is supported for RPC methods".to_string(),
			)) as Box<_>))
		}
	}

	async fn validator_groups(
		&self,
		at: &polkadot_core_primitives::BlockId,
	) -> Result<
		(
			Vec<Vec<polkadot_primitives::v2::ValidatorIndex>>,
			polkadot_primitives::v2::GroupRotationInfo<polkadot_core_primitives::BlockNumber>,
		),
		sp_api::ApiError,
	> {
		if let BlockId::Hash(hash) = at {
			self.rpc_client
				.parachain_host_validator_groups(*hash)
				.await
				.map_err(|e| sp_api::ApiError::Application(Box::new(e) as Box<_>))
		} else {
			Err(sp_api::ApiError::Application(Box::new(RelayChainError::GenericError(
				"Only hash is supported for RPC methods".to_string(),
			)) as Box<_>))
		}
	}

	async fn availability_cores(
		&self,
		at: &polkadot_core_primitives::BlockId,
	) -> Result<
		Vec<polkadot_primitives::v2::CoreState<Hash, polkadot_core_primitives::BlockNumber>>,
		sp_api::ApiError,
	> {
		if let BlockId::Hash(hash) = at {
			self.rpc_client
				.parachain_host_availability_cores(*hash)
				.await
				.map_err(|e| sp_api::ApiError::Application(Box::new(e) as Box<_>))
		} else {
			Err(sp_api::ApiError::Application(Box::new(RelayChainError::GenericError(
				"Only hash is supported for RPC methods".to_string(),
			)) as Box<_>))
		}
	}

	async fn persisted_validation_data(
		&self,
		at: &polkadot_core_primitives::BlockId,
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
		if let BlockId::Hash(hash) = at {
			self.rpc_client
				.parachain_host_persisted_validation_data(*hash, para_id, assumption)
				.await
				.map_err(|e| sp_api::ApiError::Application(Box::new(e) as Box<_>))
		} else {
			Err(sp_api::ApiError::Application(Box::new(RelayChainError::GenericError(
				"Only hash is supported for RPC methods".to_string(),
			)) as Box<_>))
		}
	}

	async fn assumed_validation_data(
		&self,
		at: &polkadot_core_primitives::BlockId,
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
		if let BlockId::Hash(hash) = at {
			self.rpc_client
				.parachain_host_assumed_validation_data(
					*hash,
					para_id,
					expected_persisted_validation_data_hash,
				)
				.await
				.map_err(|e| sp_api::ApiError::Application(Box::new(e) as Box<_>))
		} else {
			Err(sp_api::ApiError::Application(Box::new(RelayChainError::GenericError(
				"Only hash is supported for RPC methods".to_string(),
			)) as Box<_>))
		}
	}

	async fn check_validation_outputs(
		&self,
		at: &polkadot_core_primitives::BlockId,
		para_id: cumulus_primitives_core::ParaId,
		outputs: polkadot_primitives::v2::CandidateCommitments,
	) -> Result<bool, sp_api::ApiError> {
		if let BlockId::Hash(hash) = at {
			self.rpc_client
				.parachain_host_check_validation_outputs(*hash, para_id, outputs)
				.await
				.map_err(|e| sp_api::ApiError::Application(Box::new(e) as Box<_>))
		} else {
			Err(sp_api::ApiError::Application(Box::new(RelayChainError::GenericError(
				"Only hash is supported for RPC methods".to_string(),
			)) as Box<_>))
		}
	}

	async fn session_index_for_child(
		&self,
		at: &polkadot_core_primitives::BlockId,
	) -> Result<polkadot_primitives::v2::SessionIndex, sp_api::ApiError> {
		if let BlockId::Hash(hash) = at {
			self.rpc_client
				.parachain_host_session_index_for_child(*hash)
				.await
				.map_err(|e| sp_api::ApiError::Application(Box::new(e) as Box<_>))
		} else {
			Err(sp_api::ApiError::Application(Box::new(RelayChainError::GenericError(
				"Only hash is supported for RPC methods".to_string(),
			)) as Box<_>))
		}
	}

	async fn validation_code(
		&self,
		at: &polkadot_core_primitives::BlockId,
		para_id: cumulus_primitives_core::ParaId,
		assumption: polkadot_primitives::v2::OccupiedCoreAssumption,
	) -> Result<Option<polkadot_primitives::v2::ValidationCode>, sp_api::ApiError> {
		if let BlockId::Hash(hash) = at {
			self.rpc_client
				.parachain_host_validation_code(*hash, para_id, assumption)
				.await
				.map_err(|e| sp_api::ApiError::Application(Box::new(e) as Box<_>))
		} else {
			Err(sp_api::ApiError::Application(Box::new(RelayChainError::GenericError(
				"Only hash is supported for RPC methods".to_string(),
			)) as Box<_>))
		}
	}

	async fn candidate_pending_availability(
		&self,
		at: &polkadot_core_primitives::BlockId,
		para_id: cumulus_primitives_core::ParaId,
	) -> Result<Option<polkadot_primitives::v2::CommittedCandidateReceipt<Hash>>, sp_api::ApiError>
	{
		if let BlockId::Hash(hash) = at {
			self.rpc_client
				.parachain_host_candidate_pending_availability(*hash, para_id)
				.await
				.map_err(|e| sp_api::ApiError::Application(Box::new(e) as Box<_>))
		} else {
			Err(sp_api::ApiError::Application(Box::new(RelayChainError::GenericError(
				"Only hash is supported for RPC methods".to_string(),
			)) as Box<_>))
		}
	}

	async fn candidate_events(
		&self,
		at: &polkadot_core_primitives::BlockId,
	) -> Result<Vec<polkadot_primitives::v2::CandidateEvent<Hash>>, sp_api::ApiError> {
		if let BlockId::Hash(hash) = at {
			self.rpc_client
				.parachain_host_candidate_events(*hash)
				.await
				.map_err(|e| sp_api::ApiError::Application(Box::new(e) as Box<_>))
		} else {
			Err(sp_api::ApiError::Application(Box::new(RelayChainError::GenericError(
				"Only hash is supported for RPC methods".to_string(),
			)) as Box<_>))
		}
	}

	async fn dmq_contents(
		&self,
		at: &polkadot_core_primitives::BlockId,
		recipient: cumulus_primitives_core::ParaId,
	) -> Result<
		Vec<cumulus_primitives_core::InboundDownwardMessage<polkadot_core_primitives::BlockNumber>>,
		sp_api::ApiError,
	> {
		if let BlockId::Hash(hash) = at {
			self.rpc_client
				.parachain_host_dmq_contents(recipient, *hash)
				.await
				.map_err(|e| sp_api::ApiError::Application(Box::new(e) as Box<_>))
		} else {
			Err(sp_api::ApiError::Application(Box::new(RelayChainError::GenericError(
				"Only hash is supported for RPC methods".to_string(),
			)) as Box<_>))
		}
	}

	async fn inbound_hrmp_channels_contents(
		&self,
		at: &polkadot_core_primitives::BlockId,
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
		if let BlockId::Hash(hash) = at {
			self.rpc_client
				.parachain_host_inbound_hrmp_channels_contents(recipient, *hash)
				.await
				.map_err(|e| sp_api::ApiError::Application(Box::new(e) as Box<_>))
		} else {
			Err(sp_api::ApiError::Application(Box::new(RelayChainError::GenericError(
				"Only hash is supported for RPC methods".to_string(),
			)) as Box<_>))
		}
	}

	async fn validation_code_by_hash(
		&self,
		at: &polkadot_core_primitives::BlockId,
		validation_code_hash: polkadot_primitives::v2::ValidationCodeHash,
	) -> Result<Option<polkadot_primitives::v2::ValidationCode>, sp_api::ApiError> {
		if let BlockId::Hash(hash) = at {
			self.rpc_client
				.parachain_host_validation_code_by_hash(*hash, validation_code_hash)
				.await
				.map_err(|e| sp_api::ApiError::Application(Box::new(e) as Box<_>))
		} else {
			Err(sp_api::ApiError::Application(Box::new(RelayChainError::GenericError(
				"Only hash is supported for RPC methods".to_string(),
			)) as Box<_>))
		}
	}

	async fn on_chain_votes(
		&self,
		at: &polkadot_core_primitives::BlockId,
	) -> Result<Option<polkadot_primitives::v2::ScrapedOnChainVotes<Hash>>, sp_api::ApiError> {
		todo!()
	}

	async fn session_info(
		&self,
		at: &polkadot_core_primitives::BlockId,
		index: polkadot_primitives::v2::SessionIndex,
	) -> Result<Option<polkadot_primitives::v2::SessionInfo>, sp_api::ApiError> {
		if let BlockId::Hash(hash) = at {
			self.rpc_client
				.parachain_host_session_info(*hash, index)
				.await
				.map_err(|e| sp_api::ApiError::Application(Box::new(e) as Box<_>))
		} else {
			Err(sp_api::ApiError::Application(Box::new(RelayChainError::GenericError(
				"Only hash is supported for RPC methods".to_string(),
			)) as Box<_>))
		}
	}

	async fn session_info_before_version_2(
		&self,
		at: &polkadot_core_primitives::BlockId,
		index: polkadot_primitives::v2::SessionIndex,
	) -> Result<Option<polkadot_primitives::v2::SessionInfo>, sp_api::ApiError> {
		todo!()
	}

	async fn submit_pvf_check_statement(
		&self,
		at: &polkadot_core_primitives::BlockId,
		stmt: polkadot_primitives::v2::PvfCheckStatement,
		signature: polkadot_primitives::v2::ValidatorSignature,
	) -> Result<(), sp_api::ApiError> {
		todo!()
	}

	async fn pvfs_require_precheck(
		&self,
		at: &polkadot_core_primitives::BlockId,
	) -> Result<Vec<polkadot_primitives::v2::ValidationCodeHash>, sp_api::ApiError> {
		todo!()
	}

	async fn validation_code_hash(
		&self,
		at: &polkadot_core_primitives::BlockId,
		para_id: cumulus_primitives_core::ParaId,
		assumption: polkadot_primitives::v2::OccupiedCoreAssumption,
	) -> Result<Option<polkadot_primitives::v2::ValidationCodeHash>, sp_api::ApiError> {
		if let BlockId::Hash(hash) = at {
			self.rpc_client
				.parachain_host_validation_code_hash(*hash, para_id, assumption)
				.await
				.map_err(|e| sp_api::ApiError::Application(Box::new(e) as Box<_>))
		} else {
			Err(sp_api::ApiError::Application(Box::new(RelayChainError::GenericError(
				"Only hash is supported for RPC methods".to_string(),
			)) as Box<_>))
		}
	}

	async fn configuration(
		&self,
		at: &polkadot_core_primitives::BlockId,
	) -> Result<sp_consensus_babe::BabeGenesisConfiguration, sp_api::ApiError> {
		todo!()
	}

	async fn current_epoch_start(
		&self,
		at: &polkadot_core_primitives::BlockId,
	) -> Result<sp_consensus_babe::Slot, sp_api::ApiError> {
		todo!()
	}

	async fn current_epoch(
		&self,
		at: &polkadot_core_primitives::BlockId,
	) -> Result<sp_consensus_babe::Epoch, sp_api::ApiError> {
		todo!()
	}

	async fn next_epoch(
		&self,
		at: &polkadot_core_primitives::BlockId,
	) -> Result<sp_consensus_babe::Epoch, sp_api::ApiError> {
		todo!()
	}

	async fn generate_key_ownership_proof(
		&self,
		at: &polkadot_core_primitives::BlockId,
		slot: sp_consensus_babe::Slot,
		authority_id: sp_consensus_babe::AuthorityId,
	) -> Result<Option<sp_consensus_babe::OpaqueKeyOwnershipProof>, sp_api::ApiError> {
		todo!()
	}

	async fn submit_report_equivocation_unsigned_extrinsic(
		&self,
		at: &polkadot_core_primitives::BlockId,
		equivocation_proof: sp_consensus_babe::EquivocationProof<polkadot_core_primitives::Header>,
		key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
	) -> Result<Option<()>, sp_api::ApiError> {
		todo!()
	}

	async fn authorities(
		&self,
		at: &polkadot_core_primitives::BlockId,
	) -> std::result::Result<Vec<polkadot_primitives::v2::AuthorityDiscoveryId>, sp_api::ApiError> {
		todo!()
	}

	async fn api_version_parachain_host(
		&self,
		at: &polkadot_core_primitives::BlockId,
	) -> Result<Option<u32>, sp_api::ApiError> {
		Ok(Some(2))
	}

	async fn staging_get_disputes(
		&self,
		at: &BlockId,
	) -> Result<
		Vec<(
			polkadot_primitives::v2::SessionIndex,
			polkadot_primitives::v2::CandidateHash,
			polkadot_primitives::v2::DisputeState<polkadot_primitives::v2::BlockNumber>,
		)>,
		ApiError,
	> {
		if let BlockId::Hash(hash) = at {
			self.rpc_client
				.parachain_host_staging_get_disputes(*hash)
				.await
				.map_err(|e| sp_api::ApiError::Application(Box::new(e) as Box<_>))
		} else {
			Err(sp_api::ApiError::Application(Box::new(RelayChainError::GenericError(
				"Only hash is supported for RPC methods".to_string(),
			)) as Box<_>))
		}
	}
}

impl AuthorityDiscoveryWrapper<Block> for BlockChainRPCClient {
	fn authorities(
		&self,
		at: &sp_api::BlockId<Block>,
	) -> std::result::Result<Vec<polkadot_primitives::v2::AuthorityDiscoveryId>, sp_api::ApiError> {
		if let sp_api::BlockId::Hash(hash) = at {
			block_on(self.rpc_client.authority_discovery_authorities(*hash))
				.map_err(|err| sp_api::ApiError::Application(Box::new(err)))
		} else {
			todo!("BlockId number not supported")
		}
	}
}

// TODO introduce error handling
impl sc_client_api::BlockchainRPCEvents<Block> for BlockChainRPCClient {
	fn import_notification_stream(&self) -> Pin<Box<dyn Stream<Item = Header> + Send>> {
		let imported_headers_stream = block_on(self.rpc_client.subscribe_all_heads())
			.expect("subscribe_all_heads")
			.filter_map(|item| async move {
				item.map_err(|err| {
					tracing::error!(
						target: LOG_TARGET,
						"Encountered error in import notification stream: {}",
						err
					)
				})
				.ok()
			});

		imported_headers_stream.boxed()
	}

	fn finality_notification_stream(&self) -> Pin<Box<dyn Stream<Item = Header> + Send>> {
		let imported_headers_stream = block_on(self.rpc_client.subscribe_finalized_heads())
			.expect("imported_headers_stream")
			.filter_map(|item| async move {
				item.map_err(|err| {
					tracing::error!(
						target: LOG_TARGET,
						"Encountered error in finality notification stream: {}",
						err
					)
				})
				.ok()
			});

		imported_headers_stream.boxed()
	}

	fn best_head_stream(
		&self,
	) -> Pin<Box<dyn Stream<Item = <Block as polkadot_service::BlockT>::Header> + Send>> {
		let imported_headers_stream = block_on(self.rpc_client.subscribe_new_best_heads())
			.expect("new best heads")
			.filter_map(|item| async move {
				item.map_err(|err| {
					tracing::error!(
						target: LOG_TARGET,
						"Error in best block notification stream: {}",
						err
					)
				})
				.ok()
			});

		imported_headers_stream.boxed()
	}
}

fn block_local<T>(fut: impl Future<Output = T>) -> T {
	let tokio_handle = tokio::runtime::Handle::current();
	tokio::task::block_in_place(|| tokio_handle.block_on(fut))
}

impl HeaderBackend<Block> for BlockChainRPCClient {
	fn header(
		&self,
		id: sp_api::BlockId<Block>,
	) -> sp_blockchain::Result<Option<<Block as polkadot_service::BlockT>::Header>> {
		if let sp_api::BlockId::Hash(hash) = id {
			block_on(self.rpc_client.chain_get_header(Some(hash)))
				.map_err(|err| sp_blockchain::Error::Backend(err.to_string()))
		} else {
			todo!("header with number not supported")
		}
	}

	fn info(&self) -> sp_blockchain::Info<Block> {
		tracing::debug!(target: LOG_TARGET, "BlockBackend::block_status");

		let tokio_handle = tokio::runtime::Handle::current();
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
		id: sp_api::BlockId<Block>,
	) -> sp_blockchain::Result<sp_blockchain::BlockStatus> {
		todo!()
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
		todo!()
	}
}
impl ProofProvider<Block> for BlockChainRPCClient {
	fn read_proof(
		&self,
		id: &sp_api::BlockId<Block>,
		keys: &mut dyn Iterator<Item = &[u8]>,
	) -> sp_blockchain::Result<sc_client_api::StorageProof> {
		todo!()
	}

	fn read_child_proof(
		&self,
		id: &sp_api::BlockId<Block>,
		child_info: &sc_client_api::ChildInfo,
		keys: &mut dyn Iterator<Item = &[u8]>,
	) -> sp_blockchain::Result<sc_client_api::StorageProof> {
		todo!()
	}

	fn execution_proof(
		&self,
		id: &sp_api::BlockId<Block>,
		method: &str,
		call_data: &[u8],
	) -> sp_blockchain::Result<(Vec<u8>, sc_client_api::StorageProof)> {
		todo!()
	}

	fn read_proof_collection(
		&self,
		id: &sp_api::BlockId<Block>,
		start_keys: &[Vec<u8>],
		size_limit: usize,
	) -> sp_blockchain::Result<(sc_client_api::CompactProof, u32)> {
		todo!()
	}

	fn storage_collection(
		&self,
		id: &sp_api::BlockId<Block>,
		start_key: &[Vec<u8>],
		size_limit: usize,
	) -> sp_blockchain::Result<Vec<(sp_state_machine::KeyValueStorageLevel, bool)>> {
		todo!()
	}

	fn verify_range_proof(
		&self,
		root: <Block as polkadot_service::BlockT>::Hash,
		proof: sc_client_api::CompactProof,
		start_keys: &[Vec<u8>],
	) -> sp_blockchain::Result<(sc_client_api::KeyValueStates, usize)> {
		todo!()
	}
}
impl BlockIdTo<Block> for BlockChainRPCClient {
	type Error = sp_blockchain::Error;

	fn to_hash(
		&self,
		block_id: &sp_runtime::generic::BlockId<Block>,
	) -> Result<Option<<Block as polkadot_service::BlockT>::Hash>, Self::Error> {
		todo!()
	}

	fn to_number(
		&self,
		block_id: &sp_runtime::generic::BlockId<Block>,
	) -> Result<Option<polkadot_service::NumberFor<Block>>, Self::Error> {
		todo!()
	}
}

impl polkadot_service::Chain<Block> for BlockChainRPCClient {
	fn block_status(
		&self,
		id: &sp_api::BlockId<Block>,
	) -> Result<sp_consensus::BlockStatus, Box<dyn std::error::Error + Send>> {
		tracing::debug!(target: LOG_TARGET, "Chain::block_status");
		if let sp_api::BlockId::Hash(hash) = id {
			let maybe_header =
				block_local(self.rpc_client.chain_get_header(None)).expect("get_header");
			if let Some(header) = maybe_header {
				// TODO we need to check for pruned blocks here
				return Ok(sp_consensus::BlockStatus::InChainWithState)
			} else {
				return Ok(sp_consensus::BlockStatus::Unknown)
			}
		} else {
			todo!("Not supported blockId::number, block_status");
		}
	}
}

impl BlockBackend<Block> for BlockChainRPCClient {
	fn block_body(
		&self,
		id: &sp_api::BlockId<Block>,
	) -> sp_blockchain::Result<Option<Vec<<Block as polkadot_service::BlockT>::Extrinsic>>> {
		todo!()
	}

	fn block_indexed_body(
		&self,
		id: &sp_api::BlockId<Block>,
	) -> sp_blockchain::Result<Option<Vec<Vec<u8>>>> {
		todo!()
	}

	fn block(
		&self,
		id: &sp_api::BlockId<Block>,
	) -> sp_blockchain::Result<Option<polkadot_service::generic::SignedBlock<Block>>> {
		todo!()
	}

	fn block_status(
		&self,
		id: &sp_api::BlockId<Block>,
	) -> sp_blockchain::Result<sp_consensus::BlockStatus> {
		tracing::debug!(target: LOG_TARGET, "BlockBackend::block_status");
		if let sp_api::BlockId::Hash(hash) = id {
			let maybe_header =
				block_local(self.rpc_client.chain_get_header(None)).expect("get_header");
			if let Some(header) = maybe_header {
				// TODO we need to check for pruned blocks here
				return Ok(sp_consensus::BlockStatus::InChainWithState)
			} else {
				return Ok(sp_consensus::BlockStatus::Unknown)
			}
		} else {
			todo!("Not supported blockId::number, block_status");
		}
	}

	fn justifications(
		&self,
		id: &sp_api::BlockId<Block>,
	) -> sp_blockchain::Result<Option<sp_runtime::Justifications>> {
		todo!()
	}

	fn block_hash(
		&self,
		number: polkadot_service::NumberFor<Block>,
	) -> sp_blockchain::Result<Option<<Block as polkadot_service::BlockT>::Hash>> {
		todo!()
	}

	fn indexed_transaction(
		&self,
		hash: &<Block as polkadot_service::BlockT>::Hash,
	) -> sp_blockchain::Result<Option<Vec<u8>>> {
		todo!()
	}

	fn requires_full_sync(&self) -> bool {
		false
	}
}
impl HeaderMetadata<Block> for BlockChainRPCClient {
	type Error = sp_blockchain::Error;

	fn header_metadata(
		&self,
		hash: <Block as polkadot_service::BlockT>::Hash,
	) -> Result<sp_blockchain::CachedHeaderMetadata<Block>, Self::Error> {
		todo!()
	}

	fn insert_header_metadata(
		&self,
		hash: <Block as polkadot_service::BlockT>::Hash,
		header_metadata: sp_blockchain::CachedHeaderMetadata<Block>,
	) {
		todo!()
	}

	fn remove_header_metadata(&self, hash: <Block as polkadot_service::BlockT>::Hash) {
		todo!()
	}
}
