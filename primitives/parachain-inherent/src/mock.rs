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
use cumulus_primitives_core::{InboundDownwardMessage, PersistedValidationData, ParaId, relay_chain};
use sp_inherents::{InherentData, InherentDataProvider};

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
	/// The parachain id of the parachain being mocked.
	/// This field is only important if xcm is being used.
	/// If you are not interested in injecting simulated XCM message, you ca nuse any value
	pub para_id: ParaId,
	/// The starting state of the mdq_mqc_head. Also only necessary when inserting xcm. Maybe these should
	/// be grouped into a single field like mock_xcm config or something
	pub starting_dmq_mqc_head: relay_chain::Hash,
	/// The current block number of the local block chain (the parachain)
	pub current_para_block: u32,
	/// The relay block in which this parachain appeared to start. This will be the relay block
	/// number in para block #P1
	pub relay_offset: u32,
	/// The number of relay blocks that elapses between each parablock. Probably set this to 1 or 2
	/// to simulate optimistic or realistic relay chain behavior.
	pub relay_blocks_per_para_block: u32,
	/// Inbound downward XCM messages to be injected into the block.
	pub downward_messages: Vec<InboundDownwardMessage>,
	//TODO also support horizontal messages, but let's fous on downward for PoC phase.
	// Inbound Horizontal messages sorted by channel
	// pub horizontal_messages: BTreeMap<ParaId, Vec<InboundHrmpMessage>>
}

#[async_trait::async_trait]
impl InherentDataProvider for MockValidationDataInherentDataProvider {
	fn provide_inherent_data(
		&self,
		inherent_data: &mut InherentData,
	) -> Result<(), sp_inherents::Error> {
		// Use the "sproof" (spoof proof) builder to build valid mock state root and proof.
		let mut sproof_builder = RelayStateSproofBuilder::default();

		// Set the sproof builder up to match the runtime
		sproof_builder.para_id = self.para_id;
		
		// Make a MessageQueueChain object with the root that is actually in storage
		let mut dmq_mqc = crate::MessageQueueChain(self.starting_dmq_mqc_head);

		for message in &self.downward_messages {
			dmq_mqc.extend_downward(message);
		}

		sproof_builder.dmq_mqc_head = Some(dmq_mqc.0);

		let (relay_storage_root, proof) = sproof_builder.into_state_root_and_proof();

		// Calculate the mocked relay block based on the current para block
		let relay_parent_number =
			self.relay_offset + self.relay_blocks_per_para_block * self.current_para_block;

		let data = ParachainInherentData {
			validation_data: PersistedValidationData {
				parent_head: Default::default(),
				relay_parent_storage_root: relay_storage_root,
				relay_parent_number,
				max_pov_size: Default::default(),
			},
			downward_messages: self.downward_messages.clone(),
			horizontal_messages: Default::default(),
			relay_chain_state: proof,
		};

		inherent_data.put_data(INHERENT_IDENTIFIER, &data)
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
