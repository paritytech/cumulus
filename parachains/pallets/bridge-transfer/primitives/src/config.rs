// Copyright (C) 2023 Parity Technologies (UK) Ltd.
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

//! Primitives for easier configuration.

use crate::{
	AssetFilter, EnsureReachableDestination, MaybePaidLocation, ReachableDestination,
	ReachableDestinationError,
};
use frame_support::{
	traits::{ConstU32, Get},
	BoundedBTreeMap,
};
use xcm::prelude::*;
use xcm_builder::ExporterFor;

/// Configuration represents bridge to target location with possible asset filtering
#[derive(Debug)]
pub struct BridgeConfig {
	/// Location, which is able to bridge XCM messages to bridged network
	pub bridge_location: MaybePaidLocation,
	/// Contains target destination on bridged network. E.g.: MultiLocation of Statemine/t on different consensus + its configuration
	pub allowed_target_locations: sp_std::vec::Vec<(MaybePaidLocation, Option<AssetFilter>)>,
}

impl BridgeConfig {
	pub fn new(bridge_location: MaybePaidLocation) -> Self {
		Self { bridge_location, allowed_target_locations: Default::default() }
	}

	pub fn add_target_location(
		mut self,
		target_location: MaybePaidLocation,
		asset_filter: Option<AssetFilter>,
	) -> Self {
		self.allowed_target_locations.push((target_location, asset_filter));
		self
	}

	pub fn allowed_target_location_for(
		&self,
		destination: &MultiLocation,
	) -> Option<(&MaybePaidLocation, &Option<AssetFilter>)> {
		for (allowed_target_location, asset_filter) in &self.allowed_target_locations {
			if destination.starts_with(&allowed_target_location.location) ||
				destination.eq(&allowed_target_location.location)
			{
				return Some((allowed_target_location, asset_filter))
			}
		}
		None
	}
}

/// Type represents all "stored" `BridgeConfig`s.
pub type BridgesConfig = BoundedBTreeMap<NetworkId, BridgeConfig, ConstU32<24>>;

/// Builder for creating `BridgesConfig`.
#[derive(Default)]
pub struct BridgesConfigBuilder {
	configs: BoundedBTreeMap<NetworkId, BridgeConfig, ConstU32<24>>,
}
impl BridgesConfigBuilder {
	pub fn add_or_panic(mut self, network_id: NetworkId, bridge_config: BridgeConfig) -> Self {
		self.configs.try_insert(network_id, bridge_config).expect("check bounds");
		self
	}

	pub fn build(self) -> BoundedBTreeMap<NetworkId, BridgeConfig, ConstU32<24>> {
		self.configs
	}
}

/// Adapter for accessing `BridgesConfig`.
pub struct BridgesConfigAdapter<Bridges>(sp_std::marker::PhantomData<Bridges>);

/// Adapter implementation of `ExporterFor` for `BridgesConfig`
impl<Bridges: Get<BridgesConfig>> ExporterFor for BridgesConfigAdapter<Bridges> {
	fn exporter_for(
		network: &NetworkId,
		_remote_location: &InteriorMultiLocation,
		_message: &Xcm<()>,
	) -> Option<(MultiLocation, Option<MultiAsset>)> {
		Bridges::get().get(network).map(|config| {
			(config.bridge_location.location, config.bridge_location.maybe_fee.clone())
		})
	}
}

/// Adapter implementation of `EnsureReachableDestination` for `BridgesConfig`
impl<Bridges: Get<BridgesConfig>> EnsureReachableDestination for BridgesConfigAdapter<Bridges> {
	fn ensure_destination(
		remote_network: NetworkId,
		remote_destination: MultiLocation,
	) -> Result<ReachableDestination, ReachableDestinationError> {
		if let Some(config) = Bridges::get().get(&remote_network) {
			if let Some((allowed_target_location, _)) =
				config.allowed_target_location_for(&remote_destination)
			{
				return Ok(ReachableDestination {
					bridge: config.bridge_location.clone(),
					target: allowed_target_location.clone(),
					target_destination: remote_destination,
				})
			}
		}
		Err(ReachableDestinationError::UnsupportedDestination)
	}
}
