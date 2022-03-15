use cumulus_relay_chain_rpc_interface::RelayChainRPCClient;
use polkadot_core_primitives::Block;
use polkadot_node_network_protocol::Arc;
use polkadot_service::{runtime_traits::BlockIdTo, HeaderBackend};
use sc_authority_discovery::AuthorityDiscoveryWrapper;
use sc_client_api::{BlockBackend, BlockchainEvents, ProofProvider};
use sp_blockchain::HeaderMetadata;

#[derive(Clone)]
pub struct BlockChainRPCClient {
	rpc_client: Arc<RelayChainRPCClient>,
}

impl BlockChainRPCClient {
	pub fn new(rpc_client: Arc<RelayChainRPCClient>) -> Self {
		Self { rpc_client }
	}
}

impl AuthorityDiscoveryWrapper<Block> for BlockChainRPCClient {
	fn authorities(
		&self,
		at: &sp_api::BlockId<Block>,
	) -> std::result::Result<Vec<polkadot_primitives::v1::AuthorityDiscoveryId>, sp_api::ApiError> {
		todo!("AuthorityDiscoveryWrapper")
	}
}

impl BlockchainEvents<Block> for BlockChainRPCClient {
	fn import_notification_stream(&self) -> sc_client_api::ImportNotifications<Block> {
		todo!()
	}

	fn finality_notification_stream(&self) -> sc_client_api::FinalityNotifications<Block> {
		todo!()
	}

	fn storage_changes_notification_stream(
		&self,
		filter_keys: Option<&[sc_client_api::StorageKey]>,
		child_filter_keys: Option<
			&[(sc_client_api::StorageKey, Option<Vec<sc_client_api::StorageKey>>)],
		>,
	) -> sp_blockchain::Result<
		sc_client_api::StorageEventStream<<Block as polkadot_service::BlockT>::Hash>,
	> {
		todo!()
	}
}

impl HeaderBackend<Block> for BlockChainRPCClient {
	fn header(
		&self,
		id: sp_api::BlockId<Block>,
	) -> sp_blockchain::Result<Option<<Block as polkadot_service::BlockT>::Header>> {
		todo!()
	}

	fn info(&self) -> sp_blockchain::Info<Block> {
		todo!()
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
		todo!()
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
		todo!()
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
		todo!()
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
