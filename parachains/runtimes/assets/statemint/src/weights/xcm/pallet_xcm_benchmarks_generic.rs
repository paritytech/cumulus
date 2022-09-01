// Copyright 2022 Parity Technologies (UK) Ltd.
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


//! Autogenerated weights for `pallet_xcm_benchmarks::generic`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-08-25, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `bm3`, CPU: `Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("statemint-dev"), DB CACHE: 1024

// Executed Command:
// /home/benchbot/cargo_target_dir/production/polkadot-parachain
// benchmark
// pallet
// --steps=50
// --repeat=20
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --pallet=pallet_xcm_benchmarks::generic
// --chain=statemint-dev
// --json-file=/var/lib/gitlab-runner/builds/zyw4fam_/0/parity/mirrors/cumulus/.git/.artifacts/bench.json
// --header=./file_header.txt
// --template=./templates/xcm-bench-template.hbs
// --output=./parachains/runtimes/assets/statemint/src/weights/xcm/pallet_xcm_benchmarks_generic.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{RefTimeWeight, Weight}};
use sp_std::marker::PhantomData;

/// Weights for `pallet_xcm_benchmarks::generic`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo<T> {
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	pub(crate) fn query_holding() -> Weight {
		Weight::from_ref_time(682_639_000 as RefTimeWeight)
			.saturating_add(T::DbWeight::get().reads(6 as RefTimeWeight))
			.saturating_add(T::DbWeight::get().writes(2 as RefTimeWeight))
	}
	pub(crate) fn buy_execution() -> Weight {
		Weight::from_ref_time(9_272_000 as RefTimeWeight)
	}
	// Storage: PolkadotXcm Queries (r:1 w:0)
	pub(crate) fn query_response() -> Weight {
		Weight::from_ref_time(17_084_000 as RefTimeWeight)
			.saturating_add(T::DbWeight::get().reads(1 as RefTimeWeight))
	}
	pub(crate) fn transact() -> Weight {
		Weight::from_ref_time(20_265_000 as RefTimeWeight)
	}
	pub(crate) fn refund_surplus() -> Weight {
		Weight::from_ref_time(9_422_000 as RefTimeWeight)
	}
	pub(crate) fn set_error_handler() -> Weight {
		Weight::from_ref_time(5_545_000 as RefTimeWeight)
	}
	pub(crate) fn set_appendix() -> Weight {
		Weight::from_ref_time(5_450_000 as RefTimeWeight)
	}
	pub(crate) fn clear_error() -> Weight {
		Weight::from_ref_time(5_519_000 as RefTimeWeight)
	}
	pub(crate) fn descend_origin() -> Weight {
		Weight::from_ref_time(6_398_000 as RefTimeWeight)
	}
	pub(crate) fn clear_origin() -> Weight {
		Weight::from_ref_time(5_498_000 as RefTimeWeight)
	}
	// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	pub(crate) fn report_error() -> Weight {
		Weight::from_ref_time(15_784_000 as RefTimeWeight)
			.saturating_add(T::DbWeight::get().reads(5 as RefTimeWeight))
			.saturating_add(T::DbWeight::get().writes(2 as RefTimeWeight))
	}
	// Storage: PolkadotXcm AssetTraps (r:1 w:1)
	pub(crate) fn claim_asset() -> Weight {
		Weight::from_ref_time(11_861_000 as RefTimeWeight)
			.saturating_add(T::DbWeight::get().reads(1 as RefTimeWeight))
			.saturating_add(T::DbWeight::get().writes(1 as RefTimeWeight))
	}
	pub(crate) fn trap() -> Weight {
		Weight::from_ref_time(5_462_000 as RefTimeWeight)
	}
	// Storage: PolkadotXcm VersionNotifyTargets (r:1 w:1)
	// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	pub(crate) fn subscribe_version() -> Weight {
		Weight::from_ref_time(18_997_000 as RefTimeWeight)
			.saturating_add(T::DbWeight::get().reads(6 as RefTimeWeight))
			.saturating_add(T::DbWeight::get().writes(3 as RefTimeWeight))
	}
	// Storage: PolkadotXcm VersionNotifyTargets (r:0 w:1)
	pub(crate) fn unsubscribe_version() -> Weight {
		Weight::from_ref_time(8_684_000 as RefTimeWeight)
			.saturating_add(T::DbWeight::get().writes(1 as RefTimeWeight))
	}
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	pub(crate) fn initiate_reserve_withdraw() -> Weight {
		Weight::from_ref_time(883_121_000 as RefTimeWeight)
			.saturating_add(T::DbWeight::get().reads(6 as RefTimeWeight))
			.saturating_add(T::DbWeight::get().writes(2 as RefTimeWeight))
	}
}
