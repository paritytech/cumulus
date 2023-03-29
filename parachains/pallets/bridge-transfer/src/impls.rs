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

use crate::{Config, Pallet};
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
		Self::allowed_exporters(network)
			.map(|bridge_config| (bridge_config.bridge_location, bridge_config.bridge_location_fee))
	}
}

/// Verifies if we have `(MultiLocation, Junction)` in allowed universal aliases.
pub struct AllowedUniversalAliasesOf<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> Contains<(MultiLocation, Junction)> for AllowedUniversalAliasesOf<T> {
	fn contains((location, junction): &(MultiLocation, Junction)) -> bool {
		Pallet::<T>::allowed_universal_aliases(location).contains(junction)
	}
}

/// Verifies if we can allow `(MultiAsset, MultiLocation)` as trusted reserve.
pub struct IsAllowedReserveOf<T, F>(sp_std::marker::PhantomData<(T, F)>);
impl<T: Config, F: ContainsPair<MultiAsset, MultiLocation>> ContainsPair<MultiAsset, MultiLocation>
	for IsAllowedReserveOf<T, F>
{
	fn contains(asset: &MultiAsset, origin: &MultiLocation) -> bool {
		log::trace!(target: "xcm::contains", "IsAllowedReserveOf asset: {:?}, origin: {:?}", asset, origin);
		// first check - if we have configured origin as trusted reserve location
		if !Pallet::<T>::allowed_reserve_locations().contains(origin) {
			return false
		}
		// second check - we need to pass additional `(asset, origin)` filter
		F::contains(asset, origin)
	}
}
