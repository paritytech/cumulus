// Copyright 2021 Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Cumulus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

use crate::{ParachainInherentData, INHERENT_IDENTIFIER};
use cumulus_primitives_core::{InboundDownwardMessage, InboundHrmpMessage, ParaId, PersistedValidationData, relay_chain};
use sp_inherents::{InherentData, InherentDataProvider};
use std::collections::BTreeMap;
use sp_api::BlockId;
use sp_core::twox_128;
use codec::Decode;
use sc_client_api::{Backend, StorageProvider};
use sp_runtime::traits::Block;

use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;

/// Inherent data provider that supplies mocked validation data.
///
/// This is useful when running a node that is not actually backed by any relay chain.
/// For example when running a local node, or running integration tests.
///
/// We mock a relay chain block number as follows:
/// relay_block_number = offset + relay_blocks_per_para_block * current_para_block
/// To simulate a parachain that starts in relay block 1000 and gets a block in every other relay
/// block, use 1000 and 2
///
/// TODO Docs about the XCM injection
pub struct MockValidationDataInherentDataProvider {
	/// The current block number of the local block chain (the parachain)
	pub current_para_block: u32,
	/// The relay block in which this parachain appeared to start. This will be the relay block
	/// number in para block #P1
	pub relay_offset: u32,
	/// The number of relay blocks that elapses between each parablock. Probably set this to 1 or 2
	/// to simulate optimistic or realistic relay chain behavior.
	pub relay_blocks_per_para_block: u32,
	/// XCM messages and associated configuration information.
	pub xcm_config: MockXcmConfig,
	/// Inbound downward XCM messages to be injected into the block.
	pub downward_messages: Vec<Vec<u8>>,
	// Inbound Horizontal messages sorted by channel
	pub horizontal_messages: BTreeMap<ParaId, Vec<Vec<u8>>>,
}

/// Parameters for how the Mock inherent data provider should inject XCM messages
pub struct MockXcmConfig {
	/// The parachain id of the parachain being mocked.
	pub para_id: ParaId,
	/// The starting state of the dmq_mqc_head.
	pub starting_dmq_mqc_head: relay_chain::Hash,
}

impl MockXcmConfig {
	/// Utility method for creating a MockXcmConfig by reading the dmq_mqc_head directly
	/// from the storage of a previous block at common storage keys.
	pub fn from_standard_storage<B: Block, BE: Backend<B>, C: StorageProvider<B, BE>>(
		client: &C,
		parent_block: B::Hash,
		para_id: ParaId,
	) -> Self {

		let starting_dmq_mqc_head = client
			.storage(
				&BlockId::Hash(parent_block),
				&sp_storage::StorageKey(
					[twox_128(b"ParachainSystem"), twox_128(b"LastDmqMqcHead")].concat().to_vec(),
				),
			)
			.expect("We should be able to read storage from the parent block.")
			.map(|ref mut raw_data| {
				Decode::decode(&mut &raw_data.0[..])
					.expect("Stored data should decode correctly")
			})
			.unwrap_or_default();
		
		Self {
			para_id,
			starting_dmq_mqc_head,
		}
	}
}

#[async_trait::async_trait]
impl InherentDataProvider for MockValidationDataInherentDataProvider {
	fn provide_inherent_data(
		&self,
		inherent_data: &mut InherentData,
	) -> Result<(), sp_inherents::Error> {

		// Calculate the mocked relay block based on the current para block
		let relay_parent_number =
			self.relay_offset + self.relay_blocks_per_para_block * self.current_para_block;
		
		let downward_messages = self
			.downward_messages
			.iter()
			.cloned()
			.map(|msg|
				InboundDownwardMessage{
					sent_at: relay_parent_number,
					msg,
				}
			)
			.collect();

		let horizontal_messages = self
			.horizontal_messages
			.iter()
			.map(|(para_id, msgs)|
				(
					*para_id,
					msgs
						.iter()
						.map(|msg|
							InboundHrmpMessage {
								sent_at: relay_parent_number,
								data: msg.clone(),
							}
						)
						.collect()
				)
			)
			.collect();

		// Make sure the validation against the state proof passes
		let mut dmq_mqc = crate::MessageQueueChain(self.xcm_config.starting_dmq_mqc_head);
		for message in &downward_messages {
			dmq_mqc.extend_downward(message);
		}

		// Use the "sproof" (spoof proof) builder to build valid mock state root and proof.
		let mut sproof_builder = RelayStateSproofBuilder::default();
		sproof_builder.para_id = self.xcm_config.para_id;
		sproof_builder.dmq_mqc_head = Some(dmq_mqc.head());
		let (relay_parent_storage_root, proof) = sproof_builder.into_state_root_and_proof();

		inherent_data.put_data(
			INHERENT_IDENTIFIER,
			&ParachainInherentData {
				validation_data: PersistedValidationData {
					parent_head: Default::default(),
					relay_parent_storage_root,
					relay_parent_number,
					max_pov_size: Default::default(),
				},
				downward_messages,
				horizontal_messages,
				relay_chain_state: proof,
			}
		)
	}

	// Copied from the real implementation
	async fn try_handle_error(
		&self,
		_: &sp_inherents::InherentIdentifier,
		_: &[u8],
	) -> Option<Result<(), sp_inherents::Error>> {
		None
	}
}
