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
//! * Stored aliases can be accessed by [impls::AllowedUniversalAliasesOf] and configured for e.g. `xcm_executor::Config`:
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
//! * Stored reserve locations can be accessed by [impls::IsTrustedBridgedReserveForConcreteAsset] and configured for e.g. `xcm_executor::Config`:
//!   ```nocompile
//!   impl xcm_executor::Config for XcmConfig {
//! 	...
//!   	type IsReserve = IsTrustedBridgedReserveForConcreteAsset<Runtime>;
//! 	...
//!   }
//!   ```

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{transactional, BoundedBTreeSet};
use sp_std::boxed::Box;

mod types;
pub use types::*;

pub use pallet::*;
use xcm::prelude::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod impls;
pub mod weights;

/// The log target of this pallet.
pub const LOG_TARGET: &str = "runtime::bridge-transfer";

#[frame_support::pallet]
pub mod pallet {
	pub use crate::weights::WeightInfo;

	use super::*;
	use crate::filter::{AssetFilter, AssetFilterOf, MultiLocationFilterOf};
	use frame_support::{pallet_prelude::*, traits::EnsureOriginWithArg};
	use frame_system::{pallet_prelude::*, unique};
	use xcm_builder::ensure_is_remote;
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

	impl<T: Config> Pallet<T> {
		/// Validates destination and check if we support bridging to this remote global consensus
		///
		/// Returns: correct remote location, where we should be able to bridge
		pub(crate) fn ensure_reachable_remote_destination(
			remote_destination: VersionedMultiLocation,
		) -> Result<ReachableDestination<AssetFilterOf<T>>, Error<T>> {
			let remote_destination: MultiLocation = remote_destination
				.to_versioned()
				.map_err(|_| Error::<T>::UnsupportedXcmVersion)?;

			let devolved = ensure_is_remote(T::UniversalLocation::get(), remote_destination)
				.map_err(|_| Error::<T>::UnsupportedDestination)?;
			let (remote_network, _) = devolved;

			match AllowedExporters::<T>::get(remote_network) {
				Some(bridge_config) => {
					match bridge_config.allowed_target_location_for(&remote_destination) {
						Ok(Some((target_location, target_location_asset_filter))) =>
							Ok(ReachableDestination {
								bridge: bridge_config.to_bridge_location()?,
								target: target_location,
								target_asset_filter: target_location_asset_filter,
								target_destination: remote_destination,
							}),
						Ok(None) => Err(Error::<T>::UnsupportedDestination),
						Err(_) => Err(Error::<T>::UnsupportedXcmVersion),
					}
				},
				None => Err(Error::<T>::UnsupportedDestination),
			}
		}

		#[transactional]
		fn do_transfer_asset_via_bridge_in_transaction(
			origin_location: MultiLocation,
			destination: ReachableDestination<AssetFilterOf<T>>,
			assets: MultiAssets,
		) -> Result<(), DispatchError> {
			// Resolve reserve account
			let reserve_account = Self::resolve_reserve_account(&destination);

			// Target destination
			let target_location = destination.target.location;

			// UniversalLocation as sovereign account location on target_location (as target_location sees UniversalLocation)
			let universal_location_as_sovereign_account_on_target_location =
				T::UniversalLocation::get()
					.invert_target(&target_location)
					.map_err(|_| Error::<T>::InvalidConfiguration)?;

			// Prepare some XcmContext
			let xcm_context = XcmContext::with_message_id(unique(reserve_account));

			// Resolve/iterate all assets and how to transfer them
			let mut reserve_assets_deposited = xcm_executor::Assets::new();
			let mut reserved_assets_for_withdrawal = xcm_executor::Assets::new();
			for asset in assets.into_inner() {
				// first we need to know what kind of transfer it is
				match T::AssetTransferKindResolver::resolve(&asset, &target_location) {
					AssetTransferKind::ReserveBased =>
						// Move asset to reserve account
						T::AssetTransactor::transfer_asset(
							&asset,
							&origin_location,
							&reserve_account,
							&xcm_context,
						)
							.and_then(|reserved_asset| {
								Self::deposit_event(Event::ReserveAssetsDeposited {
									from: origin_location,
									to: reserve_account,
									assets: reserved_asset.clone().into(),
								});
								reserve_assets_deposited.subsume_assets(reserved_asset);
								Ok(())
							})
							.map_err(|e| {
								log::error!(
									target: LOG_TARGET,
									"AssetTransactor failed to reserve assets from origin_location: {:?} to reserve_account: {:?} for assets: {:?}, error: {:?}",
									origin_location,
									reserve_account,
									asset,
									e
								);
								Error::<T>::FailedToReserve
							})?,
					AssetTransferKind::WithdrawReserve => {
						// Just withdraw/burn asset here
						T::AssetTransactor::withdraw_asset(
							&asset,
							&origin_location,
							Some(&xcm_context),
						)
							.and_then(|withdrawn_asset| {
								Self::deposit_event(Event::AssetsWithdrawn {
									from: origin_location,
									assets: withdrawn_asset.clone().into(),
								});
								reserved_assets_for_withdrawal.subsume_assets(withdrawn_asset);
								Ok(())
							})
							.map_err(|e| {
								log::error!(
									target: LOG_TARGET,
									"AssetTransactor failed to withdraw assets from origin_location: {:?} for assets: {:?}, error: {:?}",
									origin_location,
									asset,
									e
								);
								Error::<T>::FailedToWithdraw
							})?
					}
					AssetTransferKind::Unsupported => return Err(Error::<T>::UnsupportedAssetTransferKind.into()),
				}
			}

			// Prepare xcm msg for the other side.

			// Reanchor stuff - we need to convert local asset id/MultiLocation to format that could be understood by different consensus and from their point-of-view
			reserve_assets_deposited.reanchor(&target_location, T::UniversalLocation::get(), None);
			reserved_assets_for_withdrawal.reanchor(
				&target_location,
				T::UniversalLocation::get(),
				None,
			);
			let remote_destination_reanchored = destination.target_destination
				.reanchored(&target_location, T::UniversalLocation::get())
				.map_err(|errored_dest| {
				log::error!(
					target: LOG_TARGET,
					"Failed to reanchor remote_destination: {:?} for target_destination: {:?} and universal_location: {:?}",
					errored_dest,
					target_location,
					T::UniversalLocation::get()
				);
				Error::<T>::InvalidRemoteDestination
			})?;

			// prepare xcm message
			// 1. buy execution (if needed) -> (we expect UniversalLocation's sovereign account should pay)
			let (mut xcm_instructions, maybe_buy_execution) = match destination.target.maybe_fee {
				Some(target_location_fee) => (
					sp_std::vec![
						WithdrawAsset(target_location_fee.clone().into()),
						BuyExecution { fees: target_location_fee.clone(), weight_limit: Unlimited },
					],
					Some(target_location_fee),
				),
				None => (
					sp_std::vec![UnpaidExecution { check_origin: None, weight_limit: Unlimited }],
					None,
				),
			};

			// 2. add deposit reserved asset to destination account (if any)
			if !reserve_assets_deposited.is_empty() {
				xcm_instructions.extend(sp_std::vec![
					ReserveAssetDeposited(reserve_assets_deposited.clone().into()),
					DepositAsset {
						assets: MultiAssetFilter::from(MultiAssets::from(reserve_assets_deposited)),
						beneficiary: remote_destination_reanchored
					},
				]);
			}

			// 3. add withdraw/move reserve from sovereign account to destination (if any)
			if !reserved_assets_for_withdrawal.is_empty() {
				xcm_instructions.extend(sp_std::vec![
					// we expect here, that origin is a sovereign account which was used as **reserve account**
					WithdrawAsset(reserved_assets_for_withdrawal.clone().into()),
					DepositAsset {
						assets: MultiAssetFilter::from(MultiAssets::from(
							reserved_assets_for_withdrawal
						)),
						beneficiary: remote_destination_reanchored
					},
				]);
			}

			// 4. add return unspent weight/asset back to the UniversalLocation's sovereign account on target
			if let Some(target_location_fee) = maybe_buy_execution {
				xcm_instructions.extend(sp_std::vec![
					RefundSurplus,
					DepositAsset {
						assets: MultiAssetFilter::from(MultiAssets::from(target_location_fee)),
						beneficiary: universal_location_as_sovereign_account_on_target_location
					},
				]);
			}

			Self::initiate_bridge_transfer(
				target_location,
				xcm_context.message_id,
				xcm_instructions.into(),
			)
			.map_err(Into::into)
		}

		fn initiate_bridge_transfer(
			dest: MultiLocation,
			message_id: XcmHash,
			mut xcm: Xcm<()>,
		) -> Result<(), Error<T>> {
			// append message_id
			xcm.0.extend(sp_std::vec![SetTopic(message_id)]);

			log::info!(
				target: LOG_TARGET,
				"[T::BridgeXcmSender] send to bridge, dest: {:?}, xcm: {:?}, message_id: {:?}",
				dest,
				xcm,
				message_id,
			);

			// call bridge
			// TODO: check-parameter - should we handle `sender_cost` somehow ?
			let (forwarded_message_id, sender_cost) = send_xcm::<T::BridgeXcmSender>(dest, xcm)
				.map_err(|e| {
					log::error!(
						target: LOG_TARGET,
						"[T::BridgeXcmSender] SendError occurred, error: {:?}",
						e
					);
					Error::<T>::BridgeCallError
				})?;

			// just fire event
			Self::deposit_event(Event::TransferInitiated {
				message_id,
				forwarded_message_id,
				sender_cost,
			});
			Ok(())
		}

		/// Resolve (sovereign) account which will be used as reserve account
		fn resolve_reserve_account(
			destination: &ReachableDestination<AssetFilterOf<T>>,
		) -> MultiLocation {
			destination.target.location
		}
	}
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use crate as bridge_transfer;
	use frame_support::traits::{
		AsEnsureOriginWithArg, ConstU32, Contains, ContainsPair, Currency,
	};

	use crate::{
		filter::{AssetFilter, MultiLocationFilter},
		impls::{
			AllowedUniversalAliasesOf, ConfiguredConcreteAssetTransferKindResolver,
			IsTrustedBridgedReserveForConcreteAsset,
		},
	};
	use frame_support::{
		assert_noop, assert_ok, dispatch::DispatchError, parameter_types, sp_io, sp_tracing,
		BoundedVec,
	};
	use frame_system::EnsureRoot;
	use polkadot_parachain::primitives::Sibling;
	use sp_runtime::{
		testing::{Header, H256},
		traits::{BlakeTwo256, IdentityLookup},
		AccountId32, ModuleError,
	};
	use sp_version::RuntimeVersion;
	use xcm_builder::{
		AccountId32Aliases, CurrencyAdapter, EnsureXcmOrigin, ExporterFor,
		GlobalConsensusParachainConvertsFor, IsConcrete, SiblingParachainConvertsVia,
		SignedToAccountId32, UnpaidRemoteExporter,
	};
	use xcm_executor::traits::Convert;

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
	type Block = frame_system::mocking::MockBlock<TestRuntime>;

	frame_support::construct_runtime!(
		pub enum TestRuntime where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
			Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
			BridgeTransfer: bridge_transfer::{Pallet, Call, Event<T>} = 52,
		}
	);

	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub Version: RuntimeVersion = RuntimeVersion {
			spec_name: sp_version::create_runtime_str!("test"),
			impl_name: sp_version::create_runtime_str!("system-test"),
			authoring_version: 1,
			spec_version: 1,
			impl_version: 1,
			apis: sp_version::create_apis_vec!([]),
			transaction_version: 1,
			state_version: 1,
		};
	}

	pub type AccountId = AccountId32;

	impl frame_system::Config for TestRuntime {
		type RuntimeOrigin = RuntimeOrigin;
		type RuntimeCall = RuntimeCall;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = AccountId;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type RuntimeEvent = RuntimeEvent;
		type BlockHashCount = BlockHashCount;
		type BlockLength = ();
		type BlockWeights = ();
		type Version = Version;
		type PalletInfo = PalletInfo;
		type AccountData = pallet_balances::AccountData<u64>;
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type DbWeight = ();
		type BaseCallFilter = frame_support::traits::Everything;
		type SystemWeightInfo = ();
		type SS58Prefix = ();
		type OnSetCode = ();
		type MaxConsumers = frame_support::traits::ConstU32<16>;
	}

	parameter_types! {
		pub const ExistentialDeposit: u64 = 5;
		pub const MaxReserves: u32 = 50;
	}

	impl pallet_balances::Config for TestRuntime {
		type Balance = u64;
		type RuntimeEvent = RuntimeEvent;
		type DustRemoval = ();
		type ExistentialDeposit = ExistentialDeposit;
		type AccountStore = System;
		type WeightInfo = ();
		type MaxLocks = ();
		type MaxReserves = MaxReserves;
		type ReserveIdentifier = [u8; 8];
		type RuntimeHoldReason = RuntimeHoldReason;
		type FreezeIdentifier = ();
		type MaxHolds = ConstU32<0>;
		type MaxFreezes = ConstU32<0>;
	}

	parameter_types! {
		pub const BridgedNetwork: NetworkId = NetworkId::ByGenesis([4; 32]);
		pub const RelayNetwork: NetworkId = NetworkId::ByGenesis([9; 32]);
		pub UniversalLocation: InteriorMultiLocation = X2(GlobalConsensus(RelayNetwork::get()), Parachain(1000));
		// Relay chain currency/balance location (e.g. KsmLocation, DotLocation, ..)
		pub const RelayLocation: MultiLocation = MultiLocation::parent();
	}

	std::thread_local! {
		static ROUTED_MESSAGE: std::cell::RefCell<Option<Xcm<()>>> = std::cell::RefCell::new(None);
	}

	pub struct ThreadLocalXcmRouter;
	impl SendXcm for ThreadLocalXcmRouter {
		type Ticket = Option<Xcm<()>>;

		fn validate(
			destination: &mut Option<MultiLocation>,
			message: &mut Option<Xcm<()>>,
		) -> SendResult<Self::Ticket> {
			log::info!(
				target: super::LOG_TARGET,
				"[ThreadLocalXcmRouter]: destination: {:?}, message: {:?}",
				destination,
				message
			);
			Ok((message.take(), MultiAssets::default()))
		}

		fn deliver(ticket: Self::Ticket) -> Result<XcmHash, SendError> {
			match ticket {
				Some(msg) => {
					ROUTED_MESSAGE.with(|rm| *rm.borrow_mut() = Some(msg));
					Ok([0u8; 32])
				},
				None => Err(SendError::MissingArgument),
			}
		}
	}

	pub struct NotApplicableOrFailOnParachain2222XcmRouter;
	impl SendXcm for NotApplicableOrFailOnParachain2222XcmRouter {
		type Ticket = Option<Xcm<()>>;

		fn validate(
			destination: &mut Option<MultiLocation>,
			message: &mut Option<Xcm<()>>,
		) -> SendResult<Self::Ticket> {
			log::info!(
				target: super::LOG_TARGET,
				"[NotApplicableOrFailOnParachain2222XcmRouter]: destination: {:?}, message: {:?}",
				destination,
				message
			);
			if matches!(
				destination,
				Some(MultiLocation {
					interior: X1(Parachain(Self::UNROUTABLE_PARA_ID)),
					parents: 1
				})
			) {
				Err(SendError::Transport("Simulate what ever error"))
			} else {
				Err(SendError::NotApplicable)
			}
		}

		fn deliver(ticket: Self::Ticket) -> Result<XcmHash, SendError> {
			unimplemented!("We should not come here, ticket: {:?}", ticket)
		}
	}

	impl NotApplicableOrFailOnParachain2222XcmRouter {
		const UNROUTABLE_PARA_ID: u32 = 2222;
	}

	pub type XcmRouter = (NotApplicableOrFailOnParachain2222XcmRouter, ThreadLocalXcmRouter);

	/// Bridge router, which wraps and sends xcm to BridgeHub to be delivered to the different GlobalConsensus
	pub type TestBridgeXcmSender =
		UnpaidRemoteExporter<BridgeTransfer, XcmRouter, UniversalLocation>;

	/// No local origins on this chain are allowed to dispatch XCM sends/executions.
	pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

	pub type LocationToAccountId = (
		// Sibling parachain origins convert to AccountId via the `ParaId::into`.
		SiblingParachainConvertsVia<Sibling, AccountId>,
		// Straight up local `AccountId32` origins just alias directly to `AccountId`.
		AccountId32Aliases<RelayNetwork, AccountId>,
		// Different global consensus parachain sovereign account.
		// (Used for over-bridge transfers and reserve processing)
		GlobalConsensusParachainConvertsFor<UniversalLocation, AccountId>,
	);

	/// Means for transacting the native currency on this chain.
	pub type CurrencyTransactor = CurrencyAdapter<
		// Use this currency:
		Balances,
		// Use this currency when it is a fungible asset matching the given location or name:
		IsConcrete<RelayLocation>,
		// Convert an XCM MultiLocation into a local account id:
		LocationToAccountId,
		// Our chain's account ID type (we can't get away without mentioning it explicitly):
		AccountId,
		// We don't track any teleports of `Balances`.
		(),
	>;

	/// Benchmarks helper.
	#[cfg(feature = "runtime-benchmarks")]
	pub struct TestBenchmarkHelper;

	#[cfg(feature = "runtime-benchmarks")]
	impl BenchmarkHelper<RuntimeOrigin> for TestBenchmarkHelper {
		fn bridge_config() -> (NetworkId, BridgeConfig) {
			test_bridge_config()
		}

		fn prepare_asset_transfer() -> (RuntimeOrigin, VersionedMultiAssets, VersionedMultiLocation)
		{
			let assets_count = MaxAssetsLimit::get();

			// sender account must have enough funds
			let sender_account = account(1);
			let total_deposit = ExistentialDeposit::get() * (1 + assets_count as u64);
			let _ = Balances::deposit_creating(&sender_account, total_deposit);

			// finally - prepare assets and destination
			let assets = VersionedMultiAssets::V3(
				std::iter::repeat(MultiAsset {
					fun: Fungible(ExistentialDeposit::get().into()),
					id: Concrete(RelayLocation::get()),
				})
				.take(assets_count as usize)
				.collect::<Vec<_>>()
				.into(),
			);
			let destination = VersionedMultiLocation::V3(MultiLocation::new(
				2,
				X3(GlobalConsensus(Wococo), Parachain(1000), consensus_account(Wococo, 2)),
			));

			(RuntimeOrigin::signed(sender_account), assets, destination)
		}
	}

	parameter_types! {
		pub const TrapCode: u64 = 12345;
		pub const MaxAssetsLimit: u8 = 1;
	}

	impl Config for TestRuntime {
		type RuntimeEvent = RuntimeEvent;
		type UniversalLocation = UniversalLocation;
		type WeightInfo = ();
		type AdminOrigin = EnsureRoot<AccountId>;
		type AllowReserveAssetTransferOrigin = AsEnsureOriginWithArg<EnsureRoot<AccountId>>;
		type UniversalAliasesLimit = ConstU32<2>;
		type ReserveLocationsLimit = ConstU32<2>;
		type AssetsPerReserveLocationLimit = ConstU32<2>;
		type AssetTransactor = CurrencyTransactor;
		type AssetTransferKindResolver = ConfiguredConcreteAssetTransferKindResolver<Self>;
		type BridgeXcmSender = TestBridgeXcmSender;
		type AssetTransferOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
		type MaxAssetsLimit = MaxAssetsLimit;
		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper = TestBenchmarkHelper;
	}

	pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
		sp_tracing::try_init_simple();
		let t = frame_system::GenesisConfig::default().build_storage::<TestRuntime>().unwrap();

		// with 0 block_number events dont work
		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| {
			frame_system::Pallet::<TestRuntime>::set_block_number(1u32.into());
		});

		ext
	}

	fn account(account: u8) -> AccountId32 {
		AccountId32::new([account; 32])
	}

	fn consensus_account(network: NetworkId, account: u8) -> Junction {
		xcm::prelude::AccountId32 {
			network: Some(network),
			id: AccountId32::new([account; 32]).into(),
		}
	}

	#[test]
	fn test_ensure_reachable_remote_destination() {
		new_test_ext().execute_with(|| {
			// insert exporter config + allowed target location
			let bridged_network = BridgedNetwork::get();
			let bridge_location = MultiLocation::new(1, X1(Parachain(1013)));
			assert_ok!(BridgeTransfer::add_exporter_config(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(bridge_location.clone().into_versioned()),
				None,
			));
			let target_location: MultiLocation =
				MultiLocation::new(2, X2(GlobalConsensus(bridged_network), Parachain(1000)));
			let target_location_fee: MultiAsset = (MultiLocation::parent(), 1_000_000).into();
			assert_ok!(BridgeTransfer::update_bridged_target_location(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(target_location.clone().into_versioned()),
				Some(Box::new(target_location_fee.clone().into())),
			));

			// v2 not supported
			assert_eq!(
				BridgeTransfer::ensure_reachable_remote_destination(VersionedMultiLocation::V2(
					xcm::v2::MultiLocation::default()
				)),
				Err(Error::<TestRuntime>::UnsupportedDestination)
			);

			// v3 - "parent: 0" wrong
			assert_eq!(
				BridgeTransfer::ensure_reachable_remote_destination(VersionedMultiLocation::V3(
					MultiLocation::new(0, X2(GlobalConsensus(bridged_network), Parachain(1000)))
				)),
				Err(Error::<TestRuntime>::UnsupportedDestination)
			);
			// v3 - "parent: 1" wrong
			assert_eq!(
				BridgeTransfer::ensure_reachable_remote_destination(VersionedMultiLocation::V3(
					MultiLocation::new(1, X2(GlobalConsensus(bridged_network), Parachain(1000)))
				)),
				Err(Error::<TestRuntime>::UnsupportedDestination)
			);

			// v3 - Rococo is not supported
			assert_eq!(
				BridgeTransfer::ensure_reachable_remote_destination(VersionedMultiLocation::V3(
					MultiLocation::new(2, X2(GlobalConsensus(Rococo), Parachain(1000)))
				)),
				Err(Error::<TestRuntime>::UnsupportedDestination)
			);

			// v3 - remote_destination is not allowed
			assert_eq!(
				BridgeTransfer::ensure_reachable_remote_destination(VersionedMultiLocation::V3(
					MultiLocation::new(2, X2(GlobalConsensus(bridged_network), Parachain(1234)))
				)),
				Err(Error::<TestRuntime>::UnsupportedDestination)
			);

			// v3 - ok (allowed)
			assert_ok!(
				BridgeTransfer::ensure_reachable_remote_destination(VersionedMultiLocation::V3(
					MultiLocation::new(
						2,
						X3(
							GlobalConsensus(bridged_network),
							Parachain(1000),
							consensus_account(bridged_network, 35)
						)
					),
				)),
				ReachableDestination {
					bridge: MaybePaidLocation { location: bridge_location, maybe_fee: None },
					target: MaybePaidLocation {
						location: MultiLocation::new(
							2,
							X2(GlobalConsensus(bridged_network), Parachain(1000))
						),
						maybe_fee: Some(target_location_fee),
					},
					target_asset_filter: None,
					target_destination: MultiLocation::new(
						2,
						X3(
							GlobalConsensus(bridged_network),
							Parachain(1000),
							consensus_account(bridged_network, 35)
						)
					),
				}
			);
		})
	}

	#[test]
	fn test_transfer_asset_via_bridge_for_currency_works() {
		new_test_ext().execute_with(|| {
			// initialize some Balances for user_account
			let user_account = account(1);
			let user_account_init_balance = 1000_u64;
			let _ = Balances::deposit_creating(&user_account, user_account_init_balance);
			let user_free_balance = Balances::free_balance(&user_account);
			let balance_to_transfer = 15_u64;
			assert!((user_free_balance - balance_to_transfer) >= ExistentialDeposit::get());
			// because, sovereign account needs to have ED otherwise reserve fails
			assert!(balance_to_transfer >= ExistentialDeposit::get());

			// insert bridge config
			let bridged_network = BridgedNetwork::get();
			let bridge_location: MultiLocation = (Parent, Parachain(1013)).into();
			assert_ok!(BridgeTransfer::add_exporter_config(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(bridge_location.into_versioned()),
				None,
			));
			// insert allowed reserve asset and allow all
			let target_location: MultiLocation =
				MultiLocation::new(2, X2(GlobalConsensus(bridged_network), Parachain(1000)));
			assert_ok!(BridgeTransfer::update_bridged_target_location(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(target_location.clone().into_versioned()),
				None,
			));
			assert_ok!(BridgeTransfer::allow_reserve_asset_transfer_for(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(target_location.clone().into_versioned()),
				AssetFilter::All,
			));

			// checks before
			assert!(ROUTED_MESSAGE.with(|r| r.borrow().is_none()));
			assert_eq!(Balances::free_balance(&user_account), user_account_init_balance);
			let reserve_account = LocationToAccountId::convert_ref(target_location)
				.expect("converted target_location as accountId");
			assert_eq!(Balances::free_balance(&reserve_account), 0);

			// trigger transfer_asset_via_bridge - should trigger new ROUTED_MESSAGE
			let asset = MultiAsset {
				fun: Fungible(balance_to_transfer.into()),
				id: Concrete(RelayLocation::get()),
			};
			let assets = Box::new(VersionedMultiAssets::from(MultiAssets::from(asset)));

			// destination is account from different consensus
			let destination = Box::new(VersionedMultiLocation::from(MultiLocation::new(
				2,
				X3(
					GlobalConsensus(bridged_network),
					Parachain(1000),
					consensus_account(bridged_network, 2),
				),
			)));

			// trigger asset transfer
			assert_ok!(BridgeTransfer::transfer_asset_via_bridge(
				RuntimeOrigin::signed(account(1)),
				assets,
				destination,
			));

			// check user account decressed
			assert_eq!(
				Balances::free_balance(&user_account),
				user_account_init_balance - balance_to_transfer
			);
			// check reserve account increased
			assert_eq!(Balances::free_balance(&reserve_account), 15);

			// check events
			let events = System::events();
			assert!(!events.is_empty());

			// check reserve asset deposited event
			assert!(System::events().iter().any(|r| matches!(
				r.event,
				RuntimeEvent::BridgeTransfer(Event::ReserveAssetsDeposited { .. })
			)));
			assert!(System::events().iter().any(|r| matches!(
				r.event,
				RuntimeEvent::BridgeTransfer(Event::TransferInitiated { .. })
			)));

			// check fired XCM ExportMessage to bridge-hub
			let fired_xcm =
				ROUTED_MESSAGE.with(|r| r.take().expect("xcm::ExportMessage should be here"));

			if let Some(ExportMessage { xcm, .. }) = fired_xcm.0.iter().find(|instr| {
				matches!(
					instr,
					ExportMessage {
						network,
						destination: X1(Parachain(1000)),
						..
					} if network == &bridged_network
				)
			}) {
				assert!(xcm.0.iter().any(|instr| matches!(instr, UnpaidExecution { .. })));
				assert!(xcm.0.iter().any(|instr| matches!(instr, ReserveAssetDeposited(..))));
				assert!(xcm.0.iter().any(|instr| matches!(instr, DepositAsset { .. })));
				assert!(xcm.0.iter().any(|instr| matches!(instr, SetTopic { .. })));
			} else {
				assert!(false, "Does not contains [`ExportMessage`], fired_xcm: {:?}", fired_xcm);
			}
		});
	}

	#[test]
	fn test_transfer_asset_via_bridge_in_case_of_error_transactional_works() {
		new_test_ext().execute_with(|| {
			// initialize some Balances for user_account
			let user_account = account(1);
			let user_account_init_balance = 1000_u64;
			let _ = Balances::deposit_creating(&user_account, user_account_init_balance);
			let user_free_balance = Balances::free_balance(&user_account);
			let balance_to_transfer = 15_u64;
			assert!((user_free_balance - balance_to_transfer) >= ExistentialDeposit::get());
			// because, sovereign account needs to have ED otherwise reserve fails
			assert!(balance_to_transfer >= ExistentialDeposit::get());

			// insert bridge config (with unroutable bridge_location - 2222)
			let bridged_network = BridgedNetwork::get();
			assert_ok!(BridgeTransfer::add_exporter_config(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(
					MultiLocation::new(
						1,
						Parachain(NotApplicableOrFailOnParachain2222XcmRouter::UNROUTABLE_PARA_ID)
					)
					.into_versioned()
				),
				None,
			));
			let target_location =
				MultiLocation::new(2, X2(GlobalConsensus(bridged_network), Parachain(1000)));
			assert_ok!(BridgeTransfer::update_bridged_target_location(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(target_location.clone().into_versioned()),
				None,
			));
			assert_ok!(BridgeTransfer::allow_reserve_asset_transfer_for(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(target_location.into_versioned()),
				AssetFilter::All,
			));

			// checks before
			assert!(ROUTED_MESSAGE.with(|r| r.borrow().is_none()));
			let user_balance_before = Balances::free_balance(&user_account);
			assert_eq!(user_balance_before, user_account_init_balance);
			let reserve_account = LocationToAccountId::convert_ref(target_location)
				.expect("converted target_location as accountId");
			let reserve_account_before = Balances::free_balance(&reserve_account);
			assert_eq!(reserve_account_before, 0);

			// trigger transfer_asset_via_bridge - should trigger new ROUTED_MESSAGE
			let asset = MultiAsset {
				fun: Fungible(balance_to_transfer.into()),
				id: Concrete(RelayLocation::get()),
			};
			let assets = Box::new(VersionedMultiAssets::from(MultiAssets::from(asset)));

			// destination is account from different consensus
			let destination = Box::new(VersionedMultiLocation::from(MultiLocation::new(
				2,
				X3(
					GlobalConsensus(bridged_network),
					Parachain(1000),
					consensus_account(bridged_network, 2),
				),
			)));

			// reset events
			System::reset_events();

			// trigger asset transfer
			assert_noop!(
				BridgeTransfer::transfer_asset_via_bridge(
					RuntimeOrigin::signed(account(1)),
					assets,
					destination
				),
				DispatchError::Module(ModuleError {
					index: 52,
					error: [9, 0, 0, 0],
					message: Some("BridgeCallError")
				})
			);

			// checks after
			// balances are untouched
			assert_eq!(Balances::free_balance(&user_account), user_balance_before);
			assert_eq!(Balances::free_balance(&reserve_account), reserve_account_before);
			// no xcm messages fired
			assert!(ROUTED_MESSAGE.with(|r| r.borrow().is_none()));
			// check events (no events because of rollback)
			assert!(System::events().is_empty());
		});
	}

	#[test]
	fn allowed_exporters_management_works() {
		let bridged_network = BridgedNetwork::get();
		let bridge_location = MultiLocation::new(1, X1(Parachain(1013)));
		let dummy_xcm = Xcm(vec![]);
		let dummy_remote_interior_multilocation = X1(Parachain(1234));

		new_test_ext().execute_with(|| {
			assert_eq!(AllowedExporters::<TestRuntime>::iter().count(), 0);

			// should fail - just root is allowed
			assert_noop!(
				BridgeTransfer::add_exporter_config(
					RuntimeOrigin::signed(account(1)),
					bridged_network,
					Box::new(bridge_location.clone().into_versioned()),
					None
				),
				DispatchError::BadOrigin
			);

			// should fail - we expect local bridge
			assert_noop!(
				BridgeTransfer::add_exporter_config(
					RuntimeOrigin::root(),
					bridged_network,
					Box::new(MultiLocation::new(2, X1(Parachain(1234))).into_versioned()),
					None
				),
				DispatchError::Module(ModuleError {
					index: 52,
					error: [0, 0, 0, 0],
					message: Some("InvalidConfiguration")
				})
			);
			assert_eq!(AllowedExporters::<TestRuntime>::iter().count(), 0);
			assert_eq!(
				BridgeTransfer::exporter_for(
					&bridged_network,
					&dummy_remote_interior_multilocation,
					&dummy_xcm
				),
				None
			);

			// add with root
			assert_ok!(BridgeTransfer::add_exporter_config(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(bridge_location.clone().into_versioned()),
				None
			));
			assert_eq!(AllowedExporters::<TestRuntime>::iter().count(), 1);
			assert_eq!(
				AllowedExporters::<TestRuntime>::get(bridged_network),
				Some(BridgeConfig {
					bridge_location: VersionedMultiLocation::from(bridge_location),
					bridge_location_fee: None,
					allowed_target_locations: Default::default(),
				})
			);
			assert_eq!(AllowedExporters::<TestRuntime>::get(&RelayNetwork::get()), None);
			assert_eq!(
				BridgeTransfer::exporter_for(
					&bridged_network,
					&dummy_remote_interior_multilocation,
					&dummy_xcm
				),
				Some((bridge_location.clone(), None))
			);
			assert_eq!(
				BridgeTransfer::exporter_for(
					&RelayNetwork::get(),
					&dummy_remote_interior_multilocation,
					&dummy_xcm
				),
				None
			);

			// update fee
			// remove
			assert_ok!(BridgeTransfer::update_exporter_config(
				RuntimeOrigin::root(),
				bridged_network,
				Some(VersionedMultiAsset::V3((Parent, 200u128).into()).into()),
			));
			assert_eq!(AllowedExporters::<TestRuntime>::iter().count(), 1);
			assert_eq!(
				AllowedExporters::<TestRuntime>::get(bridged_network),
				Some(BridgeConfig {
					bridge_location: VersionedMultiLocation::from(bridge_location),
					bridge_location_fee: Some((Parent, 200u128).into()),
					allowed_target_locations: Default::default(),
				})
			);
			assert_eq!(
				BridgeTransfer::exporter_for(
					&bridged_network,
					&dummy_remote_interior_multilocation,
					&dummy_xcm
				),
				Some((bridge_location, Some((Parent, 200u128).into())))
			);

			// remove
			assert_ok!(BridgeTransfer::remove_exporter_config(
				RuntimeOrigin::root(),
				bridged_network,
			));
			assert_eq!(AllowedExporters::<TestRuntime>::get(bridged_network), None);
			assert_eq!(AllowedExporters::<TestRuntime>::iter().count(), 0);
		})
	}

	#[test]
	fn allowed_universal_aliases_management_works() {
		new_test_ext().execute_with(|| {
			assert_eq!(AllowedUniversalAliases::<TestRuntime>::iter().count(), 0);

			let location1 = MultiLocation::new(1, X1(Parachain(1014)));
			let junction1 = GlobalConsensus(ByGenesis([1; 32]));
			let junction2 = GlobalConsensus(ByGenesis([2; 32]));

			// should fail - just root is allowed
			assert_noop!(
				BridgeTransfer::add_universal_alias(
					RuntimeOrigin::signed(account(1)),
					Box::new(VersionedMultiLocation::V3(location1.clone())),
					junction1.clone(),
				),
				DispatchError::BadOrigin
			);
			assert_eq!(AllowedUniversalAliases::<TestRuntime>::iter().count(), 0);
			assert!(!AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction1)));
			assert!(!AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction2)));

			// add ok
			assert_ok!(BridgeTransfer::add_universal_alias(
				RuntimeOrigin::root(),
				Box::new(VersionedMultiLocation::V3(location1.clone())),
				junction1.clone(),
			));
			assert_ok!(BridgeTransfer::add_universal_alias(
				RuntimeOrigin::root(),
				Box::new(VersionedMultiLocation::V3(location1.clone())),
				junction2.clone(),
			));
			assert!(AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction1)));
			assert!(AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction2)));
			assert_eq!(AllowedUniversalAliases::<TestRuntime>::iter().count(), 1);

			// remove ok
			assert_ok!(BridgeTransfer::remove_universal_alias(
				RuntimeOrigin::root(),
				Box::new(VersionedMultiLocation::V3(location1.clone())),
				vec![junction1.clone()],
			));
			assert!(!AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction1)));
			assert!(AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction2)));
			assert_eq!(AllowedUniversalAliases::<TestRuntime>::iter().count(), 1);

			assert_ok!(BridgeTransfer::remove_universal_alias(
				RuntimeOrigin::root(),
				Box::new(VersionedMultiLocation::V3(location1.clone())),
				vec![junction2.clone()],
			));
			assert!(!AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction1)));
			assert!(!AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction2)));
			assert_eq!(AllowedUniversalAliases::<TestRuntime>::iter().count(), 0);
		})
	}

	#[test]
	fn allowed_reserve_locations_management_works() {
		new_test_ext().execute_with(|| {
			assert_eq!(0, AllowedReserveLocations::<TestRuntime>::iter_values().count());

			let location1 = MultiLocation::new(1, X1(Parachain(1014)));
			let location1_as_latest = VersionedMultiLocation::from(location1.clone());
			let location1_as_latest_as_key =
				LatestVersionedMultiLocation::try_from(&location1_as_latest).expect("ok");
			let location2 =
				MultiLocation::new(2, X2(GlobalConsensus(ByGenesis([1; 32])), Parachain(1014)));
			let location2_as_latest = VersionedMultiLocation::from(location2.clone());
			let location2_as_key =
				LatestVersionedMultiLocation::try_from(&location2_as_latest).expect("ok");

			let asset_location = MultiLocation::parent();
			let asset: MultiAsset = (asset_location, 200u128).into();

			let asset_filter_for_asset_by_multilocation = MultiLocationFilter {
				equals_any: BoundedVec::truncate_from(vec![asset_location.into_versioned()]),
				starts_with_any: Default::default(),
			};
			let asset_filter_for_asset =
				AssetFilter::ByMultiLocation(asset_filter_for_asset_by_multilocation.clone());
			let asset_filter_for_other_by_multilocation = MultiLocationFilter {
				equals_any: BoundedVec::truncate_from(vec![
					MultiLocation::new(3, Here).into_versioned()
				]),
				starts_with_any: Default::default(),
			};
			let asset_filter_for_other =
				AssetFilter::ByMultiLocation(asset_filter_for_other_by_multilocation.clone());

			// should fail - just root is allowed
			assert_noop!(
				BridgeTransfer::add_reserve_location(
					RuntimeOrigin::signed(account(1)),
					Box::new(location1_as_latest.clone()),
					asset_filter_for_asset.clone(),
				),
				DispatchError::BadOrigin
			);
			assert!(
				AllowedReserveLocations::<TestRuntime>::get(&location1_as_latest_as_key).is_none()
			);
			assert!(AllowedReserveLocations::<TestRuntime>::get(&location2_as_key).is_none());
			assert!(!IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
				&asset, &location1
			));
			assert!(!IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
				&asset, &location2
			));
			assert_eq!(
				ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(
					&asset, &location1
				),
				AssetTransferKind::Unsupported
			);
			assert_eq!(
				ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(
					&asset, &location2
				),
				AssetTransferKind::Unsupported
			);

			// add ok
			assert_ok!(BridgeTransfer::add_reserve_location(
				RuntimeOrigin::root(),
				Box::new(VersionedMultiLocation::V3(location1.clone())),
				asset_filter_for_asset.clone()
			));
			assert_ok!(BridgeTransfer::add_reserve_location(
				RuntimeOrigin::root(),
				Box::new(VersionedMultiLocation::V3(location2.clone())),
				asset_filter_for_other.clone()
			));
			assert_eq!(2, AllowedReserveLocations::<TestRuntime>::iter_values().count());
			assert!(IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
				&asset, &location1
			));
			assert!(!IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
				&asset, &location2
			));
			assert_eq!(
				ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(
					&asset, &location1
				),
				AssetTransferKind::WithdrawReserve
			);
			assert_eq!(
				ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(
					&asset, &location2
				),
				AssetTransferKind::Unsupported
			);

			assert_ok!(BridgeTransfer::add_reserve_location(
				RuntimeOrigin::root(),
				Box::new(location2_as_latest.clone()),
				asset_filter_for_asset.clone()
			));
			assert!(IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
				&asset, &location2
			));
			assert_eq!(
				ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(
					&asset, &location2
				),
				AssetTransferKind::WithdrawReserve
			);

			// test remove
			assert_noop!(
				BridgeTransfer::remove_reserve_location(
					RuntimeOrigin::root(),
					Box::new(location1_as_latest.clone()),
					Some(asset_filter_for_other_by_multilocation.clone())
				),
				DispatchError::Module(ModuleError {
					index: 52,
					error: [0, 0, 0, 0],
					message: Some("UnavailableConfiguration")
				})
			);

			assert_ok!(BridgeTransfer::remove_reserve_location(
				RuntimeOrigin::root(),
				Box::new(location1_as_latest.clone()),
				Some(asset_filter_for_asset_by_multilocation.clone())
			));
			assert_eq!(1, AllowedReserveLocations::<TestRuntime>::iter_values().count());
			assert!(!IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
				&asset, &location1
			));
			assert!(IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
				&asset, &location2
			));
			assert_eq!(
				ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(
					&asset, &location1
				),
				AssetTransferKind::Unsupported
			);
			assert_eq!(
				ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(
					&asset, &location2
				),
				AssetTransferKind::WithdrawReserve
			);

			assert_ok!(BridgeTransfer::remove_reserve_location(
				RuntimeOrigin::root(),
				Box::new(location2_as_latest.clone()),
				Some(asset_filter_for_other_by_multilocation.clone())
			));
			assert!(!IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
				&asset, &location1
			));
			assert!(IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
				&asset, &location2
			));
			assert_eq!(
				ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(
					&asset, &location1
				),
				AssetTransferKind::Unsupported
			);
			assert_eq!(
				ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(
					&asset, &location2
				),
				AssetTransferKind::WithdrawReserve
			);

			assert_ok!(BridgeTransfer::remove_reserve_location(
				RuntimeOrigin::root(),
				Box::new(location2_as_latest),
				Some(asset_filter_for_asset_by_multilocation)
			));
			assert!(!IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
				&asset, &location1
			));
			assert!(!IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
				&asset, &location2
			));
			assert_eq!(
				ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(
					&asset, &location1
				),
				AssetTransferKind::Unsupported
			);
			assert_eq!(
				ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(
					&asset, &location2
				),
				AssetTransferKind::Unsupported
			);
		})
	}

	#[test]
	fn allowed_bridged_target_location_management_works() {
		new_test_ext().execute_with(|| {
			assert_eq!(0, AllowedExporters::<TestRuntime>::iter_values().count());

			let bridged_network = BridgedNetwork::get();
			let bridge_location: MultiLocation = (Parent, Parachain(1013)).into();
			let target_location: MultiLocation =
				MultiLocation::new(2, X2(GlobalConsensus(bridged_network), Parachain(1000)));

			// should fail - we need BridgeConfig first
			assert_noop!(
				BridgeTransfer::allow_reserve_asset_transfer_for(
					RuntimeOrigin::root(),
					bridged_network,
					Box::new(target_location.clone().into_versioned()),
					AssetFilter::All,
				),
				DispatchError::Module(ModuleError {
					index: 52,
					error: [2, 0, 0, 0],
					message: Some("UnavailableConfiguration")
				})
			);

			// add bridge config
			assert_ok!(BridgeTransfer::add_exporter_config(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(bridge_location.into_versioned()),
				None,
			));

			// should fail - we need also target_location first
			assert_noop!(
				BridgeTransfer::allow_reserve_asset_transfer_for(
					RuntimeOrigin::root(),
					bridged_network,
					Box::new(target_location.clone().into_versioned()),
					AssetFilter::All,
				),
				DispatchError::Module(ModuleError {
					index: 52,
					error: [1, 0, 0, 0],
					message: Some("InvalidBridgeConfiguration")
				})
			);

			// insert allowed target location
			assert_ok!(BridgeTransfer::update_bridged_target_location(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(target_location.clone().into_versioned()),
				None,
			));

			let asset1_location = MultiLocation::new(2, X2(Parachain(1235), Parachain(5678)));
			let asset1 = MultiAsset::from((asset1_location, 1000));
			assert_eq!(
				ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(
					&asset1,
					&target_location
				),
				AssetTransferKind::Unsupported
			);

			// now should pass - add one start_with pattern
			assert_ok!(BridgeTransfer::allow_reserve_asset_transfer_for(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(target_location.clone().into_versioned()),
				AssetFilter::ByMultiLocation(MultiLocationFilter {
					equals_any: Default::default(),
					starts_with_any: BoundedVec::truncate_from(vec![MultiLocation::new(
						2,
						X1(Parachain(2223))
					)
					.into_versioned()]),
				}),
			));

			// not allowed yet
			assert_eq!(
				ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(
					&asset1,
					&target_location
				),
				AssetTransferKind::Unsupported
			);

			// now should pass - add another start_with pattern
			assert_ok!(BridgeTransfer::allow_reserve_asset_transfer_for(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(target_location.clone().into_versioned()),
				AssetFilter::ByMultiLocation(MultiLocationFilter {
					equals_any: Default::default(),
					starts_with_any: BoundedVec::truncate_from(vec![MultiLocation::new(
						2,
						X1(Parachain(1235))
					)
					.into_versioned()]),
				}),
			));

			// ok
			assert_eq!(
				ConfiguredConcreteAssetTransferKindResolver::<TestRuntime>::resolve(
					&asset1,
					&target_location
				),
				AssetTransferKind::ReserveBased
			);
		})
	}
}
