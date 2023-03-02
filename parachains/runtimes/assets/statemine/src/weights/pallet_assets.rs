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

//! Autogenerated weights for `pallet_assets`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-02-22, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `bm6`, CPU: `Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("statemine-dev"), DB CACHE: 1024

// Executed Command:
// ./artifacts/polkadot-parachain
// benchmark
// pallet
// --chain=statemine-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_assets
// --extrinsic=*
// --steps=50
// --repeat=20
// --json
// --header=./file_header.txt
// --output=./parachains/runtimes/assets/statemine/src/weights/pallet_assets.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_assets`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_assets::WeightInfo for WeightInfo<T> {
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn create() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `141`
		//  Estimated: `5288`
		// Minimum execution time: 21_487 nanoseconds.
		Weight::from_parts(21_977_000, 0)
			.saturating_add(Weight::from_parts(0, 5288))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	fn force_create() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `6`
		//  Estimated: `2685`
		// Minimum execution time: 10_630 nanoseconds.
		Weight::from_parts(10_963_000, 0)
			.saturating_add(Weight::from_parts(0, 2685))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	fn start_destroy() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `309`
		//  Estimated: `2685`
		// Minimum execution time: 13_627 nanoseconds.
		Weight::from_parts(13_906_000, 0)
			.saturating_add(Weight::from_parts(0, 2685))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: Assets Account (r:1001 w:1000)
	/// Proof: Assets Account (max_values: None, max_size: Some(102), added: 2577, mode: MaxEncodedLen)
	/// Storage: System Account (r:1000 w:1000)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// The range of component `c` is `[0, 1000]`.
	fn destroy_accounts(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0 + c * (240 ±0)`
		//  Estimated: `5262 + c * (5180 ±0)`
		// Minimum execution time: 15_861 nanoseconds.
		Weight::from_parts(16_079_000, 0)
			.saturating_add(Weight::from_parts(0, 5262))
			// Standard Error: 10_732
			.saturating_add(Weight::from_parts(14_192_928, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(c.into())))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(c.into())))
			.saturating_add(Weight::from_parts(0, 5180).saturating_mul(c.into()))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: Assets Approvals (r:1001 w:1000)
	/// Proof: Assets Approvals (max_values: None, max_size: Some(148), added: 2623, mode: MaxEncodedLen)
	/// The range of component `a` is `[0, 1000]`.
	fn destroy_approvals(a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `446 + a * (86 ±0)`
		//  Estimated: `5308 + a * (2623 ±0)`
		// Minimum execution time: 16_455 nanoseconds.
		Weight::from_parts(16_720_000, 0)
			.saturating_add(Weight::from_parts(0, 5308))
			// Standard Error: 7_111
			.saturating_add(Weight::from_parts(13_717_750, 0).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(a.into())))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(a.into())))
			.saturating_add(Weight::from_parts(0, 2623).saturating_mul(a.into()))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: Assets Metadata (r:1 w:0)
	/// Proof: Assets Metadata (max_values: None, max_size: Some(140), added: 2615, mode: MaxEncodedLen)
	fn finish_destroy() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `275`
		//  Estimated: `5300`
		// Minimum execution time: 12_819 nanoseconds.
		Weight::from_parts(13_111_000, 0)
			.saturating_add(Weight::from_parts(0, 5300))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: Assets Account (r:1 w:1)
	/// Proof: Assets Account (max_values: None, max_size: Some(102), added: 2577, mode: MaxEncodedLen)
	fn mint() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `275`
		//  Estimated: `5262`
		// Minimum execution time: 22_897 nanoseconds.
		Weight::from_parts(23_386_000, 0)
			.saturating_add(Weight::from_parts(0, 5262))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: Assets Account (r:1 w:1)
	/// Proof: Assets Account (max_values: None, max_size: Some(102), added: 2577, mode: MaxEncodedLen)
	fn burn() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `383`
		//  Estimated: `5262`
		// Minimum execution time: 28_432 nanoseconds.
		Weight::from_parts(28_903_000, 0)
			.saturating_add(Weight::from_parts(0, 5262))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: Assets Account (r:2 w:2)
	/// Proof: Assets Account (max_values: None, max_size: Some(102), added: 2577, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `383`
		//  Estimated: `10442`
		// Minimum execution time: 39_823 nanoseconds.
		Weight::from_parts(40_380_000, 0)
			.saturating_add(Weight::from_parts(0, 10442))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: Assets Account (r:2 w:2)
	/// Proof: Assets Account (max_values: None, max_size: Some(102), added: 2577, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn transfer_keep_alive() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `383`
		//  Estimated: `10442`
		// Minimum execution time: 34_924 nanoseconds.
		Weight::from_parts(35_584_000, 0)
			.saturating_add(Weight::from_parts(0, 10442))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: Assets Account (r:2 w:2)
	/// Proof: Assets Account (max_values: None, max_size: Some(102), added: 2577, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn force_transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `383`
		//  Estimated: `10442`
		// Minimum execution time: 39_647 nanoseconds.
		Weight::from_parts(40_611_000, 0)
			.saturating_add(Weight::from_parts(0, 10442))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: Assets Asset (r:1 w:0)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: Assets Account (r:1 w:1)
	/// Proof: Assets Account (max_values: None, max_size: Some(102), added: 2577, mode: MaxEncodedLen)
	fn freeze() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `383`
		//  Estimated: `5262`
		// Minimum execution time: 16_462 nanoseconds.
		Weight::from_parts(16_821_000, 0)
			.saturating_add(Weight::from_parts(0, 5262))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:0)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: Assets Account (r:1 w:1)
	/// Proof: Assets Account (max_values: None, max_size: Some(102), added: 2577, mode: MaxEncodedLen)
	fn thaw() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `383`
		//  Estimated: `5262`
		// Minimum execution time: 16_416 nanoseconds.
		Weight::from_parts(16_758_000, 0)
			.saturating_add(Weight::from_parts(0, 5262))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	fn freeze_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `309`
		//  Estimated: `2685`
		// Minimum execution time: 12_853 nanoseconds.
		Weight::from_parts(13_257_000, 0)
			.saturating_add(Weight::from_parts(0, 2685))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	fn thaw_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `309`
		//  Estimated: `2685`
		// Minimum execution time: 12_422 nanoseconds.
		Weight::from_parts(12_763_000, 0)
			.saturating_add(Weight::from_parts(0, 2685))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: Assets Metadata (r:1 w:0)
	/// Proof: Assets Metadata (max_values: None, max_size: Some(140), added: 2615, mode: MaxEncodedLen)
	fn transfer_ownership() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `275`
		//  Estimated: `5300`
		// Minimum execution time: 13_950 nanoseconds.
		Weight::from_parts(14_242_000, 0)
			.saturating_add(Weight::from_parts(0, 5300))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	fn set_team() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `275`
		//  Estimated: `2685`
		// Minimum execution time: 12_726 nanoseconds.
		Weight::from_parts(13_072_000, 0)
			.saturating_add(Weight::from_parts(0, 2685))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:0)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: Assets Metadata (r:1 w:1)
	/// Proof: Assets Metadata (max_values: None, max_size: Some(140), added: 2615, mode: MaxEncodedLen)
	/// The range of component `n` is `[0, 50]`.
	/// The range of component `s` is `[0, 50]`.
	fn set_metadata(_n: u32, s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `275`
		//  Estimated: `5300`
		// Minimum execution time: 22_772 nanoseconds.
		Weight::from_parts(24_026_274, 0)
			.saturating_add(Weight::from_parts(0, 5300))
			// Standard Error: 1_231
			.saturating_add(Weight::from_parts(203, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:0)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: Assets Metadata (r:1 w:1)
	/// Proof: Assets Metadata (max_values: None, max_size: Some(140), added: 2615, mode: MaxEncodedLen)
	fn clear_metadata() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `471`
		//  Estimated: `5300`
		// Minimum execution time: 23_310 nanoseconds.
		Weight::from_parts(23_724_000, 0)
			.saturating_add(Weight::from_parts(0, 5300))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:0)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: Assets Metadata (r:1 w:1)
	/// Proof: Assets Metadata (max_values: None, max_size: Some(140), added: 2615, mode: MaxEncodedLen)
	/// The range of component `n` is `[0, 50]`.
	/// The range of component `s` is `[0, 50]`.
	fn force_set_metadata(n: u32, s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `82`
		//  Estimated: `5300`
		// Minimum execution time: 12_379 nanoseconds.
		Weight::from_parts(12_903_065, 0)
			.saturating_add(Weight::from_parts(0, 5300))
			// Standard Error: 330
			.saturating_add(Weight::from_parts(667, 0).saturating_mul(n.into()))
			// Standard Error: 330
			.saturating_add(Weight::from_parts(2_891, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:0)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: Assets Metadata (r:1 w:1)
	/// Proof: Assets Metadata (max_values: None, max_size: Some(140), added: 2615, mode: MaxEncodedLen)
	fn force_clear_metadata() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `471`
		//  Estimated: `5300`
		// Minimum execution time: 23_531 nanoseconds.
		Weight::from_parts(23_845_000, 0)
			.saturating_add(Weight::from_parts(0, 5300))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	fn force_asset_status() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `275`
		//  Estimated: `2685`
		// Minimum execution time: 12_426 nanoseconds.
		Weight::from_parts(12_888_000, 0)
			.saturating_add(Weight::from_parts(0, 2685))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: Assets Approvals (r:1 w:1)
	/// Proof: Assets Approvals (max_values: None, max_size: Some(148), added: 2623, mode: MaxEncodedLen)
	fn approve_transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `309`
		//  Estimated: `5308`
		// Minimum execution time: 26_365 nanoseconds.
		Weight::from_parts(26_700_000, 0)
			.saturating_add(Weight::from_parts(0, 5308))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: Assets Approvals (r:1 w:1)
	/// Proof: Assets Approvals (max_values: None, max_size: Some(148), added: 2623, mode: MaxEncodedLen)
	/// Storage: Assets Account (r:2 w:2)
	/// Proof: Assets Account (max_values: None, max_size: Some(102), added: 2577, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn transfer_approved() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `553`
		//  Estimated: `13065`
		// Minimum execution time: 53_167 nanoseconds.
		Weight::from_parts(53_564_000, 0)
			.saturating_add(Weight::from_parts(0, 13065))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: Assets Approvals (r:1 w:1)
	/// Proof: Assets Approvals (max_values: None, max_size: Some(148), added: 2623, mode: MaxEncodedLen)
	fn cancel_approval() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `479`
		//  Estimated: `5308`
		// Minimum execution time: 28_120 nanoseconds.
		Weight::from_parts(28_540_000, 0)
			.saturating_add(Weight::from_parts(0, 5308))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: Assets Approvals (r:1 w:1)
	/// Proof: Assets Approvals (max_values: None, max_size: Some(148), added: 2623, mode: MaxEncodedLen)
	fn force_cancel_approval() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `479`
		//  Estimated: `5308`
		// Minimum execution time: 28_503 nanoseconds.
		Weight::from_parts(28_988_000, 0)
			.saturating_add(Weight::from_parts(0, 5308))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	fn set_min_balance() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `383`
		//  Estimated: `3675`
		// Minimum execution time: 16_213 nanoseconds.
		Weight::from_ref_time(16_575_000)
			.saturating_add(Weight::from_proof_size(3675))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}
