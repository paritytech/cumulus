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

use sp_runtime::traits::HashFor;
use sp_state_machine::MemoryDB;
use cumulus_primitives::{ParaId, relay_chain};

/// Builds a sproof (portmanteau of 'spoof' and 'proof') of the relay chain state.
#[derive(Clone)]
pub struct RelayStateSproofBuilder {
	pub para_id: ParaId,
	pub host_config: cumulus_primitives::AbridgedHostConfiguration,
	pub relay_dispatch_queue_size: Option<(u32, u32)>,
}

impl Default for RelayStateSproofBuilder {
	fn default() -> Self {
		RelayStateSproofBuilder {
			para_id: ParaId::from(200),
			host_config: cumulus_primitives::AbridgedHostConfiguration {
				max_code_size: 2 * 1024 * 1024,
				max_head_data_size: 1024 * 1024,
				max_upward_queue_count: 8,
				max_upward_queue_size: 1024,
				max_upward_message_size: 256,
				max_upward_message_num_per_candidate: 5,
				hrmp_max_message_num_per_candidate: 5,
				validation_upgrade_frequency: 6,
				validation_upgrade_delay: 6,
			},
			relay_dispatch_queue_size: None,
		}
	}
}

impl RelayStateSproofBuilder {
	pub fn into_state_root_and_proof(
		self,
	) -> (
		polkadot_primitives::v1::Hash,
		sp_state_machine::StorageProof,
	) {
		let (db, root) = MemoryDB::<HashFor<polkadot_primitives::v1::Block>>::default_with_root();
		let mut backend = sp_state_machine::TrieBackend::new(db, root);

		let mut relevant_keys = vec![];
		{
			use codec::Encode as _;

			let mut insert = |key: Vec<u8>, value: Vec<u8>| {
				relevant_keys.push(key.clone());
				backend.insert(vec![(None, vec![(key, Some(value))])]);
			};

			insert(
				relay_chain::well_known_keys::ACTIVE_CONFIG.to_vec(),
				self.host_config.encode(),
			);
			if let Some(relay_dispatch_queue_size) = self.relay_dispatch_queue_size {
				insert(
					relay_chain::well_known_keys::relay_dispatch_queue_size(self.para_id),
					relay_dispatch_queue_size.encode(),
				);
			}
		}

		let root = backend.root().clone();
		let proof = sp_state_machine::prove_read(backend, relevant_keys).expect("prove read");
		(root, proof)
	}
}
