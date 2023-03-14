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

//! `BridgeAssetsTransfer` pallet benchmarks.

use crate::{BenchmarkHelper, Bridges, Call, Config, Event, Pallet};

use frame_benchmarking::{benchmarks, BenchmarkError};
use frame_support::traits::EnsureOrigin;
use sp_std::prelude::*;

benchmarks! {
	transfer_asset_via_bridge {
		let (bridged_network, bridge_config) = T::BenchmarkHelper::bridge_config();
		let (origin, assets, destination) = T::BenchmarkHelper::prepare_transfer();
		Bridges::<T>::insert(bridged_network, bridge_config);
	}: _<T::RuntimeOrigin>(origin, Box::new(assets), Box::new(destination))
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
