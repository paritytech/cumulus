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

//! Autogenerated weights for `pallet_multisig`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-10-19, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `bm6`, CPU: `Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("collectives-polkadot-dev"), DB CACHE: 1024

// Executed Command:
// ./artifacts/polkadot-parachain
// benchmark
// pallet
// --chain=collectives-polkadot-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_multisig
// --extrinsic=*
// --steps=50
// --repeat=20
// --json
// --header=./file_header.txt
// --output=./parachains/runtimes/collectives/collectives-polkadot/src/weights/pallet_multisig.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_multisig`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_multisig::WeightInfo for WeightInfo<T> {
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_threshold_1(z: u32, ) -> Weight {
		Weight::from_ref_time(21_577_000 as u64)
			// Standard Error: 1
			.saturating_add(Weight::from_ref_time(608 as u64).saturating_mul(z as u64))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	// Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
	/// The range of component `s` is `[2, 100]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_create(s: u32, z: u32, ) -> Weight {
		Weight::from_ref_time(49_593_000 as u64)
			// Standard Error: 1_054
			.saturating_add(Weight::from_ref_time(46_806 as u64).saturating_mul(s as u64))
			// Standard Error: 10
			.saturating_add(Weight::from_ref_time(655 as u64).saturating_mul(z as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	// Storage: Multisig Calls (r:1 w:1)
	// Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
	/// The range of component `s` is `[2, 100]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_create_store(s: u32, z: u32, ) -> Weight {
		Weight::from_ref_time(51_558_000 as u64)
			// Standard Error: 989
			.saturating_add(Weight::from_ref_time(49_648 as u64).saturating_mul(s as u64))
			// Standard Error: 9
			.saturating_add(Weight::from_ref_time(1_264 as u64).saturating_mul(z as u64))
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	/// The range of component `s` is `[3, 100]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_approve(s: u32, z: u32, ) -> Weight {
		Weight::from_ref_time(40_183_000 as u64)
			// Standard Error: 1_202
			.saturating_add(Weight::from_ref_time(38_576 as u64).saturating_mul(s as u64))
			// Standard Error: 12
			.saturating_add(Weight::from_ref_time(589 as u64).saturating_mul(z as u64))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	// Storage: Multisig Calls (r:1 w:1)
	/// The range of component `s` is `[3, 100]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_approve_store(s: u32, z: u32, ) -> Weight {
		Weight::from_ref_time(53_388_000 as u64)
			// Standard Error: 1_267
			.saturating_add(Weight::from_ref_time(44_099 as u64).saturating_mul(s as u64))
			// Standard Error: 12
			.saturating_add(Weight::from_ref_time(1_184 as u64).saturating_mul(z as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	// Storage: Multisig Calls (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	/// The range of component `s` is `[2, 100]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_complete(s: u32, z: u32, ) -> Weight {
		Weight::from_ref_time(66_476_000 as u64)
			// Standard Error: 1_250
			.saturating_add(Weight::from_ref_time(55_437 as u64).saturating_mul(s as u64))
			// Standard Error: 12
			.saturating_add(Weight::from_ref_time(1_217 as u64).saturating_mul(z as u64))
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	// Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
	/// The range of component `s` is `[2, 100]`.
	fn approve_as_multi_create(s: u32, ) -> Weight {
		Weight::from_ref_time(35_267_000 as u64)
			// Standard Error: 1_012
			.saturating_add(Weight::from_ref_time(154_096 as u64).saturating_mul(s as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	// Storage: Multisig Calls (r:1 w:0)
	/// The range of component `s` is `[2, 100]`.
	fn approve_as_multi_approve(s: u32, ) -> Weight {
		Weight::from_ref_time(26_171_000 as u64)
			// Standard Error: 408
			.saturating_add(Weight::from_ref_time(138_829 as u64).saturating_mul(s as u64))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	// Storage: Multisig Calls (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	/// The range of component `s` is `[2, 100]`.
	fn approve_as_multi_complete(s: u32, ) -> Weight {
		Weight::from_ref_time(63_602_000 as u64)
			// Standard Error: 1_154
			.saturating_add(Weight::from_ref_time(182_841 as u64).saturating_mul(s as u64))
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	// Storage: Multisig Calls (r:1 w:1)
	/// The range of component `s` is `[2, 100]`.
	fn cancel_as_multi(s: u32, ) -> Weight {
		Weight::from_ref_time(52_177_000 as u64)
			// Standard Error: 1_268
			.saturating_add(Weight::from_ref_time(159_603 as u64).saturating_mul(s as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
}
