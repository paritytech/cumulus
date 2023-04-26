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
//! DATE: 2023-03-23, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// --pallet=pallet_nfts
// --extrinsic=*
// --steps=50
// --repeat=20
// --json
// --header=./file_header.txt
// --output=./parachains/runtimes/assets/statemine/src/weights/pallet_nfts.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

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
		//  Estimated: `5038`
		// Minimum execution time: 34_100_000 picoseconds.
		Weight::from_parts(34_649_000, 0)
			.saturating_add(Weight::from_parts(0, 5038))
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
		//  Estimated: `5038`
		// Minimum execution time: 22_415_000 picoseconds.
		Weight::from_parts(22_808_000, 0)
			.saturating_add(Weight::from_parts(0, 5038))
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
	fn destroy(m: u32, _c: u32, a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `32170 + a * (366 ±0)`
		//  Estimated: `2538829 + a * (2954 ±0)`
		// Minimum execution time: 978_514_000 picoseconds.
		Weight::from_parts(915_478_956, 0)
			.saturating_add(Weight::from_parts(0, 2538829))
			// Standard Error: 4_368
			.saturating_add(Weight::from_parts(3_621, 0).saturating_mul(m.into()))
			// Standard Error: 4_368
			.saturating_add(Weight::from_parts(5_742_436, 0).saturating_mul(a.into()))
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
		//  Estimated: `18460`
		// Minimum execution time: 44_509_000 picoseconds.
		Weight::from_parts(45_090_000, 0)
			.saturating_add(Weight::from_parts(0, 18460))
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
		//  Estimated: `18460`
		// Minimum execution time: 43_761_000 picoseconds.
		Weight::from_parts(44_304_000, 0)
			.saturating_add(Weight::from_parts(0, 18460))
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
		//  Estimated: `15200`
		// Minimum execution time: 45_215_000 picoseconds.
		Weight::from_parts(46_367_000, 0)
			.saturating_add(Weight::from_parts(0, 15200))
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
		//  Estimated: `14926`
		// Minimum execution time: 35_381_000 picoseconds.
		Weight::from_parts(35_896_000, 0)
			.saturating_add(Weight::from_parts(0, 14926))
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
		//  Estimated: `8077 + i * (3336 ±0)`
		// Minimum execution time: 16_621_000 picoseconds.
		Weight::from_parts(16_839_000, 0)
			.saturating_add(Weight::from_parts(0, 8077))
			// Standard Error: 13_184
			.saturating_add(Weight::from_parts(13_274_447, 0).saturating_mul(i.into()))
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
		//  Estimated: `7047`
		// Minimum execution time: 20_314_000 picoseconds.
		Weight::from_parts(20_726_000, 0)
			.saturating_add(Weight::from_parts(0, 7047))
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
		//  Estimated: `7047`
		// Minimum execution time: 20_178_000 picoseconds.
		Weight::from_parts(20_565_000, 0)
			.saturating_add(Weight::from_parts(0, 7047))
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
		//  Estimated: `7087`
		// Minimum execution time: 17_142_000 picoseconds.
		Weight::from_parts(18_191_000, 0)
			.saturating_add(Weight::from_parts(0, 7087))
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
		//  Estimated: `7066`
		// Minimum execution time: 22_902_000 picoseconds.
		Weight::from_parts(23_495_000, 0)
			.saturating_add(Weight::from_parts(0, 7066))
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
		//  Estimated: `9627`
		// Minimum execution time: 41_436_000 picoseconds.
		Weight::from_parts(41_922_000, 0)
			.saturating_add(Weight::from_parts(0, 9627))
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
		// Minimum execution time: 19_015_000 picoseconds.
		Weight::from_parts(19_490_000, 0)
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
		// Minimum execution time: 15_532_000 picoseconds.
		Weight::from_parts(15_827_000, 0)
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
		//  Estimated: `7047`
		// Minimum execution time: 21_022_000 picoseconds.
		Weight::from_parts(21_289_000, 0)
			.saturating_add(Weight::from_parts(0, 7047))
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
		//  Estimated: `18078`
		// Minimum execution time: 47_283_000 picoseconds.
		Weight::from_parts(47_793_000, 0)
			.saturating_add(Weight::from_parts(0, 18078))
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
		//  Estimated: `7493`
		// Minimum execution time: 27_462_000 picoseconds.
		Weight::from_parts(27_798_000, 0)
			.saturating_add(Weight::from_parts(0, 7493))
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
		//  Estimated: `14540`
		// Minimum execution time: 44_392_000 picoseconds.
		Weight::from_parts(44_956_000, 0)
			.saturating_add(Weight::from_parts(0, 14540))
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
		//  Estimated: `8792`
		// Minimum execution time: 18_619_000 picoseconds.
		Weight::from_parts(18_970_000, 0)
			.saturating_add(Weight::from_parts(0, 8792))
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
		//  Estimated: `16329 + n * (2954 ±0)`
		// Minimum execution time: 28_293_000 picoseconds.
		Weight::from_parts(28_502_000, 0)
			.saturating_add(Weight::from_parts(0, 16329))
			// Standard Error: 4_215
			.saturating_add(Weight::from_parts(5_601_603, 0).saturating_mul(n.into()))
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
		//  Estimated: `17946`
		// Minimum execution time: 39_371_000 picoseconds.
		Weight::from_parts(39_852_000, 0)
			.saturating_add(Weight::from_parts(0, 17946))
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
		//  Estimated: `14408`
		// Minimum execution time: 37_535_000 picoseconds.
		Weight::from_parts(38_894_000, 0)
			.saturating_add(Weight::from_parts(0, 14408))
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
		//  Estimated: `14380`
		// Minimum execution time: 35_608_000 picoseconds.
		Weight::from_parts(35_741_000, 0)
			.saturating_add(Weight::from_parts(0, 14380))
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
		//  Estimated: `14380`
		// Minimum execution time: 33_234_000 picoseconds.
		Weight::from_parts(33_617_000, 0)
			.saturating_add(Weight::from_parts(0, 14380))
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
		//  Estimated: `7864`
		// Minimum execution time: 22_900_000 picoseconds.
		Weight::from_parts(23_351_000, 0)
			.saturating_add(Weight::from_parts(0, 7864))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Nfts Item (r:1 w:1)
	/// Proof: Nfts Item (max_values: None, max_size: Some(861), added: 3336, mode: MaxEncodedLen)
	fn cancel_approval() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `384`
		//  Estimated: `4326`
		// Minimum execution time: 20_413_000 picoseconds.
		Weight::from_parts(20_622_000, 0)
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
		// Minimum execution time: 19_132_000 picoseconds.
		Weight::from_parts(19_443_000, 0)
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
		// Minimum execution time: 16_661_000 picoseconds.
		Weight::from_parts(16_925_000, 0)
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
		//  Estimated: `7087`
		// Minimum execution time: 19_575_000 picoseconds.
		Weight::from_parts(19_826_000, 0)
			.saturating_add(Weight::from_parts(0, 7087))
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
		//  Estimated: `7072`
		// Minimum execution time: 19_749_000 picoseconds.
		Weight::from_parts(19_902_000, 0)
			.saturating_add(Weight::from_parts(0, 7072))
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
		//  Estimated: `11377`
		// Minimum execution time: 23_970_000 picoseconds.
		Weight::from_parts(24_589_000, 0)
			.saturating_add(Weight::from_parts(0, 11377))
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
		//  Estimated: `18480`
		// Minimum execution time: 43_929_000 picoseconds.
		Weight::from_parts(44_364_000, 0)
			.saturating_add(Weight::from_parts(0, 18480))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	/// The range of component `n` is `[0, 10]`.
	fn pay_tips(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 2_611_000 picoseconds.
		Weight::from_parts(4_292_527, 0)
			.saturating_add(Weight::from_parts(0, 0))
			// Standard Error: 9_304
			.saturating_add(Weight::from_parts(3_636_886, 0).saturating_mul(n.into()))
	}
	/// Storage: Nfts Item (r:2 w:0)
	/// Proof: Nfts Item (max_values: None, max_size: Some(861), added: 3336, mode: MaxEncodedLen)
	/// Storage: Nfts PendingSwapOf (r:0 w:1)
	/// Proof: Nfts PendingSwapOf (max_values: None, max_size: Some(71), added: 2546, mode: MaxEncodedLen)
	fn create_swap() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `460`
		//  Estimated: `7662`
		// Minimum execution time: 22_643_000 picoseconds.
		Weight::from_parts(22_957_000, 0)
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
		//  Estimated: `7862`
		// Minimum execution time: 21_037_000 picoseconds.
		Weight::from_parts(21_359_000, 0)
			.saturating_add(Weight::from_parts(0, 7862))
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
		//  Estimated: `24321`
		// Minimum execution time: 72_434_000 picoseconds.
		Weight::from_parts(73_184_000, 0)
			.saturating_add(Weight::from_parts(0, 24321))
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
		//  Estimated: `29399 + n * (2954 ±0)`
		// Minimum execution time: 125_554_000 picoseconds.
		Weight::from_parts(129_631_978, 0)
			.saturating_add(Weight::from_parts(0, 29399))
			// Standard Error: 20_858
			.saturating_add(Weight::from_parts(26_871_088, 0).saturating_mul(n.into()))
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
		//  Estimated: `20462 + n * (2954 ±0)`
		// Minimum execution time: 76_170_000 picoseconds.
		Weight::from_parts(85_697_599, 0)
			.saturating_add(Weight::from_parts(0, 20462))
			// Standard Error: 51_480
			.saturating_add(Weight::from_parts(26_398_485, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(n.into())))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(n.into())))
			.saturating_add(Weight::from_parts(0, 2954).saturating_mul(n.into()))
	}
}
