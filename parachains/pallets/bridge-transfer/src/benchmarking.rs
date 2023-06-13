// This file is part of Substrate.

// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! `BridgeTransfer` pallet benchmarks.

use crate::{BenchmarkHelper, Call, Config, Event, Pallet};

use frame_benchmarking::{benchmarks, BenchmarkError};
use frame_support::{
	assert_ok, ensure,
	traits::{EnsureOrigin, Get},
};
use sp_std::prelude::*;
use xcm::prelude::*;

benchmarks! {
	transfer_asset_via_bridge {
		let _ = T::AssetTransferOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		// every asset has its own configuration and ledger, so there's a performance dependency
		// (be sure to use "worst" of assets)
		let max_assets_limit = T::AssetsLimit::get();
		ensure!(max_assets_limit > 0, "MaxAssetsLimit not set up correctly.");

		// get desired remote destination
		let desired_bridged_location = T::BenchmarkHelper::desired_bridged_location()
			.ok_or(BenchmarkError::Stop("missing `desired_bridged_location` data"))?;

		// resolve expected reserve account
		let assumed_reserve_account = Pallet::<T>::resolve_reserve_account(&desired_bridged_location.1);

		// prepare all requirements
		let (origin, assets, destination) = T::BenchmarkHelper::prepare_asset_transfer_for(
			desired_bridged_location.clone(),
			assumed_reserve_account,
		).ok_or(BenchmarkError::Stop("missing `prepare_asset_transfer` data"))?;

		// check assets
		let assets_count = match &assets {
			VersionedMultiAssets::V2(assets) => assets.len(),
			VersionedMultiAssets::V3(assets) => assets.len(),
		};
		ensure!(assets_count == max_assets_limit as usize, "`assets` not set up correctly for worst case.");

		// check destination if configuration is ok
		assert_ok!(<T::BridgedDestinationValidator as pallet_bridge_transfer_primitives::EnsureReachableDestination>::ensure_destination(
			desired_bridged_location.0,
			destination.clone().try_into().map_err(|_|BenchmarkError::Stop("invalid configuration or data"))?));

	}: _<T::RuntimeOrigin>(origin, Box::new(assets), Box::new(destination))
	verify {
		// we don't care about message hash or sender cost here, just check that the transfer has been initiated
		let actual_event = frame_system::Pallet::<T>::events().pop().map(|r| r.event);
		let expected_event: <T as Config>::RuntimeEvent = Event::TransferInitiated {
			message_id: Default::default(),
			forwarded_message_id: Default::default(),
			sender_cost: Default::default(),
		}.into();
		assert!(matches!(actual_event, Some(expected_event)));
	}

	impl_benchmark_test_suite!(Pallet, crate::tests::new_test_ext(), crate::tests::TestRuntime);
}
