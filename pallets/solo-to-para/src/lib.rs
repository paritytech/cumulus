// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::weights::DispatchInfo;
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{DispatchInfoOf, Dispatchable, SignedExtension},
	transaction_validity::{
		InvalidTransaction, TransactionLongevity, TransactionPriority, TransactionValidity,
		TransactionValidityError, ValidTransaction,
	},
};

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_sudo::Config {}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo, Default)]
#[scale_info(skip_type_params(T))]
pub struct CheckSudo<T: Config + Send + Sync>(sp_std::marker::PhantomData<T>);

impl<T: Config + Send + Sync> CheckSudo<T> {
	pub fn new() -> Self {
		Self(Default::default())
	}
}

impl<T: Config + Send + Sync> sp_std::fmt::Debug for CheckSudo<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "CheckSudo")
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl<T: Config + Send + Sync> SignedExtension for CheckSudo<T>
where
	<T as frame_system::Config>::Call: Dispatchable<Info = DispatchInfo>,
{
	type AccountId = T::AccountId;
	type Call = <T as frame_system::Config>::Call;
	type AdditionalSigned = ();
	type Pre = ();
	const IDENTIFIER: &'static str = "CheckSudo";

	fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
		Ok(())
	}

	fn validate(
		&self,
		who: &Self::AccountId,
		_call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> TransactionValidity {
		let root = pallet_sudo::Pallet::<T>::key();

		if *who == root {
			Ok(ValidTransaction {
				priority: info.weight as TransactionPriority,
				longevity: TransactionLongevity::max_value(),
				propagate: true,
				..Default::default()
			})
		} else {
			Err(InvalidTransaction::Call.into())
		}
	}
}
