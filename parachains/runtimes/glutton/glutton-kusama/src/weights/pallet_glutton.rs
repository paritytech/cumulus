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

//! Autogenerated weights for `pallet_balances`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-02-23, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `bm6`, CPU: `Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("collectives-polkadot-dev"), DB CACHE: 1024

// Executed Command:
// ./artifacts/polkadot-parachain
// benchmark
// pallet
// --chain=collectives-polkadot-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_balances
// --extrinsic=*
// --steps=50
// --repeat=20
// --json
// --header=./file_header.txt
// --output=./parachains/runtimes/collectives/collectives-polkadot/src/weights/pallet_balances.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_balances`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_glutton::WeightInfo for WeightInfo<T> {
	/// Storage: Glutton TrashDataCount (r:1 w:1)
	/// Proof: Glutton TrashDataCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Glutton TrashData (r:0 w:1000)
	/// Proof: Glutton TrashData (max_values: Some(65000), max_size: Some(1036), added: 3016, mode: MaxEncodedLen)
	/// The range of component `n` is `[0, 1000]`.
	fn initialize_pallet_grow(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `4`
		//  Estimated: `499`
		// Minimum execution time: 9_248 nanoseconds.
		Weight::from_ref_time(9_582_000)
			.saturating_add(Weight::from_proof_size(499))
			// Standard Error: 1_144
			.saturating_add(Weight::from_ref_time(1_624_225).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(n.into())))
	}
	/// Storage: Glutton TrashDataCount (r:1 w:1)
	/// Proof: Glutton TrashDataCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Glutton TrashData (r:0 w:1000)
	/// Proof: Glutton TrashData (max_values: Some(65000), max_size: Some(1036), added: 3016, mode: MaxEncodedLen)
	/// The range of component `n` is `[0, 1000]`.
	fn initialize_pallet_shrink(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `65`
		//  Estimated: `499`
		// Minimum execution time: 10_210 nanoseconds.
		Weight::from_ref_time(7_260_854)
			.saturating_add(Weight::from_proof_size(499))
			// Standard Error: 1_459
			.saturating_add(Weight::from_ref_time(1_053_844).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(n.into())))
	}
	/// The range of component `i` is `[0, 100000]`.
	fn waste_ref_time_iter(i: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 605 nanoseconds.
		Weight::from_ref_time(1_102_112)
			.saturating_add(Weight::from_proof_size(0))
			// Standard Error: 28
			.saturating_add(Weight::from_ref_time(120_597).saturating_mul(i.into()))
	}
	/// Storage: Glutton TrashData (r:5000 w:0)
	/// Proof: Glutton TrashData (max_values: Some(65000), max_size: Some(1036), added: 3016, mode: MaxEncodedLen)
	/// The range of component `i` is `[0, 5000]`.
	fn waste_proof_size_some(i: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `119036 + i * (1053 ±0)`
		//  Estimated: `0 + i * (3016 ±0)`
		// Minimum execution time: 464 nanoseconds.
		Weight::from_ref_time(552_000)
			.saturating_add(Weight::from_proof_size(0))
			// Standard Error: 1_962
			.saturating_add(Weight::from_ref_time(5_780_304).saturating_mul(i.into()))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(i.into())))
			.saturating_add(Weight::from_proof_size(3016).saturating_mul(i.into()))
	}
	/// Storage: Glutton Storage (r:1 w:0)
	/// Proof: Glutton Storage (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Glutton Compute (r:1 w:0)
	/// Proof: Glutton Compute (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Glutton TrashData (r:1738 w:0)
	/// Proof: Glutton TrashData (max_values: Some(65000), max_size: Some(1036), added: 3016, mode: MaxEncodedLen)
	fn on_idle_high_proof_waste() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1955391`
		//  Estimated: `5242806`
		// Minimum execution time: 55_398_405 nanoseconds.
		Weight::from_ref_time(55_594_848_000)
			.saturating_add(Weight::from_proof_size(5242806))
			.saturating_add(T::DbWeight::get().reads(1740_u64))
	}
	/// Storage: Glutton Storage (r:1 w:0)
	/// Proof: Glutton Storage (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Glutton Compute (r:1 w:0)
	/// Proof: Glutton Compute (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Glutton TrashData (r:6 w:0)
	/// Proof: Glutton TrashData (max_values: Some(65000), max_size: Some(1036), added: 3016, mode: MaxEncodedLen)
	fn on_idle_low_proof_waste() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `11717`
		//  Estimated: `19094`
		// Minimum execution time: 98_888_667 nanoseconds.
		Weight::from_ref_time(99_157_239_000)
			.saturating_add(Weight::from_proof_size(19094))
			.saturating_add(T::DbWeight::get().reads(8_u64))
	}
	/// Storage: Glutton Storage (r:1 w:0)
	/// Proof: Glutton Storage (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Glutton Compute (r:1 w:0)
	/// Proof: Glutton Compute (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	fn empty_on_idle() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `4`
		//  Estimated: `998`
		// Minimum execution time: 3_967 nanoseconds.
		Weight::from_ref_time(4_121_000)
			.saturating_add(Weight::from_proof_size(998))
			.saturating_add(T::DbWeight::get().reads(2_u64))
	}
	/// Storage: Glutton Compute (r:0 w:1)
	/// Proof: Glutton Compute (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	fn set_compute() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_112 nanoseconds.
		Weight::from_ref_time(7_484_000)
			.saturating_add(Weight::from_proof_size(0))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: Glutton Storage (r:0 w:1)
	/// Proof: Glutton Storage (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	fn set_storage() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_056 nanoseconds.
		Weight::from_ref_time(7_414_000)
			.saturating_add(Weight::from_proof_size(0))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}