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

//! A Nimbus filter for the AuRa consensus algorithm. This filter does not use any entropy, it
//! simply rotates authors in order. A single author is eligible at each slot.
//!
//! In the Substrate ecosystem, this algorithm is typically known as AuRa (authority round).
//! There is a well known implementation in the main Substrate repository and published at
//! https://crates.io/crates/sc-consensus-aura. There are two primary differences between
//! the approaches:
//!
//! 1. This filter leverages all the heavy lifting of the Nimbus framework and consequently is
//!    capable of expressing Aura in < 100 lines of code.
//!
//!    Whereas sc-consensus-aura includes the entire consensus stack including block signing, digest
//!    formats, and slot prediction. This is a lot of overhead for a sipmle round robin
//!    consensus that basically boils down to this function
//!    https://github.com/paritytech/substrate/blob/0f849efc/client/consensus/aura/src/lib.rs#L91-L106
//!
//! 2. The Nimbus framework places the author checking logic in the runtime which makes it relatively
//!    easy for relay chain validators to confirm the author is valid.
//!
//!    Whereas sc-consensus-aura places the author checking offchain. The offchain approach is fine
//!    for standalone layer 1 blockchains, but net well suited for verification on the relay chain
//!    where validators only run a wasm blob.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;
pub use pallet::*;

#[pallet]
pub mod pallet {

	use frame_support::pallet_prelude::*;
	use sp_std::vec::Vec;

	//TODO Now that the CanAuthor trait takes a slot number, I don't think this even needs to be a pallet.
	// I think it could eb jsut a simple type.
	/// The Author Filter pallet
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// A source for the complete set of potential authors.
		/// The starting point of the filtering.
		type PotentialAuthors: Get<Vec<Self::AccountId>>;
	}

	// This code will be called by the author-inherent pallet to check whether the reported author
	// of this block is eligible at this slot. We calculate that result on demand and do not
	// record it instorage.
	impl<T: Config> nimbus_primitives::CanAuthor<T::AccountId> for Pallet<T> {
		fn can_author(account: &T::AccountId, slot: &u32) -> bool {
			let active: Vec<T::AccountId> = T::PotentialAuthors::get();

			// This is the core Aura logic right here.
			let active_author = &active[*slot as usize % active.len()];

			account == active_author
		}
	}
}
