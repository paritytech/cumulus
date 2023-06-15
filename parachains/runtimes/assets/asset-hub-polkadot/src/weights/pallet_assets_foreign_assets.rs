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

//! Autogenerated weights for `pallet_assets`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-06-15, STEPS: `2`, REPEAT: `2`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `bkontur-ThinkPad-P14s-Gen-2i`, CPU: `11th Gen Intel(R) Core(TM) i7-1185G7 @ 3.00GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("asset-hub-polkadot-dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/polkadot-parachain
// benchmark
// pallet
// --steps=2
// --repeat=2
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --json-file=./bench.json
// --header=./file_header.txt
// --chain=asset-hub-polkadot-dev
// --pallet=pallet_assets
// --output=./parachains/runtimes/assets/asset-hub-polkadot/src/weights

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_assets`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_assets::WeightInfo for WeightInfo<T> {
	/// Storage: ParachainInfo ParachainId (r:1 w:0)
	/// Proof: ParachainInfo ParachainId (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn create() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `107`
		//  Estimated: `4273`
		// Minimum execution time: 37_287_000 picoseconds.
		Weight::from_parts(41_350_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	fn force_create() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `4`
		//  Estimated: `4273`
		// Minimum execution time: 14_862_000 picoseconds.
		Weight::from_parts(17_084_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	fn start_destroy() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `276`
		//  Estimated: `4273`
		// Minimum execution time: 15_974_000 picoseconds.
		Weight::from_parts(18_461_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Account (r:1001 w:1000)
	/// Proof: ForeignAssets Account (max_values: None, max_size: Some(732), added: 3207, mode: MaxEncodedLen)
	/// Storage: System Account (r:1000 w:1000)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// The range of component `c` is `[0, 1000]`.
	/// The range of component `c` is `[0, 1000]`.
	fn destroy_accounts(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `242 + c * (208 ±0)`
		//  Estimated: `4273 + c * (3207 ±0)`
		// Minimum execution time: 20_611_000 picoseconds.
		Weight::from_parts(23_087_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			// Standard Error: 383_890
			.saturating_add(Weight::from_parts(13_875_751, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(c.into())))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(c.into())))
			.saturating_add(Weight::from_parts(0, 3207).saturating_mul(c.into()))
	}
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Approvals (r:1001 w:1000)
	/// Proof: ForeignAssets Approvals (max_values: None, max_size: Some(746), added: 3221, mode: MaxEncodedLen)
	/// The range of component `a` is `[0, 1000]`.
	/// The range of component `a` is `[0, 1000]`.
	fn destroy_approvals(a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `276 + a * (86 ±0)`
		//  Estimated: `4273 + a * (3221 ±0)`
		// Minimum execution time: 22_419_000 picoseconds.
		Weight::from_parts(23_993_500, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			// Standard Error: 641_080
			.saturating_add(Weight::from_parts(18_210_812, 0).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(a.into())))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(a.into())))
			.saturating_add(Weight::from_parts(0, 3221).saturating_mul(a.into()))
	}
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Metadata (r:1 w:0)
	/// Proof: ForeignAssets Metadata (max_values: None, max_size: Some(738), added: 3213, mode: MaxEncodedLen)
	fn finish_destroy() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `242`
		//  Estimated: `4273`
		// Minimum execution time: 17_500_000 picoseconds.
		Weight::from_parts(21_725_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Account (r:1 w:1)
	/// Proof: ForeignAssets Account (max_values: None, max_size: Some(732), added: 3207, mode: MaxEncodedLen)
	fn mint() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `242`
		//  Estimated: `4273`
		// Minimum execution time: 31_134_000 picoseconds.
		Weight::from_parts(34_636_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Account (r:1 w:1)
	/// Proof: ForeignAssets Account (max_values: None, max_size: Some(732), added: 3207, mode: MaxEncodedLen)
	fn burn() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `350`
		//  Estimated: `4273`
		// Minimum execution time: 38_540_000 picoseconds.
		Weight::from_parts(41_880_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Account (r:2 w:2)
	/// Proof: ForeignAssets Account (max_values: None, max_size: Some(732), added: 3207, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `350`
		//  Estimated: `7404`
		// Minimum execution time: 53_139_000 picoseconds.
		Weight::from_parts(60_514_000, 0)
			.saturating_add(Weight::from_parts(0, 7404))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Account (r:2 w:2)
	/// Proof: ForeignAssets Account (max_values: None, max_size: Some(732), added: 3207, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn transfer_keep_alive() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `350`
		//  Estimated: `7404`
		// Minimum execution time: 46_891_000 picoseconds.
		Weight::from_parts(51_853_000, 0)
			.saturating_add(Weight::from_parts(0, 7404))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Account (r:2 w:2)
	/// Proof: ForeignAssets Account (max_values: None, max_size: Some(732), added: 3207, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn force_transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `350`
		//  Estimated: `7404`
		// Minimum execution time: 52_990_000 picoseconds.
		Weight::from_parts(56_258_000, 0)
			.saturating_add(Weight::from_parts(0, 7404))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: ForeignAssets Asset (r:1 w:0)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Account (r:1 w:1)
	/// Proof: ForeignAssets Account (max_values: None, max_size: Some(732), added: 3207, mode: MaxEncodedLen)
	fn freeze() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `350`
		//  Estimated: `4273`
		// Minimum execution time: 20_713_000 picoseconds.
		Weight::from_parts(34_730_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: ForeignAssets Asset (r:1 w:0)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Account (r:1 w:1)
	/// Proof: ForeignAssets Account (max_values: None, max_size: Some(732), added: 3207, mode: MaxEncodedLen)
	fn thaw() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `350`
		//  Estimated: `4273`
		// Minimum execution time: 21_035_000 picoseconds.
		Weight::from_parts(23_011_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	fn freeze_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `276`
		//  Estimated: `4273`
		// Minimum execution time: 15_926_000 picoseconds.
		Weight::from_parts(18_525_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	fn thaw_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `276`
		//  Estimated: `4273`
		// Minimum execution time: 15_974_000 picoseconds.
		Weight::from_parts(18_236_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Metadata (r:1 w:0)
	/// Proof: ForeignAssets Metadata (max_values: None, max_size: Some(738), added: 3213, mode: MaxEncodedLen)
	fn transfer_ownership() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `242`
		//  Estimated: `4273`
		// Minimum execution time: 18_380_000 picoseconds.
		Weight::from_parts(21_431_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	fn set_team() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `242`
		//  Estimated: `4273`
		// Minimum execution time: 16_903_000 picoseconds.
		Weight::from_parts(19_449_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: ForeignAssets Asset (r:1 w:0)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Metadata (r:1 w:1)
	/// Proof: ForeignAssets Metadata (max_values: None, max_size: Some(738), added: 3213, mode: MaxEncodedLen)
	/// The range of component `n` is `[0, 50]`.
	/// The range of component `s` is `[0, 50]`.
	/// The range of component `n` is `[0, 50]`.
	/// The range of component `s` is `[0, 50]`.
	fn set_metadata(n: u32, s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `242`
		//  Estimated: `4273`
		// Minimum execution time: 34_820_000 picoseconds.
		Weight::from_parts(34_178_250, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			// Standard Error: 48_050
			.saturating_add(Weight::from_parts(67_285, 0).saturating_mul(n.into()))
			// Standard Error: 48_050
			.saturating_add(Weight::from_parts(64_875, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: ForeignAssets Asset (r:1 w:0)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Metadata (r:1 w:1)
	/// Proof: ForeignAssets Metadata (max_values: None, max_size: Some(738), added: 3213, mode: MaxEncodedLen)
	fn clear_metadata() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `406`
		//  Estimated: `4273`
		// Minimum execution time: 35_129_000 picoseconds.
		Weight::from_parts(36_646_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: ForeignAssets Asset (r:1 w:0)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Metadata (r:1 w:1)
	/// Proof: ForeignAssets Metadata (max_values: None, max_size: Some(738), added: 3213, mode: MaxEncodedLen)
	/// The range of component `n` is `[0, 50]`.
	/// The range of component `s` is `[0, 50]`.
	/// The range of component `n` is `[0, 50]`.
	/// The range of component `s` is `[0, 50]`.
	fn force_set_metadata(n: u32, s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `81`
		//  Estimated: `4273`
		// Minimum execution time: 16_158_000 picoseconds.
		Weight::from_parts(17_126_250, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			// Standard Error: 14_938
			.saturating_add(Weight::from_parts(2_715, 0).saturating_mul(n.into()))
			// Standard Error: 14_938
			.saturating_add(Weight::from_parts(1_015, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: ForeignAssets Asset (r:1 w:0)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Metadata (r:1 w:1)
	/// Proof: ForeignAssets Metadata (max_values: None, max_size: Some(738), added: 3213, mode: MaxEncodedLen)
	fn force_clear_metadata() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `406`
		//  Estimated: `4273`
		// Minimum execution time: 35_109_000 picoseconds.
		Weight::from_parts(35_975_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	fn force_asset_status() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `242`
		//  Estimated: `4273`
		// Minimum execution time: 15_069_000 picoseconds.
		Weight::from_parts(16_850_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Approvals (r:1 w:1)
	/// Proof: ForeignAssets Approvals (max_values: None, max_size: Some(746), added: 3221, mode: MaxEncodedLen)
	fn approve_transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `276`
		//  Estimated: `4273`
		// Minimum execution time: 39_402_000 picoseconds.
		Weight::from_parts(41_160_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Approvals (r:1 w:1)
	/// Proof: ForeignAssets Approvals (max_values: None, max_size: Some(746), added: 3221, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Account (r:2 w:2)
	/// Proof: ForeignAssets Account (max_values: None, max_size: Some(732), added: 3207, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn transfer_approved() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `520`
		//  Estimated: `7404`
		// Minimum execution time: 76_315_000 picoseconds.
		Weight::from_parts(78_873_000, 0)
			.saturating_add(Weight::from_parts(0, 7404))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Approvals (r:1 w:1)
	/// Proof: ForeignAssets Approvals (max_values: None, max_size: Some(746), added: 3221, mode: MaxEncodedLen)
	fn cancel_approval() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `446`
		//  Estimated: `4273`
		// Minimum execution time: 41_786_000 picoseconds.
		Weight::from_parts(44_567_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Approvals (r:1 w:1)
	/// Proof: ForeignAssets Approvals (max_values: None, max_size: Some(746), added: 3221, mode: MaxEncodedLen)
	fn force_cancel_approval() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `446`
		//  Estimated: `4273`
		// Minimum execution time: 43_008_000 picoseconds.
		Weight::from_parts(45_273_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	fn set_min_balance() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `242`
		//  Estimated: `4273`
		// Minimum execution time: 16_607_000 picoseconds.
		Weight::from_parts(18_578_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: ForeignAssets Account (r:1 w:1)
	/// Proof: ForeignAssets Account (max_values: None, max_size: Some(732), added: 3207, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn touch() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `345`
		//  Estimated: `4273`
		// Minimum execution time: 40_765_000 picoseconds.
		Weight::from_parts(42_772_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: ForeignAssets Account (r:1 w:1)
	/// Proof: ForeignAssets Account (max_values: None, max_size: Some(732), added: 3207, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	fn touch_other() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `242`
		//  Estimated: `4273`
		// Minimum execution time: 38_835_000 picoseconds.
		Weight::from_parts(40_890_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: ForeignAssets Account (r:1 w:1)
	/// Proof: ForeignAssets Account (max_values: None, max_size: Some(732), added: 3207, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn refund() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `471`
		//  Estimated: `4273`
		// Minimum execution time: 37_138_000 picoseconds.
		Weight::from_parts(38_251_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: ForeignAssets Account (r:1 w:1)
	/// Proof: ForeignAssets Account (max_values: None, max_size: Some(732), added: 3207, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Asset (r:1 w:1)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	fn refund_other() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `401`
		//  Estimated: `4273`
		// Minimum execution time: 34_151_000 picoseconds.
		Weight::from_parts(36_837_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: ForeignAssets Asset (r:1 w:0)
	/// Proof: ForeignAssets Asset (max_values: None, max_size: Some(808), added: 3283, mode: MaxEncodedLen)
	/// Storage: ForeignAssets Account (r:1 w:1)
	/// Proof: ForeignAssets Account (max_values: None, max_size: Some(732), added: 3207, mode: MaxEncodedLen)
	fn block() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `350`
		//  Estimated: `4273`
		// Minimum execution time: 20_311_000 picoseconds.
		Weight::from_parts(23_004_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
