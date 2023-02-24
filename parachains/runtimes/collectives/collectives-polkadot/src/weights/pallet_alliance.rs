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

//! Autogenerated weights for `pallet_alliance`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-02-24, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `bm6`, CPU: `Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("collectives-polkadot-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/polkadot-parachain
// benchmark
// pallet
// --chain=collectives-polkadot-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_alliance
// --no-storage-info
// --no-median-slopes
// --no-min-squares
// --extrinsic=*
// --steps=50
// --repeat=20
// --json
// --header=./file_header.txt
// --output=./parachains/runtimes/collectives/collectives-polkadot/src/weights/pallet_alliance.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_alliance`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_alliance::WeightInfo for WeightInfo<T> {
	/// Storage: Alliance Members (r:1 w:0)
	/// Proof: Alliance Members (max_values: None, max_size: Some(3211), added: 5686, mode: MaxEncodedLen)
	/// Storage: AllianceMotion ProposalOf (r:1 w:1)
	/// Proof Skipped: AllianceMotion ProposalOf (max_values: None, max_size: None, mode: Measured)
	/// Storage: AllianceMotion Proposals (r:1 w:1)
	/// Proof Skipped: AllianceMotion Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: AllianceMotion ProposalCount (r:1 w:1)
	/// Proof Skipped: AllianceMotion ProposalCount (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: AllianceMotion Voting (r:0 w:1)
	/// Proof Skipped: AllianceMotion Voting (max_values: None, max_size: None, mode: Measured)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[2, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn propose_proposed(b: u32, m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `502 + m * (32 ±0) + p * (36 ±0)`
		//  Estimated: `10983 + m * (128 ±0) + p * (144 ±0)`
		// Minimum execution time: 26_818 nanoseconds.
		Weight::from_ref_time(28_910_195)
			.saturating_add(Weight::from_proof_size(10983))
			// Standard Error: 69
			.saturating_add(Weight::from_ref_time(260).saturating_mul(b.into()))
			// Standard Error: 727
			.saturating_add(Weight::from_ref_time(17_637).saturating_mul(m.into()))
			// Standard Error: 718
			.saturating_add(Weight::from_ref_time(86_764).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
			.saturating_add(Weight::from_proof_size(128).saturating_mul(m.into()))
			.saturating_add(Weight::from_proof_size(144).saturating_mul(p.into()))
	}
	/// Storage: Alliance Members (r:1 w:0)
	/// Proof: Alliance Members (max_values: None, max_size: Some(3211), added: 5686, mode: MaxEncodedLen)
	/// Storage: AllianceMotion Voting (r:1 w:1)
	/// Proof Skipped: AllianceMotion Voting (max_values: None, max_size: None, mode: Measured)
	/// The range of component `m` is `[5, 100]`.
	fn vote(m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `932 + m * (64 ±0)`
		//  Estimated: `9092 + m * (64 ±0)`
		// Minimum execution time: 23_096 nanoseconds.
		Weight::from_ref_time(23_809_814)
			.saturating_add(Weight::from_proof_size(9092))
			// Standard Error: 499
			.saturating_add(Weight::from_ref_time(42_842).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(Weight::from_proof_size(64).saturating_mul(m.into()))
	}
	/// Storage: Alliance Members (r:1 w:0)
	/// Proof: Alliance Members (max_values: None, max_size: Some(3211), added: 5686, mode: MaxEncodedLen)
	/// Storage: AllianceMotion Voting (r:1 w:1)
	/// Proof Skipped: AllianceMotion Voting (max_values: None, max_size: None, mode: Measured)
	/// Storage: AllianceMotion Members (r:1 w:0)
	/// Proof Skipped: AllianceMotion Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: AllianceMotion Proposals (r:1 w:1)
	/// Proof Skipped: AllianceMotion Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: AllianceMotion ProposalOf (r:0 w:1)
	/// Proof Skipped: AllianceMotion ProposalOf (max_values: None, max_size: None, mode: Measured)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_disapproved(m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `438 + m * (96 ±0) + p * (36 ±0)`
		//  Estimated: `11067 + m * (388 ±0) + p * (144 ±0)`
		// Minimum execution time: 33_026 nanoseconds.
		Weight::from_ref_time(30_778_569)
			.saturating_add(Weight::from_proof_size(11067))
			// Standard Error: 559
			.saturating_add(Weight::from_ref_time(42_382).saturating_mul(m.into()))
			// Standard Error: 545
			.saturating_add(Weight::from_ref_time(80_327).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_proof_size(388).saturating_mul(m.into()))
			.saturating_add(Weight::from_proof_size(144).saturating_mul(p.into()))
	}
	/// Storage: Alliance Members (r:1 w:0)
	/// Proof: Alliance Members (max_values: None, max_size: Some(3211), added: 5686, mode: MaxEncodedLen)
	/// Storage: AllianceMotion Voting (r:1 w:1)
	/// Proof Skipped: AllianceMotion Voting (max_values: None, max_size: None, mode: Measured)
	/// Storage: AllianceMotion Members (r:1 w:0)
	/// Proof Skipped: AllianceMotion Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: AllianceMotion ProposalOf (r:1 w:1)
	/// Proof Skipped: AllianceMotion ProposalOf (max_values: None, max_size: None, mode: Measured)
	/// Storage: AllianceMotion Proposals (r:1 w:1)
	/// Proof Skipped: AllianceMotion Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_approved(_b: u32, m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `923 + m * (96 ±0) + p * (41 ±0)`
		//  Estimated: `15422 + m * (388 ±0) + p * (160 ±0)`
		// Minimum execution time: 42_776 nanoseconds.
		Weight::from_ref_time(42_634_632)
			.saturating_add(Weight::from_proof_size(15422))
			// Standard Error: 1_837
			.saturating_add(Weight::from_ref_time(54_576).saturating_mul(m.into()))
			// Standard Error: 1_790
			.saturating_add(Weight::from_ref_time(92_042).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_proof_size(388).saturating_mul(m.into()))
			.saturating_add(Weight::from_proof_size(160).saturating_mul(p.into()))
	}
	/// Storage: Alliance Members (r:1 w:0)
	/// Proof: Alliance Members (max_values: None, max_size: Some(3211), added: 5686, mode: MaxEncodedLen)
	/// Storage: AllianceMotion Voting (r:1 w:1)
	/// Proof Skipped: AllianceMotion Voting (max_values: None, max_size: None, mode: Measured)
	/// Storage: AllianceMotion Members (r:1 w:0)
	/// Proof Skipped: AllianceMotion Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: AllianceMotion Prime (r:1 w:0)
	/// Proof Skipped: AllianceMotion Prime (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: AllianceMotion ProposalOf (r:1 w:1)
	/// Proof Skipped: AllianceMotion ProposalOf (max_values: None, max_size: None, mode: Measured)
	/// Storage: AllianceMotion Proposals (r:1 w:1)
	/// Proof Skipped: AllianceMotion Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Alliance Rule (r:0 w:1)
	/// Proof: Alliance Rule (max_values: Some(1), max_size: Some(87), added: 582, mode: MaxEncodedLen)
	/// The range of component `m` is `[2, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_disapproved(m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `677 + m * (96 ±0) + p * (41 ±0)`
		//  Estimated: `14232 + m * (507 ±0) + p * (202 ±0)`
		// Minimum execution time: 39_952 nanoseconds.
		Weight::from_ref_time(42_118_417)
			.saturating_add(Weight::from_proof_size(14232))
			// Standard Error: 3_385
			.saturating_add(Weight::from_ref_time(97_927).saturating_mul(m.into()))
			// Standard Error: 3_343
			.saturating_add(Weight::from_ref_time(107_913).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(4))
			.saturating_add(Weight::from_proof_size(507).saturating_mul(m.into()))
			.saturating_add(Weight::from_proof_size(202).saturating_mul(p.into()))
	}
	/// Storage: Alliance Members (r:1 w:0)
	/// Proof: Alliance Members (max_values: None, max_size: Some(3211), added: 5686, mode: MaxEncodedLen)
	/// Storage: AllianceMotion Voting (r:1 w:1)
	/// Proof Skipped: AllianceMotion Voting (max_values: None, max_size: None, mode: Measured)
	/// Storage: AllianceMotion Members (r:1 w:0)
	/// Proof Skipped: AllianceMotion Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: AllianceMotion Prime (r:1 w:0)
	/// Proof Skipped: AllianceMotion Prime (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: AllianceMotion Proposals (r:1 w:1)
	/// Proof Skipped: AllianceMotion Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: AllianceMotion ProposalOf (r:0 w:1)
	/// Proof Skipped: AllianceMotion ProposalOf (max_values: None, max_size: None, mode: Measured)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[5, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_approved(_b: u32, m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `545 + m * (96 ±0) + p * (36 ±0)`
		//  Estimated: `12436 + m * (480 ±0) + p * (180 ±0)`
		// Minimum execution time: 34_010 nanoseconds.
		Weight::from_ref_time(32_643_642)
			.saturating_add(Weight::from_proof_size(12436))
			// Standard Error: 642
			.saturating_add(Weight::from_ref_time(41_276).saturating_mul(m.into()))
			// Standard Error: 619
			.saturating_add(Weight::from_ref_time(79_639).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_proof_size(480).saturating_mul(m.into()))
			.saturating_add(Weight::from_proof_size(180).saturating_mul(p.into()))
	}
	/// Storage: Alliance Members (r:2 w:2)
	/// Proof: Alliance Members (max_values: None, max_size: Some(3211), added: 5686, mode: MaxEncodedLen)
	/// Storage: AllianceMotion Members (r:1 w:1)
	/// Proof Skipped: AllianceMotion Members (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `m` is `[1, 100]`.
	/// The range of component `z` is `[0, 100]`.
	fn init_members(m: u32, z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `12`
		//  Estimated: `11879`
		// Minimum execution time: 27_583 nanoseconds.
		Weight::from_ref_time(17_006_319)
			.saturating_add(Weight::from_proof_size(11879))
			// Standard Error: 527
			.saturating_add(Weight::from_ref_time(125_728).saturating_mul(m.into()))
			// Standard Error: 521
			.saturating_add(Weight::from_ref_time(108_294).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: Alliance Members (r:2 w:2)
	/// Proof: Alliance Members (max_values: None, max_size: Some(3211), added: 5686, mode: MaxEncodedLen)
	/// Storage: AllianceMotion Proposals (r:1 w:0)
	/// Proof Skipped: AllianceMotion Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Alliance DepositOf (r:200 w:50)
	/// Proof: Alliance DepositOf (max_values: None, max_size: Some(64), added: 2539, mode: MaxEncodedLen)
	/// Storage: System Account (r:50 w:50)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: AllianceMotion Members (r:0 w:1)
	/// Proof Skipped: AllianceMotion Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: AllianceMotion Prime (r:0 w:1)
	/// Proof Skipped: AllianceMotion Prime (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `x` is `[1, 100]`.
	/// The range of component `y` is `[0, 100]`.
	/// The range of component `z` is `[0, 50]`.
	fn disband(x: u32, y: u32, z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0 + x * (52 ±0) + y * (53 ±0) + z * (282 ±0)`
		//  Estimated: `31580 + x * (2590 ±0) + y * (2590 ±0) + z * (3200 ±1)`
		// Minimum execution time: 224_479 nanoseconds.
		Weight::from_ref_time(224_779_000)
			.saturating_add(Weight::from_proof_size(31580))
			// Standard Error: 18_186
			.saturating_add(Weight::from_ref_time(437_934).saturating_mul(x.into()))
			// Standard Error: 18_099
			.saturating_add(Weight::from_ref_time(444_305).saturating_mul(y.into()))
			// Standard Error: 36_165
			.saturating_add(Weight::from_ref_time(9_610_862).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(x.into())))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(y.into())))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(z.into())))
			.saturating_add(T::DbWeight::get().writes(4))
			.saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(z.into())))
			.saturating_add(Weight::from_proof_size(2590).saturating_mul(x.into()))
			.saturating_add(Weight::from_proof_size(2590).saturating_mul(y.into()))
			.saturating_add(Weight::from_proof_size(3200).saturating_mul(z.into()))
	}
	/// Storage: Alliance Rule (r:0 w:1)
	/// Proof: Alliance Rule (max_values: Some(1), max_size: Some(87), added: 582, mode: MaxEncodedLen)
	fn set_rule() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_075 nanoseconds.
		Weight::from_ref_time(8_307_000)
			.saturating_add(Weight::from_proof_size(0))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Alliance Announcements (r:1 w:1)
	/// Proof: Alliance Announcements (max_values: Some(1), max_size: Some(8702), added: 9197, mode: MaxEncodedLen)
	fn announce() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `76`
		//  Estimated: `9197`
		// Minimum execution time: 10_765 nanoseconds.
		Weight::from_ref_time(10_979_000)
			.saturating_add(Weight::from_proof_size(9197))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Alliance Announcements (r:1 w:1)
	/// Proof: Alliance Announcements (max_values: Some(1), max_size: Some(8702), added: 9197, mode: MaxEncodedLen)
	fn remove_announcement() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `181`
		//  Estimated: `9197`
		// Minimum execution time: 11_486 nanoseconds.
		Weight::from_ref_time(11_842_000)
			.saturating_add(Weight::from_proof_size(9197))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Alliance Members (r:3 w:1)
	/// Proof: Alliance Members (max_values: None, max_size: Some(3211), added: 5686, mode: MaxEncodedLen)
	/// Storage: Alliance UnscrupulousAccounts (r:1 w:0)
	/// Proof: Alliance UnscrupulousAccounts (max_values: Some(1), max_size: Some(3202), added: 3697, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: Alliance DepositOf (r:0 w:1)
	/// Proof: Alliance DepositOf (max_values: None, max_size: Some(64), added: 2539, mode: MaxEncodedLen)
	fn join_alliance() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `388`
		//  Estimated: `23358`
		// Minimum execution time: 38_930 nanoseconds.
		Weight::from_ref_time(39_402_000)
			.saturating_add(Weight::from_proof_size(23358))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: Alliance Members (r:3 w:1)
	/// Proof: Alliance Members (max_values: None, max_size: Some(3211), added: 5686, mode: MaxEncodedLen)
	/// Storage: Alliance UnscrupulousAccounts (r:1 w:0)
	/// Proof: Alliance UnscrupulousAccounts (max_values: Some(1), max_size: Some(3202), added: 3697, mode: MaxEncodedLen)
	fn nominate_ally() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `255`
		//  Estimated: `20755`
		// Minimum execution time: 27_830 nanoseconds.
		Weight::from_ref_time(28_321_000)
			.saturating_add(Weight::from_proof_size(20755))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Alliance Members (r:2 w:2)
	/// Proof: Alliance Members (max_values: None, max_size: Some(3211), added: 5686, mode: MaxEncodedLen)
	/// Storage: AllianceMotion Proposals (r:1 w:0)
	/// Proof Skipped: AllianceMotion Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: AllianceMotion Members (r:0 w:1)
	/// Proof Skipped: AllianceMotion Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: AllianceMotion Prime (r:0 w:1)
	/// Proof Skipped: AllianceMotion Prime (max_values: Some(1), max_size: None, mode: Measured)
	fn elevate_ally() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `298`
		//  Estimated: `12761`
		// Minimum execution time: 23_160 nanoseconds.
		Weight::from_ref_time(23_622_000)
			.saturating_add(Weight::from_proof_size(12761))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: Alliance Members (r:4 w:2)
	/// Proof: Alliance Members (max_values: None, max_size: Some(3211), added: 5686, mode: MaxEncodedLen)
	/// Storage: AllianceMotion Proposals (r:1 w:0)
	/// Proof Skipped: AllianceMotion Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: AllianceMotion Members (r:0 w:1)
	/// Proof Skipped: AllianceMotion Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: AllianceMotion Prime (r:0 w:1)
	/// Proof Skipped: AllianceMotion Prime (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Alliance RetiringMembers (r:0 w:1)
	/// Proof: Alliance RetiringMembers (max_values: None, max_size: Some(52), added: 2527, mode: MaxEncodedLen)
	fn give_retirement_notice() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `298`
		//  Estimated: `24133`
		// Minimum execution time: 31_000 nanoseconds.
		Weight::from_ref_time(31_845_000)
			.saturating_add(Weight::from_proof_size(24133))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	/// Storage: Alliance RetiringMembers (r:1 w:1)
	/// Proof: Alliance RetiringMembers (max_values: None, max_size: Some(52), added: 2527, mode: MaxEncodedLen)
	/// Storage: Alliance Members (r:1 w:1)
	/// Proof: Alliance Members (max_values: None, max_size: Some(3211), added: 5686, mode: MaxEncodedLen)
	/// Storage: Alliance DepositOf (r:1 w:1)
	/// Proof: Alliance DepositOf (max_values: None, max_size: Some(64), added: 2539, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn retire() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `580`
		//  Estimated: `13355`
		// Minimum execution time: 32_769 nanoseconds.
		Weight::from_ref_time(33_083_000)
			.saturating_add(Weight::from_proof_size(13355))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: Alliance Members (r:3 w:1)
	/// Proof: Alliance Members (max_values: None, max_size: Some(3211), added: 5686, mode: MaxEncodedLen)
	/// Storage: AllianceMotion Proposals (r:1 w:0)
	/// Proof Skipped: AllianceMotion Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Alliance DepositOf (r:1 w:1)
	/// Proof: Alliance DepositOf (max_values: None, max_size: Some(64), added: 2539, mode: MaxEncodedLen)
	/// Storage: System Account (r:2 w:2)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: ParachainInfo ParachainId (r:1 w:0)
	/// Proof: ParachainInfo ParachainId (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SupportedVersion (max_values: None, max_size: None, mode: Measured)
	/// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	/// Proof Skipped: PolkadotXcm VersionDiscoveryQueue (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SafeXcmVersion (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	/// Proof Skipped: ParachainSystem HostConfiguration (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	/// Proof Skipped: ParachainSystem PendingUpwardMessages (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: AllianceMotion Members (r:0 w:1)
	/// Proof Skipped: AllianceMotion Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: AllianceMotion Prime (r:0 w:1)
	/// Proof Skipped: AllianceMotion Prime (max_values: Some(1), max_size: None, mode: Measured)
	fn kick_member() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `716`
		//  Estimated: `35980`
		// Minimum execution time: 118_219 nanoseconds.
		Weight::from_ref_time(120_320_000)
			.saturating_add(Weight::from_proof_size(35980))
			.saturating_add(T::DbWeight::get().reads(13))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	/// Storage: Alliance UnscrupulousAccounts (r:1 w:1)
	/// Proof: Alliance UnscrupulousAccounts (max_values: Some(1), max_size: Some(3202), added: 3697, mode: MaxEncodedLen)
	/// Storage: Alliance UnscrupulousWebsites (r:1 w:1)
	/// Proof: Alliance UnscrupulousWebsites (max_values: Some(1), max_size: Some(25702), added: 26197, mode: MaxEncodedLen)
	/// The range of component `n` is `[0, 100]`.
	/// The range of component `l` is `[0, 255]`.
	fn add_unscrupulous_items(n: u32, l: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `76`
		//  Estimated: `29894`
		// Minimum execution time: 6_691 nanoseconds.
		Weight::from_ref_time(6_850_000)
			.saturating_add(Weight::from_proof_size(29894))
			// Standard Error: 2_838
			.saturating_add(Weight::from_ref_time(1_196_960).saturating_mul(n.into()))
			// Standard Error: 1_111
			.saturating_add(Weight::from_ref_time(65_116).saturating_mul(l.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Alliance UnscrupulousAccounts (r:1 w:1)
	/// Proof: Alliance UnscrupulousAccounts (max_values: Some(1), max_size: Some(3202), added: 3697, mode: MaxEncodedLen)
	/// Storage: Alliance UnscrupulousWebsites (r:1 w:1)
	/// Proof: Alliance UnscrupulousWebsites (max_values: Some(1), max_size: Some(25702), added: 26197, mode: MaxEncodedLen)
	/// The range of component `n` is `[0, 100]`.
	/// The range of component `l` is `[0, 255]`.
	fn remove_unscrupulous_items(n: u32, l: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0 + n * (289 ±0) + l * (100 ±0)`
		//  Estimated: `29894`
		// Minimum execution time: 6_846 nanoseconds.
		Weight::from_ref_time(6_963_000)
			.saturating_add(Weight::from_proof_size(29894))
			// Standard Error: 171_069
			.saturating_add(Weight::from_ref_time(13_569_517).saturating_mul(n.into()))
			// Standard Error: 66_998
			.saturating_add(Weight::from_ref_time(464_176).saturating_mul(l.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Alliance Members (r:3 w:2)
	/// Proof: Alliance Members (max_values: None, max_size: Some(3211), added: 5686, mode: MaxEncodedLen)
	/// Storage: AllianceMotion Proposals (r:1 w:0)
	/// Proof Skipped: AllianceMotion Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: AllianceMotion Members (r:0 w:1)
	/// Proof Skipped: AllianceMotion Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: AllianceMotion Prime (r:0 w:1)
	/// Proof Skipped: AllianceMotion Prime (max_values: Some(1), max_size: None, mode: Measured)
	fn abdicate_fellow_status() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `298`
		//  Estimated: `18447`
		// Minimum execution time: 29_822 nanoseconds.
		Weight::from_ref_time(30_595_000)
			.saturating_add(Weight::from_proof_size(18447))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
}
