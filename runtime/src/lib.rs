// Copyright 2019 Parity Technologies (UK) Ltd.
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

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_runtime::traits::Block as BlockT;
///! The Cumulus runtime to make a runtime a parachain.
use sp_std::vec::Vec;

#[cfg(not(feature = "std"))]
#[doc(hidden)]
pub use sp_std::slice;

#[macro_use]
pub mod validate_block;

/// The witness data type.
type WitnessData = Vec<Vec<u8>>;

/// The parachain block that is created on a collator and validated by a validator.
#[derive(Encode, Decode)]
pub struct ParachainBlockData<B: BlockT> {
	/// The header of the parachain block.
	header: <B as BlockT>::Header,
	/// The extrinsics of the parachain block without the `PolkadotInherent`.
	extrinsics: Vec<<B as BlockT>::Extrinsic>,
	/// The data that is required to emulate the storage accesses executed by all extrinsics.
	witness_data: WitnessData,
	/// The storage root of the witness data.
	witness_data_storage_root: <B as BlockT>::Hash,
}

impl<B: BlockT> ParachainBlockData<B> {
	pub fn new(
		header: <B as BlockT>::Header,
		extrinsics: Vec<<B as BlockT>::Extrinsic>,
		witness_data: WitnessData,
		witness_data_storage_root: <B as BlockT>::Hash,
	) -> Self {
		Self {
			header,
			extrinsics,
			witness_data,
			witness_data_storage_root,
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
}


pub mod validation_function_params {
	use codec::{Decode, Encode};
	use parachain::primitives::{RelayChainBlockNumber, ValidationParams};
	use sp_inherents::{InherentIdentifier, InherentData};

	/// Current validation function parameters.
	pub const VALIDATION_FUNCTION_PARAMS: &'static [u8] = b":validation_function_params";

	/// Code upgarde (set as appropriate by a pallet).
	pub const NEW_VALIDATION_CODE: &'static [u8] = b":new_validation_code";

	/// Validation Function Parameters
	///
	/// This struct is the subset of [`ValidationParams`](../../polkadot_parachain/primitives/struct.ValidationParams.html)
	/// which is of interest when upgrading parachain validation functions.
	#[derive(PartialEq, Eq, Encode, Decode, Clone, Copy)]
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

	/// Auxiliary trait to extract timestamp inherent data.
	pub trait ValidationFunctionParamsInherentData {
		/// Get `ValidationFunctionParams` inherent data.
		fn validation_function_params_inherent_data(&self) -> Result<InherentType, sp_inherents::Error>;
	}

	impl ValidationFunctionParamsInherentData for InherentData {
		fn validation_function_params_inherent_data(&self) -> Result<InherentType, sp_inherents::Error> {
			self.get_data(&INHERENT_IDENTIFIER)
				.and_then(|r| r.ok_or_else(|| "Timestamp inherent data not found".into()))
		}
	}
}
