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
//! DATE: 2023-06-19, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `bm3`, CPU: `Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("asset-hub-kusama-dev"), DB CACHE: 1024

// Executed Command:
// target/production/polkadot-parachain
// benchmark
// pallet
// --steps=50
// --repeat=20
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --json-file=/var/lib/gitlab-runner/builds/zyw4fam_/0/parity/mirrors/cumulus/.git/.artifacts/bench.json
// --pallet=pallet_assets
// --chain=asset-hub-kusama-dev
// --header=./file_header.txt
// --output=./parachains/runtimes/assets/asset-hub-kusama/src/weights/

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
		// Minimum execution time: 31_332_000 picoseconds.
		Weight::from_parts(31_795_000, 0)
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
		// Minimum execution time: 13_419_000 picoseconds.
		Weight::from_parts(13_849_000, 0)
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
		// Minimum execution time: 15_917_000 picoseconds.
		Weight::from_parts(16_378_000, 0)
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
		//  Measured:  `0 + c * (208 ±0)`
		//  Estimated: `4273 + c * (3207 ±0)`
		// Minimum execution time: 19_319_000 picoseconds.
		Weight::from_parts(19_539_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			// Standard Error: 6_789
			.saturating_add(Weight::from_parts(12_067_134, 0).saturating_mul(c.into()))
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
		//  Measured:  `413 + a * (86 ±0)`
		//  Estimated: `4273 + a * (3221 ±0)`
		// Minimum execution time: 20_676_000 picoseconds.
		Weight::from_parts(20_831_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			// Standard Error: 3_434
			.saturating_add(Weight::from_parts(13_968_086, 0).saturating_mul(a.into()))
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
		// Minimum execution time: 16_160_000 picoseconds.
		Weight::from_parts(16_516_000, 0)
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
		// Minimum execution time: 27_656_000 picoseconds.
		Weight::from_parts(28_025_000, 0)
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
		// Minimum execution time: 33_830_000 picoseconds.
		Weight::from_parts(34_918_000, 0)
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
		// Minimum execution time: 45_780_000 picoseconds.
		Weight::from_parts(46_441_000, 0)
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
		// Minimum execution time: 40_920_000 picoseconds.
		Weight::from_parts(41_274_000, 0)
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
		// Minimum execution time: 46_034_000 picoseconds.
		Weight::from_parts(46_648_000, 0)
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
		// Minimum execution time: 19_383_000 picoseconds.
		Weight::from_parts(19_768_000, 0)
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
		// Minimum execution time: 19_245_000 picoseconds.
		Weight::from_parts(19_668_000, 0)
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
		// Minimum execution time: 15_991_000 picoseconds.
		Weight::from_parts(16_359_000, 0)
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
		// Minimum execution time: 15_749_000 picoseconds.
		Weight::from_parts(16_129_000, 0)
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
		// Minimum execution time: 17_299_000 picoseconds.
		Weight::from_parts(17_645_000, 0)
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
		// Minimum execution time: 15_564_000 picoseconds.
		Weight::from_parts(15_815_000, 0)
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
		// Minimum execution time: 30_451_000 picoseconds.
		Weight::from_parts(30_994_269, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			// Standard Error: 246
			.saturating_add(Weight::from_parts(1_329, 0).saturating_mul(n.into()))
			// Standard Error: 246
			.saturating_add(Weight::from_parts(2_803, 0).saturating_mul(s.into()))
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
		// Minimum execution time: 31_002_000 picoseconds.
		Weight::from_parts(31_584_000, 0)
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
	fn force_set_metadata(_n: u32, _s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `81`
		//  Estimated: `4273`
		// Minimum execution time: 14_540_000 picoseconds.
		Weight::from_parts(15_190_542, 0)
			.saturating_add(Weight::from_parts(0, 4273))
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
		// Minimum execution time: 29_925_000 picoseconds.
		Weight::from_parts(30_405_000, 0)
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
		// Minimum execution time: 14_522_000 picoseconds.
		Weight::from_parts(14_740_000, 0)
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
		// Minimum execution time: 33_704_000 picoseconds.
		Weight::from_parts(34_186_000, 0)
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
		// Minimum execution time: 64_650_000 picoseconds.
		Weight::from_parts(65_291_000, 0)
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
		// Minimum execution time: 35_832_000 picoseconds.
		Weight::from_parts(36_468_000, 0)
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
		// Minimum execution time: 36_699_000 picoseconds.
		Weight::from_parts(37_187_000, 0)
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
		// Minimum execution time: 16_524_000 picoseconds.
		Weight::from_parts(16_952_000, 0)
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
		// Minimum execution time: 35_747_000 picoseconds.
		Weight::from_parts(36_196_000, 0)
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
		// Minimum execution time: 33_921_000 picoseconds.
		Weight::from_parts(34_210_000, 0)
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
		// Minimum execution time: 32_240_000 picoseconds.
		Weight::from_parts(32_607_000, 0)
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
		// Minimum execution time: 30_307_000 picoseconds.
		Weight::from_parts(30_855_000, 0)
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
		// Minimum execution time: 19_206_000 picoseconds.
		Weight::from_parts(19_582_000, 0)
			.saturating_add(Weight::from_parts(0, 4273))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
