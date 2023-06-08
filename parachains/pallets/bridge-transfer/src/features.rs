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
	AssetTransferKind, BridgeConfig, Config, Pallet, ResolveAssetTransferKind, UsingVersioned,
	LOG_TARGET,
};
use frame_support::traits::{Contains, ContainsPair};
use xcm::prelude::*;
use xcm_builder::ExporterFor;

/// `ExporterFor` implementation to check if we can transfer anything to `NetworkId`
impl<T: Config> ExporterFor for Pallet<T> {
	fn exporter_for(
		network: &NetworkId,
		_remote_location: &InteriorMultiLocation,
		_message: &Xcm<()>,
	) -> Option<(MultiLocation, Option<MultiAsset>)> {
		match Self::allowed_exporters(network) {
			Some(BridgeConfig { bridge_location, bridge_location_fee, .. }) => {
				let bridge_location = match bridge_location.to_versioned() {
					Ok(versioned) => versioned,
					Err(e) => {
						log::warn!(
							target: LOG_TARGET,
							"Exporter for network: {:?} - bridge_location has error: {:?}!",
							network, e
						);
						return None
					},
				};
				let bridge_location_fee = match bridge_location_fee {
					Some(fee) => match fee.to_versioned() {
						Ok(versioned) => Some(versioned),
						Err(e) => {
							log::warn!(
								target: LOG_TARGET,
								"ExporterFor network: {:?} - bridge_location_fee has error: {:?}!",
								network, e
							);
							return None
						},
					},
					None => None,
				};
				Some((bridge_location, bridge_location_fee))
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

/// Implementation of `ResolveTransferKind` which tries to resolve all kinds of transfer.
pub struct ConcreteAssetTransferKindResolver<
	IsReserveLocationForAsset,
	IsAllowedReserveBasedTransferForAsset,
>(sp_std::marker::PhantomData<(IsReserveLocationForAsset, IsAllowedReserveBasedTransferForAsset)>);

impl<
		IsReserveLocationForAsset: ContainsPair<MultiAsset, MultiLocation>,
		IsAllowedReserveBasedTransferForAsset: ContainsPair<MultiAsset, MultiLocation>,
	> ResolveAssetTransferKind
	for ConcreteAssetTransferKindResolver<
		IsReserveLocationForAsset,
		IsAllowedReserveBasedTransferForAsset,
	>
{
	fn resolve(asset: &MultiAsset, target_location: &MultiLocation) -> AssetTransferKind {
		log::trace!(
			target: LOG_TARGET,
			"ConcreteAssetTransferKindResolver resolve asset: {:?}, target_location: {:?}",
			asset, target_location
		);

		// accepts only Concrete
		match &asset.id {
			Concrete(_) => (),
			_ => return AssetTransferKind::Unsupported,
		};

		// first, we check, if we are trying to transfer back reserve-deposited assets
		// other words: if target_location is a known reserve location for asset
		if IsReserveLocationForAsset::contains(asset, target_location) {
			return AssetTransferKind::WithdrawReserve
		}

		// second, we check, if target_location is allowed for requested asset to be transferred there
		if IsAllowedReserveBasedTransferForAsset::contains(asset, target_location) {
			return AssetTransferKind::ReserveBased
		}

		AssetTransferKind::Unsupported
	}
}
