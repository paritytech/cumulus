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
use cumulus_pallet_parachain_system as parachain_system;
use frame_support::{dispatch::DispatchResult, pallet_prelude::*, weights::DispatchInfo};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{DispatchInfoOf, Dispatchable, SignedExtension},
	transaction_validity::{
		InvalidTransaction, TransactionLongevity, TransactionPriority, TransactionValidity,
		TransactionValidityError, ValidTransaction,
	},
};
use sp_std::{prelude::*, vec::Vec};

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config:
		frame_system::Config + cumulus_pallet_parachain_system::Config + pallet_sudo::Config
	{
		type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// In case of a scheduled migration, this storage field contains the custom head data to be applied.
	#[pallet::storage]
	pub(super) type PendingCustomValidationHeadData<T: Config> =
		StorageValue<_, Vec<u8>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event {
		MigrationScheduled,
		/// The custom validation head data has been scheduled to apply.
		CustomValidationHeadDataStored,
		/// The custom validation head data was applied as of the contained relay chain block number.
		CustomValidationHeadDataApplied, //(RelayChainBlockNumber),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// CustomHeadData is not stored in storage.
		NoCustomHeadData,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn schedule_migration(
			origin: OriginFor<T>,
			code: Vec<u8>,
			head_data: Vec<u8>,
		) -> DispatchResult {
			ensure_root(origin)?;

			parachain_system::Pallet::<T>::set_code_impl(code)?;
			Self::store_pending_custom_validation_head_data(head_data);

			Self::deposit_event(Event::MigrationScheduled);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Set a custom head data that should only be applied when upgradeGoAheadSignal from
		/// the Relay Chain is GoAhead
		fn store_pending_custom_validation_head_data(head_data: Vec<u8>) {
			PendingCustomValidationHeadData::<T>::put(head_data);
			Self::deposit_event(Event::CustomValidationHeadDataStored);
		}

		pub fn set_pending_custom_validation_head_data() -> Result<(), Error<T>> {
			match <PendingCustomValidationHeadData<T>>::get() {
				Some(head_data) => {
					parachain_system::Pallet::<T>::set_custom_validation_head_data(head_data);
					<PendingCustomValidationHeadData<T>>::kill();
					Self::deposit_event(Event::CustomValidationHeadDataApplied);
				},
				None => return Err((Error::<T>::NoCustomHeadData).into()),
			}
			Ok(())
		}
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

		fn pre_dispatch(
			self,
			who: &Self::AccountId,
			call: &Self::Call,
			info: &DispatchInfoOf<Self::Call>,
			len: usize,
		) -> Result<Self::Pre, TransactionValidityError> {
			Ok(self.validate(who, call, info, len).map(|_| ())?)
		}

		fn validate(
			&self,
			who: &Self::AccountId,
			_call: &Self::Call,
			info: &DispatchInfoOf<Self::Call>,
			_len: usize,
		) -> TransactionValidity {
			let root_account = match pallet_sudo::Pallet::<T>::key() {
				Some(account) => account,
				None => return Err(InvalidTransaction::Call.into()),
			};

			if *who == root_account {
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
}
