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

//! Autogenerated weights for `pallet_bridge_relayers`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-05-05, STEPS: `2`, REPEAT: `2`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `bkontur-ThinkPad-P14s-Gen-2i`, CPU: `11th Gen Intel(R) Core(TM) i7-1185G7 @ 3.00GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("bridge-hub-kusama-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/polkadot-parachain
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
// --chain=bridge-hub-kusama-dev
// --pallet=pallet_bridge_relayers
// --output=./parachains/runtimes/bridge-hubs/bridge-hub-kusama/src/weights

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_bridge_relayers`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_bridge_relayers::WeightInfo for WeightInfo<T> {
	/// Storage: BridgeRelayers RelayerRewards (r:1 w:1)
	/// Proof: BridgeRelayers RelayerRewards (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn claim_rewards() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `207`
		//  Estimated: `3593`
		// Minimum execution time: 52_874_000 picoseconds.
		Weight::from_parts(72_930_000, 0)
			.saturating_add(Weight::from_parts(0, 3593))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: BridgeRelayers RegisteredRelayers (r:1 w:1)
	/// Proof: BridgeRelayers RegisteredRelayers (max_values: None, max_size: Some(68), added: 2543, mode: MaxEncodedLen)
	/// Storage: unknown `0x1e8445dc201eeb8560e5579a5dd54655` (r:1 w:0)
	/// Proof Skipped: unknown `0x1e8445dc201eeb8560e5579a5dd54655` (r:1 w:0)
	/// Storage: Balances Reserves (r:1 w:1)
	/// Proof: Balances Reserves (max_values: None, max_size: Some(1249), added: 3724, mode: MaxEncodedLen)
	fn register() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `61`
		//  Estimated: `4714`
		// Minimum execution time: 28_476_000 picoseconds.
		Weight::from_parts(32_338_000, 0)
			.saturating_add(Weight::from_parts(0, 4714))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: BridgeRelayers RegisteredRelayers (r:1 w:1)
	/// Proof: BridgeRelayers RegisteredRelayers (max_values: None, max_size: Some(68), added: 2543, mode: MaxEncodedLen)
	/// Storage: Balances Reserves (r:1 w:1)
	/// Proof: Balances Reserves (max_values: None, max_size: Some(1249), added: 3724, mode: MaxEncodedLen)
	fn deregister() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `160`
		//  Estimated: `4714`
		// Minimum execution time: 28_815_000 picoseconds.
		Weight::from_parts(30_180_000, 0)
			.saturating_add(Weight::from_parts(0, 4714))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: BridgeRelayers RegisteredRelayers (r:1 w:1)
	/// Proof: BridgeRelayers RegisteredRelayers (max_values: None, max_size: Some(68), added: 2543, mode: MaxEncodedLen)
	/// Storage: Balances Reserves (r:1 w:1)
	/// Proof: Balances Reserves (max_values: None, max_size: Some(1249), added: 3724, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn slash_and_deregister() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `263`
		//  Estimated: `4714`
		// Minimum execution time: 29_426_000 picoseconds.
		Weight::from_parts(31_393_000, 0)
			.saturating_add(Weight::from_parts(0, 4714))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: BridgeRelayers RelayerRewards (r:1 w:1)
	/// Proof: BridgeRelayers RelayerRewards (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	fn register_relayer_reward() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `6`
		//  Estimated: `3538`
		// Minimum execution time: 3_035_000 picoseconds.
		Weight::from_parts(3_572_000, 0)
			.saturating_add(Weight::from_parts(0, 3538))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
