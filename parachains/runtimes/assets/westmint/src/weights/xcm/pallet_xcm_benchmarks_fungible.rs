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


//! Autogenerated weights for `pallet_xcm_benchmarks::fungible`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-08-17, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("westmint-dev"), DB CACHE: 1024

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
// --pallet=pallet_xcm_benchmarks::fungible
// --chain=westmint-dev
// --json-file=/var/lib/gitlab-runner/builds/zyw4fam_/0/parity/mirrors/cumulus/.git/.artifacts/bench.json
// --header=./file_header.txt
// --template=./templates/xcm-bench-template.hbs
// --output=./parachains/runtimes/assets/westmint/src/weights/xcm/pallet_xcm_benchmarks_fungible.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{RefTimeWeight, Weight}};
use sp_std::marker::PhantomData;

/// Weights for `pallet_xcm_benchmarks::fungible`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo<T> {
	// Storage: System Account (r:1 w:1)
	pub(crate) fn withdraw_asset() -> Weight {
		Weight::from_ref_time(28_725_000 as RefTimeWeight)
			.saturating_add(T::DbWeight::get().reads(1 as RefTimeWeight))
			.saturating_add(T::DbWeight::get().writes(1 as RefTimeWeight))
	}
	// Storage: System Account (r:2 w:2)
	pub(crate) fn transfer_asset() -> Weight {
		Weight::from_ref_time(33_865_000 as RefTimeWeight)
			.saturating_add(T::DbWeight::get().reads(2 as RefTimeWeight))
			.saturating_add(T::DbWeight::get().writes(2 as RefTimeWeight))
	}
	// Storage: System Account (r:2 w:2)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	pub(crate) fn transfer_reserve_asset() -> Weight {
		Weight::from_ref_time(48_839_000 as RefTimeWeight)
			.saturating_add(T::DbWeight::get().reads(8 as RefTimeWeight))
			.saturating_add(T::DbWeight::get().writes(4 as RefTimeWeight))
	}
	pub(crate) fn receive_teleported_asset() -> Weight {
		Weight::from_ref_time(4_849_000 as RefTimeWeight)
	}
	// Storage: System Account (r:1 w:1)
	pub(crate) fn deposit_asset() -> Weight {
		Weight::from_ref_time(29_778_000 as RefTimeWeight)
			.saturating_add(T::DbWeight::get().reads(1 as RefTimeWeight))
			.saturating_add(T::DbWeight::get().writes(1 as RefTimeWeight))
	}
	// Storage: System Account (r:1 w:1)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	pub(crate) fn deposit_reserve_asset() -> Weight {
		Weight::from_ref_time(45_355_000 as RefTimeWeight)
			.saturating_add(T::DbWeight::get().reads(7 as RefTimeWeight))
			.saturating_add(T::DbWeight::get().writes(3 as RefTimeWeight))
	}
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	pub(crate) fn initiate_teleport() -> Weight {
		Weight::from_ref_time(23_451_000 as RefTimeWeight)
			.saturating_add(T::DbWeight::get().reads(6 as RefTimeWeight))
			.saturating_add(T::DbWeight::get().writes(2 as RefTimeWeight))
	}
}
