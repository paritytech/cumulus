// Copyright Parity Technologies (UK) Ltd.
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

//! Autogenerated weights for `pallet_core_fellowship`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-05-19, STEPS: `2`, REPEAT: `1`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `cob`, CPU: `<UNKNOWN>`
//! EXECUTION: None, WASM-EXECUTION: Compiled, CHAIN: Some("collectives-polkadot-dev"), DB CACHE: 1024

// Executed Command:
// ./target/debug/polkadot-parachain
// benchmark
// pallet
// --chain=collectives-polkadot-dev
// --steps=2
// --repeat=1
// --pallet=pallet_core_fellowship
// --extrinsic=*
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./parachains/runtimes/collectives/collectives-polkadot/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_core_fellowship`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_core_fellowship::WeightInfo for WeightInfo<T> {
	/// Storage: FellowshipCore Params (r:0 w:1)
	/// Proof: FellowshipCore Params (max_values: Some(1), max_size: Some(364), added: 859, mode: MaxEncodedLen)
	fn set_params() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 65_000_000 picoseconds.
		Weight::from_parts(65_000_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: FellowshipCore Member (r:1 w:1)
	/// Proof: FellowshipCore Member (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
	/// Storage: FellowshipCollective Members (r:1 w:1)
	/// Proof: FellowshipCollective Members (max_values: None, max_size: Some(42), added: 2517, mode: MaxEncodedLen)
	/// Storage: FellowshipCore Params (r:1 w:0)
	/// Proof: FellowshipCore Params (max_values: Some(1), max_size: Some(364), added: 859, mode: MaxEncodedLen)
	/// Storage: FellowshipCollective MemberCount (r:1 w:1)
	/// Proof: FellowshipCollective MemberCount (max_values: None, max_size: Some(14), added: 2489, mode: MaxEncodedLen)
	/// Storage: FellowshipCollective IdToIndex (r:1 w:0)
	/// Proof: FellowshipCollective IdToIndex (max_values: None, max_size: Some(54), added: 2529, mode: MaxEncodedLen)
	/// Storage: FellowshipCore MemberEvidence (r:1 w:1)
	/// Proof: FellowshipCore MemberEvidence (max_values: None, max_size: Some(1067), added: 3542, mode: MaxEncodedLen)
	fn bump_offboard() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1562`
		//  Estimated: `4532`
		// Minimum execution time: 300_000_000 picoseconds.
		Weight::from_parts(300_000_000, 0)
			.saturating_add(Weight::from_parts(0, 4532))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: FellowshipCore Member (r:1 w:1)
	/// Proof: FellowshipCore Member (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
	/// Storage: FellowshipCollective Members (r:1 w:1)
	/// Proof: FellowshipCollective Members (max_values: None, max_size: Some(42), added: 2517, mode: MaxEncodedLen)
	/// Storage: FellowshipCore Params (r:1 w:0)
	/// Proof: FellowshipCore Params (max_values: Some(1), max_size: Some(364), added: 859, mode: MaxEncodedLen)
	/// Storage: FellowshipCollective MemberCount (r:1 w:1)
	/// Proof: FellowshipCollective MemberCount (max_values: None, max_size: Some(14), added: 2489, mode: MaxEncodedLen)
	/// Storage: FellowshipCollective IdToIndex (r:1 w:0)
	/// Proof: FellowshipCollective IdToIndex (max_values: None, max_size: Some(54), added: 2529, mode: MaxEncodedLen)
	/// Storage: FellowshipCore MemberEvidence (r:1 w:1)
	/// Proof: FellowshipCore MemberEvidence (max_values: None, max_size: Some(1067), added: 3542, mode: MaxEncodedLen)
	fn bump_demote() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1672`
		//  Estimated: `4532`
		// Minimum execution time: 339_000_000 picoseconds.
		Weight::from_parts(339_000_000, 0)
			.saturating_add(Weight::from_parts(0, 4532))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: FellowshipCollective Members (r:1 w:0)
	/// Proof: FellowshipCollective Members (max_values: None, max_size: Some(42), added: 2517, mode: MaxEncodedLen)
	/// Storage: FellowshipCore Member (r:1 w:1)
	/// Proof: FellowshipCore Member (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
	fn set_active() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `427`
		//  Estimated: `3514`
		// Minimum execution time: 150_000_000 picoseconds.
		Weight::from_parts(150_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3514))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: FellowshipCore Member (r:1 w:1)
	/// Proof: FellowshipCore Member (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
	/// Storage: FellowshipCollective Members (r:1 w:1)
	/// Proof: FellowshipCollective Members (max_values: None, max_size: Some(42), added: 2517, mode: MaxEncodedLen)
	/// Storage: FellowshipCollective MemberCount (r:1 w:1)
	/// Proof: FellowshipCollective MemberCount (max_values: None, max_size: Some(14), added: 2489, mode: MaxEncodedLen)
	/// Storage: FellowshipCollective IndexToId (r:0 w:1)
	/// Proof: FellowshipCollective IndexToId (max_values: None, max_size: Some(54), added: 2529, mode: MaxEncodedLen)
	/// Storage: FellowshipCollective IdToIndex (r:0 w:1)
	/// Proof: FellowshipCollective IdToIndex (max_values: None, max_size: Some(54), added: 2529, mode: MaxEncodedLen)
	fn induct() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `185`
		//  Estimated: `3514`
		// Minimum execution time: 166_000_000 picoseconds.
		Weight::from_parts(166_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3514))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	/// Storage: FellowshipCollective Members (r:1 w:1)
	/// Proof: FellowshipCollective Members (max_values: None, max_size: Some(42), added: 2517, mode: MaxEncodedLen)
	/// Storage: FellowshipCore Member (r:1 w:1)
	/// Proof: FellowshipCore Member (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
	/// Storage: FellowshipCore Params (r:1 w:0)
	/// Proof: FellowshipCore Params (max_values: Some(1), max_size: Some(364), added: 859, mode: MaxEncodedLen)
	/// Storage: FellowshipCollective MemberCount (r:1 w:1)
	/// Proof: FellowshipCollective MemberCount (max_values: None, max_size: Some(14), added: 2489, mode: MaxEncodedLen)
	/// Storage: FellowshipCore MemberEvidence (r:1 w:1)
	/// Proof: FellowshipCore MemberEvidence (max_values: None, max_size: Some(1067), added: 3542, mode: MaxEncodedLen)
	/// Storage: FellowshipCollective IndexToId (r:0 w:1)
	/// Proof: FellowshipCollective IndexToId (max_values: None, max_size: Some(54), added: 2529, mode: MaxEncodedLen)
	/// Storage: FellowshipCollective IdToIndex (r:0 w:1)
	/// Proof: FellowshipCollective IdToIndex (max_values: None, max_size: Some(54), added: 2529, mode: MaxEncodedLen)
	fn promote() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1540`
		//  Estimated: `4532`
		// Minimum execution time: 308_000_000 picoseconds.
		Weight::from_parts(308_000_000, 0)
			.saturating_add(Weight::from_parts(0, 4532))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	/// Storage: FellowshipCollective Members (r:1 w:0)
	/// Proof: FellowshipCollective Members (max_values: None, max_size: Some(42), added: 2517, mode: MaxEncodedLen)
	/// Storage: FellowshipCore Member (r:1 w:1)
	/// Proof: FellowshipCore Member (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
	/// Storage: FellowshipCore MemberEvidence (r:0 w:1)
	/// Proof: FellowshipCore MemberEvidence (max_values: None, max_size: Some(1067), added: 3542, mode: MaxEncodedLen)
	fn offboard() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `398`
		//  Estimated: `3514`
		// Minimum execution time: 141_000_000 picoseconds.
		Weight::from_parts(141_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3514))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: FellowshipCore Member (r:1 w:1)
	/// Proof: FellowshipCore Member (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
	/// Storage: FellowshipCollective Members (r:1 w:0)
	/// Proof: FellowshipCollective Members (max_values: None, max_size: Some(42), added: 2517, mode: MaxEncodedLen)
	fn import() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `352`
		//  Estimated: `3514`
		// Minimum execution time: 131_000_000 picoseconds.
		Weight::from_parts(131_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3514))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: FellowshipCollective Members (r:1 w:0)
	/// Proof: FellowshipCollective Members (max_values: None, max_size: Some(42), added: 2517, mode: MaxEncodedLen)
	/// Storage: FellowshipCore Member (r:1 w:1)
	/// Proof: FellowshipCore Member (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
	/// Storage: FellowshipCore MemberEvidence (r:1 w:1)
	/// Proof: FellowshipCore MemberEvidence (max_values: None, max_size: Some(1067), added: 3542, mode: MaxEncodedLen)
	fn approve() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1518`
		//  Estimated: `4532`
		// Minimum execution time: 198_000_000 picoseconds.
		Weight::from_parts(198_000_000, 0)
			.saturating_add(Weight::from_parts(0, 4532))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: FellowshipCore Member (r:1 w:0)
	/// Proof: FellowshipCore Member (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
	/// Storage: FellowshipCore MemberEvidence (r:1 w:1)
	/// Proof: FellowshipCore MemberEvidence (max_values: None, max_size: Some(1067), added: 3542, mode: MaxEncodedLen)
	fn submit_evidence() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `151`
		//  Estimated: `4532`
		// Minimum execution time: 99_000_000 picoseconds.
		Weight::from_parts(99_000_000, 0)
			.saturating_add(Weight::from_parts(0, 4532))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
