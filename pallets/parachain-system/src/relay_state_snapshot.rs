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

use codec::{Decode, Encode};
use cumulus_primitives_core::{
	relay_chain, AbridgedHostConfiguration, AbridgedHrmpChannel, ParaId,
};
use sp_trie::{MemoryDB, HashDBT, EMPTY_PREFIX};
use sp_runtime::traits::HashFor;
use sp_state_machine::{Backend, TrieBackend};
use sp_std::vec::Vec;
use sp_trie::StorageProof;

/// A snapshot of some messaging related state of relay chain pertaining to the current parachain.
///
/// This data is essential for making sure that the parachain is aware of current resource use on
/// the relay chain and that the candidates produced for this parachain do not exceed any of these
/// limits.
#[derive(Clone, Encode, Decode)]
pub struct MessagingStateSnapshot {
	/// The current message queue chain head for downward message queue.
	///
	/// If the value is absent on the relay chain this will be set to all zeros.
	pub dmq_mqc_head: relay_chain::Hash,

	/// The current capacity of the upward message queue of the current parachain on the relay chain.
	///
	/// The capacity is represented by a tuple that consist of the `count` of the messages and the
	/// `total_size` expressed as the sum of byte sizes of all messages in the queue.
	pub relay_dispatch_queue_size: (u32, u32),

	/// Information about all the inbound HRMP channels.
	///
	/// These are structured as a list of tuples. The para id in the tuple specifies the sender
	/// of the channel. Obviously, the recipient is the current parachain.
	///
	/// The channels are sorted by the sender para id ascension.
	pub ingress_channels: Vec<(ParaId, AbridgedHrmpChannel)>,

	/// Information about all the outbound HRMP channels.
	///
	/// These are structured as a list of tuples. The para id in the tuple specifies the recipient
	/// of the channel. Obviously, the sender is the current parachain.
	///
	/// The channels are sorted by the recipient para id ascension.
	pub egress_channels: Vec<(ParaId, AbridgedHrmpChannel)>,
}

#[derive(Debug)]
pub enum Error {
	/// The provided proof was created against unexpected storage root.
	RootMismatch,
	/// The slot cannot be extracted.
	Slot(ReadEntryErr),
	/// The host configuration cannot be extracted.
	Config(ReadEntryErr),
	/// The DMQ MQC head cannot be extracted.
	DmqMqcHead(ReadEntryErr),
	/// Relay dispatch queue cannot be extracted.
	RelayDispatchQueueSize(ReadEntryErr),
	/// The hrmp inress channel index cannot be extracted.
	HrmpIngressChannelIndex(ReadEntryErr),
	/// The hrmp egress channel index cannot be extracted.
	HrmpEgressChannelIndex(ReadEntryErr),
	/// The channel identified by the sender and receiver cannot be extracted.
	HrmpChannel(ParaId, ParaId, ReadEntryErr),
}

#[derive(Debug)]
pub enum ReadEntryErr {
	/// The value cannot be extracted from the proof.
	Proof,
	/// The value cannot be decoded.
	Decode,
	/// The value is expected to be present on the relay chain, but it doesn't exist.
	Absent,
}

/// Read an entry given by the key and try to decode it. If the value specified by the key according
/// to the proof is empty, the `fallback` value will be returned.
///
/// Returns `Err` in case the backend can't return the value under the specific key (likely due to
/// a malformed proof), in case the decoding fails, or in case where the value is empty in the relay
/// chain state and no fallback was provided.
fn read_entry<T, B>(backend: &B, key: &[u8], fallback: Option<T>) -> Result<T, ReadEntryErr>
where
	T: Decode,
	B: Backend<HashFor<relay_chain::Block>>,
{
	backend
		.storage(key)
		.map_err(|_| ReadEntryErr::Proof)?
		.map(|raw_entry| T::decode(&mut &raw_entry[..]).map_err(|_| ReadEntryErr::Decode))
		.transpose()?
		.or(fallback)
		.ok_or(ReadEntryErr::Absent)
}

/// A state proof extracted from the relay chain.
///
/// This state proof is extracted from the relay chain block we are building on top of.
pub struct RelayChainStateProof {
	para_id: ParaId,
	trie_backend: TrieBackend<MemoryDB<HashFor<relay_chain::Block>>, HashFor<relay_chain::Block>>,
}

impl RelayChainStateProof {
	/// Create a new instance of `Self`.
	///
	/// Returns an error if the given `relay_parent_storage_root` is not the root of the given
	/// `proof`.
	pub fn new(
		para_id: ParaId,
		relay_parent_storage_root: relay_chain::v1::Hash,
		proof: StorageProof,
	) -> Result<Self, Error> {
		let db = proof.into_memory_db::<HashFor<relay_chain::Block>>();
		if !db.contains(&relay_parent_storage_root, EMPTY_PREFIX) {
			return Err(Error::RootMismatch);
		}
		let trie_backend = TrieBackend::new(db, relay_parent_storage_root);

		Ok(Self {
			para_id,
			trie_backend,
		})
	}

	/// Read the [`MessagingStateSnapshot`] from the relay chain state proof.
	///
	/// Returns an error if anything failed at reading or decoding.
	pub fn read_messaging_state_snapshot(&self) -> Result<MessagingStateSnapshot, Error> {
		let dmq_mqc_head: relay_chain::Hash = read_entry(
			&self.trie_backend,
			&relay_chain::well_known_keys::dmq_mqc_head(self.para_id),
			Some(Default::default()),
		)
		.map_err(Error::DmqMqcHead)?;

		let relay_dispatch_queue_size: (u32, u32) = read_entry(
			&self.trie_backend,
			&relay_chain::well_known_keys::relay_dispatch_queue_size(self.para_id),
			Some((0, 0)),
		)
		.map_err(Error::RelayDispatchQueueSize)?;

		let ingress_channel_index: Vec<ParaId> = read_entry(
			&self.trie_backend,
			&relay_chain::well_known_keys::hrmp_ingress_channel_index(self.para_id),
			Some(Vec::new()),
		)
		.map_err(Error::HrmpIngressChannelIndex)?;

		let egress_channel_index: Vec<ParaId> = read_entry(
			&self.trie_backend,
			&relay_chain::well_known_keys::hrmp_egress_channel_index(self.para_id),
			Some(Vec::new()),
		)
		.map_err(Error::HrmpEgressChannelIndex)?;

		let mut ingress_channels = Vec::with_capacity(ingress_channel_index.len());
		for sender in ingress_channel_index {
			let channel_id = relay_chain::v1::HrmpChannelId {
				sender,
				recipient: self.para_id,
			};
			let hrmp_channel: AbridgedHrmpChannel = read_entry(
				&self.trie_backend,
				&relay_chain::well_known_keys::hrmp_channels(channel_id),
				None,
			)
			.map_err(|read_err| Error::HrmpChannel(sender, self.para_id, read_err))?;
			ingress_channels.push((sender, hrmp_channel));
		}

		let mut egress_channels = Vec::with_capacity(egress_channel_index.len());
		for recipient in egress_channel_index {
			let channel_id = relay_chain::v1::HrmpChannelId {
				sender: self.para_id,
				recipient,
			};
			let hrmp_channel: AbridgedHrmpChannel = read_entry(
				&self.trie_backend,
				&relay_chain::well_known_keys::hrmp_channels(channel_id),
				None,
			)
			.map_err(|read_err| Error::HrmpChannel(self.para_id, recipient, read_err))?;
			egress_channels.push((recipient, hrmp_channel));
		}

		// NOTE that ingress_channels and egress_channels promise to be sorted. We satisfy this property
		// by relying on the fact that `ingress_channel_index` and `egress_channel_index` are themselves sorted.
		Ok(MessagingStateSnapshot {
			dmq_mqc_head,
			relay_dispatch_queue_size,
			ingress_channels,
			egress_channels,
		})
	}

	/// Read the [`AbridgedHostConfiguration`] from the relay chain state proof.
	///
	/// Returns an error if anything failed at reading or decoding.
	pub fn read_abridged_host_configuration(&self) -> Result<AbridgedHostConfiguration, Error> {
		read_entry(&self.trie_backend, relay_chain::well_known_keys::ACTIVE_CONFIG, None)
			.map_err(Error::Config)
	}

	/// Read the [`Slot`](relay_chain::v1::Slot) from the relay chain state proof.
	///
	/// The slot is slot of the relay chain block this state proof was extracted from.
	///
	/// Returns an error if anything failed at reading or decoding.
	pub fn read_slot(&self) -> Result<relay_chain::v1::Slot, Error> {
		read_entry(&self.trie_backend, relay_chain::well_known_keys::CURRENT_SLOT, None).map_err(Error::Slot)
	}
}
