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

use crate::{
	types::{AssetFilterT, LatestVersionedMultiLocation},
	AllowedExporters, Config, Pallet, LOG_TARGET,
};
use frame_support::traits::{Contains, ContainsPair};
use pallet_bridge_transfer_primitives::{
	EnsureReachableDestination, MaybePaidLocation, ReachableDestination, ReachableDestinationError,
};
use xcm::prelude::*;
use xcm_builder::ExporterFor;

/// Implementation of `EnsureReachableDestination` which checks on-chain configuration with `AllowedExporters`
impl<T: Config> EnsureReachableDestination for Pallet<T> {
	fn ensure_destination(
		remote_network: NetworkId,
		remote_destination: MultiLocation,
	) -> Result<ReachableDestination, ReachableDestinationError> {
		match AllowedExporters::<T>::get(&remote_network) {
			Some(bridge_config) => {
				match bridge_config.allowed_target_location_for(&remote_destination) {
					Ok(Some((target_location, _))) => Ok(ReachableDestination {
						bridge: bridge_config.to_bridge_location()?,
						target: target_location,
						target_destination: remote_destination,
					}),
					Ok(None) => Err(ReachableDestinationError::UnsupportedDestination),
					Err(_) => Err(ReachableDestinationError::UnsupportedXcmVersion),
				}
			},
			None => Err(ReachableDestinationError::UnsupportedDestination),
		}
	}
}

/// `ExporterFor` implementation to check if we can transfer anything to `NetworkId`
impl<T: Config> ExporterFor for Pallet<T> {
	fn exporter_for(
		network: &NetworkId,
		_remote_location: &InteriorMultiLocation,
		_message: &Xcm<()>,
	) -> Option<(MultiLocation, Option<MultiAsset>)> {
		match Self::allowed_exporters(network) {
			Some(bridge_config) => match bridge_config.to_bridge_location() {
				Ok(MaybePaidLocation { location, maybe_fee }) => Some((location, maybe_fee)),
				Err(e) => {
					log::warn!(
						target: LOG_TARGET,
						"Exporter for network: {:?} has error: {:?}!",
						network, e
					);
					None
				},
			},
			None => None,
		}
	}
}

/// Verifies if we have `(MultiLocation, Junction)` in allowed universal aliases.
pub struct AllowedUniversalAliasesOf<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> Contains<(MultiLocation, Junction)> for AllowedUniversalAliasesOf<T> {
	fn contains((location, junction): &(MultiLocation, Junction)) -> bool {
		log::trace!(
			target: LOG_TARGET,
			"AllowedUniversalAliasesOf location: {:?}, junction: {:?}", location, junction
		);

		Pallet::<T>::allowed_universal_aliases(LatestVersionedMultiLocation(location))
			.contains(junction)
	}
}

/// Verifies if we can allow `(MultiAsset, MultiLocation)` as trusted reserve received from "bridge".
pub struct IsTrustedBridgedReserveForConcreteAsset<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> ContainsPair<MultiAsset, MultiLocation>
	for IsTrustedBridgedReserveForConcreteAsset<T>
{
	fn contains(asset: &MultiAsset, origin: &MultiLocation) -> bool {
		log::trace!(
			target: LOG_TARGET,
			"IsTrustedBridgedReserveForConcreteAsset asset: {:?}, origin: {:?}",
			asset, origin
		);

		let asset_location = match &asset.id {
			Concrete(location) => location,
			_ => return false,
		};

		let versioned_origin = LatestVersionedMultiLocation(origin);
		Pallet::<T>::allowed_reserve_locations(versioned_origin)
			.map_or(false, |filter| match filter.matches(asset_location) {
				Ok(result) => result,
				Err(e) => {
					log::error!(
						target: LOG_TARGET,
						"IsTrustedBridgedReserveForConcreteAsset has error: {:?} for asset_location: {:?} and origin: {:?}!",
						e, asset_location, origin
					);
					false
				},
			})
	}
}

/// Verifies if it is allowed for `(MultiAsset, MultiLocation)` to be transferred out as reserve based transfer over "bridge".
pub struct IsAllowedReserveBasedAssetTransferForConcreteAsset<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> ContainsPair<MultiAsset, MultiLocation>
	for IsAllowedReserveBasedAssetTransferForConcreteAsset<T>
{
	fn contains(asset: &MultiAsset, target_location: &MultiLocation) -> bool {
		log::trace!(
			target: LOG_TARGET,
			"IsAllowedReserveBasedAssetTransferForConcreteAsset asset: {:?}, target_location: {:?}",
			asset, target_location
		);

		let asset_location = match &asset.id {
			Concrete(location) => location,
			_ => return false,
		};

		match target_location.interior.global_consensus() {
			Ok(target_consensus) => {
				if let Some(bridge_config) = Pallet::<T>::allowed_exporters(target_consensus) {
					match bridge_config.allowed_target_location_for(target_location) {
						Ok(Some((_, Some(asset_filter)))) => {
							match asset_filter.matches(asset_location) {
								Ok(matches) => matches,
								Err(e) => {
									log::error!(
										target: LOG_TARGET,
										"IsAllowedReserveBasedAssetTransferForConcreteAsset `asset_filter.matches` has error: {:?} for asset_location: {:?} and target_location: {:?}!",
										e, asset_location, target_location
									);
									false
								},
							}
						},
						Ok(_) => false,
						Err(e) => {
							log::warn!(
								target: LOG_TARGET,
								"IsAllowedReserveBasedAssetTransferForConcreteAsset allowed_target_location_for has error: {:?} for target_location: {:?}!",
								e, target_location
							);
							false
						},
					}
				} else {
					log::warn!(
						target: LOG_TARGET,
						"IsAllowedReserveBasedAssetTransferForConcreteAsset missing configuration for target_consensus: {:?} of target_location: {:?}!",
						target_consensus, target_location
					);
					false
				}
			},
			Err(_) => {
				log::warn!(
					target: LOG_TARGET,
					"IsAllowedReserveBasedAssetTransferForConcreteAsset target_location: {:?} does not contain global consensus!",
					target_location
				);
				false
			},
		}
	}
}
