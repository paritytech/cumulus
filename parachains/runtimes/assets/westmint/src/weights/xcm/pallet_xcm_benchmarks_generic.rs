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
//! DATE: 2022-09-30, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `bm3`, CPU: `Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("westmint-dev"), DB CACHE: 1024

// Executed Command:
// ./artifacts/polkadot-parachain
// benchmark
// pallet
// --template=./templates/xcm-bench-template.hbs
// --chain=westmint-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_xcm_benchmarks::generic
// --extrinsic=*
// --steps=50
// --repeat=20
// --json
// --header=./file_header.txt
// --template=./templates/xcm-bench-template.hbs
// --output=./parachains/runtimes/assets/westmint/src/weights/xcm/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
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
	pub(crate) fn report_holding() -> Weight {
		Weight::from_ref_time(1_324_853_000 as u64)
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	pub(crate) fn buy_execution() -> Weight {
		Weight::from_ref_time(8_533_000 as u64)
	}
	// Storage: PolkadotXcm Queries (r:1 w:0)
	pub(crate) fn query_response() -> Weight {
		Weight::from_ref_time(19_435_000 as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
	}
	pub(crate) fn transact() -> Weight {
		Weight::from_ref_time(22_656_000 as u64)
	}
	pub(crate) fn refund_surplus() -> Weight {
		Weight::from_ref_time(8_900_000 as u64)
	}
	pub(crate) fn set_error_handler() -> Weight {
		Weight::from_ref_time(6_255_000 as u64)
	}
	pub(crate) fn set_appendix() -> Weight {
		Weight::from_ref_time(6_268_000 as u64)
	}
	pub(crate) fn clear_error() -> Weight {
		Weight::from_ref_time(6_304_000 as u64)
	}
	pub(crate) fn descend_origin() -> Weight {
		Weight::from_ref_time(7_279_000 as u64)
	}
	pub(crate) fn clear_origin() -> Weight {
		Weight::from_ref_time(6_297_000 as u64)
	}
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	pub(crate) fn report_error() -> Weight {
		Weight::from_ref_time(23_025_000 as u64)
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: PolkadotXcm AssetTraps (r:1 w:1)
	pub(crate) fn claim_asset() -> Weight {
		Weight::from_ref_time(13_001_000 as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	pub(crate) fn trap() -> Weight {
		Weight::from_ref_time(6_266_000 as u64)
	}
	// Storage: PolkadotXcm VersionNotifyTargets (r:1 w:1)
	// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	pub(crate) fn subscribe_version() -> Weight {
		Weight::from_ref_time(31_348_000 as u64)
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: PolkadotXcm VersionNotifyTargets (r:0 w:1)
	pub(crate) fn unsubscribe_version() -> Weight {
		Weight::from_ref_time(9_534_000 as u64)
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	pub(crate) fn initiate_reserve_withdraw() -> Weight {
		Weight::from_ref_time(1_558_814_000 as u64)
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	pub(crate) fn burn_asset() -> Weight {
		Weight::from_ref_time(496_802_000 as u64)
	}
	pub(crate) fn expect_asset() -> Weight {
		Weight::from_ref_time(38_299_000 as u64)
	}
	pub(crate) fn expect_origin() -> Weight {
		Weight::from_ref_time(6_354_000 as u64)
	}
	pub(crate) fn expect_error() -> Weight {
		Weight::from_ref_time(6_234_000 as u64)
	}
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	pub(crate) fn query_pallet() -> Weight {
		Weight::from_ref_time(25_150_000 as u64)
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	pub(crate) fn expect_pallet() -> Weight {
		Weight::from_ref_time(7_969_000 as u64)
	}
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	pub(crate) fn report_transact_status() -> Weight {
		Weight::from_ref_time(23_099_000 as u64)
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	pub(crate) fn clear_transact_status() -> Weight {
		Weight::from_ref_time(6_366_000 as u64)
	}
	pub(crate) fn set_topic() -> Weight {
		Weight::from_ref_time(6_422_000 as u64)
	}
	pub(crate) fn clear_topic() -> Weight {
		Weight::from_ref_time(6_405_000 as u64)
	}
	pub(crate) fn set_fees_mode() -> Weight {
		Weight::from_ref_time(6_392_000 as u64)
	}
	pub(crate) fn unpaid_execution() -> Weight {
		Weight::from_ref_time(3_111_000 as u64)
	}
}
