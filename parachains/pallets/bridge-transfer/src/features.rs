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
	types::{AssetTransferKind, ResolveAssetTransferKind},
	LOG_TARGET,
};
use frame_support::traits::{ContainsPair, Get};
use pallet_bridge_transfer_primitives::{BridgesConfig, MatchAssetLocation};
use xcm::prelude::*;
use xcm_builder::ensure_is_remote;
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

/// Adapter verifies if it is allowed to transfer out `MultiAsset` to the `MultiLocation`.
///
/// Note: `MultiLocation` has to be from different global consensus.
pub struct IsAllowedReserveBasedTransferForConcreteAssetToBridgedLocation<
	UniversalLocation,
	Bridges,
>(sp_std::marker::PhantomData<(UniversalLocation, Bridges)>);
impl<UniversalLocation: Get<InteriorMultiLocation>, Bridges: Get<BridgesConfig>>
	ContainsPair<MultiAsset, MultiLocation>
	for IsAllowedReserveBasedTransferForConcreteAssetToBridgedLocation<UniversalLocation, Bridges>
{
	fn contains(asset: &MultiAsset, to_location: &MultiLocation) -> bool {
		let universal_source = UniversalLocation::get();
		log::trace!(
			target: "xcm::contains",
			"IsAllowedReserveBasedTransferForConcreteAssetToBridgedLocation asset: {:?}, to_location: {:?}, universal_source: {:?}",
			asset, to_location, universal_source
		);

		// check remote origin
		let (remote_network, _) = match ensure_is_remote(universal_source, *to_location) {
			Ok(devolved) => devolved,
			Err(_) => {
				log::trace!(
					target: "xcm::contains",
					"IsAllowedReserveBasedTransferForConcreteAssetToBridgedLocation to_location: {:?} is not remote to the universal_source: {:?}",
					to_location, universal_source
				);
				return false
			},
		};

		// check asset location
		let asset_location = match &asset.id {
			Concrete(location) => location,
			_ => return false,
		};

		// check asset according to the config
		if let Some(config) = Bridges::get().get(&remote_network) {
			if let Some((_, Some(asset_filter))) = config.allowed_target_location_for(&to_location)
			{
				return asset_filter.matches(asset_location)
			}
		}

		false
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
