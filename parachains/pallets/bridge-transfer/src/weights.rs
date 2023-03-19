// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Weights trait for the `pallet_bridge_assets_transfer` pallet.

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_bridge_assets_transfer.
pub trait WeightInfo {
	/// Weight of the `transfer_asset_via_bridge` call.
	fn transfer_asset_via_bridge() -> Weight;
	/// Weight of the `ping_via_bridge` call.
	fn ping_via_bridge() -> Weight;

	/// Weight of the `add_bridge_config` call.
	fn add_bridge_config() -> Weight;
	/// Weight of the `remove_bridge_config` call.
	fn remove_bridge_config() -> Weight;
	/// Weight of the `update_bridge_config` call.
	fn update_bridge_config() -> Weight;
}

// Zero weights to use in tests
impl WeightInfo for () {
	fn transfer_asset_via_bridge() -> Weight {
		Weight::zero()
	}

	fn ping_via_bridge() -> Weight {
		Weight::zero()
	}

	fn add_bridge_config() -> Weight {
		Weight::zero()
	}

	fn remove_bridge_config() -> Weight {
		Weight::zero()
	}

	fn update_bridge_config() -> Weight {
		Weight::zero()
	}
}
