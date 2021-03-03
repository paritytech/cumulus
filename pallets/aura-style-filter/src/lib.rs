// Copyright 2019-2020 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! Small pallet responsible determining which accounts are eligible to author at the current
//! slot. The slot is determined by the relay parent block number from the parachain inherent.
//!
//! This pallet does not use any entropy, it simply rotates authors in order. A single author is
//! eligible at each slot.
//!
//! In the Substrate ecosystem, this algorithm is typically known as AuRa (authority Round). There
//! is a well known implementatino in the main substrate repo. However that implementation does all
//! of the author checking offchain, which is not suitable for parachain use. This pallet
//! implements the same author selection algorithm in the Author Filter framework.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;

pub use pallet::*;

#[pallet]
pub mod pallet {

	// use frame_support::debug::debug;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::Vec;
	use frame_system::pallet_prelude::*;

	/// The Author Filter pallet
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config + cumulus_pallet_parachain_system::Config {
		/// A source for the complete set of potential authors.
		/// The starting point of the filtering.
		type PotentialAuthors: Get<Vec<Self::AccountId>>;
	}

	// This code will be called by the author-inherent pallet to check whether the reported author
	// of this block is eligible at this slot. We calculate that result on demand and do not
	// record it instorage.
	impl<T: Config> pallet_author_inherent::CanAuthor<T::AccountId> for Pallet<T> {
		fn can_author(account: &T::AccountId) -> bool {

			// Grab the relay parent height as a temporary source of relay-based entropy
			let validation_data = cumulus_pallet_parachain_system::Module::<T>::validation_data()
				.expect("validation data was set in parachain system inherent");
			let relay_height = validation_data.relay_parent_number;

			Self::can_author_helper(account, relay_height)
		}
	}

	impl<T: Config> Pallet<T> {
		/// Helper method to calculate eligible authors
		pub fn can_author_helper(account: &T::AccountId, relay_height: u32) -> bool {
			let active: Vec<T::AccountId> = T::PotentialAuthors::get();

			// This is the core Aura logic right here.
			let active_author = &active[relay_height as usize % active.len()];

			account == active_author
		}
	}

	// No hooks
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// TODO do I need this?
	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	// /// The percentage of active authors that will be eligible at each height.
	// #[pallet::storage]
	// pub type EligibleRatio<T: Config> = StorageValue<_, Percent, ValueQuery, Half<T>>;
	//
	// // Default value for the `EligibleRatio` is one half.
	// #[pallet::type_value]
	// pub fn Half<T: Config>() -> Percent {
	// 	Percent::from_percent(50)
	// }
	//
	// #[pallet::genesis_config]
	// pub struct GenesisConfig {
	// 	pub eligible_ratio: u8,
	// }
	//
	// #[cfg(feature = "std")]
	// impl Default for GenesisConfig {
	// 	fn default() -> Self {
	// 		Self {
	// 			eligible_ratio: 50,
	// 		}
	// 	}
	// }
	//
	// #[pallet::genesis_build]
	// impl<T: Config> GenesisBuild<T> for GenesisConfig {
	// 	fn build(&self) {
	// 		//TODO ensure that the u8 is less than or equal 100 so it can be apercent
	//
	// 		EligibleRatio::<T>::put(Percent::from_percent(self.eligible_ratio));
	// 	}
	// }
	//
	// #[pallet::event]
	// #[pallet::generate_deposit(fn deposit_event)]
	// pub enum Event {
	// 	/// The amount of eligible authors for the filter to select has been changed.
	// 	EligibleUpdated(Percent),
	// }
}
