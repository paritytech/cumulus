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

//! Autogenerated weights for `pallet_nfts`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-05-05, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `bm6`, CPU: `Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("statemine-dev"), DB CACHE: 1024

// Executed Command:
// ./artifacts/polkadot-parachain
// benchmark
// pallet
// --chain=statemint-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_nfts
// --extrinsic=*
// --steps=50
// --repeat=20
// --json
// --header=./file_header.txt
// --output=./parachains/runtimes/assets/asset-hub-polkadot/src/weights/pallet_nfts.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_nfts`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_nfts::WeightInfo for WeightInfo<T> {
	/// Storage: Nfts NextCollectionId (r:1 w:1)
	/// Proof: Nfts NextCollectionId (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Nfts Collection (r:1 w:1)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionRoleOf (r:0 w:1)
	/// Proof: Nfts CollectionRoleOf (max_values: None, max_size: Some(69), added: 2544, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionConfigOf (r:0 w:1)
	/// Proof: Nfts CollectionConfigOf (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionAccount (r:0 w:1)
	/// Proof: Nfts CollectionAccount (max_values: None, max_size: Some(68), added: 2543, mode: MaxEncodedLen)
	fn create() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `145`
		//  Estimated: `3549`
		// Minimum execution time: 39_589_000 picoseconds.
		Weight::from_parts(40_305_000, 0)
			.saturating_add(Weight::from_parts(0, 3549))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	/// Storage: Nfts NextCollectionId (r:1 w:1)
	/// Proof: Nfts NextCollectionId (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Nfts Collection (r:1 w:1)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionRoleOf (r:0 w:1)
	/// Proof: Nfts CollectionRoleOf (max_values: None, max_size: Some(69), added: 2544, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionConfigOf (r:0 w:1)
	/// Proof: Nfts CollectionConfigOf (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionAccount (r:0 w:1)
	/// Proof: Nfts CollectionAccount (max_values: None, max_size: Some(68), added: 2543, mode: MaxEncodedLen)
	fn force_create() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `42`
		//  Estimated: `3549`
		// Minimum execution time: 23_945_000 picoseconds.
		Weight::from_parts(24_351_000, 0)
			.saturating_add(Weight::from_parts(0, 3549))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	/// Storage: Nfts Collection (r:1 w:1)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts ItemMetadataOf (r:1 w:0)
	/// Proof: Nfts ItemMetadataOf (max_values: None, max_size: Some(347), added: 2822, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionRoleOf (r:1 w:1)
	/// Proof: Nfts CollectionRoleOf (max_values: None, max_size: Some(69), added: 2544, mode: MaxEncodedLen)
	/// Storage: Nfts Attribute (r:1001 w:1000)
	/// Proof: Nfts Attribute (max_values: None, max_size: Some(479), added: 2954, mode: MaxEncodedLen)
	/// Storage: Nfts ItemConfigOf (r:1000 w:1000)
	/// Proof: Nfts ItemConfigOf (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionMetadataOf (r:0 w:1)
	/// Proof: Nfts CollectionMetadataOf (max_values: None, max_size: Some(294), added: 2769, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionConfigOf (r:0 w:1)
	/// Proof: Nfts CollectionConfigOf (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionAccount (r:0 w:1)
	/// Proof: Nfts CollectionAccount (max_values: None, max_size: Some(68), added: 2543, mode: MaxEncodedLen)
	/// The range of component `m` is `[0, 1000]`.
	/// The range of component `c` is `[0, 1000]`.
	/// The range of component `a` is `[0, 1000]`.
	fn destroy(_m: u32, _c: u32, a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `32170 + a * (366 ±0)`
		//  Estimated: `2523990 + a * (2954 ±0)`
		// Minimum execution time: 992_533_000 picoseconds.
		Weight::from_parts(941_813_359, 0)
			.saturating_add(Weight::from_parts(0, 2523990))
			// Standard Error: 3_954
			.saturating_add(Weight::from_parts(5_784_754, 0).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(1004))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(a.into())))
			.saturating_add(T::DbWeight::get().writes(1005))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(a.into())))
			.saturating_add(Weight::from_parts(0, 2954).saturating_mul(a.into()))
	}
	/// Storage: Nfts CollectionConfigOf (r:1 w:0)
	/// Proof: Nfts CollectionConfigOf (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	/// Storage: Nfts Item (r:1 w:1)
	/// Proof: Nfts Item (max_values: None, max_size: Some(861), added: 3336, mode: MaxEncodedLen)
	/// Storage: Nfts Collection (r:1 w:1)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionRoleOf (r:1 w:0)
	/// Proof: Nfts CollectionRoleOf (max_values: None, max_size: Some(69), added: 2544, mode: MaxEncodedLen)
	/// Storage: Nfts ItemConfigOf (r:1 w:1)
	/// Proof: Nfts ItemConfigOf (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Nfts Account (r:0 w:1)
	/// Proof: Nfts Account (max_values: None, max_size: Some(88), added: 2563, mode: MaxEncodedLen)
	fn mint() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `421`
		//  Estimated: `4326`
		// Minimum execution time: 49_305_000 picoseconds.
		Weight::from_parts(50_143_000, 0)
			.saturating_add(Weight::from_parts(0, 4326))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: Nfts CollectionRoleOf (r:1 w:0)
	/// Proof: Nfts CollectionRoleOf (max_values: None, max_size: Some(69), added: 2544, mode: MaxEncodedLen)
	/// Storage: Nfts Item (r:1 w:1)
	/// Proof: Nfts Item (max_values: None, max_size: Some(861), added: 3336, mode: MaxEncodedLen)
	/// Storage: Nfts Collection (r:1 w:1)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionConfigOf (r:1 w:0)
	/// Proof: Nfts CollectionConfigOf (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	/// Storage: Nfts ItemConfigOf (r:1 w:1)
	/// Proof: Nfts ItemConfigOf (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Nfts Account (r:0 w:1)
	/// Proof: Nfts Account (max_values: None, max_size: Some(88), added: 2563, mode: MaxEncodedLen)
	fn force_mint() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `421`
		//  Estimated: `4326`
		// Minimum execution time: 48_627_000 picoseconds.
		Weight::from_parts(48_954_000, 0)
			.saturating_add(Weight::from_parts(0, 4326))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: Nfts ItemConfigOf (r:1 w:1)
	/// Proof: Nfts ItemConfigOf (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Nfts Collection (r:1 w:1)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts Item (r:1 w:1)
	/// Proof: Nfts Item (max_values: None, max_size: Some(861), added: 3336, mode: MaxEncodedLen)
	/// Storage: Nfts ItemMetadataOf (r:1 w:0)
	/// Proof: Nfts ItemMetadataOf (max_values: None, max_size: Some(347), added: 2822, mode: MaxEncodedLen)
	/// Storage: Nfts Account (r:0 w:1)
	/// Proof: Nfts Account (max_values: None, max_size: Some(88), added: 2563, mode: MaxEncodedLen)
	/// Storage: Nfts ItemPriceOf (r:0 w:1)
	/// Proof: Nfts ItemPriceOf (max_values: None, max_size: Some(89), added: 2564, mode: MaxEncodedLen)
	/// Storage: Nfts ItemAttributesApprovalsOf (r:0 w:1)
	/// Proof: Nfts ItemAttributesApprovalsOf (max_values: None, max_size: Some(1001), added: 3476, mode: MaxEncodedLen)
	/// Storage: Nfts PendingSwapOf (r:0 w:1)
	/// Proof: Nfts PendingSwapOf (max_values: None, max_size: Some(71), added: 2546, mode: MaxEncodedLen)
	fn burn() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `530`
		//  Estimated: `4326`
		// Minimum execution time: 49_958_000 picoseconds.
		Weight::from_parts(50_387_000, 0)
			.saturating_add(Weight::from_parts(0, 4326))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	/// Storage: Nfts Collection (r:1 w:0)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionConfigOf (r:1 w:0)
	/// Proof: Nfts CollectionConfigOf (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	/// Storage: Nfts ItemConfigOf (r:1 w:0)
	/// Proof: Nfts ItemConfigOf (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Nfts Item (r:1 w:1)
	/// Proof: Nfts Item (max_values: None, max_size: Some(861), added: 3336, mode: MaxEncodedLen)
	/// Storage: Nfts Account (r:0 w:2)
	/// Proof: Nfts Account (max_values: None, max_size: Some(88), added: 2563, mode: MaxEncodedLen)
	/// Storage: Nfts ItemPriceOf (r:0 w:1)
	/// Proof: Nfts ItemPriceOf (max_values: None, max_size: Some(89), added: 2564, mode: MaxEncodedLen)
	/// Storage: Nfts PendingSwapOf (r:0 w:1)
	/// Proof: Nfts PendingSwapOf (max_values: None, max_size: Some(71), added: 2546, mode: MaxEncodedLen)
	fn transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `559`
		//  Estimated: `4326`
		// Minimum execution time: 36_267_000 picoseconds.
		Weight::from_parts(36_712_000, 0)
			.saturating_add(Weight::from_parts(0, 4326))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	/// Storage: Nfts Collection (r:1 w:0)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionConfigOf (r:1 w:0)
	/// Proof: Nfts CollectionConfigOf (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	/// Storage: Nfts Item (r:5000 w:5000)
	/// Proof: Nfts Item (max_values: None, max_size: Some(861), added: 3336, mode: MaxEncodedLen)
	/// The range of component `i` is `[0, 5000]`.
	fn redeposit(i: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `729 + i * (108 ±0)`
		//  Estimated: `3549 + i * (3336 ±0)`
		// Minimum execution time: 17_738_000 picoseconds.
		Weight::from_parts(17_801_000, 0)
			.saturating_add(Weight::from_parts(0, 3549))
			// Standard Error: 13_596
			.saturating_add(Weight::from_parts(15_695_790, 0).saturating_mul(i.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(i.into())))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(i.into())))
			.saturating_add(Weight::from_parts(0, 3336).saturating_mul(i.into()))
	}
	/// Storage: Nfts CollectionRoleOf (r:1 w:0)
	/// Proof: Nfts CollectionRoleOf (max_values: None, max_size: Some(69), added: 2544, mode: MaxEncodedLen)
	/// Storage: Nfts ItemConfigOf (r:1 w:1)
	/// Proof: Nfts ItemConfigOf (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	fn lock_item_transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `401`
		//  Estimated: `3534`
		// Minimum execution time: 20_845_000 picoseconds.
		Weight::from_parts(21_133_000, 0)
			.saturating_add(Weight::from_parts(0, 3534))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Nfts CollectionRoleOf (r:1 w:0)
	/// Proof: Nfts CollectionRoleOf (max_values: None, max_size: Some(69), added: 2544, mode: MaxEncodedLen)
	/// Storage: Nfts ItemConfigOf (r:1 w:1)
	/// Proof: Nfts ItemConfigOf (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	fn unlock_item_transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `401`
		//  Estimated: `3534`
		// Minimum execution time: 20_862_000 picoseconds.
		Weight::from_parts(21_105_000, 0)
			.saturating_add(Weight::from_parts(0, 3534))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Nfts Collection (r:1 w:0)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionConfigOf (r:1 w:1)
	/// Proof: Nfts CollectionConfigOf (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	fn lock_collection() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `306`
		//  Estimated: `3549`
		// Minimum execution time: 18_196_000 picoseconds.
		Weight::from_parts(18_333_000, 0)
			.saturating_add(Weight::from_parts(0, 3549))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Nfts OwnershipAcceptance (r:1 w:1)
	/// Proof: Nfts OwnershipAcceptance (max_values: None, max_size: Some(52), added: 2527, mode: MaxEncodedLen)
	/// Storage: Nfts Collection (r:1 w:1)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionAccount (r:0 w:2)
	/// Proof: Nfts CollectionAccount (max_values: None, max_size: Some(68), added: 2543, mode: MaxEncodedLen)
	fn transfer_ownership() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `354`
		//  Estimated: `3549`
		// Minimum execution time: 24_025_000 picoseconds.
		Weight::from_parts(24_277_000, 0)
			.saturating_add(Weight::from_parts(0, 3549))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: Nfts Collection (r:1 w:1)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionRoleOf (r:2 w:4)
	/// Proof: Nfts CollectionRoleOf (max_values: None, max_size: Some(69), added: 2544, mode: MaxEncodedLen)
	fn set_team() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `335`
		//  Estimated: `6078`
		// Minimum execution time: 40_974_000 picoseconds.
		Weight::from_parts(41_706_000, 0)
			.saturating_add(Weight::from_parts(0, 6078))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	/// Storage: Nfts Collection (r:1 w:1)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionAccount (r:0 w:2)
	/// Proof: Nfts CollectionAccount (max_values: None, max_size: Some(68), added: 2543, mode: MaxEncodedLen)
	fn force_collection_owner() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `277`
		//  Estimated: `3549`
		// Minimum execution time: 19_044_000 picoseconds.
		Weight::from_parts(19_465_000, 0)
			.saturating_add(Weight::from_parts(0, 3549))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: Nfts Collection (r:1 w:0)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionConfigOf (r:0 w:1)
	/// Proof: Nfts CollectionConfigOf (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	fn force_collection_config() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `242`
		//  Estimated: `3549`
		// Minimum execution time: 15_591_000 picoseconds.
		Weight::from_parts(15_858_000, 0)
			.saturating_add(Weight::from_parts(0, 3549))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Nfts CollectionRoleOf (r:1 w:0)
	/// Proof: Nfts CollectionRoleOf (max_values: None, max_size: Some(69), added: 2544, mode: MaxEncodedLen)
	/// Storage: Nfts ItemConfigOf (r:1 w:1)
	/// Proof: Nfts ItemConfigOf (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	fn lock_item_properties() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `401`
		//  Estimated: `3534`
		// Minimum execution time: 20_831_000 picoseconds.
		Weight::from_parts(21_121_000, 0)
			.saturating_add(Weight::from_parts(0, 3534))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Nfts Collection (r:1 w:1)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionRoleOf (r:1 w:0)
	/// Proof: Nfts CollectionRoleOf (max_values: None, max_size: Some(69), added: 2544, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionConfigOf (r:1 w:0)
	/// Proof: Nfts CollectionConfigOf (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	/// Storage: Nfts ItemConfigOf (r:1 w:0)
	/// Proof: Nfts ItemConfigOf (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Nfts Attribute (r:1 w:1)
	/// Proof: Nfts Attribute (max_values: None, max_size: Some(479), added: 2954, mode: MaxEncodedLen)
	fn set_attribute() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `505`
		//  Estimated: `3944`
		// Minimum execution time: 50_650_000 picoseconds.
		Weight::from_parts(51_315_000, 0)
			.saturating_add(Weight::from_parts(0, 3944))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Nfts Collection (r:1 w:1)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts Attribute (r:1 w:1)
	/// Proof: Nfts Attribute (max_values: None, max_size: Some(479), added: 2954, mode: MaxEncodedLen)
	fn force_set_attribute() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `310`
		//  Estimated: `3944`
		// Minimum execution time: 28_244_000 picoseconds.
		Weight::from_parts(28_627_000, 0)
			.saturating_add(Weight::from_parts(0, 3944))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Nfts Attribute (r:1 w:1)
	/// Proof: Nfts Attribute (max_values: None, max_size: Some(479), added: 2954, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionRoleOf (r:1 w:0)
	/// Proof: Nfts CollectionRoleOf (max_values: None, max_size: Some(69), added: 2544, mode: MaxEncodedLen)
	/// Storage: Nfts ItemConfigOf (r:1 w:0)
	/// Proof: Nfts ItemConfigOf (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Nfts Collection (r:1 w:1)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	fn clear_attribute() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `949`
		//  Estimated: `3944`
		// Minimum execution time: 47_299_000 picoseconds.
		Weight::from_parts(47_921_000, 0)
			.saturating_add(Weight::from_parts(0, 3944))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Nfts Item (r:1 w:0)
	/// Proof: Nfts Item (max_values: None, max_size: Some(861), added: 3336, mode: MaxEncodedLen)
	/// Storage: Nfts ItemAttributesApprovalsOf (r:1 w:1)
	/// Proof: Nfts ItemAttributesApprovalsOf (max_values: None, max_size: Some(1001), added: 3476, mode: MaxEncodedLen)
	fn approve_item_attributes() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `347`
		//  Estimated: `4466`
		// Minimum execution time: 19_400_000 picoseconds.
		Weight::from_parts(19_601_000, 0)
			.saturating_add(Weight::from_parts(0, 4466))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Nfts Item (r:1 w:0)
	/// Proof: Nfts Item (max_values: None, max_size: Some(861), added: 3336, mode: MaxEncodedLen)
	/// Storage: Nfts ItemAttributesApprovalsOf (r:1 w:1)
	/// Proof: Nfts ItemAttributesApprovalsOf (max_values: None, max_size: Some(1001), added: 3476, mode: MaxEncodedLen)
	/// Storage: Nfts Attribute (r:1001 w:1000)
	/// Proof: Nfts Attribute (max_values: None, max_size: Some(479), added: 2954, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// The range of component `n` is `[0, 1000]`.
	fn cancel_item_attributes_approval(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `726 + n * (398 ±0)`
		//  Estimated: `4466 + n * (2954 ±0)`
		// Minimum execution time: 28_552_000 picoseconds.
		Weight::from_parts(28_822_000, 0)
			.saturating_add(Weight::from_parts(0, 4466))
			// Standard Error: 3_265
			.saturating_add(Weight::from_parts(5_570_824, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(n.into())))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(n.into())))
			.saturating_add(Weight::from_parts(0, 2954).saturating_mul(n.into()))
	}
	/// Storage: Nfts CollectionRoleOf (r:1 w:0)
	/// Proof: Nfts CollectionRoleOf (max_values: None, max_size: Some(69), added: 2544, mode: MaxEncodedLen)
	/// Storage: Nfts Collection (r:1 w:1)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts ItemConfigOf (r:1 w:0)
	/// Proof: Nfts ItemConfigOf (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionConfigOf (r:1 w:0)
	/// Proof: Nfts CollectionConfigOf (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	/// Storage: Nfts ItemMetadataOf (r:1 w:1)
	/// Proof: Nfts ItemMetadataOf (max_values: None, max_size: Some(347), added: 2822, mode: MaxEncodedLen)
	fn set_metadata() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `505`
		//  Estimated: `3812`
		// Minimum execution time: 42_425_000 picoseconds.
		Weight::from_parts(42_883_000, 0)
			.saturating_add(Weight::from_parts(0, 3812))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Nfts CollectionRoleOf (r:1 w:0)
	/// Proof: Nfts CollectionRoleOf (max_values: None, max_size: Some(69), added: 2544, mode: MaxEncodedLen)
	/// Storage: Nfts ItemMetadataOf (r:1 w:1)
	/// Proof: Nfts ItemMetadataOf (max_values: None, max_size: Some(347), added: 2822, mode: MaxEncodedLen)
	/// Storage: Nfts Collection (r:1 w:1)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts ItemConfigOf (r:1 w:0)
	/// Proof: Nfts ItemConfigOf (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	fn clear_metadata() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `815`
		//  Estimated: `3812`
		// Minimum execution time: 40_219_000 picoseconds.
		Weight::from_parts(41_709_000, 0)
			.saturating_add(Weight::from_parts(0, 3812))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Nfts CollectionRoleOf (r:1 w:0)
	/// Proof: Nfts CollectionRoleOf (max_values: None, max_size: Some(69), added: 2544, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionConfigOf (r:1 w:0)
	/// Proof: Nfts CollectionConfigOf (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	/// Storage: Nfts Collection (r:1 w:1)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionMetadataOf (r:1 w:1)
	/// Proof: Nfts CollectionMetadataOf (max_values: None, max_size: Some(294), added: 2769, mode: MaxEncodedLen)
	fn set_collection_metadata() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `364`
		//  Estimated: `3759`
		// Minimum execution time: 39_376_000 picoseconds.
		Weight::from_parts(39_895_000, 0)
			.saturating_add(Weight::from_parts(0, 3759))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Nfts CollectionRoleOf (r:1 w:0)
	/// Proof: Nfts CollectionRoleOf (max_values: None, max_size: Some(69), added: 2544, mode: MaxEncodedLen)
	/// Storage: Nfts Collection (r:1 w:0)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionConfigOf (r:1 w:0)
	/// Proof: Nfts CollectionConfigOf (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionMetadataOf (r:1 w:1)
	/// Proof: Nfts CollectionMetadataOf (max_values: None, max_size: Some(294), added: 2769, mode: MaxEncodedLen)
	fn clear_collection_metadata() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `682`
		//  Estimated: `3759`
		// Minimum execution time: 38_414_000 picoseconds.
		Weight::from_parts(38_627_000, 0)
			.saturating_add(Weight::from_parts(0, 3759))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Nfts Item (r:1 w:1)
	/// Proof: Nfts Item (max_values: None, max_size: Some(861), added: 3336, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionConfigOf (r:1 w:0)
	/// Proof: Nfts CollectionConfigOf (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	fn approve_transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `376`
		//  Estimated: `4326`
		// Minimum execution time: 22_896_000 picoseconds.
		Weight::from_parts(23_137_000, 0)
			.saturating_add(Weight::from_parts(0, 4326))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Nfts Item (r:1 w:1)
	/// Proof: Nfts Item (max_values: None, max_size: Some(861), added: 3336, mode: MaxEncodedLen)
	fn cancel_approval() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `384`
		//  Estimated: `4326`
		// Minimum execution time: 20_602_000 picoseconds.
		Weight::from_parts(20_869_000, 0)
			.saturating_add(Weight::from_parts(0, 4326))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Nfts Item (r:1 w:1)
	/// Proof: Nfts Item (max_values: None, max_size: Some(861), added: 3336, mode: MaxEncodedLen)
	fn clear_all_transfer_approvals() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `384`
		//  Estimated: `4326`
		// Minimum execution time: 19_415_000 picoseconds.
		Weight::from_parts(19_594_000, 0)
			.saturating_add(Weight::from_parts(0, 4326))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Nfts OwnershipAcceptance (r:1 w:1)
	/// Proof: Nfts OwnershipAcceptance (max_values: None, max_size: Some(52), added: 2527, mode: MaxEncodedLen)
	fn set_accept_ownership() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `42`
		//  Estimated: `3517`
		// Minimum execution time: 16_784_000 picoseconds.
		Weight::from_parts(17_133_000, 0)
			.saturating_add(Weight::from_parts(0, 3517))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Nfts CollectionConfigOf (r:1 w:1)
	/// Proof: Nfts CollectionConfigOf (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	/// Storage: Nfts Collection (r:1 w:0)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	fn set_collection_max_supply() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `306`
		//  Estimated: `3549`
		// Minimum execution time: 20_391_000 picoseconds.
		Weight::from_parts(20_710_000, 0)
			.saturating_add(Weight::from_parts(0, 3549))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Nfts CollectionRoleOf (r:1 w:0)
	/// Proof: Nfts CollectionRoleOf (max_values: None, max_size: Some(69), added: 2544, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionConfigOf (r:1 w:1)
	/// Proof: Nfts CollectionConfigOf (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	fn update_mint_settings() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `289`
		//  Estimated: `3538`
		// Minimum execution time: 19_989_000 picoseconds.
		Weight::from_parts(20_179_000, 0)
			.saturating_add(Weight::from_parts(0, 3538))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Nfts Item (r:1 w:0)
	/// Proof: Nfts Item (max_values: None, max_size: Some(861), added: 3336, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionConfigOf (r:1 w:0)
	/// Proof: Nfts CollectionConfigOf (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	/// Storage: Nfts ItemConfigOf (r:1 w:0)
	/// Proof: Nfts ItemConfigOf (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Nfts ItemPriceOf (r:0 w:1)
	/// Proof: Nfts ItemPriceOf (max_values: None, max_size: Some(89), added: 2564, mode: MaxEncodedLen)
	fn set_price() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `484`
		//  Estimated: `4326`
		// Minimum execution time: 24_308_000 picoseconds.
		Weight::from_parts(24_721_000, 0)
			.saturating_add(Weight::from_parts(0, 4326))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Nfts Item (r:1 w:1)
	/// Proof: Nfts Item (max_values: None, max_size: Some(861), added: 3336, mode: MaxEncodedLen)
	/// Storage: Nfts ItemPriceOf (r:1 w:1)
	/// Proof: Nfts ItemPriceOf (max_values: None, max_size: Some(89), added: 2564, mode: MaxEncodedLen)
	/// Storage: Nfts Collection (r:1 w:0)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionConfigOf (r:1 w:0)
	/// Proof: Nfts CollectionConfigOf (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	/// Storage: Nfts ItemConfigOf (r:1 w:0)
	/// Proof: Nfts ItemConfigOf (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Nfts Account (r:0 w:2)
	/// Proof: Nfts Account (max_values: None, max_size: Some(88), added: 2563, mode: MaxEncodedLen)
	/// Storage: Nfts PendingSwapOf (r:0 w:1)
	/// Proof: Nfts PendingSwapOf (max_values: None, max_size: Some(71), added: 2546, mode: MaxEncodedLen)
	fn buy_item() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `671`
		//  Estimated: `4326`
		// Minimum execution time: 45_626_000 picoseconds.
		Weight::from_parts(46_030_000, 0)
			.saturating_add(Weight::from_parts(0, 4326))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	/// The range of component `n` is `[0, 10]`.
	fn pay_tips(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 2_654_000 picoseconds.
		Weight::from_parts(4_301_940, 0)
			.saturating_add(Weight::from_parts(0, 0))
			// Standard Error: 9_223
			.saturating_add(Weight::from_parts(3_945_966, 0).saturating_mul(n.into()))
	}
	/// Storage: Nfts Item (r:2 w:0)
	/// Proof: Nfts Item (max_values: None, max_size: Some(861), added: 3336, mode: MaxEncodedLen)
	/// Storage: Nfts PendingSwapOf (r:0 w:1)
	/// Proof: Nfts PendingSwapOf (max_values: None, max_size: Some(71), added: 2546, mode: MaxEncodedLen)
	fn create_swap() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `460`
		//  Estimated: `7662`
		// Minimum execution time: 23_071_000 picoseconds.
		Weight::from_parts(23_535_000, 0)
			.saturating_add(Weight::from_parts(0, 7662))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Nfts PendingSwapOf (r:1 w:1)
	/// Proof: Nfts PendingSwapOf (max_values: None, max_size: Some(71), added: 2546, mode: MaxEncodedLen)
	/// Storage: Nfts Item (r:1 w:0)
	/// Proof: Nfts Item (max_values: None, max_size: Some(861), added: 3336, mode: MaxEncodedLen)
	fn cancel_swap() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `479`
		//  Estimated: `4326`
		// Minimum execution time: 21_554_000 picoseconds.
		Weight::from_parts(21_941_000, 0)
			.saturating_add(Weight::from_parts(0, 4326))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Nfts Item (r:2 w:2)
	/// Proof: Nfts Item (max_values: None, max_size: Some(861), added: 3336, mode: MaxEncodedLen)
	/// Storage: Nfts PendingSwapOf (r:1 w:2)
	/// Proof: Nfts PendingSwapOf (max_values: None, max_size: Some(71), added: 2546, mode: MaxEncodedLen)
	/// Storage: Nfts Collection (r:1 w:0)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionConfigOf (r:1 w:0)
	/// Proof: Nfts CollectionConfigOf (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	/// Storage: Nfts ItemConfigOf (r:2 w:0)
	/// Proof: Nfts ItemConfigOf (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: Nfts Account (r:0 w:4)
	/// Proof: Nfts Account (max_values: None, max_size: Some(88), added: 2563, mode: MaxEncodedLen)
	/// Storage: Nfts ItemPriceOf (r:0 w:2)
	/// Proof: Nfts ItemPriceOf (max_values: None, max_size: Some(89), added: 2564, mode: MaxEncodedLen)
	fn claim_swap() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `800`
		//  Estimated: `7662`
		// Minimum execution time: 74_272_000 picoseconds.
		Weight::from_parts(75_374_000, 0)
			.saturating_add(Weight::from_parts(0, 7662))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(10))
	}
	/// Storage: Nfts CollectionRoleOf (r:2 w:0)
	/// Proof: Nfts CollectionRoleOf (max_values: None, max_size: Some(69), added: 2544, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionConfigOf (r:1 w:0)
	/// Proof: Nfts CollectionConfigOf (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	/// Storage: Nfts Item (r:1 w:1)
	/// Proof: Nfts Item (max_values: None, max_size: Some(861), added: 3336, mode: MaxEncodedLen)
	/// Storage: Nfts Collection (r:1 w:1)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts ItemConfigOf (r:1 w:1)
	/// Proof: Nfts ItemConfigOf (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: Nfts Attribute (r:10 w:10)
	/// Proof: Nfts Attribute (max_values: None, max_size: Some(479), added: 2954, mode: MaxEncodedLen)
	/// Storage: Nfts ItemMetadataOf (r:1 w:1)
	/// Proof: Nfts ItemMetadataOf (max_values: None, max_size: Some(347), added: 2822, mode: MaxEncodedLen)
	/// Storage: Nfts Account (r:0 w:1)
	/// Proof: Nfts Account (max_values: None, max_size: Some(88), added: 2563, mode: MaxEncodedLen)
	/// The range of component `n` is `[0, 10]`.
	fn mint_pre_signed(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `524`
		//  Estimated: `6078 + n * (2954 ±0)`
		// Minimum execution time: 133_545_000 picoseconds.
		Weight::from_parts(137_797_962, 0)
			.saturating_add(Weight::from_parts(0, 6078))
			// Standard Error: 33_124
			.saturating_add(Weight::from_parts(29_785_862, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(n.into())))
			.saturating_add(T::DbWeight::get().writes(6))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(n.into())))
			.saturating_add(Weight::from_parts(0, 2954).saturating_mul(n.into()))
	}
	/// Storage: Nfts Item (r:1 w:0)
	/// Proof: Nfts Item (max_values: None, max_size: Some(861), added: 3336, mode: MaxEncodedLen)
	/// Storage: Nfts ItemAttributesApprovalsOf (r:1 w:1)
	/// Proof: Nfts ItemAttributesApprovalsOf (max_values: None, max_size: Some(1001), added: 3476, mode: MaxEncodedLen)
	/// Storage: Nfts CollectionConfigOf (r:1 w:0)
	/// Proof: Nfts CollectionConfigOf (max_values: None, max_size: Some(73), added: 2548, mode: MaxEncodedLen)
	/// Storage: Nfts Collection (r:1 w:1)
	/// Proof: Nfts Collection (max_values: None, max_size: Some(84), added: 2559, mode: MaxEncodedLen)
	/// Storage: Nfts Attribute (r:10 w:10)
	/// Proof: Nfts Attribute (max_values: None, max_size: Some(479), added: 2954, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// The range of component `n` is `[0, 10]`.
	fn set_attributes_pre_signed(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `554`
		//  Estimated: `4466 + n * (2954 ±0)`
		// Minimum execution time: 77_475_000 picoseconds.
		Weight::from_parts(88_353_947, 0)
			.saturating_add(Weight::from_parts(0, 4466))
			// Standard Error: 60_491
			.saturating_add(Weight::from_parts(29_507_037, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(n.into())))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(n.into())))
			.saturating_add(Weight::from_parts(0, 2954).saturating_mul(n.into()))
	}
}