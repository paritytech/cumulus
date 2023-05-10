// Copyright 2023 Parity Technologies (UK) Ltd.
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

//! The definition of a [`FixedVelocityConsensusHook`] for consensus logic to manage
//! block velocity.
//!
//! The velocity `V` refers to the rate of block processing by the relay chain.

use super::pallet;
use frame_support::pallet_prelude::*;
use sp_std::marker::PhantomData;

/// A consensus hook for a fixed block processing velocity.
pub struct FixedVelocityConsensusHook<T, const V: u32, const C: u32>(PhantomData<T>);

impl<T: pallet::Config, const V: u32, const C: u32> FixedVelocityConsensusHook<T, V, C> {
	/// Validates the number of authored blocks within the slot with respect to the `V + 1` limit.
	pub fn validate_slot() -> Weight {
		// Ensure velocity is non-zero.
		let velocity = V.max(1);

		let authored = pallet::Pallet::<T>::slot_info()
			.map(|(_slot, authored)| authored)
			.expect("slot info is inserted on block initialization");
		if authored > velocity + 1 {
			panic!("authored blocks limit is reached for the slot")
		}

		T::DbWeight::get().reads(1)
	}
}
