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

use crate::{BenchmarkHelper, Bridges, Call, Config, Event, Pallet, PingMessageBuilder};

use frame_benchmarking::{benchmarks, BenchmarkError, BenchmarkResult};
use frame_support::{traits::EnsureOrigin, weights::Weight};
use sp_std::prelude::*;

benchmarks! {
	transfer_asset_via_bridge {
		// every asset has its own configuration and ledger, so there's a performance dependency
		// TODO: add proper range after once pallet works with multiple assets
		// (be sure to use "worst" of assets)
		// let a in 1 .. 1;
		let (bridged_network, bridge_config) = T::BenchmarkHelper::bridge_config();
		let (origin, assets, destination) = T::BenchmarkHelper::prepare_asset_transfer(1);
		Bridges::<T>::insert(bridged_network, bridge_config);
	}: _<T::RuntimeOrigin>(origin, Box::new(assets), Box::new(destination))
	verify {
		// we don't care about message hash here, just check that the transfer has been initiated
		let actual_event = frame_system::Pallet::<T>::events().pop().map(|r| r.event);
		let expected_event: <T as Config>::RuntimeEvent = Event::TransferInitiated(Default::default()).into();
		assert!(matches!(actual_event, Some(expected_event)));
	}

	ping_via_bridge {
		let (bridged_network, bridge_config) = T::BenchmarkHelper::bridge_config();
		Bridges::<T>::insert(bridged_network, bridge_config);

		let (origin, destination) = T::BenchmarkHelper::prepare_ping();

		let origin_location = T::TransferPingOrigin::ensure_origin(origin.clone()).map_err(|_|
			BenchmarkError::Override(BenchmarkResult::from_weight(Weight::MAX)),
		)?;
		let (_, _, destination_location) = Pallet::<T>::ensure_remote_destination(destination.clone()).map_err(|_|
			BenchmarkError::Override(BenchmarkResult::from_weight(Weight::MAX)),
		)?;
		let _ = T::PingMessageBuilder::try_build(&origin_location, &bridged_network, &destination_location).ok_or(
			BenchmarkError::Override(BenchmarkResult::from_weight(Weight::MAX)),
		)?;
	}: _<T::RuntimeOrigin>(origin, Box::new(destination))
	verify {
		// we don't care about message hash here, just check that the transfer has been initiated
		let actual_event = frame_system::Pallet::<T>::events().pop().map(|r| r.event);
		let expected_event: <T as Config>::RuntimeEvent = Event::TransferInitiated(Default::default()).into();
		assert!(matches!(actual_event, Some(expected_event)));
	}

	add_bridge_config {
		let origin = T::AdminOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let (bridged_network, bridge_config) = T::BenchmarkHelper::bridge_config();
	}: _<T::RuntimeOrigin>(origin, bridged_network, Box::new(bridge_config.clone()))
	verify {
		assert_eq!(Bridges::<T>::get(bridged_network), Some(bridge_config));
	}

	remove_bridge_config {
		let origin = T::AdminOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let (bridged_network, bridge_config) = T::BenchmarkHelper::bridge_config();
		Bridges::<T>::insert(bridged_network, bridge_config);
	}: _<T::RuntimeOrigin>(origin, bridged_network)
	verify {
		assert_eq!(Bridges::<T>::get(bridged_network), None);
	}

	update_bridge_config {
		let origin = T::AdminOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let (bridged_network, bridge_config) = T::BenchmarkHelper::bridge_config();
		Bridges::<T>::insert(bridged_network, bridge_config);

		let fee = None;
	}: _<T::RuntimeOrigin>(origin, bridged_network, fee)
	verify {
		assert_eq!(Bridges::<T>::get(bridged_network).unwrap().fee, None);
	}

	impl_benchmark_test_suite!(Pallet, crate::tests::new_test_ext(), crate::tests::TestRuntime);
}
