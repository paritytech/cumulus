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

//! Autogenerated weights for `pallet_collator_selection`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-04-25, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("westmint-dev"), DB CACHE: 1024

// Executed Command:
// ./artifacts/polkadot-collator
// benchmark
// pallet
// --chain=westmint-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_collator_selection
// --extrinsic=*
// --steps=50
// --repeat=20
// --json
// --header=./file_header.txt
// --output=./polkadot-parachains/westmint/src/weights

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_collator_selection`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_collator_selection::WeightInfo for WeightInfo<T> {
	// Storage: Session NextKeys (r:1 w:0)
	// Storage: CollatorSelection Invulnerables (r:0 w:1)
	fn set_invulnerables(b: u32, ) -> Weight {
		(7_171_000 as Weight)
			// Standard Error: 3_000
			.saturating_add((3_220_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(b as Weight)))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: CollatorSelection DesiredCandidates (r:0 w:1)
	fn set_desired_candidates() -> Weight {
		(8_492_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: CollatorSelection CandidacyBond (r:0 w:1)
	fn set_candidacy_bond() -> Weight {
		(8_543_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: CollatorSelection Candidates (r:1 w:1)
	// Storage: CollatorSelection DesiredCandidates (r:1 w:0)
	// Storage: CollatorSelection Invulnerables (r:1 w:0)
	// Storage: Session NextKeys (r:1 w:0)
	// Storage: CollatorSelection CandidacyBond (r:1 w:0)
	// Storage: CollatorSelection LastAuthoredBlock (r:0 w:1)
	fn register_as_candidate(c: u32, ) -> Weight {
		(50_704_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((82_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: CollatorSelection Candidates (r:1 w:1)
	// Storage: CollatorSelection LastAuthoredBlock (r:0 w:1)
	fn leave_intent(c: u32, ) -> Weight {
		(52_488_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((95_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: System Account (r:2 w:2)
	// Storage: System BlockWeight (r:1 w:1)
	// Storage: CollatorSelection LastAuthoredBlock (r:0 w:1)
	fn note_author() -> Weight {
		(31_751_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	// Storage: CollatorSelection Candidates (r:1 w:1)
	// Storage: CollatorSelection LastAuthoredBlock (r:1000 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: CollatorSelection Invulnerables (r:1 w:0)
	// Storage: System BlockWeight (r:1 w:1)
	fn new_session(r: u32, c: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 1_807_000
			.saturating_add((9_559_000 as Weight).saturating_mul(r as Weight))
			// Standard Error: 1_807_000
			.saturating_add((48_490_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(c as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(r as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(c as Weight)))
	}
}
