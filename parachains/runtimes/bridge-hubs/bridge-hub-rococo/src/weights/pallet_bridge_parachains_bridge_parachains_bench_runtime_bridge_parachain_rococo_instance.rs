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

//! Autogenerated weights for `pallet_bridge_parachains`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-07-31, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `runner-ynta1nyy-project-238-concurrent-0`, CPU: `Intel(R) Xeon(R) CPU @ 2.60GHz`
//! EXECUTION: ``, WASM-EXECUTION: `Compiled`, CHAIN: `Some("bridge-hub-rococo-dev")`, DB CACHE: 1024

// Executed Command:
// ./target/production/polkadot-parachain
// benchmark
// pallet
// --chain=bridge-hub-rococo-dev
// --wasm-execution=compiled
// --pallet=pallet_bridge_parachains
// --no-storage-info
// --no-median-slopes
// --no-min-squares
// --extrinsic=*
// --steps=50
// --repeat=20
// --json
// --header=./file_header.txt
// --output=./parachains/runtimes/bridge-hubs/bridge-hub-rococo/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_bridge_parachains`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_bridge_parachains::WeightInfo for WeightInfo<T> {
	/// Storage: `BridgeRococoParachain::PalletOperatingMode` (r:1 w:0)
	/// Proof: `BridgeRococoParachain::PalletOperatingMode` (`max_values`: Some(1), `max_size`: Some(1), added: 496, mode: `MaxEncodedLen`)
	/// Storage: `BridgeRococoGrandpa::ImportedHeaders` (r:1 w:0)
	/// Proof: `BridgeRococoGrandpa::ImportedHeaders` (`max_values`: Some(1024), `max_size`: Some(68), added: 1553, mode: `MaxEncodedLen`)
	/// Storage: `BridgeRococoParachain::ParasInfo` (r:1 w:1)
	/// Proof: `BridgeRococoParachain::ParasInfo` (`max_values`: Some(1), `max_size`: Some(60), added: 555, mode: `MaxEncodedLen`)
	/// Storage: `BridgeRococoParachain::ImportedParaHashes` (r:1 w:1)
	/// Proof: `BridgeRococoParachain::ImportedParaHashes` (`max_values`: Some(64), `max_size`: Some(64), added: 1054, mode: `MaxEncodedLen`)
	/// Storage: `BridgeRococoParachain::ImportedParaHeads` (r:0 w:1)
	/// Proof: `BridgeRococoParachain::ImportedParaHeads` (`max_values`: Some(64), `max_size`: Some(196), added: 1186, mode: `MaxEncodedLen`)
	/// The range of component `p` is `[1, 2]`.
	/// The range of component `p` is `[1, 2]`.
	fn submit_parachain_heads_with_n_parachains(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `294`
		//  Estimated: `2543`
		// Minimum execution time: 33_519_000 picoseconds.
		Weight::from_parts(34_527_779, 0)
			.saturating_add(Weight::from_parts(0, 2543))
			// Standard Error: 45_887
			.saturating_add(Weight::from_parts(120_010, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `BridgeRococoParachain::PalletOperatingMode` (r:1 w:0)
	/// Proof: `BridgeRococoParachain::PalletOperatingMode` (`max_values`: Some(1), `max_size`: Some(1), added: 496, mode: `MaxEncodedLen`)
	/// Storage: `BridgeRococoGrandpa::ImportedHeaders` (r:1 w:0)
	/// Proof: `BridgeRococoGrandpa::ImportedHeaders` (`max_values`: Some(1024), `max_size`: Some(68), added: 1553, mode: `MaxEncodedLen`)
	/// Storage: `BridgeRococoParachain::ParasInfo` (r:1 w:1)
	/// Proof: `BridgeRococoParachain::ParasInfo` (`max_values`: Some(1), `max_size`: Some(60), added: 555, mode: `MaxEncodedLen`)
	/// Storage: `BridgeRococoParachain::ImportedParaHashes` (r:1 w:1)
	/// Proof: `BridgeRococoParachain::ImportedParaHashes` (`max_values`: Some(64), `max_size`: Some(64), added: 1054, mode: `MaxEncodedLen`)
	/// Storage: `BridgeRococoParachain::ImportedParaHeads` (r:0 w:1)
	/// Proof: `BridgeRococoParachain::ImportedParaHeads` (`max_values`: Some(64), `max_size`: Some(196), added: 1186, mode: `MaxEncodedLen`)
	fn submit_parachain_heads_with_1kb_proof() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `294`
		//  Estimated: `2543`
		// Minimum execution time: 35_140_000 picoseconds.
		Weight::from_parts(35_801_000, 0)
			.saturating_add(Weight::from_parts(0, 2543))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `BridgeRococoParachain::PalletOperatingMode` (r:1 w:0)
	/// Proof: `BridgeRococoParachain::PalletOperatingMode` (`max_values`: Some(1), `max_size`: Some(1), added: 496, mode: `MaxEncodedLen`)
	/// Storage: `BridgeRococoGrandpa::ImportedHeaders` (r:1 w:0)
	/// Proof: `BridgeRococoGrandpa::ImportedHeaders` (`max_values`: Some(1024), `max_size`: Some(68), added: 1553, mode: `MaxEncodedLen`)
	/// Storage: `BridgeRococoParachain::ParasInfo` (r:1 w:1)
	/// Proof: `BridgeRococoParachain::ParasInfo` (`max_values`: Some(1), `max_size`: Some(60), added: 555, mode: `MaxEncodedLen`)
	/// Storage: `BridgeRococoParachain::ImportedParaHashes` (r:1 w:1)
	/// Proof: `BridgeRococoParachain::ImportedParaHashes` (`max_values`: Some(64), `max_size`: Some(64), added: 1054, mode: `MaxEncodedLen`)
	/// Storage: `BridgeRococoParachain::ImportedParaHeads` (r:0 w:1)
	/// Proof: `BridgeRococoParachain::ImportedParaHeads` (`max_values`: Some(64), `max_size`: Some(196), added: 1186, mode: `MaxEncodedLen`)
	fn submit_parachain_heads_with_16kb_proof() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `294`
		//  Estimated: `2543`
		// Minimum execution time: 67_110_000 picoseconds.
		Weight::from_parts(67_951_000, 0)
			.saturating_add(Weight::from_parts(0, 2543))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
}
