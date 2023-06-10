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

//! `BridgeTransfer` on-chain configuration pallet benchmarks.

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
	traits::{EnsureOrigin, EnsureOriginWithArg, Get},
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
	add_exporter_config {
		let origin = T::AdminOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let (bridged_network, maybe_paid_bridge) = T::BenchmarkHelper::bridge_location()
			.ok_or(BenchmarkError::Stop("missing `bridge_config` data"))?;
	}: _<T::RuntimeOrigin>(
		origin,
		bridged_network,
		Box::new(maybe_paid_bridge.location.clone().into_versioned()),
		maybe_paid_bridge.maybe_fee.clone().map(|fee| Box::new(VersionedMultiAsset::V3(fee)))
	)
	verify {
		assert_eq!(
			AllowedExporters::<T>::get(bridged_network).unwrap().to_bridge_location().expect("stored config"),
			maybe_paid_bridge
		);
	}

	remove_exporter_config {
		let origin = T::AdminOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let (bridged_network, maybe_paid_bridge) = T::BenchmarkHelper::bridge_location()
			.ok_or(BenchmarkError::Stop("missing `bridge_config` data"))?;
		AllowedExporters::<T>::insert(
			bridged_network,
			BridgeConfig::new(
				maybe_paid_bridge.location.into_versioned(),
				maybe_paid_bridge.maybe_fee.map(|fee| VersionedMultiAsset::V3(fee))
			)
		);
	}: _<T::RuntimeOrigin>(origin, bridged_network)
	verify {
		assert_eq!(AllowedExporters::<T>::get(bridged_network), None);
	}

	update_exporter_config {
		let origin = T::AdminOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let (bridged_network, maybe_paid_bridge) = T::BenchmarkHelper::bridge_location()
			.ok_or(BenchmarkError::Stop("missing `bridge_config` data"))?;
		AllowedExporters::<T>::insert(
			bridged_network,
			BridgeConfig::new(
				maybe_paid_bridge.location.into_versioned(),
				None
			)
		);

		let new_bridge_location_fee = Some(MultiAsset {
			id: Concrete(MultiLocation::parent()),
			fun: Fungible(1_000_0000),
		});
	}: _<T::RuntimeOrigin>(
		origin,
		bridged_network,
		new_bridge_location_fee.clone().map(|fee| Box::new(VersionedMultiAsset::V3(fee)))
	)
	verify {
		assert_eq!(
			AllowedExporters::<T>::get(bridged_network).unwrap().to_bridge_location().expect("stored config").maybe_fee,
			new_bridge_location_fee
		);
	}

	update_bridged_target_location {
		let origin = T::AdminOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		if T::TargetLocationsPerExporterLimit::get() <= 0 {
			return Err(BenchmarkError::Weightless);
		}
		let (bridged_network, maybe_paid_bridge) = T::BenchmarkHelper::bridge_location()
			.ok_or(BenchmarkError::Stop("missing `bridge_config` data"))?;
		let allowed_bridged_target_location = T::BenchmarkHelper::allowed_bridged_target_location()
			.ok_or(BenchmarkError::Stop("missing `allowed_bridged_target_location` data"))?;
		AllowedExporters::<T>::insert(
			bridged_network,
			BridgeConfig::new(
				maybe_paid_bridge.location.into_versioned(),
				None
			)
		);
		let new_target_location_fee = Some(MultiAsset {
			id: Concrete(MultiLocation::parent()),
			fun: Fungible(1_000_0000),
		});
	}: _<T::RuntimeOrigin>(
		origin,
		bridged_network,
		Box::new(allowed_bridged_target_location.location.clone().into_versioned()),
		new_target_location_fee.clone().map(|fee| Box::new(VersionedMultiAsset::V3(fee)))
	)
	verify {
		let bridge_config = AllowedExporters::<T>::get(bridged_network).unwrap();
		assert!(
			bridge_config.allowed_target_location_for(&allowed_bridged_target_location.location).expect("stored target_location").is_some()
		);
	}

	allow_reserve_asset_transfer_for {
		let asset_filter = AssetFilterOf::<T>::All;

		let origin = T::AllowReserveAssetTransferOrigin::try_successful_origin(&asset_filter).map_err(|_| BenchmarkError::Weightless)?;
		if T::TargetLocationsPerExporterLimit::get() <= 0 {
			return Err(BenchmarkError::Weightless);
		}

		let (bridged_network, maybe_paid_bridge) = T::BenchmarkHelper::bridge_location()
			.ok_or(BenchmarkError::Stop("missing `bridge_config` data"))?;
		let allowed_bridged_target_location = T::BenchmarkHelper::allowed_bridged_target_location()
			.ok_or(BenchmarkError::Stop("missing `allowed_bridged_target_location` data"))?;
		let mut bridge_config = BridgeConfig::new(
			maybe_paid_bridge.location.into_versioned(),
			None
		);
		assert_ok!(bridge_config.update_allowed_target_location(
			allowed_bridged_target_location.location.into_versioned(),
			allowed_bridged_target_location.maybe_fee.map(|fee| VersionedMultiAsset::V3(fee))
		));
		AllowedExporters::<T>::insert(bridged_network, bridge_config);

	}: _<T::RuntimeOrigin>(
		origin,
		bridged_network,
		Box::new(allowed_bridged_target_location.location.clone().into_versioned()),
		asset_filter.clone()
	)
	verify {
		let bridge_config = AllowedExporters::<T>::get(bridged_network).unwrap();
		let allowed_target_location = bridge_config.allowed_target_location_for(
			&allowed_bridged_target_location.location
		).expect("stored target_location").unwrap();
		assert_eq!(
			allowed_target_location.1,
			Some(asset_filter)
		);
	}

	add_universal_alias {
		let origin = T::AdminOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let (location, junction) = match T::BenchmarkHelper::universal_alias() {
			Some(alias) => alias,
			None => match T::UniversalAliasesLimit::get() > 0_u32 {
				true => return Err(BenchmarkError::Stop("missing `universal_alias` data")),
				false => return Err(BenchmarkError::Weightless),
			}
		};
	}: _<T::RuntimeOrigin>(origin, Box::new(location.clone()), junction)
	verify {
		assert!(
			AllowedUniversalAliases::<T>::get(LatestVersionedMultiLocation::try_from(&location).expect("ok")).contains(&junction)
		);
	}

	remove_universal_alias {
		let origin = T::AdminOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let (location, junction) = match T::BenchmarkHelper::universal_alias() {
			Some(alias) => alias,
			None => match T::UniversalAliasesLimit::get() > 0_u32 {
				true => return Err(BenchmarkError::Stop("missing `universal_alias` data")),
				false => return Err(BenchmarkError::Weightless),
			}
		};
		Pallet::<T>::insert_universal_alias_for_benchmarks((location.clone().try_into().unwrap(), junction));
	}: _<T::RuntimeOrigin>(origin, Box::new(location.clone()), vec![junction.clone()])
	verify {
		assert!(!AllowedUniversalAliases::<T>::get(LatestVersionedMultiLocation::try_from(&location).expect("ok")).contains(&junction));
	}

	add_reserve_location {
		let origin = T::AdminOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let reserve_location = if T::ReserveLocationsLimit::get() > 0_u32 {
			match T::BenchmarkHelper::reserve_location() {
				Some(location) => location,
				None => return Err(BenchmarkError::Stop("missing `reserve_location` data")),
			}
		} else {
			return Err(BenchmarkError::Weightless);
		};
		let asset_filter: AssetFilterOf<T> = if T::AssetsPerReserveLocationLimit::get() > 0_u32 {
			let assets = (0..T::AssetsPerReserveLocationLimit::get())
				.map(|i| MultiLocation::new(1, X1(Parachain(i))).into_versioned())
				.collect::<Vec<_>>();
			MultiLocationFilter {
				equals_any: BoundedVec::truncate_from(assets),
				starts_with_any: Default::default(),
			}.into()
		} else {
			return Err(BenchmarkError::Weightless);
		};
	}: _<T::RuntimeOrigin>(origin, Box::new(reserve_location.clone()), asset_filter.clone())
	verify {
		assert_eq!(
			AllowedReserveLocations::<T>::get(LatestVersionedMultiLocation::try_from(&reserve_location).expect("ok")),
			Some(asset_filter)
		);
	}

	remove_reserve_location {
		let origin = T::AdminOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let reserve_location = if T::ReserveLocationsLimit::get() > 0_u32 {
			match T::BenchmarkHelper::reserve_location() {
				Some(location) => location,
				None => return Err(BenchmarkError::Stop("missing `reserve_location` data")),
			}
		} else {
			return Err(BenchmarkError::Weightless);
		};
		AllowedReserveLocations::<T>::insert(reserve_location.clone(), AssetFilterOf::<T>::All);
		assert!(AllowedReserveLocations::<T>::contains_key(LatestVersionedMultiLocation::try_from(&reserve_location).expect("ok")));
	}: _<T::RuntimeOrigin>(origin, Box::new(reserve_location.clone()), None)
	verify {
		assert!(!AllowedReserveLocations::<T>::contains_key(LatestVersionedMultiLocation::try_from(&reserve_location).expect("ok")));
	}

	impl_benchmark_test_suite!(Pallet, crate::tests::new_test_ext(), crate::tests::TestRuntime);
}
