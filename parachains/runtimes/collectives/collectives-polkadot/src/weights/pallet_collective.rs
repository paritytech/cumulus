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

//! Autogenerated weights for `pallet_collective`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-07-31, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `runner-ynta1nyy-project-238-concurrent-0`, CPU: `Intel(R) Xeon(R) CPU @ 2.60GHz`
//! EXECUTION: ``, WASM-EXECUTION: `Compiled`, CHAIN: `Some("collectives-polkadot-dev")`, DB CACHE: 1024

// Executed Command:
// ./target/production/polkadot-parachain
// benchmark
// pallet
// --chain=collectives-polkadot-dev
// --wasm-execution=compiled
// --pallet=pallet_collective
// --no-storage-info
// --no-median-slopes
// --no-min-squares
// --extrinsic=*
// --steps=50
// --repeat=20
// --json
// --header=./file_header.txt
// --output=./parachains/runtimes/collectives/collectives-polkadot/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_collective`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_collective::WeightInfo for WeightInfo<T> {
	/// Storage: `AllianceMotion::Members` (r:1 w:1)
	/// Proof: `AllianceMotion::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::Proposals` (r:1 w:0)
	/// Proof: `AllianceMotion::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::Voting` (r:100 w:100)
	/// Proof: `AllianceMotion::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::Prime` (r:0 w:1)
	/// Proof: `AllianceMotion::Prime` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `m` is `[0, 100]`.
	/// The range of component `n` is `[0, 100]`.
	/// The range of component `p` is `[0, 100]`.
	fn set_members(m: u32, _n: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0 + m * (3232 ±0) + p * (3190 ±0)`
		//  Estimated: `15691 + m * (1967 ±23) + p * (4332 ±23)`
		// Minimum execution time: 16_410_000 picoseconds.
		Weight::from_parts(16_816_000, 0)
			.saturating_add(Weight::from_parts(0, 15691))
			// Standard Error: 59_812
			.saturating_add(Weight::from_parts(4_516_537, 0).saturating_mul(m.into()))
			// Standard Error: 59_812
			.saturating_add(Weight::from_parts(7_992_168, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(p.into())))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(p.into())))
			.saturating_add(Weight::from_parts(0, 1967).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 4332).saturating_mul(p.into()))
	}
	/// Storage: `AllianceMotion::Members` (r:1 w:0)
	/// Proof: `AllianceMotion::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn execute(b: u32, m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `32 + m * (32 ±0)`
		//  Estimated: `1518 + m * (32 ±0)`
		// Minimum execution time: 14_418_000 picoseconds.
		Weight::from_parts(13_588_617, 0)
			.saturating_add(Weight::from_parts(0, 1518))
			// Standard Error: 21
			.saturating_add(Weight::from_parts(1_711, 0).saturating_mul(b.into()))
			// Standard Error: 223
			.saturating_add(Weight::from_parts(13_836, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(Weight::from_parts(0, 32).saturating_mul(m.into()))
	}
	/// Storage: `AllianceMotion::Members` (r:1 w:0)
	/// Proof: `AllianceMotion::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::ProposalOf` (r:1 w:0)
	/// Proof: `AllianceMotion::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn propose_execute(b: u32, m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `32 + m * (32 ±0)`
		//  Estimated: `3498 + m * (32 ±0)`
		// Minimum execution time: 17_174_000 picoseconds.
		Weight::from_parts(16_192_764, 0)
			.saturating_add(Weight::from_parts(0, 3498))
			// Standard Error: 27
			.saturating_add(Weight::from_parts(1_672, 0).saturating_mul(b.into()))
			// Standard Error: 280
			.saturating_add(Weight::from_parts(24_343, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(Weight::from_parts(0, 32).saturating_mul(m.into()))
	}
	/// Storage: `AllianceMotion::Members` (r:1 w:0)
	/// Proof: `AllianceMotion::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::ProposalOf` (r:1 w:1)
	/// Proof: `AllianceMotion::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::Proposals` (r:1 w:1)
	/// Proof: `AllianceMotion::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::ProposalCount` (r:1 w:1)
	/// Proof: `AllianceMotion::ProposalCount` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::Voting` (r:0 w:1)
	/// Proof: `AllianceMotion::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[2, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn propose_proposed(b: u32, m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `322 + m * (32 ±0) + p * (36 ±0)`
		//  Estimated: `3714 + m * (33 ±0) + p * (36 ±0)`
		// Minimum execution time: 23_970_000 picoseconds.
		Weight::from_parts(23_004_052, 0)
			.saturating_add(Weight::from_parts(0, 3714))
			// Standard Error: 123
			.saturating_add(Weight::from_parts(2_728, 0).saturating_mul(b.into()))
			// Standard Error: 1_291
			.saturating_add(Weight::from_parts(32_731, 0).saturating_mul(m.into()))
			// Standard Error: 1_275
			.saturating_add(Weight::from_parts(199_537, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
			.saturating_add(Weight::from_parts(0, 33).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 36).saturating_mul(p.into()))
	}
	/// Storage: `AllianceMotion::Members` (r:1 w:0)
	/// Proof: `AllianceMotion::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::Voting` (r:1 w:1)
	/// Proof: `AllianceMotion::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `m` is `[5, 100]`.
	fn vote(m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `771 + m * (64 ±0)`
		//  Estimated: `4235 + m * (64 ±0)`
		// Minimum execution time: 25_843_000 picoseconds.
		Weight::from_parts(26_092_578, 0)
			.saturating_add(Weight::from_parts(0, 4235))
			// Standard Error: 1_785
			.saturating_add(Weight::from_parts(67_298, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(Weight::from_parts(0, 64).saturating_mul(m.into()))
	}
	/// Storage: `AllianceMotion::Voting` (r:1 w:1)
	/// Proof: `AllianceMotion::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::Members` (r:1 w:0)
	/// Proof: `AllianceMotion::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::Proposals` (r:1 w:1)
	/// Proof: `AllianceMotion::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::ProposalOf` (r:0 w:1)
	/// Proof: `AllianceMotion::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_disapproved(m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `360 + m * (64 ±0) + p * (36 ±0)`
		//  Estimated: `3805 + m * (65 ±0) + p * (36 ±0)`
		// Minimum execution time: 27_543_000 picoseconds.
		Weight::from_parts(26_505_473, 0)
			.saturating_add(Weight::from_parts(0, 3805))
			// Standard Error: 1_054
			.saturating_add(Weight::from_parts(35_295, 0).saturating_mul(m.into()))
			// Standard Error: 1_028
			.saturating_add(Weight::from_parts(190_508, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_parts(0, 65).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 36).saturating_mul(p.into()))
	}
	/// Storage: `AllianceMotion::Voting` (r:1 w:1)
	/// Proof: `AllianceMotion::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::Members` (r:1 w:0)
	/// Proof: `AllianceMotion::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::ProposalOf` (r:1 w:1)
	/// Proof: `AllianceMotion::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::Proposals` (r:1 w:1)
	/// Proof: `AllianceMotion::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_approved(b: u32, m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `662 + b * (1 ±0) + m * (64 ±0) + p * (40 ±0)`
		//  Estimated: `3979 + b * (1 ±0) + m * (66 ±0) + p * (40 ±0)`
		// Minimum execution time: 40_375_000 picoseconds.
		Weight::from_parts(34_081_294, 0)
			.saturating_add(Weight::from_parts(0, 3979))
			// Standard Error: 196
			.saturating_add(Weight::from_parts(3_796, 0).saturating_mul(b.into()))
			// Standard Error: 2_072
			.saturating_add(Weight::from_parts(50_954, 0).saturating_mul(m.into()))
			// Standard Error: 2_020
			.saturating_add(Weight::from_parts(246_000, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_parts(0, 1).saturating_mul(b.into()))
			.saturating_add(Weight::from_parts(0, 66).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 40).saturating_mul(p.into()))
	}
	/// Storage: `AllianceMotion::Voting` (r:1 w:1)
	/// Proof: `AllianceMotion::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::Members` (r:1 w:0)
	/// Proof: `AllianceMotion::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::Prime` (r:1 w:0)
	/// Proof: `AllianceMotion::Prime` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::Proposals` (r:1 w:1)
	/// Proof: `AllianceMotion::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::ProposalOf` (r:0 w:1)
	/// Proof: `AllianceMotion::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_disapproved(m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `458 + m * (48 ±0) + p * (36 ±0)`
		//  Estimated: `3898 + m * (49 ±0) + p * (36 ±0)`
		// Minimum execution time: 28_793_000 picoseconds.
		Weight::from_parts(29_656_832, 0)
			.saturating_add(Weight::from_parts(0, 3898))
			// Standard Error: 1_214
			.saturating_add(Weight::from_parts(22_148, 0).saturating_mul(m.into()))
			// Standard Error: 1_184
			.saturating_add(Weight::from_parts(189_860, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_parts(0, 49).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 36).saturating_mul(p.into()))
	}
	/// Storage: `AllianceMotion::Voting` (r:1 w:1)
	/// Proof: `AllianceMotion::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::Members` (r:1 w:0)
	/// Proof: `AllianceMotion::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::Prime` (r:1 w:0)
	/// Proof: `AllianceMotion::Prime` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::ProposalOf` (r:1 w:1)
	/// Proof: `AllianceMotion::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::Proposals` (r:1 w:1)
	/// Proof: `AllianceMotion::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_approved(b: u32, m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `682 + b * (1 ±0) + m * (64 ±0) + p * (40 ±0)`
		//  Estimated: `3999 + b * (1 ±0) + m * (66 ±0) + p * (40 ±0)`
		// Minimum execution time: 40_887_000 picoseconds.
		Weight::from_parts(39_529_567, 0)
			.saturating_add(Weight::from_parts(0, 3999))
			// Standard Error: 191
			.saturating_add(Weight::from_parts(2_802, 0).saturating_mul(b.into()))
			// Standard Error: 2_021
			.saturating_add(Weight::from_parts(35_956, 0).saturating_mul(m.into()))
			// Standard Error: 1_970
			.saturating_add(Weight::from_parts(235_154, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_parts(0, 1).saturating_mul(b.into()))
			.saturating_add(Weight::from_parts(0, 66).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 40).saturating_mul(p.into()))
	}
	/// Storage: `AllianceMotion::Proposals` (r:1 w:1)
	/// Proof: `AllianceMotion::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::Voting` (r:0 w:1)
	/// Proof: `AllianceMotion::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AllianceMotion::ProposalOf` (r:0 w:1)
	/// Proof: `AllianceMotion::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `p` is `[1, 100]`.
	fn disapprove_proposal(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `189 + p * (32 ±0)`
		//  Estimated: `1674 + p * (32 ±0)`
		// Minimum execution time: 14_040_000 picoseconds.
		Weight::from_parts(15_075_964, 0)
			.saturating_add(Weight::from_parts(0, 1674))
			// Standard Error: 854
			.saturating_add(Weight::from_parts(159_597, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_parts(0, 32).saturating_mul(p.into()))
	}
}
