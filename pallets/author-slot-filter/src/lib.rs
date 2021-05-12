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
//! Using a randomness beacon supplied by the `Randomness` trait, this pallet takes the set of
//! currently active accounts from pallet stake, and filters them down to a pseudorandom subset.
//! The current technique gives no preference to any particular author. In the future, we could
//! disfavor authors who are authoring a disproportionate amount of the time in an attempt to
//! "even the playing field".

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;

pub use pallet::*;

#[pallet]
pub mod pallet {

	use log::debug;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::Randomness;
	use sp_std::vec::Vec;
	use frame_system::pallet_prelude::*;
	use sp_core::H256;
	use sp_runtime::Percent;
	use nimbus_primitives::CanAuthor;

	/// The Author Filter pallet
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config + cumulus_pallet_parachain_system::Config {
		/// The identifier type for an author.
		// TODO All the trait bounds? I already removed RuntimeAppPublic.
		type AuthorId: Member;

		/// The overarching event type
		type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;
		/// Deterministic on-chain pseudo-randomness used to do the filtering
		type RandomnessSource: Randomness<H256, Self::BlockNumber>;
		//TODO introduce a new trait for exhaustive sets and use it here.
		/// A source for the complete set of potential authors.
		/// The starting point of the filtering.
		type PotentialAuthors: Get<Vec<Self::AuthorId>>;
	}

	// This code will be called by the author-inherent pallet to check whether the reported author
	// of this block is eligible in this slot. We calculate that result on demand and do not
	// record it in storage (although we do emit a debugging event for now).
	impl<T: Config> CanAuthor<T::AuthorId> for Pallet<T> {
		fn can_author(author: &T::AuthorId, slot: &u32) -> bool {
			let mut active: Vec<T::AuthorId> = T::PotentialAuthors::get();

			let num_eligible = EligibleRatio::<T>::get().mul_ceil(active.len());
			let mut eligible = Vec::with_capacity(num_eligible);

			for i in 0..num_eligible {
				// A context identifier for grabbing the randomness. Consists of three parts
				// - The constant string *b"filter" - to identify this pallet
				// - The index `i` when we're selecting the ith eligible author
				// - The relay parent block number so that the eligible authors at the next height
				//   change. Avoids liveness attacks from colluding minorities of active authors.
				// Third one may not be necessary once we leverage the relay chain's randomness.
				let subject: [u8; 8] = [
					b'f',
					b'i',
					b'l',
					b't',
					b'e',
					b'r',
					i as u8,
					*slot as u8,
				];
				let (randomness, _) = T::RandomnessSource::random(&subject);
				debug!(target: "author-filter", "🎲Randomness sample {}: {:?}", i, &randomness);

				// Cast to u32 first so we get consistent results on 32- and 64-bit platforms.
				let index = (randomness.to_fixed_bytes()[0] as u32) as usize;

				// Move the selected author from the original vector into the eligible vector
				// TODO we could short-circuit this check by returning early when the claimed
				// author is selected. For now I'll leave it like this because:
				// 1. it is easier to understand what our core filtering logic is
				// 2. we currently show the entire filtered set in the debug event
				eligible.push(active.remove(index % active.len()));
			}

			// Print some logs for debugging purposes.
			debug!(target: "author-filter", "Eligible Authors: {:?}", eligible);
			debug!(target: "author-filter", "Ineligible Authors: {:?}", &active);
			debug!(target: "author-filter",
				"Current author, {:?}, is eligible: {}",
				author,
				eligible.contains(author)
			);

			eligible.contains(author)
		}
	}

	// No hooks
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Update the eligible ratio. Intended to be called by governance.
		#[pallet::weight(0)]
		pub fn set_eligible(origin: OriginFor<T>, new: Percent) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			EligibleRatio::<T>::put(&new);
			<Pallet<T>>::deposit_event(Event::EligibleUpdated(new));

			Ok(Default::default())
		}
	}

	/// The percentage of active authors that will be eligible at each height.
	#[pallet::storage]
	pub type EligibleRatio<T: Config> = StorageValue<_, Percent, ValueQuery, Half<T>>;

	// Default value for the `EligibleRatio` is one half.
	#[pallet::type_value]
	pub fn Half<T: Config>() -> Percent {
		Percent::from_percent(50)
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub eligible_ratio: u8,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self {
				eligible_ratio: 50,
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			//TODO ensure that the u8 is less than or equal 100 so it can be apercent

			EligibleRatio::<T>::put(Percent::from_percent(self.eligible_ratio));
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event {
		/// The amount of eligible authors for the filter to select has been changed.
		EligibleUpdated(Percent),
	}
}
