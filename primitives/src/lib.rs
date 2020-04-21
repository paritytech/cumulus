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

/// Identifiers and types related to Cumulus Inherents
pub mod inherents {
	use sp_inherents::InherentIdentifier;

	/// Inherent identifier for downward messages.
	pub const DOWNWARD_MESSAGES_IDENTIFIER: InherentIdentifier = *b"cumdownm";

	/// The type of the inherent downward messages.
	pub type DownwardMessagesType = sp_std::vec::Vec<()>;
}

/// Well known keys for values in the storage.
pub mod well_known_keys {
	/// The storage key for the upward messages.
	///
	/// The upward messages are stored as SCALE encoded `Vec<()>`.
	pub const UPWARD_MESSAGES: &'static [u8] = b":cumulus_upward_messages:";
}

/// Something that should be called when a downward message is received.
#[impl_trait_for_tuples::impl_for_tuples(30)]
pub trait DownwardMessageHandler {
	/// Handle the given downward message.
	fn handle_downward_message(msg: &());
}

/// Something that can send upward messages.
pub trait UpwardMessageSender {
	/// Send an upward message to the relay chain.
	///
	/// Returns an error if sending failed.
	fn send_upward_message(msg: &()) -> Result<(), ()>;
}

/// Validation Function Parameters govern the ability of parachains to upgrade their validation functions.
pub mod validation_function_params {
	use codec::{Decode, Encode};
	use polkadot_parachain::primitives::{RelayChainBlockNumber, ValidationParams};
	use sp_inherents::InherentIdentifier;

	/// Current validation function parameters.
	pub const VALIDATION_FUNCTION_PARAMS: &'static [u8] = b":validation_function_params";

	/// Code upgarde (set as appropriate by a pallet).
	pub const NEW_VALIDATION_CODE: &'static [u8] = b":new_validation_code";

	/// Validation Function Parameters
	///
	/// This struct is the subset of [`ValidationParams`](../../polkadot_parachain/primitives/struct.ValidationParams.html)
	/// which is of interest when upgrading parachain validation functions.
	#[derive(PartialEq, Eq, Encode, Decode, Clone, Copy, Default)]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct ValidationFunctionParams {
		/// The maximum code size permitted, in bytes.
		pub max_code_size: u32,
		/// The current relay-chain block number.
		pub relay_chain_height: RelayChainBlockNumber,
		/// Whether a code upgrade is allowed or not, and at which height the upgrade
		/// would be applied after, if so. The parachain logic should apply any upgrade
		/// issued in this block after the first block
		/// with `relay_chain_height` at least this value, if `Some`. if `None`, issue
		/// no upgrade.
		pub code_upgrade_allowed: Option<RelayChainBlockNumber>,
	}

	impl ValidationFunctionParams {
		pub fn new(vp: &ValidationParams) -> ValidationFunctionParams {
			ValidationFunctionParams {
				max_code_size: vp.max_code_size,
				relay_chain_height: vp.relay_chain_height,
				code_upgrade_allowed: vp.code_upgrade_allowed,
			}
		}
	}

	/// The identifier for the `validation_function_params` inherent.
	pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"valfunp0";
	/// The type of the inherent.
	pub type InherentType = ValidationFunctionParams;
}
