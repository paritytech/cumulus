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
		v1::{CommittedCandidateReceipt, OccupiedCoreAssumption, SessionIndex, ValidatorId},
		Block as PBlock, BlockId, Hash as PHash, InboundHrmpMessage,
	},
	InboundDownwardMessage, ParaId, PersistedValidationData,
};
use polkadot_service::Handle;
use sc_client_api::{blockchain::BlockStatus, StorageProof};

use sp_api::ApiError;
use sp_core::sp_std::collections::btree_map::BTreeMap;
use sp_state_machine::StorageValue;

/// Should be used for all interaction with the relay chain in cumulus.
pub trait RelayChainInterface: Send + Sync {
	fn get_storage_by_key(&self, block_id: &BlockId, key: &[u8]) -> Option<StorageValue>;

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

	fn import_notification_stream(&self) -> sc_client_api::ImportNotifications<PBlock>;

	/// Check if block is in chain. If it is, we return nothing.
	/// If it is not in the chain, we return a listener that can be used to wait on the block.
	fn check_block_in_chain(
		&self,
		block_id: BlockId,
	) -> Result<Option<sc_client_api::ImportNotifications<PBlock>>, sp_blockchain::Error>;

	fn finality_notification_stream(&self) -> sc_client_api::FinalityNotifications<PBlock>;

	fn storage_changes_notification_stream(
		&self,
		filter_keys: Option<&[sc_client_api::StorageKey]>,
		child_filter_keys: Option<
			&[(sc_client_api::StorageKey, Option<Vec<sc_client_api::StorageKey>>)],
		>,
	) -> sc_client_api::blockchain::Result<sc_client_api::StorageEventStream<PHash>>;

	fn is_major_syncing(&self) -> bool;

	fn overseer_handle(&self) -> Option<Handle>;

	fn prove_read(
		&self,
		block_id: &BlockId,
		relevant_keys: &Vec<Vec<u8>>,
	) -> Result<Option<StorageProof>, Box<dyn sp_state_machine::Error>>;
}

impl<T> RelayChainInterface for Arc<T>
where
	T: RelayChainInterface + ?Sized,
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

	fn import_notification_stream(&self) -> sc_client_api::ImportNotifications<PBlock> {
		(**self).import_notification_stream()
	}

	fn finality_notification_stream(&self) -> sc_client_api::FinalityNotifications<PBlock> {
		(**self).finality_notification_stream()
	}

	fn storage_changes_notification_stream(
		&self,
		filter_keys: Option<&[sc_client_api::StorageKey]>,
		child_filter_keys: Option<
			&[(sc_client_api::StorageKey, Option<Vec<sc_client_api::StorageKey>>)],
		>,
	) -> sc_client_api::blockchain::Result<sc_client_api::StorageEventStream<PHash>> {
		(**self).storage_changes_notification_stream(filter_keys, child_filter_keys)
	}

	fn best_block_hash(&self) -> PHash {
		(**self).best_block_hash()
	}

	fn block_status(&self, block_id: BlockId) -> Result<BlockStatus, sp_blockchain::Error> {
		(**self).block_status(block_id)
	}

	fn is_major_syncing(&self) -> bool {
		(**self).is_major_syncing()
	}

	fn overseer_handle(&self) -> Option<Handle> {
		(**self).overseer_handle()
	}

	fn get_storage_by_key(&self, block_id: &BlockId, key: &[u8]) -> Option<StorageValue> {
		(**self).get_storage_by_key(block_id, key)
	}

	fn prove_read(
		&self,
		block_id: &BlockId,
		relevant_keys: &Vec<Vec<u8>>,
	) -> Result<Option<StorageProof>, Box<dyn sp_state_machine::Error>> {
		(**self).prove_read(block_id, relevant_keys)
	}

	fn check_block_in_chain(
		&self,
		block_id: BlockId,
	) -> Result<Option<sc_client_api::ImportNotifications<PBlock>>, sp_blockchain::Error> {
		(**self).check_block_in_chain(block_id)
	}
}
