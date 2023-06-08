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

//! # Bridge Transfer Pallet
//!
//! Module helps with different transfers through bridges between different consensus chain.
//!
//! One of possible scenarios which supports directly is "transfer asset over bridge (back and forth)".
//! With fine-grained configuration you can control transferred assets (out/in) between different consensus chain.
//! E.g. you can allow just some assets to go out/in based on their `MultiLocation` patterns.
//! "Transfer asset over bridge" recognize two kinds of transfer see [AssetTransferKind].
//!
//! ## Overview
//!
//! Pallet supports configuration for several independent scenarios:
//!
//! ### 1. Support to store on-chain bridge locations
//!
//! * Bridge locations/fees can be stored on-chain in [AllowedExporters]
//! * Managing bridge locations can be done with dedicated extrinsics: `add_exporter_config / remove_exporter_config / update_exporter_config`
//! * Pallet implements `xcm_builder::ExporterFor` which can be used with `xcm_builder::UnpaidRemoteExporter/SovereignPaidRemoteExporter`
//!
//! ### 2. Support to store on-chain allowed bridged target locations with asset filters
//!
//! * (Used for transfer assets out)
//! * If we want to use `transfer_asset_via_bridge` we should setup the target location with asset filter to allow reserve asset transfer to this location over bridge.
//! * Managing target location and asset filer can be done with dedicated extrinsics:
//! * - `update_bridged_target_location / remove_bridged_target_location` - managing target location behind bridge
//! * - `allow_reserve_asset_transfer_for / disallow_reserve_asset_transfer_for` - managing asset filters
//!
//! ### 3. Support to store on-chain allowed universal aliases/origins
//!
//! * (Used for transfer assets in)
//! * Aliases can be stored on-chain in [AllowedUniversalAliases]
//! * Managing bridge locations can be done with dedicated extrinsics: `add_universal_alias / remove_universal_alias`
//! * Stored aliases can be accessed by [features::AllowedUniversalAliasesOf] and configured for e.g. `xcm_executor::Config`:
//!   ```nocompile
//!   impl xcm_executor::Config for XcmConfig {
//! 	...
//!   	type UniversalAliases = AllowedUniversalAliasesOf<Runtime>;
//! 	...
//!   }
//!   ```
//!
//! ### 4. Support to store on-chain allowed reserve locations with allowed asset filters
//!
//! * (Used for transfer assets in)
//! * Reserve locations with asset filters can be stored on-chain in [AllowedReserveLocations]
//! * Managing bridge locations can be done with dedicated extrinsics: `add_reserve_location / remove_reserve_location`
//! * Stored reserve locations can be accessed by [features::IsTrustedBridgedReserveForConcreteAsset] and configured for e.g. `xcm_executor::Config`:
//!   ```nocompile
//!   impl xcm_executor::Config for XcmConfig {
//! 	...
//!   	type IsReserve = IsTrustedBridgedReserveForConcreteAsset<Runtime>;
//! 	...
//!   }
//!   ```

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::boxed::Box;

mod types;
pub use types::*;

pub use pallet::*;

pub mod features;
mod impls;
pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod tests;

/// The log target of this pallet.
pub const LOG_TARGET: &str = "runtime::bridge-transfer";

#[frame_support::pallet]
pub mod pallet {
	pub use crate::weights::WeightInfo;

	use super::*;
	use crate::filter::{AssetFilter, AssetFilterOf, MultiLocationFilterOf};
	use frame_support::{pallet_prelude::*, traits::EnsureOriginWithArg, BoundedBTreeSet};
	use frame_system::pallet_prelude::*;
	use xcm::prelude::*;
	use xcm_executor::traits::TransactAsset;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Everything we need to run benchmarks.
	#[cfg(feature = "runtime-benchmarks")]
	pub trait BenchmarkHelper<RuntimeOrigin> {
		/// Returns proper bridge configuration, supported by the runtime.
		///
		/// We expect that the XCM environment (`BridgeXcmSender`) has everything enabled
		/// to support transfer to this destination **after** `prepare_asset_transfer` call.
		fn bridge_config() -> Option<(NetworkId, BridgeConfig)> {
			None
		}

		/// Prepare environment for assets transfer and return transfer origin and assets
		/// to transfer. After this function is called, we expect `transfer_asset_via_bridge`
		/// to succeed, so in proper environment, it should:
		///
		/// - deposit enough funds (fee from `bridge_config()` and transferred assets) to the sender account;
		///
		/// - ensure that the `BridgeXcmSender` is properly configured for the transfer;
		///
		/// - be close to the worst possible scenario - i.e. if some account may need to be created during
		///   the assets transfer, it should be created. If there are multiple bridges, the "worst possible"
		///   (in terms of performance) bridge must be selected for the transfer.
		fn prepare_asset_transfer(
		) -> Option<(RuntimeOrigin, VersionedMultiAssets, VersionedMultiLocation)> {
			None
		}

		fn universal_alias() -> Option<(VersionedMultiLocation, Junction)> {
			None
		}

		fn reserve_location() -> Option<VersionedMultiLocation> {
			None
		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Runtime's universal location
		type UniversalLocation: Get<InteriorMultiLocation>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// The configurable origin to allow bridges configuration management.
		type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// The configurable origin to allow transfer out asset filters management.
		type AllowReserveAssetTransferOrigin: EnsureOriginWithArg<
			Self::RuntimeOrigin,
			AssetFilterOf<Self>,
		>;

		/// Max allowed universal aliases per one `MultiLocation`
		/// (Config for transfer in)
		#[pallet::constant]
		type UniversalAliasesLimit: Get<u32>;
		/// Max allowed reserve locations
		/// (Config for transfer out)
		#[pallet::constant]
		type ReserveLocationsLimit: Get<u32>;
		/// Max allowed assets for one reserve location
		/// (Config for transfer in/out)
		#[pallet::constant]
		type AssetsPerReserveLocationLimit: Get<u32>;

		/// How to withdraw and deposit an asset for reserve.
		/// (Config for transfer out)
		type AssetTransactor: TransactAsset;
		/// Transfer type resolver for `asset` to `target_location`.
		type AssetTransferKindResolver: ResolveAssetTransferKind;
		/// XCM sender which sends messages to the BridgeHub
		/// (Config for transfer out)
		type BridgeXcmSender: SendXcm;
		/// Required origin for asset transfer. If successful, it resolves to `MultiLocation`.
		/// (Config for transfer out)
		type AssetTransferOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = MultiLocation>;
		/// Max count of assets in one call
		/// (Config for transfer out)
		type MaxAssetsLimit: Get<u8>;

		/// Benchmarks helper.
		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper: BenchmarkHelper<Self::RuntimeOrigin>;
	}

	/// Details of configured bridges which are allowed for **transfer out**.
	/// (Config for transfer out)
	#[pallet::storage]
	#[pallet::getter(fn allowed_exporters)]
	pub(super) type AllowedExporters<T: Config> =
		StorageMap<_, Blake2_128Concat, NetworkId, BridgeConfig<T>>;

	/// Holds allowed mappings `MultiLocation->Junction` for `UniversalAliases`
	/// E.g:
	/// 	BridgeHubMultiLocation1 -> NetworkId::Kusama
	/// 	BridgeHubMultiLocation1 -> NetworkId::Polkadot
	/// (Config for transfer in)
	#[pallet::storage]
	#[pallet::getter(fn allowed_universal_aliases)]
	pub(super) type AllowedUniversalAliases<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		VersionedMultiLocation,
		BoundedBTreeSet<Junction, T::UniversalAliasesLimit>,
		ValueQuery,
	>;

	/// Holds allowed mappings `MultiLocation` as trusted reserve locations for allowed assets
	/// (Config for transfer in)
	#[pallet::storage]
	#[pallet::getter(fn allowed_reserve_locations)]
	pub(super) type AllowedReserveLocations<T: Config> =
		StorageMap<_, Blake2_128Concat, VersionedMultiLocation, AssetFilterOf<T>, OptionQuery>;

	#[pallet::error]
	#[cfg_attr(test, derive(PartialEq))]
	pub enum Error<T> {
		InvalidConfiguration,
		InvalidBridgeConfiguration,
		UnavailableConfiguration,
		ConfigurationAlreadyExists,
		InvalidAssets,
		MaxAssetsLimitReached,
		UnsupportedDestination,
		UnsupportedXcmVersion,
		InvalidRemoteDestination,
		BridgeCallError,
		FailedToReserve,
		FailedToWithdraw,
		UnsupportedAssetTransferKind,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Transfer was successfully entered to the system (does not mean already delivered)
		TransferInitiated {
			/// `XcmHash` from `XcmContext` which is used for `AssetTransactor` processing and is related to the original message constructed here
			message_id: XcmHash,
			/// `XcmHash` from `SendXcm` (which is used for `ExportMessage` envelope)
			forwarded_message_id: XcmHash,
			/// `SendXcm` cost
			sender_cost: MultiAssets,
		},

		/// Reserve assets passed
		ReserveAssetsDeposited { from: MultiLocation, to: MultiLocation, assets: MultiAssets },
		/// Assets were withdrawn
		AssetsWithdrawn { from: MultiLocation, assets: MultiAssets },

		/// New bridge configuration was added
		BridgeAdded,
		/// Bridge configuration was removed
		BridgeRemoved,
		/// Bridge fee configuration was updated
		BridgeFeeUpdated,
		/// Target location for bridge was updated
		BridgeTargetLocationUpdated,

		/// New universal alias was added
		UniversalAliasAdded,
		/// New universal alias was removed
		UniversalAliasRemoved { junction: Junction },

		/// New reserve location was added
		ReserveLocationAdded,
		/// New reserve location was removed
		ReserveLocationRemoved,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Transfer asset via bridge to different global consensus
		///
		/// Parameters:
		///
		/// * `assets`:
		/// * `destination`: Different consensus location, where the assets will be deposited, e.g. Polkadot's Statemint: `2, X2(GlobalConsensus(NetworkId::Polkadot), Parachain(1000))`
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::transfer_asset_via_bridge())]
		pub fn transfer_asset_via_bridge(
			origin: OriginFor<T>,
			assets: Box<VersionedMultiAssets>,
			destination: Box<VersionedMultiLocation>,
		) -> DispatchResult {
			// Check origin
			let origin_location = T::AssetTransferOrigin::ensure_origin(origin)?;

			// Check if remote destination is reachable
			let destination = Self::ensure_reachable_remote_destination(*destination)?;

			// Check assets (lets leave others checks on `AssetTransactor`)
			let assets: MultiAssets =
				(*assets).try_into().map_err(|()| Error::<T>::InvalidAssets)?;
			ensure!(
				assets.len() <= T::MaxAssetsLimit::get() as usize,
				Error::<T>::MaxAssetsLimitReached
			);

			// Do this in transaction (explicitly), the rollback should occur in case of any error and no assets will be trapped or lost
			Self::do_transfer_asset_via_bridge_in_transaction(origin_location, destination, assets)
		}

		/// Adds new bridge exporter, which allows transfer to this `bridged_network`.
		///
		/// Parameters:
		///
		/// * `bridged_network`: Network where we want to allow transfer funds
		/// * `bridge_config`: contains location for BridgeHub in our network + fee
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::add_exporter_config())]
		pub fn add_exporter_config(
			origin: OriginFor<T>,
			bridged_network: NetworkId,
			bridge_location: Box<VersionedMultiLocation>,
			bridge_location_fee: Option<Box<VersionedMultiAsset>>,
		) -> DispatchResult {
			let _ = T::AdminOrigin::ensure_origin(origin)?;

			// allow just one exporter for one network (can be changed in future)
			ensure!(
				!AllowedExporters::<T>::contains_key(bridged_network),
				Error::<T>::ConfigurationAlreadyExists
			);

			// check bridge is from local consensus
			ensure!(
				bridge_location
					.using_versioned_ref(|versioned| versioned.parents)
					.map_err(|_| Error::<T>::UnsupportedXcmVersion)? <=
					1,
				Error::<T>::InvalidConfiguration
			);

			// insert
			AllowedExporters::<T>::insert(
				bridged_network,
				BridgeConfig {
					bridge_location: *bridge_location,
					bridge_location_fee: bridge_location_fee.map(|fee| *fee),
					allowed_target_locations: Default::default(),
				},
			);

			Self::deposit_event(Event::BridgeAdded);
			Ok(())
		}

		/// Remove bridge configuration for specified `bridged_network`.
		///
		/// Parameters:
		///
		/// * `bridged_network`: Network where we want to remove
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::remove_exporter_config())]
		pub fn remove_exporter_config(
			origin: OriginFor<T>,
			bridged_network: NetworkId,
		) -> DispatchResult {
			let _ = T::AdminOrigin::ensure_origin(origin)?;
			ensure!(
				AllowedExporters::<T>::contains_key(bridged_network),
				Error::<T>::UnavailableConfiguration
			);

			AllowedExporters::<T>::remove(bridged_network);
			Self::deposit_event(Event::BridgeRemoved);
			Ok(())
		}

		/// Updates bridge configuration for specified `bridged_network`.
		///
		/// Parameters:
		///
		/// * `bridged_network`: Network where we want to remove
		/// * `bridge_location_fee`: New fee to update
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::update_exporter_config())]
		pub fn update_exporter_config(
			origin: OriginFor<T>,
			bridged_network: NetworkId,
			bridge_location_fee: Option<Box<VersionedMultiAsset>>,
		) -> DispatchResult {
			let _ = T::AdminOrigin::ensure_origin(origin)?;

			AllowedExporters::<T>::try_mutate_exists(bridged_network, |maybe_bridge_config| {
				let bridge_config =
					maybe_bridge_config.as_mut().ok_or(Error::<T>::UnavailableConfiguration)?;
				bridge_config.bridge_location_fee = bridge_location_fee.map(|fee| *fee);
				Self::deposit_event(Event::BridgeFeeUpdated);
				Ok(())
			})
		}

		/// Alters bridge configuration with asset filter, which allows to send out matched assets to the `target_location`
		///
		/// Parameters:
		///
		/// * `bridged_network`: bridge identifier
		/// * `target_location`: target location where we want to allow send assets
		/// * `asset_filter`: asset filter to add
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::update_bridged_target_location())]
		pub fn update_bridged_target_location(
			origin: OriginFor<T>,
			bridged_network: NetworkId,
			target_location: Box<VersionedMultiLocation>,
			max_target_location_fee: Option<Box<VersionedMultiAsset>>,
		) -> DispatchResult {
			let _ = T::AdminOrigin::ensure_origin(origin)?;

			AllowedExporters::<T>::try_mutate_exists(bridged_network, |maybe_bridge_config| {
				let bridge_config =
					maybe_bridge_config.as_mut().ok_or(Error::<T>::UnavailableConfiguration)?;
				bridge_config
					.update_allowed_target_location(
						*target_location,
						max_target_location_fee.map(|fee| *fee),
					)
					.map_err(|_| Error::<T>::InvalidBridgeConfiguration)?;
				Self::deposit_event(Event::BridgeTargetLocationUpdated);
				Ok(())
			})
		}

		/// Alters bridge configuration with asset filter, which allows to send out matched assets to the `target_location`
		///
		/// Parameters:
		///
		/// * `bridged_network`: bridge identifier
		/// * `target_location`: target location where we want to allow send assets
		/// * `asset_filter`: asset filter to add
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::allow_reserve_asset_transfer_for())]
		pub fn allow_reserve_asset_transfer_for(
			origin: OriginFor<T>,
			bridged_network: NetworkId,
			target_location: Box<VersionedMultiLocation>,
			asset_filter: AssetFilterOf<T>,
		) -> DispatchResult {
			let _ = T::AllowReserveAssetTransferOrigin::ensure_origin(origin, &asset_filter)?;

			AllowedExporters::<T>::try_mutate_exists(bridged_network, |maybe_bridge_config| {
				let bridge_config =
					maybe_bridge_config.as_mut().ok_or(Error::<T>::UnavailableConfiguration)?;
				let was_added = bridge_config
					.add_allowed_target_location_filter_for(*target_location, asset_filter)
					.map_err(|_| Error::<T>::InvalidBridgeConfiguration)?;
				if was_added {
					Self::deposit_event(Event::ReserveLocationAdded);
				}
				Ok(())
			})
		}

		/// Add `(MultiLocation, Junction)` mapping to `AllowedUniversalAliases`
		///
		/// Parameters:
		///
		/// * `location`: key
		/// * `junction`: value
		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::add_universal_alias())]
		pub fn add_universal_alias(
			origin: OriginFor<T>,
			location: Box<VersionedMultiLocation>,
			junction: Junction,
		) -> DispatchResult {
			let _ = T::AdminOrigin::ensure_origin(origin)?;

			let versioned_location = LatestVersionedMultiLocation::try_from(location.as_ref())
				.map_err(|_| Error::<T>::UnsupportedXcmVersion)?;
			AllowedUniversalAliases::<T>::try_mutate(versioned_location, |junctions| {
				if junctions.try_insert(junction).map_err(|_| Error::<T>::InvalidConfiguration)? {
					Self::deposit_event(Event::UniversalAliasAdded);
				}
				Ok(())
			})
		}

		/// Remove `(MultiLocation, Junction)` mapping from `AllowedUniversalAliases`
		///
		/// Parameters:
		///
		/// * `location`: key
		/// * `junction`: value
		#[pallet::call_index(9)]
		#[pallet::weight(T::WeightInfo::remove_universal_alias())]
		pub fn remove_universal_alias(
			origin: OriginFor<T>,
			location: Box<VersionedMultiLocation>,
			junctions_to_remove: sp_std::prelude::Vec<Junction>,
		) -> DispatchResult {
			let _ = T::AdminOrigin::ensure_origin(origin)?;

			let versioned_location = LatestVersionedMultiLocation::try_from(location.as_ref())
				.map_err(|_| Error::<T>::UnsupportedXcmVersion)?;
			AllowedUniversalAliases::<T>::try_mutate_exists(versioned_location, |maybe_junctions| {
				let junctions =
					maybe_junctions.as_mut().ok_or(Error::<T>::UnavailableConfiguration)?;
				for jtr in junctions_to_remove {
					if junctions.remove(&jtr) {
						Self::deposit_event(Event::UniversalAliasRemoved { junction: jtr });
					}
				}

				if junctions.is_empty() {
					// no junctions, so remove entry from storage
					*maybe_junctions = None;
				}

				Ok(())
			})
		}

		/// Add `MultiLocation` + `AssetFilter` mapping to `AllowedReserveLocations`
		///
		/// Parameters:
		///
		/// * `location`: as reserve `MultiLocation`
		/// * `asset_filter_to_add`: filter we want to add for that location
		#[pallet::call_index(10)]
		#[pallet::weight(T::WeightInfo::add_reserve_location())]
		pub fn add_reserve_location(
			origin: OriginFor<T>,
			location: Box<VersionedMultiLocation>,
			asset_filter_to_add: AssetFilterOf<T>,
		) -> DispatchResult {
			let _ = T::AdminOrigin::ensure_origin(origin)?;

			let versioned_location = LatestVersionedMultiLocation::try_from(location.as_ref())
				.map_err(|_| Error::<T>::UnsupportedXcmVersion)?;
			AllowedReserveLocations::<T>::try_mutate(versioned_location, |filter| {
				let was_added = match filter {
					Some(old_filter) => old_filter
						.add(asset_filter_to_add)
						.map_err(|_| Error::<T>::InvalidConfiguration)?,
					None => {
						*filter = Some(asset_filter_to_add);
						true
					},
				};
				if was_added {
					Self::deposit_event(Event::ReserveLocationAdded);
				}
				Ok(())
			})
		}

		/// Remove `MultiLocation` mapping from `AllowedReserveLocations`.
		///
		/// Parameters:
		///
		/// * `location`: as reserve `MultiLocation`
		/// * `asset_filter_to_remove`: if `None` the whole `location` entry will be removed, otherwise only selected filters
		#[pallet::call_index(11)]
		#[pallet::weight(T::WeightInfo::remove_reserve_location())]
		pub fn remove_reserve_location(
			origin: OriginFor<T>,
			location: Box<VersionedMultiLocation>,
			asset_filter_to_remove: Option<MultiLocationFilterOf<T>>,
		) -> DispatchResult {
			let _ = T::AdminOrigin::ensure_origin(origin)?;

			let versioned_location = LatestVersionedMultiLocation::try_from(location.as_ref())
				.map_err(|_| Error::<T>::UnsupportedXcmVersion)?;
			let (mut was_removed, remove_location) = if let Some(asset_filter_to_remove) =
				asset_filter_to_remove
			{
				AllowedReserveLocations::<T>::try_mutate_exists(
					versioned_location,
					|maybe_filter| {
						let filter =
							maybe_filter.as_mut().ok_or(Error::<T>::UnavailableConfiguration)?;
						match filter {
							AssetFilter::ByMultiLocation(old) => {
								let was_removed = old.remove(asset_filter_to_remove);
								was_removed.map(|wr| (wr, old.is_empty())).map_err(Into::into)
							},
							AssetFilter::All => Err(Error::<T>::UnavailableConfiguration),
						}
					},
				)?
			} else {
				(false, true)
			};

			if remove_location {
				was_removed |= AllowedReserveLocations::<T>::try_mutate_exists(location, |value| {
					let exists = value.is_some();
					if exists {
						*value = None;
					}
					Ok::<_, Error<T>>(exists)
				})
				.map_err(|_| Error::<T>::InvalidConfiguration)?;
			}

			if was_removed {
				Self::deposit_event(Event::ReserveLocationRemoved);
				Ok(())
			} else {
				Err(Error::<T>::InvalidConfiguration.into())
			}
		}
	}
}
