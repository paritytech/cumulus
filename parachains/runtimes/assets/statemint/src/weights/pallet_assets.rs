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
//! DATE: 2022-11-10, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `bm6`, CPU: `Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("statemint-dev"), DB CACHE: 1024

// Executed Command:
// ./artifacts/polkadot-parachain
// benchmark
// pallet
// --chain=statemint-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_assets
// --extrinsic=*
// --steps=50
// --repeat=20
// --json
// --header=./file_header.txt
// --output=./parachains/runtimes/assets/statemint/src/weights/pallet_assets.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_assets`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_assets::WeightInfo for WeightInfo<T> {
	// Storage: Assets Asset (r:1 w:1)
	fn create() -> Weight {
		// Minimum execution time: 32_921 nanoseconds.
		Weight::from_ref_time(33_633_000 as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Assets Asset (r:1 w:1)
	fn force_create() -> Weight {
		// Minimum execution time: 19_787 nanoseconds.
		Weight::from_ref_time(20_249_000 as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:5002 w:5001)
	// Storage: System Account (r:5000 w:5000)
	// Storage: Assets Metadata (r:1 w:0)
	// Storage: Assets Approvals (r:501 w:500)
	/// The range of component `c` is `[0, 5000]`.
	/// The range of component `s` is `[0, 5000]`.
	/// The range of component `a` is `[0, 500]`.
	fn destroy(c: u32, s: u32, a: u32, ) -> Weight {
		// Minimum execution time: 74_325_463 nanoseconds.
		Weight::from_ref_time(74_947_012_000 as u64)
			// Standard Error: 139_062
			.saturating_add(Weight::from_ref_time(10_871_043 as u64).saturating_mul(c as u64))
			// Standard Error: 139_062
			.saturating_add(Weight::from_ref_time(11_485_328 as u64).saturating_mul(s as u64))
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().reads((2 as u64).saturating_mul(c as u64)))
			.saturating_add(T::DbWeight::get().reads((2 as u64).saturating_mul(s as u64)))
			.saturating_add(T::DbWeight::get().reads((1 as u64).saturating_mul(a as u64)))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
			.saturating_add(T::DbWeight::get().writes((2 as u64).saturating_mul(c as u64)))
			.saturating_add(T::DbWeight::get().writes((2 as u64).saturating_mul(s as u64)))
			.saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(a as u64)))
	}
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	fn mint() -> Weight {
		// Minimum execution time: 37_940 nanoseconds.
		Weight::from_ref_time(38_473_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	fn burn() -> Weight {
		// Minimum execution time: 41_145 nanoseconds.
		Weight::from_ref_time(41_670_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	fn transfer() -> Weight {
		// Minimum execution time: 53_947 nanoseconds.
		Weight::from_ref_time(54_412_000 as u64)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	fn transfer_keep_alive() -> Weight {
		// Minimum execution time: 47_048 nanoseconds.
		Weight::from_ref_time(47_946_000 as u64)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	fn force_transfer() -> Weight {
		// Minimum execution time: 55_147 nanoseconds.
		Weight::from_ref_time(55_548_000 as u64)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Assets Asset (r:1 w:0)
	// Storage: Assets Account (r:1 w:1)
	fn freeze() -> Weight {
		// Minimum execution time: 25_841 nanoseconds.
		Weight::from_ref_time(26_506_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Assets Asset (r:1 w:0)
	// Storage: Assets Account (r:1 w:1)
	fn thaw() -> Weight {
		// Minimum execution time: 26_639 nanoseconds.
		Weight::from_ref_time(27_081_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Assets Asset (r:1 w:1)
	fn freeze_asset() -> Weight {
		// Minimum execution time: 22_413 nanoseconds.
		Weight::from_ref_time(23_213_000 as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Assets Asset (r:1 w:1)
	fn thaw_asset() -> Weight {
		// Minimum execution time: 21_963 nanoseconds.
		Weight::from_ref_time(22_479_000 as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Metadata (r:1 w:0)
	fn transfer_ownership() -> Weight {
		// Minimum execution time: 25_533 nanoseconds.
		Weight::from_ref_time(25_979_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Assets Asset (r:1 w:1)
	fn set_team() -> Weight {
		// Minimum execution time: 23_829 nanoseconds.
		Weight::from_ref_time(24_876_000 as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Assets Asset (r:1 w:0)
	// Storage: Assets Metadata (r:1 w:1)
	/// The range of component `n` is `[0, 50]`.
	/// The range of component `s` is `[0, 50]`.
	fn set_metadata(_n: u32, s: u32, ) -> Weight {
		// Minimum execution time: 36_438 nanoseconds.
		Weight::from_ref_time(38_367_681 as u64)
			// Standard Error: 1_203
			.saturating_add(Weight::from_ref_time(7_334 as u64).saturating_mul(s as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Assets Asset (r:1 w:0)
	// Storage: Assets Metadata (r:1 w:1)
	fn clear_metadata() -> Weight {
		// Minimum execution time: 40_670 nanoseconds.
		Weight::from_ref_time(41_049_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Assets Asset (r:1 w:0)
	// Storage: Assets Metadata (r:1 w:1)
	/// The range of component `n` is `[0, 50]`.
	/// The range of component `s` is `[0, 50]`.
	fn force_set_metadata(n: u32, s: u32, ) -> Weight {
		// Minimum execution time: 24_647 nanoseconds.
		Weight::from_ref_time(25_823_064 as u64)
			// Standard Error: 623
			.saturating_add(Weight::from_ref_time(1_109 as u64).saturating_mul(n as u64))
			// Standard Error: 623
			.saturating_add(Weight::from_ref_time(1_174 as u64).saturating_mul(s as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Assets Asset (r:1 w:0)
	// Storage: Assets Metadata (r:1 w:1)
	fn force_clear_metadata() -> Weight {
		// Minimum execution time: 41_329 nanoseconds.
		Weight::from_ref_time(41_824_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Assets Asset (r:1 w:1)
	fn force_asset_status() -> Weight {
		// Minimum execution time: 23_806 nanoseconds.
		Weight::from_ref_time(24_275_000 as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Approvals (r:1 w:1)
	fn approve_transfer() -> Weight {
		// Minimum execution time: 40_971 nanoseconds.
		Weight::from_ref_time(41_749_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: Assets Approvals (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	fn transfer_approved() -> Weight {
		// Minimum execution time: 70_934 nanoseconds.
		Weight::from_ref_time(71_774_000 as u64)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Approvals (r:1 w:1)
	fn cancel_approval() -> Weight {
		// Minimum execution time: 43_221 nanoseconds.
		Weight::from_ref_time(43_660_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Approvals (r:1 w:1)
	fn force_cancel_approval() -> Weight {
		// Minimum execution time: 43_115 nanoseconds.
		Weight::from_ref_time(43_577_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
}
