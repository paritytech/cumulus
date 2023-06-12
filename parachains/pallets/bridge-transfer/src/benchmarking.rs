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

use crate::{
	types::{
		filter::MultiLocationFilter, AssetFilterOf, BridgeConfig, LatestVersionedMultiLocation,
	},
	AllowedExporters, AllowedReserveLocations, AllowedUniversalAliases, BenchmarkHelper, Call,
	Config, Event, Pallet,
};

use frame_benchmarking::{benchmarks, BenchmarkError};
use frame_support::{
	assert_ok, ensure,
	traits::{EnsureOrigin, Get},
	BoundedVec,
};
use sp_std::prelude::*;
use xcm::prelude::*;

#[cfg(feature = "runtime-benchmarks")]
impl<T: Config> Pallet<T> {
	#[cfg(feature = "runtime-benchmarks")]
	pub fn insert_universal_alias_for_benchmarks((location, junction): (MultiLocation, Junction)) {
		assert!(matches!(
			AllowedUniversalAliases::<T>::try_mutate(
				LatestVersionedMultiLocation(&location),
				|junctions| junctions.try_insert(junction)
			),
			Ok(true)
		));
	}
}

benchmarks! {
	transfer_asset_via_bridge {
		let _ = T::AssetTransferOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		// every asset has its own configuration and ledger, so there's a performance dependency
		// (be sure to use "worst" of assets)
		let max_assets_limit = T::AssetsLimit::get();
		ensure!(max_assets_limit > 0, "MaxAssetsLimit not set up correctly.");

		let (bridged_network, maybe_paid_bridge) = T::BenchmarkHelper::bridge_location()
			.ok_or(BenchmarkError::Stop("missing `bridge_location` data"))?;
		let mut bridge_config = BridgeConfig::new(
			maybe_paid_bridge.location.into_versioned(),
			maybe_paid_bridge.maybe_fee.map(|fee| fee.into())
		);

		let allowed_bridged_target_location = T::BenchmarkHelper::allowed_bridged_target_location()
			.ok_or(BenchmarkError::Stop("missing `allowed_bridged_target_location` data"))?;
		assert_ok!(bridge_config.update_allowed_target_location(
			allowed_bridged_target_location.location.into_versioned(),
			allowed_bridged_target_location.maybe_fee.map(|fee| VersionedMultiAsset::V3(fee))
		));
		assert_ok!(bridge_config.add_allowed_target_location_filter_for(
			allowed_bridged_target_location.location.into_versioned(),
			AssetFilterOf::<T>::All,
		));
		AllowedExporters::<T>::insert(bridged_network, bridge_config);

		let (origin, assets, destination) = T::BenchmarkHelper::prepare_asset_transfer()
			.ok_or(BenchmarkError::Stop("missing `prepare_asset_transfer` data"))?;
		let assets_count = match &assets {
			VersionedMultiAssets::V2(assets) => assets.len(),
			VersionedMultiAssets::V3(assets) => assets.len(),
		};
		ensure!(assets_count == max_assets_limit as usize, "`assets` not set up correctly for worst case.");

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
