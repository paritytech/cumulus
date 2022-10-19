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
// --pallet=pallet_alliance
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
	// Storage: Alliance Members (r:1 w:0)
	// Storage: AllianceMotion ProposalOf (r:1 w:1)
	// Storage: AllianceMotion Proposals (r:1 w:1)
	// Storage: AllianceMotion ProposalCount (r:1 w:1)
	// Storage: AllianceMotion Voting (r:0 w:1)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `x` is `[2, 10]`.
	/// The range of component `y` is `[0, 90]`.
	/// The range of component `p` is `[1, 100]`.
	fn propose_proposed(b: u32, _x: u32, y: u32, p: u32, ) -> Weight {
		Weight::from_ref_time(41_971_000 as u64)
			// Standard Error: 202
			.saturating_add(Weight::from_ref_time(1_452 as u64).saturating_mul(b as u64))
			// Standard Error: 2_287
			.saturating_add(Weight::from_ref_time(23_477 as u64).saturating_mul(y as u64))
			// Standard Error: 2_076
			.saturating_add(Weight::from_ref_time(166_128 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Alliance Members (r:2 w:0)
	// Storage: AllianceMotion Voting (r:1 w:1)
	/// The range of component `x` is `[3, 10]`.
	/// The range of component `y` is `[2, 90]`.
	fn vote(x: u32, y: u32, ) -> Weight {
		Weight::from_ref_time(45_843_000 as u64)
			// Standard Error: 18_702
			.saturating_add(Weight::from_ref_time(109_438 as u64).saturating_mul(x as u64))
			// Standard Error: 2_133
			.saturating_add(Weight::from_ref_time(94_046 as u64).saturating_mul(y as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Alliance Members (r:1 w:0)
	// Storage: AllianceMotion ProposalOf (r:1 w:1)
	// Storage: AllianceMotion Proposals (r:1 w:1)
	// Storage: AllianceMotion Voting (r:0 w:1)
	/// The range of component `p` is `[1, 100]`.
	fn veto(p: u32, ) -> Weight {
		Weight::from_ref_time(32_810_000 as u64)
			// Standard Error: 1_482
			.saturating_add(Weight::from_ref_time(238_662 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Alliance Members (r:1 w:0)
	// Storage: AllianceMotion Voting (r:1 w:1)
	// Storage: AllianceMotion Members (r:1 w:0)
	// Storage: AllianceMotion Proposals (r:1 w:1)
	// Storage: AllianceMotion ProposalOf (r:0 w:1)
	/// The range of component `x` is `[2, 10]`.
	/// The range of component `y` is `[2, 90]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_disapproved(_x: u32, y: u32, p: u32, ) -> Weight {
		Weight::from_ref_time(50_261_000 as u64)
			// Standard Error: 1_934
			.saturating_add(Weight::from_ref_time(39_028 as u64).saturating_mul(y as u64))
			// Standard Error: 1_734
			.saturating_add(Weight::from_ref_time(132_016 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Alliance Members (r:1 w:0)
	// Storage: AllianceMotion Voting (r:1 w:1)
	// Storage: AllianceMotion Members (r:1 w:0)
	// Storage: AllianceMotion ProposalOf (r:1 w:1)
	// Storage: AllianceMotion Proposals (r:1 w:1)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `x` is `[2, 10]`.
	/// The range of component `y` is `[2, 90]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_approved(b: u32, _x: u32, y: u32, p: u32, ) -> Weight {
		Weight::from_ref_time(60_581_000 as u64)
			// Standard Error: 231
			.saturating_add(Weight::from_ref_time(843 as u64).saturating_mul(b as u64))
			// Standard Error: 2_660
			.saturating_add(Weight::from_ref_time(37_765 as u64).saturating_mul(y as u64))
			// Standard Error: 2_381
			.saturating_add(Weight::from_ref_time(163_757 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Alliance Members (r:1 w:0)
	// Storage: AllianceMotion Voting (r:1 w:1)
	// Storage: AllianceMotion Members (r:1 w:0)
	// Storage: AllianceMotion Prime (r:1 w:0)
	// Storage: AllianceMotion ProposalOf (r:1 w:1)
	// Storage: AllianceMotion Proposals (r:1 w:1)
	// Storage: Alliance Rule (r:0 w:1)
	/// The range of component `x` is `[2, 10]`.
	/// The range of component `y` is `[2, 90]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_disapproved(_x: u32, y: u32, p: u32, ) -> Weight {
		Weight::from_ref_time(67_103_000 as u64)
			// Standard Error: 1_978
			.saturating_add(Weight::from_ref_time(54_602 as u64).saturating_mul(y as u64))
			// Standard Error: 1_773
			.saturating_add(Weight::from_ref_time(172_664 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Alliance Members (r:1 w:0)
	// Storage: AllianceMotion Voting (r:1 w:1)
	// Storage: AllianceMotion Members (r:1 w:0)
	// Storage: AllianceMotion Prime (r:1 w:0)
	// Storage: AllianceMotion Proposals (r:1 w:1)
	// Storage: AllianceMotion ProposalOf (r:0 w:1)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `x` is `[2, 10]`.
	/// The range of component `y` is `[2, 90]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_approved(_b: u32, x: u32, y: u32, p: u32, ) -> Weight {
		Weight::from_ref_time(51_035_000 as u64)
			// Standard Error: 20_467
			.saturating_add(Weight::from_ref_time(128_374 as u64).saturating_mul(x as u64))
			// Standard Error: 2_114
			.saturating_add(Weight::from_ref_time(44_758 as u64).saturating_mul(y as u64))
			// Standard Error: 1_892
			.saturating_add(Weight::from_ref_time(143_724 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Alliance Members (r:3 w:3)
	// Storage: AllianceMotion Members (r:1 w:1)
	/// The range of component `x` is `[1, 10]`.
	/// The range of component `y` is `[0, 90]`.
	/// The range of component `z` is `[0, 100]`.
	fn init_members(_x: u32, y: u32, z: u32, ) -> Weight {
		Weight::from_ref_time(48_550_000 as u64)
			// Standard Error: 1_509
			.saturating_add(Weight::from_ref_time(113_586 as u64).saturating_mul(y as u64))
			// Standard Error: 1_359
			.saturating_add(Weight::from_ref_time(86_465 as u64).saturating_mul(z as u64))
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Alliance Members (r:3 w:3)
	// Storage: AllianceMotion Proposals (r:1 w:0)
	// Storage: Alliance DepositOf (r:101 w:50)
	// Storage: System Account (r:50 w:50)
	// Storage: AllianceMotion Members (r:0 w:1)
	// Storage: AllianceMotion Prime (r:0 w:1)
	/// The range of component `x` is `[1, 100]`.
	/// The range of component `y` is `[0, 100]`.
	/// The range of component `z` is `[0, 50]`.
	fn disband(x: u32, y: u32, z: u32, ) -> Weight {
		Weight::from_ref_time(265_459_000 as u64)
			// Standard Error: 20_711
			.saturating_add(Weight::from_ref_time(462_927 as u64).saturating_mul(x as u64))
			// Standard Error: 20_611
			.saturating_add(Weight::from_ref_time(477_895 as u64).saturating_mul(y as u64))
			// Standard Error: 41_186
			.saturating_add(Weight::from_ref_time(9_255_397 as u64).saturating_mul(z as u64))
			.saturating_add(T::DbWeight::get().reads(154 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
			.saturating_add(T::DbWeight::get().writes((2 as u64).saturating_mul(z as u64)))
	}
	// Storage: Alliance Rule (r:0 w:1)
	fn set_rule() -> Weight {
		Weight::from_ref_time(18_726_000 as u64)
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Alliance Announcements (r:1 w:1)
	fn announce() -> Weight {
		Weight::from_ref_time(20_945_000 as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Alliance Announcements (r:1 w:1)
	fn remove_announcement() -> Weight {
		Weight::from_ref_time(23_006_000 as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Alliance Members (r:4 w:1)
	// Storage: Alliance UnscrupulousAccounts (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: Alliance DepositOf (r:0 w:1)
	fn join_alliance() -> Weight {
		Weight::from_ref_time(55_374_000 as u64)
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Alliance Members (r:4 w:1)
	// Storage: Alliance UnscrupulousAccounts (r:1 w:0)
	fn nominate_ally() -> Weight {
		Weight::from_ref_time(42_073_000 as u64)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Alliance Members (r:3 w:2)
	// Storage: AllianceMotion Proposals (r:1 w:0)
	// Storage: AllianceMotion Members (r:0 w:1)
	// Storage: AllianceMotion Prime (r:0 w:1)
	fn elevate_ally() -> Weight {
		Weight::from_ref_time(38_040_000 as u64)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Alliance Members (r:4 w:2)
	// Storage: AllianceMotion Proposals (r:1 w:0)
	// Storage: AllianceMotion Members (r:0 w:1)
	// Storage: AllianceMotion Prime (r:0 w:1)
	// Storage: Alliance RetiringMembers (r:0 w:1)
	fn give_retirement_notice() -> Weight {
		Weight::from_ref_time(40_888_000 as u64)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
	// Storage: Alliance RetiringMembers (r:1 w:1)
	// Storage: Alliance Members (r:1 w:1)
	// Storage: Alliance DepositOf (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn retire() -> Weight {
		Weight::from_ref_time(45_167_000 as u64)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Alliance Members (r:3 w:1)
	// Storage: AllianceMotion Proposals (r:1 w:0)
	// Storage: Alliance DepositOf (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	// Storage: AllianceMotion Members (r:0 w:1)
	// Storage: AllianceMotion Prime (r:0 w:1)
	fn kick_member() -> Weight {
		Weight::from_ref_time(127_449_000 as u64)
			.saturating_add(T::DbWeight::get().reads(13 as u64))
			.saturating_add(T::DbWeight::get().writes(8 as u64))
	}
	// Storage: Alliance UnscrupulousAccounts (r:1 w:1)
	// Storage: Alliance UnscrupulousWebsites (r:1 w:1)
	/// The range of component `n` is `[1, 100]`.
	/// The range of component `l` is `[1, 255]`.
	fn add_unscrupulous_items(n: u32, l: u32, ) -> Weight {
		Weight::from_ref_time(25_056_000 as u64)
			// Standard Error: 3_388
			.saturating_add(Weight::from_ref_time(1_261_054 as u64).saturating_mul(n as u64))
			// Standard Error: 1_327
			.saturating_add(Weight::from_ref_time(50_999 as u64).saturating_mul(l as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: Alliance UnscrupulousAccounts (r:1 w:1)
	// Storage: Alliance UnscrupulousWebsites (r:1 w:1)
	/// The range of component `n` is `[1, 100]`.
	/// The range of component `l` is `[1, 255]`.
	fn remove_unscrupulous_items(n: u32, l: u32, ) -> Weight {
		Weight::from_ref_time(29_081_000 as u64)
			// Standard Error: 175_459
			.saturating_add(Weight::from_ref_time(14_861_926 as u64).saturating_mul(n as u64))
			// Standard Error: 68_746
			.saturating_add(Weight::from_ref_time(379_239 as u64).saturating_mul(l as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
}
