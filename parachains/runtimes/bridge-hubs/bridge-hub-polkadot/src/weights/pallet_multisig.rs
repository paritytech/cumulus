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
//! DATE: 2023-02-23, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `bm3`, CPU: `Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("bridge-hub-polkadot-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/polkadot-parachain
// benchmark
// pallet
// --chain=bridge-hub-polkadot-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_multisig
// --no-storage-info
// --no-median-slopes
// --no-min-squares
// --extrinsic=*
// --steps=50
// --repeat=20
// --json
// --header=./file_header.txt
// --output=./parachains/runtimes/bridge-hubs/bridge-hub-polkadot/src/weights/pallet_multisig.rs

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
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 11_727 nanoseconds.
		Weight::from_ref_time(12_260_035)
			.saturating_add(Weight::from_proof_size(0))
			// Standard Error: 4
			.saturating_add(Weight::from_ref_time(479).saturating_mul(z.into()))
	}
	/// Storage: Multisig Multisigs (r:1 w:1)
	/// Proof: Multisig Multisigs (max_values: None, max_size: Some(3346), added: 5821, mode: MaxEncodedLen)
	/// The range of component `s` is `[2, 100]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_create(s: u32, z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `314 + s * (2 ±0)`
		//  Estimated: `5821`
		// Minimum execution time: 35_341 nanoseconds.
		Weight::from_ref_time(30_373_960)
			.saturating_add(Weight::from_proof_size(5821))
			// Standard Error: 2_451
			.saturating_add(Weight::from_ref_time(66_083).saturating_mul(s.into()))
			// Standard Error: 24
			.saturating_add(Weight::from_ref_time(1_396).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Multisig Multisigs (r:1 w:1)
	/// Proof: Multisig Multisigs (max_values: None, max_size: Some(3346), added: 5821, mode: MaxEncodedLen)
	/// The range of component `s` is `[3, 100]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_approve(s: u32, z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `313`
		//  Estimated: `5821`
		// Minimum execution time: 26_382 nanoseconds.
		Weight::from_ref_time(20_382_119)
			.saturating_add(Weight::from_proof_size(5821))
			// Standard Error: 713
			.saturating_add(Weight::from_ref_time(67_344).saturating_mul(s.into()))
			// Standard Error: 6
			.saturating_add(Weight::from_ref_time(1_500).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Multisig Multisigs (r:1 w:1)
	/// Proof: Multisig Multisigs (max_values: None, max_size: Some(3346), added: 5821, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// The range of component `s` is `[2, 100]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_complete(s: u32, z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `451 + s * (33 ±0)`
		//  Estimated: `8424`
		// Minimum execution time: 39_804 nanoseconds.
		Weight::from_ref_time(32_281_318)
			.saturating_add(Weight::from_proof_size(8424))
			// Standard Error: 1_039
			.saturating_add(Weight::from_ref_time(84_946).saturating_mul(s.into()))
			// Standard Error: 10
			.saturating_add(Weight::from_ref_time(1_523).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Multisig Multisigs (r:1 w:1)
	/// Proof: Multisig Multisigs (max_values: None, max_size: Some(3346), added: 5821, mode: MaxEncodedLen)
	/// The range of component `s` is `[2, 100]`.
	fn approve_as_multi_create(s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `321 + s * (2 ±0)`
		//  Estimated: `5821`
		// Minimum execution time: 25_656 nanoseconds.
		Weight::from_ref_time(28_619_860)
			.saturating_add(Weight::from_proof_size(5821))
			// Standard Error: 3_488
			.saturating_add(Weight::from_ref_time(56_060).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Multisig Multisigs (r:1 w:1)
	/// Proof: Multisig Multisigs (max_values: None, max_size: Some(3346), added: 5821, mode: MaxEncodedLen)
	/// The range of component `s` is `[2, 100]`.
	fn approve_as_multi_approve(s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `313`
		//  Estimated: `5821`
		// Minimum execution time: 17_037 nanoseconds.
		Weight::from_ref_time(18_447_067)
			.saturating_add(Weight::from_proof_size(5821))
			// Standard Error: 2_101
			.saturating_add(Weight::from_ref_time(78_578).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Multisig Multisigs (r:1 w:1)
	/// Proof: Multisig Multisigs (max_values: None, max_size: Some(3346), added: 5821, mode: MaxEncodedLen)
	/// The range of component `s` is `[2, 100]`.
	fn cancel_as_multi(s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `517 + s * (1 ±0)`
		//  Estimated: `5821`
		// Minimum execution time: 26_423 nanoseconds.
		Weight::from_ref_time(28_078_539)
			.saturating_add(Weight::from_proof_size(5821))
			// Standard Error: 717
			.saturating_add(Weight::from_ref_time(80_919).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
