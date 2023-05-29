// Copyright Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! A module that is responsible for migration of storage for Collator Selection.

use super::*;
use frame_support::{log, traits::OnRuntimeUpgrade};

/// Version 1 Migration
/// This migration ensures that any existing `Invulnerables` storage lists are sorted.
pub mod v1 {
	use super::*;
	use sp_std::prelude::*;
	use frame_support::pallet_prelude::*;

	pub struct MigrateToV1<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
		fn on_runtime_upgrade() -> Weight {
			let onchain_version = Pallet::<T>::on_chain_storage_version();
			if onchain_version == 0 {
				let invulnerables_len = Invulnerables::<T>::get().to_vec().len();
				<Invulnerables<T>>::mutate(|invulnerables| {
					invulnerables.sort();
				});

				StorageVersion::new(1).put::<Pallet<T>>();
				log::info!(
					target: LOG_TARGET,
					"Sorted {} Invulnerables, upgraded storage to version 1",
					invulnerables_len,
				);
				// Similar complexity to `set_invulnerables` (put storage value)
				// Plus 1 read for length, 1 read for `onchain_version`, 1 write to put version
				T::WeightInfo::set_invulnerables(invulnerables_len as u32)
					.saturating_add(T::DbWeight::get().reads_writes(2, 1))
			} else {
				log::info!(
					target: LOG_TARGET,
					"Migration did not execute. This probably should be removed"
				);
				T::DbWeight::get().reads(1)
			}
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
			let number_of_invulnerables = Invulnerables::<T>::get().to_vec().len();
			Ok((number_of_invulnerables as u32).encode())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(number_of_invulnerables: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
			let stored_invulnerables = Invulnerables::<T>::get().to_vec();
			let mut sorted_invulnerables = stored_invulnerables.clone();
			sorted_invulnerables.sort();
			assert_eq!(
				stored_invulnerables, sorted_invulnerables,
				"after migration, the stored invulnerables should be sorted"
			);

			let number_of_invulnerables: u32 = Decode::decode(
				&mut number_of_invulnerables.as_slice(),
			)
			.expect("the state parameter should be something that was generated by pre_upgrade");
			let stored_invulnerables_len = stored_invulnerables.len() as u32;
			assert_eq!(
				number_of_invulnerables, stored_invulnerables_len,
				"after migration, there should be the same number of invulnerables"
			);

			let onchain_version = Pallet::<T>::on_chain_storage_version();
			frame_support::ensure!(onchain_version >= 1, "must_upgrade");

			Ok(())
		}
	}
}
