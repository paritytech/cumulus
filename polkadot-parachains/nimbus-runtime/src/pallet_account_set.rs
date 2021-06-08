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

//! Small pallet responsible for storing a set of accounts, and their associated session keys.
//! This is a minimal solution where staking would be used in practice.
//! The accounts are set and genesis and never change.
//!
//! The Substrate ecosystem has a wide variety of real-world solutions and examples of what this
//! pallet could be replaced with.
//! Gautam's validator set pallet - https://github.com/paritytech/substrate/tree/master/frame/staking/
//! Parity's pallet staking - https://github.com/paritytech/substrate/tree/master/frame/staking/
//! Moonbeam's Parachain Staking - https://github.com/PureStake/moonbeam/tree/master/pallets/parachain-staking
//! Recipe for AccountSet, VecSet, and MapSet

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;

pub use pallet::*;

#[pallet]
pub mod pallet {

	use log::warn;
	use frame_support::pallet_prelude::*;
	use sp_std::vec::Vec;
	use nimbus_primitives::{AccountLookup, CanAuthor};

	/// The Account Set pallet
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config  {
		/// The identifier type for an author.
		type AuthorId: Member + Parameter + MaybeSerializeDeserialize;
	}

	/// The set of accounts that is stored in this pallet.
	#[pallet::storage]
	pub type StoredAccounts<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	impl<T: Config> Get<Vec<T::AccountId>> for Pallet<T> {
		fn get() -> Vec<T::AccountId> {
			StoredAccounts::<T>::get()
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn account_id_of)]
	/// A mapping from the AuthorIds used in the consensus layer
	/// to the AccountIds runtime.
	type Mapping<T: Config> = StorageMap<_, Twox64Concat, T::AuthorId, T::AccountId, OptionQuery>;

	#[pallet::genesis_config]
	/// Genesis config for author mapping pallet
	pub struct GenesisConfig<T: Config> {
		/// The associations that should exist at chain genesis
		pub mapping: Vec<(T::AuthorId, T::AccountId)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { mapping: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			if self.mapping.is_empty() {
				warn!(target: "account-set", "No mappings at genesis. Your chain will have no valid authors.");
			}
			for (author_id, account_id) in &self.mapping {
				Mapping::<T>::insert(author_id, account_id);
				StoredAccounts::<T>::append(account_id);
			}
		}
	}

	/// This pallet is compatible with nimbus's author filtering system. Any account stored in this pallet
	/// is a valid author. Notice that this implementation does not have an inner filter, so it
	/// can only be the beginning of the nimbus filter pipeline.
	impl<T: Config> CanAuthor<T::AccountId> for Pallet<T> {
		fn can_author(author: &T::AccountId, _slot: &u32) -> bool {
			StoredAccounts::<T>::get().contains(author)
		}
	}

	impl<T: Config> AccountLookup<T::AuthorId, T::AccountId> for Pallet<T> {
		fn lookup_account(author: &T::AuthorId) -> Option<T::AccountId> {
			Mapping::<T>::get(&author)
		}
	}
}
