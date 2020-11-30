// Copyright 2020 Parity Technologies (UK) Ltd.
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

//! Cumulus related primitive types and traits.

#![cfg_attr(not(feature = "std"), no_std)]

pub use polkadot_core_primitives as relay_chain;
pub use polkadot_core_primitives::InboundDownwardMessage;
pub use polkadot_parachain::primitives::{Id as ParaId, UpwardMessage, ValidationParams};
pub use polkadot_primitives::v1::{
	PersistedValidationData, TransientValidationData, ValidationData,
};

#[cfg(feature = "std")]
pub mod genesis;
pub mod xcmp;

/// An inbound HRMP message.
pub type InboundHrmpMessage = polkadot_primitives::v1::InboundHrmpMessage<relay_chain::BlockNumber>;

/// Identifiers and types related to Cumulus Inherents
pub mod inherents {
	use sp_inherents::InherentIdentifier;
	use sp_std::{
		vec::Vec,
		collections::btree_map::BTreeMap,
	};
	use super::{InboundDownwardMessage, InboundHrmpMessage, ParaId};

	/// Inherent identifier for message ingestion inherent.
	pub const MESSAGE_INGESTION_IDENTIFIER: InherentIdentifier = *b"msgingst";
	/// The data passed via a message ingestion inherent. Consists of a bundle of
	/// DMP and HRMP messages.
	#[derive(codec::Encode, codec::Decode, sp_core::RuntimeDebug, Clone, PartialEq)]
	pub struct MessageIngestionType {
		/// Downward messages in the order they were sent.
		pub dmp: Vec<InboundDownwardMessage>,
		/// HRMP messages grouped by channels. The messages in the inner vec must be in order they
		/// were sent. In combination with the rule of no more than one message in a channel per block,
		/// this means `sent_at` is **strictly** greater than the previous one (if any).
		pub hrmp: BTreeMap<ParaId, Vec<InboundHrmpMessage>>,
	}

	/// The identifier for the `set_validation_data` inherent.
	pub const VALIDATION_DATA_IDENTIFIER: InherentIdentifier = *b"valfunp0";
	/// The type of the inherent.
	pub type ValidationDataType = crate::ValidationData;
}

/// Well known keys for values in the storage.
pub mod well_known_keys {
	/// The storage key for the upward messages.
	///
	/// The upward messages are stored as SCALE encoded `Vec<UpwardMessage>`.
	pub const UPWARD_MESSAGES: &'static [u8] = b":cumulus_upward_messages:";

	/// Current validation data.
	pub const VALIDATION_DATA: &'static [u8] = b":cumulus_validation_data:";

	/// Code upgarde (set as appropriate by a pallet).
	pub const NEW_VALIDATION_CODE: &'static [u8] = b":cumulus_new_validation_code:";

	/// The storage key for communicating the HRMP watermark from the runtime to the PVF. Cleared by
	/// the runtime each block and set after  message ingestion, but only if there were messages.
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

#[impl_trait_for_tuples::impl_for_tuples(30)]
pub trait HrmpMessageHandler {
	fn handle_hrmp_message(sender: ParaId, msg: InboundHrmpMessage);
}

/// A trait which is called when the validation data is set.
#[impl_trait_for_tuples::impl_for_tuples(30)]
pub trait OnValidationData {
	fn on_validation_data(data: ValidationData);
}
