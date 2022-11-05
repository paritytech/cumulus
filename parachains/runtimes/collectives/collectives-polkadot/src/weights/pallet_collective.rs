
//! Autogenerated weights for `pallet_collective`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-11-04, STEPS: `20`, REPEAT: 1, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `cob`, CPU: `<UNKNOWN>`
//! EXECUTION: None, WASM-EXECUTION: Compiled, CHAIN: Some("collectives-polkadot-dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/polkadot-parachain
// benchmark
// pallet
// --chain=collectives-polkadot-dev
// --steps=20
// --repeat=1
// --pallet=pallet_collective
// --extrinsic=*
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./parachains/runtimes/collectives/collectives-polkadot/src/weights/pallet_collective.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_collective`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_collective::WeightInfo for WeightInfo<T> {
	// Storage: AllianceMotion Members (r:1 w:1)
	// Storage: AllianceMotion Proposals (r:1 w:0)
	// Storage: AllianceMotion Prime (r:0 w:1)
	// Storage: AllianceMotion Voting (r:100 w:100)
	/// The range of component `m` is `[0, 100]`.
	/// The range of component `n` is `[0, 100]`.
	/// The range of component `p` is `[0, 100]`.
	fn set_members(m: u32, _n: u32, p: u32, ) -> Weight {
		// Minimum execution time: 14_000 nanoseconds.
		Weight::from_ref_time(14_000_000 as u64)
			// Standard Error: 199_241
			.saturating_add(Weight::from_ref_time(1_684_468 as u64).saturating_mul(m as u64))
			// Standard Error: 199_241
			.saturating_add(Weight::from_ref_time(3_940_701 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().reads((1 as u64).saturating_mul(p as u64)))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
			.saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(p as u64)))
	}
	// Storage: AllianceMotion Members (r:1 w:0)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn execute(b: u32, m: u32, ) -> Weight {
		// Minimum execution time: 17_000 nanoseconds.
		Weight::from_ref_time(16_643_798 as u64)
			// Standard Error: 2_622
			.saturating_add(Weight::from_ref_time(2_896 as u64).saturating_mul(b as u64))
			// Standard Error: 26_994
			.saturating_add(Weight::from_ref_time(15_081 as u64).saturating_mul(m as u64))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
	}
	// Storage: AllianceMotion Members (r:1 w:0)
	// Storage: AllianceMotion ProposalOf (r:1 w:0)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn propose_execute(b: u32, m: u32, ) -> Weight {
		// Minimum execution time: 18_000 nanoseconds.
		Weight::from_ref_time(19_485_559 as u64)
			// Standard Error: 2_752
			.saturating_add(Weight::from_ref_time(1_497 as u64).saturating_mul(b as u64))
			// Standard Error: 28_323
			.saturating_add(Weight::from_ref_time(3_190 as u64).saturating_mul(m as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
	}
	// Storage: AllianceMotion Members (r:1 w:0)
	// Storage: AllianceMotion ProposalOf (r:1 w:1)
	// Storage: AllianceMotion Proposals (r:1 w:1)
	// Storage: AllianceMotion ProposalCount (r:1 w:1)
	// Storage: AllianceMotion Voting (r:0 w:1)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[2, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn propose_proposed(b: u32, m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 21_000 nanoseconds.
		Weight::from_ref_time(25_273_683 as u64)
			// Standard Error: 1_539
			.saturating_add(Weight::from_ref_time(709 as u64).saturating_mul(b as u64))
			// Standard Error: 16_003
			.saturating_add(Weight::from_ref_time(9_535 as u64).saturating_mul(m as u64))
			// Standard Error: 15_844
			.saturating_add(Weight::from_ref_time(54_487 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: AllianceMotion Members (r:1 w:0)
	// Storage: AllianceMotion Voting (r:1 w:1)
	/// The range of component `m` is `[5, 100]`.
	fn vote(m: u32, ) -> Weight {
		// Minimum execution time: 20_000 nanoseconds.
		Weight::from_ref_time(20_957_894 as u64)
			// Standard Error: 11_970
			.saturating_add(Weight::from_ref_time(42_706 as u64).saturating_mul(m as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: AllianceMotion Voting (r:1 w:1)
	// Storage: AllianceMotion Members (r:1 w:0)
	// Storage: AllianceMotion Proposals (r:1 w:1)
	// Storage: AllianceMotion ProposalOf (r:0 w:1)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_disapproved(m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 25_000 nanoseconds.
		Weight::from_ref_time(25_615_237 as u64)
			// Standard Error: 8_271
			.saturating_add(Weight::from_ref_time(10_262 as u64).saturating_mul(m as u64))
			// Standard Error: 8_001
			.saturating_add(Weight::from_ref_time(58_408 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: AllianceMotion Voting (r:1 w:1)
	// Storage: AllianceMotion Members (r:1 w:0)
	// Storage: AllianceMotion ProposalOf (r:1 w:1)
	// Storage: AllianceMotion Proposals (r:1 w:1)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_approved(_b: u32, m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 34_000 nanoseconds.
		Weight::from_ref_time(35_090_450 as u64)
			// Standard Error: 24_312
			.saturating_add(Weight::from_ref_time(4_182 as u64).saturating_mul(m as u64))
			// Standard Error: 23_524
			.saturating_add(Weight::from_ref_time(80_145 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: AllianceMotion Voting (r:1 w:1)
	// Storage: AllianceMotion Members (r:1 w:0)
	// Storage: AllianceMotion Prime (r:1 w:0)
	// Storage: AllianceMotion Proposals (r:1 w:1)
	// Storage: AllianceMotion ProposalOf (r:0 w:1)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_disapproved(m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 26_000 nanoseconds.
		Weight::from_ref_time(26_620_699 as u64)
			// Standard Error: 17_432
			.saturating_add(Weight::from_ref_time(24_962 as u64).saturating_mul(m as u64))
			// Standard Error: 16_864
			.saturating_add(Weight::from_ref_time(61_816 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: AllianceMotion Voting (r:1 w:1)
	// Storage: AllianceMotion Members (r:1 w:0)
	// Storage: AllianceMotion Prime (r:1 w:0)
	// Storage: AllianceMotion ProposalOf (r:1 w:1)
	// Storage: AllianceMotion Proposals (r:1 w:1)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_approved(_b: u32, m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 34_000 nanoseconds.
		Weight::from_ref_time(35_643_124 as u64)
			// Standard Error: 16_696
			.saturating_add(Weight::from_ref_time(30_049 as u64).saturating_mul(m as u64))
			// Standard Error: 16_155
			.saturating_add(Weight::from_ref_time(68_833 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: AllianceMotion Proposals (r:1 w:1)
	// Storage: AllianceMotion Voting (r:0 w:1)
	// Storage: AllianceMotion ProposalOf (r:0 w:1)
	/// The range of component `p` is `[1, 100]`.
	fn disapprove_proposal(p: u32, ) -> Weight {
		// Minimum execution time: 18_000 nanoseconds.
		Weight::from_ref_time(18_621_969 as u64)
			// Standard Error: 8_814
			.saturating_add(Weight::from_ref_time(41_519 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
}
