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
	AllowedExporters, AllowedReserveLocations, AllowedUniversalAliases, BenchmarkHelper, Call,
	Config, Event, Pallet, PingMessageBuilder,
};

use frame_benchmarking::{benchmarks, BenchmarkError};
use frame_support::traits::{EnsureOrigin, Get};
use sp_std::prelude::*;
use xcm::prelude::*;

#[cfg(feature = "runtime-benchmarks")]
impl<T: Config> Pallet<T> {
	#[cfg(feature = "runtime-benchmarks")]
	pub fn insert_universal_alias_for_benchmarks((location, junction): (MultiLocation, Junction)) {
		assert!(matches!(
			AllowedUniversalAliases::<T>::try_mutate(location, |junctions| junctions
				.try_insert(junction)),
			Ok(true)
		));
	}
}

benchmarks! {
	transfer_asset_via_bridge {
		let _ = T::TransferAssetOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		// every asset has its own configuration and ledger, so there's a performance dependency
		// TODO: add proper range after once pallet works with multiple assets
		// (be sure to use "worst" of assets)
		// let a in 1 .. 1;
		let (bridged_network, bridge_config) = T::BenchmarkHelper::bridge_config()
			.ok_or(BenchmarkError::Stop("missing `bridge_config` data"))?;
		let (origin, assets, destination) = T::BenchmarkHelper::prepare_asset_transfer(1)
			.ok_or(BenchmarkError::Stop("missing `prepare_asset_transfer` data"))?;
		AllowedExporters::<T>::insert(bridged_network, bridge_config);
	}: _<T::RuntimeOrigin>(origin, Box::new(assets), Box::new(destination))
	verify {
		// we don't care about message hash here, just check that the transfer has been initiated
		let actual_event = frame_system::Pallet::<T>::events().pop().map(|r| r.event);
		let expected_event: <T as Config>::RuntimeEvent = Event::TransferInitiated(Default::default()).into();
		assert!(matches!(actual_event, Some(expected_event)));
	}

	ping_via_bridge {
		let _ = T::TransferPingOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;

		let (bridged_network, bridge_config) = T::BenchmarkHelper::bridge_config()
			.ok_or(BenchmarkError::Stop("missing `bridge_config` data"))?;
		AllowedExporters::<T>::insert(bridged_network, bridge_config);

		let (origin, destination) = T::BenchmarkHelper::prepare_ping_transfer()
			.ok_or(BenchmarkError::Stop("missing `prepare_ping_transfer` data"))?;

		let origin_location = T::TransferPingOrigin::ensure_origin(origin.clone()).map_err(|_| BenchmarkError::Stop("invalid `origin`"),
		)?;
		let (_, _, destination_location) = Pallet::<T>::ensure_remote_destination(destination.clone()).map_err(|_|
			BenchmarkError::Stop("invalid `destination_location`"),
		)?;
		let _ = T::PingMessageBuilder::try_build(&origin_location, &bridged_network, &destination_location).ok_or(
			BenchmarkError::Stop("invalid `PingMessageBuilder`"),
		)?;
	}: _<T::RuntimeOrigin>(origin, Box::new(destination))
	verify {
		// we don't care about message hash here, just check that the transfer has been initiated
		let actual_event = frame_system::Pallet::<T>::events().pop().map(|r| r.event);
		let expected_event: <T as Config>::RuntimeEvent = Event::TransferInitiated(Default::default()).into();
		assert!(matches!(actual_event, Some(expected_event)));
	}

	add_exporter_config {
		let origin = T::AdminOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let (bridged_network, bridge_config) = T::BenchmarkHelper::bridge_config()
			.ok_or(BenchmarkError::Stop("missing `bridge_config` data"))?;
	}: _<T::RuntimeOrigin>(origin, bridged_network, Box::new(bridge_config.clone()))
	verify {
		assert_eq!(AllowedExporters::<T>::get(bridged_network), Some(bridge_config));
	}

	remove_exporter_config {
		let origin = T::AdminOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let (bridged_network, bridge_config) = T::BenchmarkHelper::bridge_config()
			.ok_or(BenchmarkError::Stop("missing `bridge_config` data"))?;
		AllowedExporters::<T>::insert(bridged_network, bridge_config);
	}: _<T::RuntimeOrigin>(origin, bridged_network)
	verify {
		assert_eq!(AllowedExporters::<T>::get(bridged_network), None);
	}

	update_exporter_config {
		let origin = T::AdminOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let (bridged_network, bridge_config) = T::BenchmarkHelper::bridge_config()
			.ok_or(BenchmarkError::Stop("missing `bridge_config` data"))?;
		AllowedExporters::<T>::insert(bridged_network, bridge_config);

		let bridge_location_fee = None;
		let target_location_fee = Some(xcm::VersionedMultiAsset::V3(MultiAsset {
			id: Concrete(MultiLocation::parent()),
			fun: Fungible(1_000_0000),
		}));
	}: _<T::RuntimeOrigin>(origin, bridged_network, bridge_location_fee.clone().map(Box::new), target_location_fee.clone().map(Box::new))
	verify {
		let exporter = AllowedExporters::<T>::get(bridged_network).unwrap();
		assert_eq!(exporter.bridge_location_fee, bridge_location_fee.map(|fee| MultiAsset::try_from(fee).unwrap()));
		assert_eq!(exporter.target_location_fee, target_location_fee.map(|fee| MultiAsset::try_from(fee).unwrap()));
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
		assert!(AllowedUniversalAliases::<T>::get(&location.try_as().unwrap()).contains(&junction));
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
		assert!(!AllowedUniversalAliases::<T>::get(&location.try_as().unwrap()).contains(&junction));
	}

	add_reserve_location {
		let origin = T::AdminOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let location = match T::BenchmarkHelper::reserve_location() {
			Some(location) => location,
			None => match T::ReserveLocationsLimit::get() > 0_u32 {
				true => return Err(BenchmarkError::Stop("missing `reserve_location` data")),
				false => return Err(BenchmarkError::Weightless),
			}
		};
	}: _<T::RuntimeOrigin>(origin, Box::new(location.clone()))
	verify {
		assert!(AllowedReserveLocations::<T>::get().contains(&location.try_as().unwrap()));
	}

	remove_reserve_location {
		let origin = T::AdminOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let location = match T::BenchmarkHelper::reserve_location() {
			Some(location) => location,
			None => match T::ReserveLocationsLimit::get() > 0_u32 {
				true => return Err(BenchmarkError::Stop("missing `reserve_location` data")),
				false => return Err(BenchmarkError::Weightless),
			}
		};
		let multilocation: MultiLocation = location.clone().try_into().unwrap();
		assert!(AllowedReserveLocations::<T>::try_mutate(|locations| locations.try_insert(multilocation)).unwrap());
	}: _<T::RuntimeOrigin>(origin, vec![location.clone()])
	verify {
		assert!(!AllowedReserveLocations::<T>::get().contains(&location.try_as().unwrap()));
	}

	impl_benchmark_test_suite!(Pallet, crate::tests::new_test_ext(), crate::tests::TestRuntime);
}
