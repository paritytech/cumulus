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

use crate::Config;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::{CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound};
use scale_info::TypeInfo;
use xcm::prelude::*;

pub use crate::{
	config::BridgeConfig,
	filter::{AssetFilterOf, AssetFilterT, MultiLocationFilterOf},
	xcm_version::{LatestVersionedMultiLocation, UnsupportedXcmVersionError, UsingVersioned},
};

/// Pallet support two kinds of transfer.
#[cfg_attr(feature = "std", derive(Debug, PartialEq))]
pub enum AssetTransferKind {
	/// When we need to do a **reserve** on source chain to **reserve account** and then send it as `ReserveAssetDeposited`
	ReserveBased,
	/// When we need to do a opposite direction, withdraw/burn asset on source chain and send it as `Withdraw/Burn` on target chain from **reserve account**.
	WithdrawReserve,
	/// If not supported/permitted (e.g. missing configuration of trusted reserve location, ...).
	Unsupported,
}

/// Trait for resolving a transfer type for `asset` to `target_location`
pub trait ResolveAssetTransferKind {
	fn resolve(asset: &MultiAsset, target_location: &MultiLocation) -> AssetTransferKind;
}

impl ResolveAssetTransferKind for () {
	fn resolve(_asset: &MultiAsset, _target_location: &MultiLocation) -> AssetTransferKind {
		AssetTransferKind::Unsupported
	}
}

#[cfg_attr(feature = "std", derive(Debug, PartialEq))]
pub struct MaybePaidLocation {
	pub location: MultiLocation,
	pub maybe_fee: Option<MultiAsset>,
}

#[cfg_attr(feature = "std", derive(Debug, PartialEq))]
pub struct ReachableDestination<AssetFilter> {
	/// Bridge location
	pub bridge: MaybePaidLocation,
	/// Target location (e.g. remote parachain in different consensus)
	pub target: MaybePaidLocation,
	/// Optional asset filter, which we can send to target location
	pub target_asset_filter: Option<AssetFilter>,
	/// Destination on target location (e.g. account on remote parachain in different consensus)
	pub target_destination: MultiLocation,
}

pub mod config {
	use super::*;
	use crate::{
		filter::{AssetFilterError, AssetFilterOf},
		UnsupportedXcmVersionError,
	};
	use frame_support::BoundedBTreeMap;

	#[derive(
		Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebugNoBound, CloneNoBound, PartialEqNoBound,
	)]
	#[scale_info(skip_type_params(T))]
	pub struct BridgeConfig<T: Config> {
		/// Contains location, which is able to bridge XCM messages to bridged network
		pub bridge_location: VersionedMultiLocation,
		/// Fee which could be needed to pay in `bridge_location`
		/// `MultiAsset` is here from the point of view of `bridge_location`, e.g.: `MultiLocation::parent()` means relay chain token of `bridge_location`
		pub bridge_location_fee: Option<VersionedMultiAsset>,

		/// Contains target destination on bridged network. E.g.: MultiLocation of Statemine/t on different consensus + its configuration
		pub(crate) allowed_target_locations: BoundedBTreeMap<
			VersionedMultiLocation,
			TargetLocationConfig<T>,
			T::ReserveLocationsLimit,
		>,
	}

	impl<T: Config> BridgeConfig<T> {
		/// Tries to find target destination configuration for destination.
		pub fn allowed_target_location_for(
			&self,
			destination: &MultiLocation,
		) -> Result<Option<(MaybePaidLocation, Option<AssetFilterOf<T>>)>, UnsupportedXcmVersionError>
		{
			for (target_location, cfg) in &self.allowed_target_locations {
				let target_location = target_location.as_versioned_ref()?;
				// if destination matches target_location, we found our configuration
				if destination.starts_with(target_location) || destination.eq(target_location) {
					let maybe_fee = match &cfg.max_target_location_fee {
						Some(fee) => Some(fee.as_versioned_ref()?.clone()),
						None => None,
					};
					return Ok(Some((
						MaybePaidLocation { location: target_location.clone(), maybe_fee },
						cfg.allowed_reserve_assets.clone(),
					)))
				}
			}
			Ok(None)
		}

		pub fn update_allowed_target_location(
			&mut self,
			target_location: VersionedMultiLocation,
			max_target_location_fee: Option<VersionedMultiAsset>,
		) -> Result<(), BridgeConfigError> {
			// lets try update existing
			for (stored_target, cfg) in self.allowed_target_locations.iter_mut() {
				let stored_target = stored_target.as_versioned_ref()?;
				let target_location = target_location.as_versioned_ref()?;
				// if stored_target matches target_location, we found our configuration
				if stored_target.eq(target_location) {
					cfg.max_target_location_fee = max_target_location_fee;
					return Ok(())
				}
			}

			// if not found, just insert
			self.allowed_target_locations
				.try_insert(
					target_location,
					TargetLocationConfig { max_target_location_fee, allowed_reserve_assets: None },
				)
				.map(|_| ())
				.map_err(|_| BridgeConfigError::LimitReached)
		}

		pub fn add_allowed_target_location_filter_for(
			&mut self,
			target_location: VersionedMultiLocation,
			new_asset_filter: AssetFilterOf<T>,
		) -> Result<bool, BridgeConfigError> {
			for (stored_target, cfg) in self.allowed_target_locations.iter_mut() {
				let stored_target = stored_target.as_versioned_ref()?;
				let target_location = target_location.as_versioned_ref()?;
				// if stored_target matches target_location, we found our configuration
				if stored_target.eq(target_location) {
					let was_added = match &mut cfg.allowed_reserve_assets {
						Some(old_asset_filter) => old_asset_filter.add(new_asset_filter)?,
						None => {
							cfg.allowed_reserve_assets = Some(new_asset_filter);
							true
						},
					};
					return Ok(was_added)
				}
			}
			return Err(BridgeConfigError::MissingTargetLocation)
		}

		pub fn to_bridge_location(self) -> Result<MaybePaidLocation, UnsupportedXcmVersionError> {
			let maybe_fee = match self.bridge_location_fee {
				Some(fee) => Some(fee.to_versioned()?),
				None => None,
			};
			Ok(MaybePaidLocation { location: self.bridge_location.to_versioned()?, maybe_fee })
		}
	}

	#[derive(
		Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebugNoBound, CloneNoBound, PartialEqNoBound,
	)]
	#[scale_info(skip_type_params(T))]
	pub struct TargetLocationConfig<T: Config> {
		/// If `None` then `UnpaidExecution` is used, else `Withdraw(target_location_fee)/BuyExecution(target_location_fee, Unlimited)`
		/// `MultiAsset` is here from the point of view of `allowed_target_location`, e.g.: `MultiLocation::parent()` means relay chain token of `allowed_target_location`
		pub max_target_location_fee: Option<VersionedMultiAsset>,

		/// Filter for allowed asset that can be send out to the target location.
		pub allowed_reserve_assets: Option<AssetFilterOf<T>>,
	}

	pub enum BridgeConfigError {
		AssetFilterError(AssetFilterError),
		MissingTargetLocation,
		UnsupportedXcmVersionError(UnsupportedXcmVersionError),
		LimitReached,
	}

	impl From<UnsupportedXcmVersionError> for BridgeConfigError {
		fn from(e: UnsupportedXcmVersionError) -> Self {
			Self::UnsupportedXcmVersionError(e)
		}
	}

	impl From<AssetFilterError> for BridgeConfigError {
		fn from(e: AssetFilterError) -> Self {
			Self::AssetFilterError(e)
		}
	}
}

pub mod filter {
	use super::*;

	use crate::UnsupportedXcmVersionError;
	use frame_support::{pallet_prelude::Get, BoundedVec};

	pub type AssetFilterOf<T> = AssetFilter<<T as Config>::AssetsPerReserveLocationLimit>;
	pub type MultiLocationFilterOf<T> =
		MultiLocationFilter<<T as Config>::AssetsPerReserveLocationLimit>;

	pub trait AssetFilterT {
		fn matches(&self, location: &MultiLocation) -> Result<bool, UnsupportedXcmVersionError>;
	}

	pub enum AssetFilterError {
		UnsupportedXcmVersionError,
		LimitReached,
	}

	impl From<UnsupportedXcmVersionError> for AssetFilterError {
		fn from(_: UnsupportedXcmVersionError) -> Self {
			Self::UnsupportedXcmVersionError
		}
	}

	impl<T: Config> From<AssetFilterError> for crate::Error<T> {
		fn from(value: AssetFilterError) -> Self {
			match value {
				AssetFilterError::UnsupportedXcmVersionError =>
					crate::Error::<T>::UnsupportedXcmVersion,
				AssetFilterError::LimitReached => crate::Error::<T>::InvalidConfiguration,
			}
		}
	}

	#[derive(
		CloneNoBound,
		Encode,
		Decode,
		Eq,
		PartialEqNoBound,
		RuntimeDebugNoBound,
		TypeInfo,
		MaxEncodedLen,
		Ord,
		PartialOrd,
	)]
	#[scale_info(skip_type_params(AssetsLimit))]
	pub enum AssetFilter<AssetsLimit: Get<u32>> {
		ByMultiLocation(MultiLocationFilter<AssetsLimit>),
		All,
	}

	impl<AssetsLimit: Get<u32>> AssetFilterT for AssetFilter<AssetsLimit> {
		fn matches(&self, location: &MultiLocation) -> Result<bool, UnsupportedXcmVersionError> {
			match self {
				AssetFilter::ByMultiLocation(by_location) => by_location.matches(location),
				AssetFilter::All => Ok(true),
			}
		}
	}

	impl<AssetsLimit: Get<u32>> AssetFilter<AssetsLimit> {
		pub fn add(
			&mut self,
			new_filter: AssetFilter<AssetsLimit>,
		) -> Result<bool, AssetFilterError> {
			match new_filter {
				AssetFilter::ByMultiLocation(by_location) => match self {
					AssetFilter::ByMultiLocation(old_by_location) =>
						old_by_location.add(by_location),
					AssetFilter::All => {
						*self = AssetFilter::ByMultiLocation(by_location);
						Ok(true)
					},
				},
				AssetFilter::All => {
					let mut was_added = false;
					if !matches!(self, AssetFilter::All) {
						*self = AssetFilter::All;
						was_added = true
					}
					Ok(was_added)
				},
			}
		}
	}

	#[derive(
		CloneNoBound,
		Encode,
		Decode,
		Eq,
		PartialEqNoBound,
		RuntimeDebugNoBound,
		Default,
		TypeInfo,
		MaxEncodedLen,
		Ord,
		PartialOrd,
	)]
	#[scale_info(skip_type_params(PatternsLimit))]
	pub struct MultiLocationFilter<PatternsLimit: Get<u32>> {
		/// Requested location equals to `MultiLocation`
		pub equals_any: BoundedVec<VersionedMultiLocation, PatternsLimit>,
		/// Requested location starts with `MultiLocation`
		pub starts_with_any: BoundedVec<VersionedMultiLocation, PatternsLimit>,
	}

	impl<PatternsLimit: Get<u32>> MultiLocationFilter<PatternsLimit> {
		fn add(
			&mut self,
			to_add: MultiLocationFilter<PatternsLimit>,
		) -> Result<bool, AssetFilterError> {
			let add_if_not_found = |olds: &mut BoundedVec<VersionedMultiLocation, _>,
			                        news: BoundedVec<VersionedMultiLocation, _>|
			 -> Result<bool, AssetFilterError> {
				let mut was_added = false;
				for new_ta in news {
					if MultiLocationFilter::find_index(&olds, &new_ta)?.is_none() {
						olds.try_push(new_ta).map_err(|_| AssetFilterError::LimitReached)?;
						was_added = true;
					}
				}
				Ok(was_added)
			};

			Ok(add_if_not_found(&mut self.equals_any, to_add.equals_any)? |
				add_if_not_found(&mut self.starts_with_any, to_add.starts_with_any)?)
		}

		pub fn remove(
			&mut self,
			to_remove: MultiLocationFilter<PatternsLimit>,
		) -> Result<bool, AssetFilterError> {
			let remove_from = |olds: &mut BoundedVec<VersionedMultiLocation, _>,
			                   to_remove: BoundedVec<VersionedMultiLocation, _>|
			 -> Result<bool, AssetFilterError> {
				let mut was_removed = false;
				for tr in to_remove {
					let found = MultiLocationFilter::find_index(olds, &tr)?;
					if let Some(idx) = found {
						_ = olds.remove(idx);
						was_removed = true;
					}
				}
				Ok(was_removed)
			};

			Ok(remove_from(&mut self.equals_any, to_remove.equals_any)? |
				remove_from(&mut self.starts_with_any, to_remove.starts_with_any)?)
		}

		fn find_index(
			items: &BoundedVec<VersionedMultiLocation, PatternsLimit>,
			searched: &VersionedMultiLocation,
		) -> Result<Option<usize>, UnsupportedXcmVersionError> {
			let searched = searched.as_versioned_ref()?;
			for (idx, item) in items.iter().enumerate() {
				let item = item.as_versioned_ref()?;
				if item.eq(searched) {
					return Ok(Some(idx))
				}
			}
			Ok(None)
		}

		pub fn is_empty(&self) -> bool {
			self.equals_any.is_empty() && self.starts_with_any.is_empty()
		}
	}

	impl<PatternsLimit: Get<u32>> AssetFilterT for MultiLocationFilter<PatternsLimit> {
		fn matches(&self, location: &MultiLocation) -> Result<bool, UnsupportedXcmVersionError> {
			for filter in &self.equals_any {
				if filter.using_versioned_ref(|versioned| location.eq(versioned))? {
					return Ok(true)
				}
			}
			for filter in &self.starts_with_any {
				if filter.using_versioned_ref(|versioned| location.starts_with(versioned))? {
					return Ok(true)
				}
			}
			Ok(false)
		}
	}
}

pub mod xcm_version {
	use super::*;
	use crate::Config;
	use codec::EncodeLike;
	use xcm::TryAs;

	/// For simplified version mismatch handling
	#[derive(Debug)]
	pub struct UnsupportedXcmVersionError;

	impl<T: Config> From<UnsupportedXcmVersionError> for crate::Error<T> {
		fn from(_: UnsupportedXcmVersionError) -> Self {
			Self::UnsupportedXcmVersion
		}
	}

	/// Simplified adapter between `xcm::latest` and `Versioned*` stuff.
	pub trait UsingVersioned<T>: TryAs<T> + TryInto<T> {
		fn using_versioned_ref<R, F: FnOnce(&T) -> R>(
			&self,
			f: F,
		) -> Result<R, UnsupportedXcmVersionError> {
			let versioned = self.try_as().map_err(|_| UnsupportedXcmVersionError)?;
			Ok(f(versioned))
		}

		fn as_versioned_ref(&self) -> Result<&T, UnsupportedXcmVersionError> {
			let versioned = self.try_as().map_err(|_| UnsupportedXcmVersionError)?;
			Ok(versioned)
		}

		fn to_versioned(self) -> Result<T, UnsupportedXcmVersionError> {
			let versioned = self.try_into().map_err(|_| UnsupportedXcmVersionError)?;
			Ok(versioned)
		}
	}

	impl UsingVersioned<MultiLocation> for VersionedMultiLocation {}

	impl UsingVersioned<MultiAsset> for VersionedMultiAsset {}

	/// `KeyArg` wrapper for `VersionedMultiLocation`.
	/// This feature allows to use `VersionedMultiLocation` as key, e.g. for `StorageMap`.
	///
	/// NOTE: Because something like this does not work:
	/// ```nocompile
	/// let a = VersionedMultiLocation::V2(xcm::v2::MultiLocation::new(1, xcm::v2::Junctions::X1(xcm::v2::Junction::Parachain(1000))));
	/// let b = VersionedMultiLocation::V3(xcm::v3::MultiLocation::new(1, xcm::v3::Junctions::X1(xcm::v3::Junction::Parachain(1000))));
	/// assert_eq!(key_a, key_b);  // <-- fails
	/// ```
	#[derive(Copy, Clone)]
	pub struct LatestVersionedMultiLocation<'a>(pub(crate) &'a MultiLocation);

	impl<'a> EncodeLike<VersionedMultiLocation> for LatestVersionedMultiLocation<'a> {}

	impl<'a> EncodeLike<VersionedMultiLocation> for &LatestVersionedMultiLocation<'a> {}

	impl<'a> Encode for LatestVersionedMultiLocation<'a> {
		fn encode(&self) -> sp_std::vec::Vec<u8> {
			let mut r = VersionedMultiLocation::from(MultiLocation::default()).encode();
			r.truncate(1);
			self.0.using_encoded(|d| r.extend_from_slice(d));
			r
		}
	}

	impl<'a> TryFrom<&'a VersionedMultiLocation> for LatestVersionedMultiLocation<'a> {
		type Error = UnsupportedXcmVersionError;

		fn try_from(value: &'a VersionedMultiLocation) -> Result<Self, Self::Error> {
			value.as_versioned_ref().map(|v| LatestVersionedMultiLocation(v))
		}
	}
}
