// Copyright 2020-2021 Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

//! Cumulus related core primitive types and traits.

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use codec::{Encode, Decode};
use sp_runtime::{RuntimeDebug, traits::Block as BlockT};
use xcm::VersionedXcm;

pub use polkadot_core_primitives::InboundDownwardMessage;
pub use polkadot_parachain::primitives::{Id as ParaId, UpwardMessage, ValidationParams};
pub use polkadot_primitives::v1::{
	PersistedValidationData, AbridgedHostConfiguration, AbridgedHrmpChannel,
};

/// A module that re-exports relevant relay chain definitions.
pub mod relay_chain {
	pub use polkadot_core_primitives::*;
	pub use polkadot_primitives::v1;
	pub use polkadot_primitives::v1::well_known_keys;
}

/// An inbound HRMP message.
pub type InboundHrmpMessage = polkadot_primitives::v1::InboundHrmpMessage<relay_chain::BlockNumber>;

/// And outbound HRMP message
pub type OutboundHrmpMessage = polkadot_primitives::v1::OutboundHrmpMessage<ParaId>;

/// Error description of a message send failure.
pub enum MessageSendError {
	/// The dispatch queue is full.
	QueueFull,
	/// There does not exist a channel for sending the message.
	NoChannel,
	/// The message is too big to ever fit in a channel.
	TooBig,
	/// Some other error.
	Other(&'static str),
}

/// Well known keys for values in the storage.
pub mod well_known_keys {
	/// The storage key for the upward messages.
	///
	/// The upward messages are stored as SCALE encoded `Vec<UpwardMessage>`.
	pub const UPWARD_MESSAGES: &'static [u8] = b":cumulus_upward_messages:";

	/// Code upgarde (set as appropriate by a pallet).
	pub const NEW_VALIDATION_CODE: &'static [u8] = b":cumulus_new_validation_code:";

	/// The storage key with which the runtime passes outbound HRMP messages it wants to send to the
	/// PVF.
	///
	/// The value is stored as SCALE encoded `Vec<OutboundHrmpMessage>`
	pub const HRMP_OUTBOUND_MESSAGES: &'static [u8] = b":cumulus_hrmp_outbound_messages:";

	/// The storage key for communicating the HRMP watermark from the runtime to the PVF. Cleared by
	/// the runtime each block and set after message inclusion, but only if there were messages.
	///
	/// The value is stored as SCALE encoded relay-chain's `BlockNumber`.
	pub const HRMP_WATERMARK: &'static [u8] = b":cumulus_hrmp_watermark:";

	/// The storage key for the processed downward messages.
	///
	/// The value is stored as SCALE encoded `u32`.
	pub const PROCESSED_DOWNWARD_MESSAGES: &'static [u8] = b":cumulus_processed_downward_messages:";
}

/// Something that should be called when a downward message is received.
#[impl_trait_for_tuples::impl_for_tuples(30)]
pub trait DownwardMessageHandler {
	/// Handle the given downward message.
	fn handle_downward_message(msg: InboundDownwardMessage);
}

// TODO: Rename Hrmp -> Xcmp
/// Something that should be called for each fragment message received over XCMP.
#[impl_trait_for_tuples::impl_for_tuples(30)]
pub trait HrmpMessageHandler {
	/// Handle the given blob XCMP message.
	fn handle_blob_message(sender: ParaId, sent_at: relay_chain::BlockNumber, msg: Vec<u8>);

	/// Handle the given Xcm message that has been received over XCMP.
	fn handle_xcm_message(sender: ParaId, sent_at: relay_chain::BlockNumber, xcm: VersionedXcm);
}

/// Something that should be called when sending an upward message.
pub trait UpwardMessageSender {
	/// Send the given UMP message; return the expected number of blocks before the message will
	/// be dispatched or an error if the message cannot be sent.
	fn send_upward_message(msg: UpwardMessage) -> Result<u32, MessageSendError>;
}

/// The "quality of service" considerations for message sending.
#[derive(Eq, PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug)]
pub enum ServiceQuality {
	/// Ensure that this message is dispatched in the same relative order as any other messages that
	/// were also sent with `Ordered`. This only guarantees message ordering on the dispatch side,
	/// and not necessarily on the execution side.
	Ordered,
	/// Ensure that the message is dispatched as soon as possible, which could result in it being
	/// dispatched before other messages which are larger and/or rely on relative ordering.
	Fast,
}

// TODO: Rename Hrmp -> Xcmp
/// Something that should be called in order to send a message over XCMP.
pub trait HrmpMessageSender {
	/// Send the given abstract HRMP message; return the expected number of blocks before the
	/// message will be dispatched or an error if the message cannot be sent.
	///
	/// NOTE: The number of blocks is a guideline only and may take more or less depending on
	/// various factors. However, it should generally return at least one non-zero successful
	/// result before returning a `QueueFull` error.
	///
	/// NOTE: Using this when `send_xcm_message` has already been used may result in additional
	/// aggregate messages being sent.
	fn send_blob_message(dest: ParaId, msg: Vec<u8>, qos: ServiceQuality)
		-> Result<u32, MessageSendError>;

	/// Send the given XCM message through HMP; return the expected number of blocks before the
	/// message will be dispatched or an error if the message cannot be sent.
	///
	/// NOTE: The number of blocks is a guideline only and may take more or less depending on
	/// various factors. However, it should generally return at least one non-zero successful
	/// result before returning a `QueueFull` error.
	fn send_xcm_message(dest: ParaId, msg: VersionedXcm, qos: ServiceQuality)
		-> Result<u32, MessageSendError>;
}

/// A trait which is called when the validation data is set.
#[impl_trait_for_tuples::impl_for_tuples(30)]
pub trait OnValidationData {
	fn on_validation_data(data: &PersistedValidationData);
}

/// The parachain block that is created by a collator.
///
/// This is send as PoV (proof of validity block) to the relay-chain validators. There it will be
/// passed to the parachain validation Wasm blob to be validated.
#[derive(codec::Encode, codec::Decode)]
pub struct ParachainBlockData<B: BlockT> {
	/// The header of the parachain block.
	header: B::Header,
	/// The extrinsics of the parachain block.
	extrinsics: sp_std::vec::Vec<B::Extrinsic>,
	/// The data that is required to emulate the storage accesses executed by all extrinsics.
	storage_proof: sp_trie::StorageProof,
}

impl<B: BlockT> ParachainBlockData<B> {
	/// Creates a new instance of `Self`.
	pub fn new(
		header: <B as BlockT>::Header,
		extrinsics: sp_std::vec::Vec<<B as BlockT>::Extrinsic>,
		storage_proof: sp_trie::StorageProof,
	) -> Self {
		Self {
			header,
			extrinsics,
			storage_proof,
		}
	}

	/// Convert `self` into the stored header.
	pub fn into_header(self) -> B::Header {
		self.header
	}

	/// Returns the header.
	pub fn header(&self) -> &B::Header {
		&self.header
	}

	/// Returns the extrinsics.
	pub fn extrinsics(&self) -> &[B::Extrinsic] {
		&self.extrinsics
	}

	/// Returns the [`StorageProof`](sp_trie::StorageProof).
	pub fn storage_proof(&self) -> &sp_trie::StorageProof {
		&self.storage_proof
	}

	/// Deconstruct into the inner parts.
	pub fn deconstruct(self) -> (B::Header, sp_std::vec::Vec<B::Extrinsic>, sp_trie::StorageProof) {
		(self.header, self.extrinsics, self.storage_proof)
	}
}
