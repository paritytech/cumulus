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

use cumulus_primitives_core::ParaId;
use frame_support::{
	pallet_prelude::Get,
	traits::{Contains, ContainsPair},
};
use xcm::{
	latest::prelude::{MultiAsset, MultiLocation},
	prelude::*,
};
use xcm_builder::ensure_is_remote;

pub struct StartsWith<T>(sp_std::marker::PhantomData<T>);
impl<Location: Get<MultiLocation>> Contains<MultiLocation> for StartsWith<Location> {
	fn contains(t: &MultiLocation) -> bool {
		t.starts_with(&Location::get())
	}
}

pub struct Equals<T>(sp_std::marker::PhantomData<T>);
impl<Location: Get<MultiLocation>> Contains<MultiLocation> for Equals<Location> {
	fn contains(t: &MultiLocation) -> bool {
		t == &Location::get()
	}
}

pub struct StartsWithExplicitGlobalConsensus<T>(sp_std::marker::PhantomData<T>);
impl<Network: Get<NetworkId>> Contains<MultiLocation>
	for StartsWithExplicitGlobalConsensus<Network>
{
	fn contains(t: &MultiLocation) -> bool {
		matches!(t.interior.global_consensus(), Ok(requested_network) if requested_network.eq(&Network::get()))
	}
}

frame_support::parameter_types! {
	pub LocalMultiLocationPattern: MultiLocation = MultiLocation::new(0, Here);
	pub ParentLocation: MultiLocation = MultiLocation::parent();
}

/// Accepts an asset if it is from the origin.
pub struct IsForeignConcreteAsset<IsForeign>(sp_std::marker::PhantomData<IsForeign>);
impl<IsForeign: ContainsPair<MultiLocation, MultiLocation>> ContainsPair<MultiAsset, MultiLocation>
	for IsForeignConcreteAsset<IsForeign>
{
	fn contains(asset: &MultiAsset, origin: &MultiLocation) -> bool {
		log::trace!(target: "xcm::contains", "IsForeignConcreteAsset asset: {:?}, origin: {:?}", asset, origin);
		matches!(asset.id, Concrete(ref id) if IsForeign::contains(id, origin))
	}
}

/// Checks if `a` is from sibling location `b`. Checks that `MultiLocation-a` starts with
/// `MultiLocation-b`, and that the `ParaId` of `b` is not equal to `a`.
pub struct FromSiblingParachain<SelfParaId>(sp_std::marker::PhantomData<SelfParaId>);
impl<SelfParaId: Get<ParaId>> ContainsPair<MultiLocation, MultiLocation>
	for FromSiblingParachain<SelfParaId>
{
	fn contains(&a: &MultiLocation, b: &MultiLocation) -> bool {
		// `a` needs to be from `b` at least
		if !a.starts_with(b) {
			return false
		}

		// here we check if sibling
		match a {
			MultiLocation { parents: 1, interior } =>
				matches!(interior.first(), Some(Parachain(sibling_para_id)) if sibling_para_id.ne(&u32::from(SelfParaId::get()))),
			_ => false,
		}
	}
}

/// Adapter verifies if it is allowed to receive `MultiAsset` from `MultiLocation`.
///
/// Note: `MultiLocation` has to be from different global consensus.
pub struct IsTrustedBridgedReserveLocationForConcreteAsset<UniversalLocation, Reserves>(
	sp_std::marker::PhantomData<(UniversalLocation, Reserves)>,
);
impl<
		UniversalLocation: Get<InteriorMultiLocation>,
		Reserves: Get<sp_std::vec::Vec<FilteredReserveLocation>>,
	> ContainsPair<MultiAsset, MultiLocation>
	for IsTrustedBridgedReserveLocationForConcreteAsset<UniversalLocation, Reserves>
{
	fn contains(asset: &MultiAsset, origin: &MultiLocation) -> bool {
		let universal_source = UniversalLocation::get();
		log::trace!(
			target: "xcm::contains",
			"IsTrustedBridgedReserveLocationForConcreteAsset asset: {:?}, origin: {:?}, universal_source: {:?}",
			asset, origin, universal_source
		);

		// check remote origin
		let _ = match ensure_is_remote(universal_source, *origin) {
			Ok(devolved) => devolved,
			Err(_) => {
				log::trace!(
					target: "xcm::contains",
					"IsTrustedBridgedReserveLocationForConcreteAsset origin: {:?} is not remote to the universal_source: {:?}",
					origin, universal_source
				);
				return false
			},
		};

		// check asset location
		let asset_location = match &asset.id {
			Concrete(location) => location,
			_ => return false,
		};

		// check asset according to the configured reserve locations
		for (reserve_location, asset_filter) in Reserves::get() {
			if origin.eq(&reserve_location) {
				if asset_filter.matches(asset_location) {
					return true
				}
			}
		}

		false
	}
}

/// Reserve location as `MultiLocation` with `AssetFilter`
pub type FilteredReserveLocation = (MultiLocation, AssetFilter);

/// Trait for matching asset location
pub trait MatchAssetLocation {
	fn matches(&self, location: &MultiLocation) -> bool;
}

/// Simple asset location filter
#[derive(Debug)]
pub enum AssetFilter {
	ByMultiLocation(MultiLocationFilter),
}

impl MatchAssetLocation for AssetFilter {
	fn matches(&self, asset_location: &MultiLocation) -> bool {
		match self {
			AssetFilter::ByMultiLocation(by_location) => by_location.matches(asset_location),
		}
	}
}

#[derive(Debug, Default)]
pub struct MultiLocationFilter {
	/// Requested location equals to `MultiLocation`
	pub equals_any: sp_std::vec::Vec<MultiLocation>,
	/// Requested location starts with `MultiLocation`
	pub starts_with_any: sp_std::vec::Vec<MultiLocation>,
}

impl MultiLocationFilter {
	pub fn add_equals(mut self, filter: MultiLocation) -> Self {
		self.equals_any.push(filter);
		self
	}
	pub fn add_starts_with(mut self, filter: MultiLocation) -> Self {
		self.starts_with_any.push(filter);
		self
	}
}

impl MatchAssetLocation for MultiLocationFilter {
	fn matches(&self, location: &MultiLocation) -> bool {
		for filter in &self.equals_any {
			if location.eq(filter) {
				return true
			}
		}
		for filter in &self.starts_with_any {
			if location.starts_with(filter) {
				return true
			}
		}
		false
	}
}
