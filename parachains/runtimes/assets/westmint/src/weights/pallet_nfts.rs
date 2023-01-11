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

//! Autogenerated weights for `pallet_nfts`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-01-04, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `Jegors-MBP.lan`, CPU: `<UNKNOWN>`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("westmint-dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/polkadot-parachain
// benchmark
// pallet
// --chain=westmint-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_nfts
// --extrinsic=*
// --steps=50
// --repeat=20
// --json
// --header=./file_header.txt
// --output=./parachains/runtimes/assets/westmint/src/weights/pallet_nfts.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_nfts`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_nfts::WeightInfo for WeightInfo<T> {
	// Storage: Nfts NextCollectionId (r:1 w:1)
	// Storage: Nfts Collection (r:1 w:1)
	// Storage: Nfts CollectionRoleOf (r:0 w:1)
	// Storage: Nfts CollectionConfigOf (r:0 w:1)
	// Storage: Nfts CollectionAccount (r:0 w:1)
	fn create() -> Weight {
		// Minimum execution time: 37_000 nanoseconds.
		Weight::from_ref_time(38_000_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	// Storage: Nfts NextCollectionId (r:1 w:1)
	// Storage: Nfts Collection (r:1 w:1)
	// Storage: Nfts CollectionRoleOf (r:0 w:1)
	// Storage: Nfts CollectionConfigOf (r:0 w:1)
	// Storage: Nfts CollectionAccount (r:0 w:1)
	fn force_create() -> Weight {
		// Minimum execution time: 26_000 nanoseconds.
		Weight::from_ref_time(26_000_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	// Storage: Nfts Collection (r:1 w:1)
	// Storage: Nfts Item (r:1001 w:1000)
	// Storage: Nfts Attribute (r:1001 w:1000)
	// Storage: Nfts ItemMetadataOf (r:0 w:1000)
	// Storage: Nfts CollectionRoleOf (r:0 w:1)
	// Storage: Nfts CollectionMetadataOf (r:0 w:1)
	// Storage: Nfts CollectionConfigOf (r:0 w:1)
	// Storage: Nfts ItemConfigOf (r:0 w:1000)
	// Storage: Nfts Account (r:0 w:1000)
	// Storage: Nfts CollectionAccount (r:0 w:1)
	/// The range of component `n` is `[0, 1000]`.
	/// The range of component `m` is `[0, 1000]`.
	/// The range of component `a` is `[0, 1000]`.
	fn destroy(n: u32, m: u32, a: u32, ) -> Weight {
		// Minimum execution time: 21_464_000 nanoseconds.
		Weight::from_ref_time(17_877_474_846)
			// Standard Error: 113_501
			.saturating_add(Weight::from_ref_time(1_677_234).saturating_mul(n.into()))
			// Standard Error: 113_501
			.saturating_add(Weight::from_ref_time(2_499_409).saturating_mul(m.into()))
			// Standard Error: 113_501
			.saturating_add(Weight::from_ref_time(9_962_607).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(1003))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(a.into())))
			.saturating_add(T::DbWeight::get().writes(3005))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(m.into())))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(a.into())))
	}
	// Storage: Nfts CollectionConfigOf (r:1 w:0)
	// Storage: Nfts Item (r:1 w:1)
	// Storage: Nfts Collection (r:1 w:1)
	// Storage: Nfts CollectionRoleOf (r:1 w:0)
	// Storage: Nfts ItemConfigOf (r:1 w:1)
	// Storage: Nfts Account (r:0 w:1)
	fn mint() -> Weight {
		// Minimum execution time: 47_000 nanoseconds.
		Weight::from_ref_time(51_000_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: Nfts CollectionRoleOf (r:1 w:0)
	// Storage: Nfts Item (r:1 w:1)
	// Storage: Nfts Collection (r:1 w:1)
	// Storage: Nfts CollectionConfigOf (r:1 w:0)
	// Storage: Nfts ItemConfigOf (r:1 w:1)
	// Storage: Nfts Account (r:0 w:1)
	fn force_mint() -> Weight {
		// Minimum execution time: 47_000 nanoseconds.
		Weight::from_ref_time(49_000_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: Nfts Collection (r:1 w:1)
	// Storage: Nfts Item (r:1 w:1)
	// Storage: Nfts CollectionRoleOf (r:1 w:0)
	// Storage: Nfts ItemConfigOf (r:1 w:1)
	// Storage: Nfts Account (r:0 w:1)
	// Storage: Nfts ItemPriceOf (r:0 w:1)
	// Storage: Nfts ItemAttributesApprovalsOf (r:0 w:1)
	// Storage: Nfts PendingSwapOf (r:0 w:1)
	fn burn() -> Weight {
		// Minimum execution time: 49_000 nanoseconds.
		Weight::from_ref_time(52_000_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	// Storage: Nfts Collection (r:1 w:0)
	// Storage: Nfts CollectionConfigOf (r:1 w:0)
	// Storage: Nfts ItemConfigOf (r:1 w:0)
	// Storage: Nfts Item (r:1 w:1)
	// Storage: Nfts CollectionRoleOf (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: Nfts Account (r:0 w:2)
	// Storage: Nfts ItemPriceOf (r:0 w:1)
	// Storage: Nfts PendingSwapOf (r:0 w:1)
	fn transfer() -> Weight {
		// Minimum execution time: 55_000 nanoseconds.
		Weight::from_ref_time(56_000_000)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	// Storage: Nfts Collection (r:1 w:0)
	// Storage: Nfts CollectionConfigOf (r:1 w:0)
	// Storage: Nfts Item (r:102 w:102)
	/// The range of component `i` is `[0, 5000]`.
	fn redeposit(i: u32, ) -> Weight {
		// Minimum execution time: 19_000 nanoseconds.
		Weight::from_ref_time(20_000_000)
			// Standard Error: 31_499
			.saturating_add(Weight::from_ref_time(14_469_164).saturating_mul(i.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(i.into())))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(i.into())))
	}
	// Storage: Nfts CollectionRoleOf (r:1 w:0)
	// Storage: Nfts ItemConfigOf (r:1 w:1)
	fn lock_item_transfer() -> Weight {
		// Minimum execution time: 22_000 nanoseconds.
		Weight::from_ref_time(23_000_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Nfts CollectionRoleOf (r:1 w:0)
	// Storage: Nfts ItemConfigOf (r:1 w:1)
	fn unlock_item_transfer() -> Weight {
		// Minimum execution time: 22_000 nanoseconds.
		Weight::from_ref_time(22_000_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Nfts CollectionRoleOf (r:1 w:0)
	// Storage: Nfts CollectionConfigOf (r:1 w:1)
	fn lock_collection() -> Weight {
		// Minimum execution time: 20_000 nanoseconds.
		Weight::from_ref_time(21_000_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Nfts OwnershipAcceptance (r:1 w:1)
	// Storage: Nfts Collection (r:1 w:1)
	// Storage: Nfts CollectionAccount (r:0 w:2)
	fn transfer_ownership() -> Weight {
		// Minimum execution time: 25_000 nanoseconds.
		Weight::from_ref_time(26_000_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: Nfts Collection (r:1 w:1)
	// Storage: Nfts CollectionRoleOf (r:0 w:4)
	fn set_team() -> Weight {
		// Minimum execution time: 27_000 nanoseconds.
		Weight::from_ref_time(28_000_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	// Storage: Nfts Collection (r:1 w:1)
	// Storage: Nfts CollectionAccount (r:0 w:2)
	fn force_collection_owner() -> Weight {
		// Minimum execution time: 20_000 nanoseconds.
		Weight::from_ref_time(21_000_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Nfts Collection (r:1 w:0)
	// Storage: Nfts CollectionConfigOf (r:0 w:1)
	fn force_collection_config() -> Weight {
		// Minimum execution time: 17_000 nanoseconds.
		Weight::from_ref_time(18_000_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Nfts Collection (r:1 w:0)
	// Storage: Nfts ItemConfigOf (r:1 w:1)
	fn lock_item_properties() -> Weight {
		// Minimum execution time: 20_000 nanoseconds.
		Weight::from_ref_time(22_000_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Nfts Collection (r:1 w:1)
	// Storage: Nfts CollectionConfigOf (r:1 w:0)
	// Storage: Nfts ItemConfigOf (r:1 w:0)
	// Storage: Nfts Attribute (r:1 w:1)
	fn set_attribute() -> Weight {
		// Minimum execution time: 45_000 nanoseconds.
		Weight::from_ref_time(46_000_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Nfts Collection (r:1 w:1)
	// Storage: Nfts Attribute (r:1 w:1)
	fn force_set_attribute() -> Weight {
		// Minimum execution time: 27_000 nanoseconds.
		Weight::from_ref_time(28_000_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Nfts Attribute (r:1 w:1)
	// Storage: Nfts Collection (r:1 w:1)
	// Storage: Nfts ItemConfigOf (r:1 w:0)
	fn clear_attribute() -> Weight {
		// Minimum execution time: 40_000 nanoseconds.
		Weight::from_ref_time(42_000_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Nfts Item (r:1 w:0)
	// Storage: Nfts ItemAttributesApprovalsOf (r:1 w:1)
	fn approve_item_attributes() -> Weight {
		// Minimum execution time: 21_000 nanoseconds.
		Weight::from_ref_time(21_000_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Nfts Item (r:1 w:0)
	// Storage: Nfts ItemAttributesApprovalsOf (r:1 w:1)
	// Storage: Nfts Attribute (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	/// The range of component `n` is `[0, 1000]`.
	fn cancel_item_attributes_approval(n: u32, ) -> Weight {
		// Minimum execution time: 28_000 nanoseconds.
		Weight::from_ref_time(29_000_000)
			// Standard Error: 28_180
			.saturating_add(Weight::from_ref_time(8_101_928).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(n.into())))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(n.into())))
	}
	// Storage: Nfts Collection (r:1 w:1)
	// Storage: Nfts ItemConfigOf (r:1 w:0)
	// Storage: Nfts CollectionConfigOf (r:1 w:0)
	// Storage: Nfts ItemMetadataOf (r:1 w:1)
	fn set_metadata() -> Weight {
		// Minimum execution time: 39_000 nanoseconds.
		Weight::from_ref_time(40_000_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Nfts Collection (r:1 w:1)
	// Storage: Nfts ItemConfigOf (r:1 w:0)
	// Storage: Nfts ItemMetadataOf (r:1 w:1)
	fn clear_metadata() -> Weight {
		// Minimum execution time: 37_000 nanoseconds.
		Weight::from_ref_time(38_000_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Nfts CollectionConfigOf (r:1 w:0)
	// Storage: Nfts Collection (r:1 w:1)
	// Storage: Nfts CollectionMetadataOf (r:1 w:1)
	fn set_collection_metadata() -> Weight {
		// Minimum execution time: 34_000 nanoseconds.
		Weight::from_ref_time(36_000_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Nfts Collection (r:1 w:0)
	// Storage: Nfts CollectionConfigOf (r:1 w:0)
	// Storage: Nfts CollectionMetadataOf (r:1 w:1)
	fn clear_collection_metadata() -> Weight {
		// Minimum execution time: 34_000 nanoseconds.
		Weight::from_ref_time(36_000_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Nfts Item (r:1 w:1)
	// Storage: Nfts CollectionConfigOf (r:1 w:0)
	// Storage: Nfts CollectionRoleOf (r:1 w:0)
	fn approve_transfer() -> Weight {
		// Minimum execution time: 26_000 nanoseconds.
		Weight::from_ref_time(27_000_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Nfts Item (r:1 w:1)
	// Storage: Nfts CollectionRoleOf (r:1 w:0)
	fn cancel_approval() -> Weight {
		// Minimum execution time: 24_000 nanoseconds.
		Weight::from_ref_time(25_000_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Nfts Item (r:1 w:1)
	// Storage: Nfts CollectionRoleOf (r:1 w:0)
	fn clear_all_transfer_approvals() -> Weight {
		// Minimum execution time: 24_000 nanoseconds.
		Weight::from_ref_time(25_000_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Nfts OwnershipAcceptance (r:1 w:1)
	fn set_accept_ownership() -> Weight {
		// Minimum execution time: 20_000 nanoseconds.
		Weight::from_ref_time(22_000_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Nfts CollectionConfigOf (r:1 w:1)
	// Storage: Nfts Collection (r:1 w:0)
	fn set_collection_max_supply() -> Weight {
		// Minimum execution time: 22_000 nanoseconds.
		Weight::from_ref_time(23_000_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Nfts Collection (r:1 w:0)
	// Storage: Nfts CollectionConfigOf (r:1 w:1)
	fn update_mint_settings() -> Weight {
		// Minimum execution time: 20_000 nanoseconds.
		Weight::from_ref_time(21_000_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Nfts Item (r:1 w:0)
	// Storage: Nfts CollectionConfigOf (r:1 w:0)
	// Storage: Nfts ItemConfigOf (r:1 w:0)
	// Storage: Nfts ItemPriceOf (r:0 w:1)
	fn set_price() -> Weight {
		// Minimum execution time: 25_000 nanoseconds.
		Weight::from_ref_time(26_000_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Nfts Item (r:1 w:1)
	// Storage: Nfts ItemPriceOf (r:1 w:1)
	// Storage: Nfts Collection (r:1 w:0)
	// Storage: Nfts CollectionConfigOf (r:1 w:0)
	// Storage: Nfts ItemConfigOf (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: Nfts Account (r:0 w:2)
	// Storage: Nfts PendingSwapOf (r:0 w:1)
	fn buy_item() -> Weight {
		// Minimum execution time: 60_000 nanoseconds.
		Weight::from_ref_time(62_000_000)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	/// The range of component `n` is `[0, 10]`.
	fn pay_tips(n: u32, ) -> Weight {
		// Minimum execution time: 3_000 nanoseconds.
		Weight::from_ref_time(7_424_228)
			// Standard Error: 21_679
			.saturating_add(Weight::from_ref_time(4_176_055).saturating_mul(n.into()))
	}
	// Storage: Nfts Item (r:2 w:0)
	// Storage: Nfts PendingSwapOf (r:0 w:1)
	fn create_swap() -> Weight {
		// Minimum execution time: 23_000 nanoseconds.
		Weight::from_ref_time(24_000_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Nfts PendingSwapOf (r:1 w:1)
	// Storage: Nfts Item (r:1 w:0)
	fn cancel_swap() -> Weight {
		// Minimum execution time: 26_000 nanoseconds.
		Weight::from_ref_time(27_000_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Nfts Item (r:2 w:2)
	// Storage: Nfts PendingSwapOf (r:1 w:2)
	// Storage: Nfts Collection (r:1 w:0)
	// Storage: Nfts CollectionConfigOf (r:1 w:0)
	// Storage: Nfts ItemConfigOf (r:2 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: Nfts Account (r:0 w:4)
	// Storage: Nfts ItemPriceOf (r:0 w:2)
	fn claim_swap() -> Weight {
		// Minimum execution time: 87_000 nanoseconds.
		Weight::from_ref_time(90_000_000)
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(11))
	}
}
