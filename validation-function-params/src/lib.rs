// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Core types and inherents for validation function parameters.
//!
//! Note that this inherent can't be provided normally, because it depends
//! on data which originates in the relay chain. Instead, it is injected
//! within Cumulus; see
//! https://github.com/paritytech/cumulus/blob/8169b45d66a797c6786b0178121afad17a218998/collator/src/lib.rs#L151-L179

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
#[cfg(feature = "std")]
use codec::Decode;
use sp_inherents::{InherentIdentifier, IsFatalError, InherentData};
pub use cumulus_runtime::ValidationFunctionParams;

/// The identifier for the `validation_function_params` inherent.
pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"valfunp0";
/// The type of the inherent.
pub type InherentType = ValidationFunctionParams;

/// Errors that can occur while checking the timestamp inherent.
// until we come up with some variants which make sense, this type can't be instantiated
#[derive(Encode, sp_runtime::RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Decode))]
pub enum InherentError {}

impl IsFatalError for InherentError {
	fn is_fatal_error(&self) -> bool {
		true // how'd we get an instance, anyway?
	}
}

impl InherentError {
	/// Try to create an instance ouf of the given identifier and data.
	#[cfg(feature = "std")]
	pub fn try_from(id: &InherentIdentifier, data: &[u8]) -> Option<Self> {
		if id == &INHERENT_IDENTIFIER {
			<InherentError as codec::Decode>::decode(&mut &data[..]).ok()
		} else {
			None
		}
	}
}

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
