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
//! Module which could help with different transfers through bridges,
//! e.g. move assets between different global consensus...
//!
//! ## Overview
//!
//! Pallet supports configuration for two independent scenarios:
//!
//! ### Transfer out
//!
//! * see (Config for transfer out) in the code
//! * if you want to allow initiate bridge transfer from runtime,
//!   actually pallet supports asset transfer and ping with dedicated extrinsics `transfer_asset_via_bridge` / `ping_via_bridge`
//! * e.g. for asset transfer with correct configuration it sends `ReserveAssetDeposited` over bridge,
//!   you can configure bridge location and allowed target location with `AllowedExporters`
//!
//! ### Transfer in
//!
//! * see (Config for transfer in) in the code
//! * e.g. if you want to allow process xcm `UniversalOrigin` instruction,
//!   you can configure "allowed universal aliases" here and then use it for `xcm_executor::Config`:
//!   `type UniversalAliases = AllowedUniversalAliasesOf<Runtime>;`
//! * e.g. if you want to allow process xcm `ReserveAssetDeposited` instruction,
//!   you can configure "allowed reserve locations" here and then use it for `xcm_executor::Config`:
//!   ```nocompile
//!   type IsReserve = IsAllowedReserveOf<
//!			Runtime,
//!			IsDifferentGlobalConsensusConcreteAsset<UniversalLocationNetworkId>,
//!	  >;
//!   ```
//!
//! Transfer in/out are independent so you can configure just to receive or just to send part.
//! All configuration is done by dedicated extrinsics under `AdminOrigin` so for example runtime can allow to change this configuration just by governance.

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{transactional, BoundedBTreeSet};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::boxed::Box;

pub use pallet::*;
use xcm::prelude::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod impls;
pub mod weights;

/// The log target of this pallet.
pub const LOG_TARGET: &str = "runtime::bridge-transfer";

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct BridgeConfig {
	/// Contains location, which is able to bridge XCM messages to bridged network
	pub bridge_location: MultiLocation,
	/// Fee which could be needed to pay in `bridge_location`
	/// `MultiAsset` is here from the point of view of `bridge_location`, e.g.: `MultiLocation::parent()` means relay chain token of `bridge_location`
	pub bridge_location_fee: Option<MultiAsset>,

	/// Contains target destination on bridged network. E.g.: MultiLocation of Statemine/t on different consensus
	// TODO:check-parameter - lets start with 1..1, maybe later we could extend this with BoundedVec
	// TODO: bridged bridge-hub should have router for this
	pub allowed_target_location: MultiLocation,
	// TODO:check-parameter - can we store Option<Weight> and then aviod using `Unlimited`?
	/// If `None` then `UnpaidExecution` is used, else `Withdraw(target_location_fee)/BuyExecution(target_location_fee, Unlimited)`
	/// `MultiAsset` is here from the point of view of `allowed_target_location`, e.g.: `MultiLocation::parent()` means relay chain token of `allowed_target_location`
	pub max_target_location_fee: Option<MultiAsset>,
}

/// Trait for constructing ping message.
pub trait PingMessageBuilder {
	fn try_build(
		local_origin: &MultiLocation,
		network: &NetworkId,
		remote_destination: &MultiLocation,
	) -> Option<Xcm<()>>;
}

impl PingMessageBuilder for () {
	fn try_build(_: &MultiLocation, _: &NetworkId, _: &MultiLocation) -> Option<Xcm<()>> {
		None
	}
}

/// Builder creates xcm message just with `Trap` instruction.
pub struct UnpaidTrapMessageBuilder<TrapCode>(sp_std::marker::PhantomData<TrapCode>);
impl<TrapCode: frame_support::traits::Get<u64>> PingMessageBuilder
	for UnpaidTrapMessageBuilder<TrapCode>
{
	fn try_build(_: &MultiLocation, _: &NetworkId, _: &MultiLocation) -> Option<Xcm<()>> {
		Some(Xcm(sp_std::vec![Trap(TrapCode::get())]))
	}
}

#[frame_support::pallet]
pub mod pallet {
	pub use crate::weights::WeightInfo;

	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
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

		/// Prepare environment for ping transfer and return transfer origin and assets
		/// to transfer. After this function is called, we expect `ping_via_bridge`
		/// to succeed, so in proper environment, it should:
		///
		/// - deposit enough funds (fee from `bridge_config()`) to the sender account;
		///
		/// - ensure that the `BridgeXcmSender` is properly configured for the transfer;
		///
		/// - be close to the worst possible scenario - i.e. if some account may need to be created during
		///  it should be created. If there are multiple bridges, the "worst possible"
		///   (in terms of performance) bridge must be selected for the transfer.
		fn prepare_ping_transfer() -> Option<(RuntimeOrigin, VersionedMultiLocation)> {
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

		/// The configurable origin to allow bridges configuration management
		type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Max allowed universal aliases per one `MultiLocation`
		/// (Config for transfer in)
		type UniversalAliasesLimit: Get<u32>;
		/// Max allowed reserve locations
		/// (Config for transfer in)
		type ReserveLocationsLimit: Get<u32>;

		/// How to withdraw and deposit an asset for reserve.
		/// (Config for transfer out)
		type AssetTransactor: TransactAsset;
		/// XCM sender which sends messages to the BridgeHub
		/// (Config for transfer out)
		type BridgeXcmSender: SendXcm;
		/// Required origin for asset transfer. If successful, it resolves to `MultiLocation`.
		/// (Config for transfer out)
		type TransferAssetOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = MultiLocation>;
		/// Max count of assets in one call
		/// (Config for transfer out)
		type MaxAssetsLimit: Get<u8>;
		/// Required origin for ping transfer. If successful, it resolves to `MultiLocation`.
		/// (Config for transfer out)
		type TransferPingOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = MultiLocation>;
		/// Configurable ping message, `None` means no message will be transferred.
		/// (Config for transfer out)
		type PingMessageBuilder: PingMessageBuilder;

		/// Benchmarks helper.
		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper: BenchmarkHelper<Self::RuntimeOrigin>;
	}

	/// Details of configured bridges which are allowed for **transfer out**.
	/// (Config for transfer out)
	#[pallet::storage]
	#[pallet::getter(fn allowed_exporters)]
	pub(super) type AllowedExporters<T: Config> =
		StorageMap<_, Blake2_128Concat, NetworkId, BridgeConfig>;

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
		MultiLocation,
		BoundedBTreeSet<Junction, T::UniversalAliasesLimit>,
		ValueQuery,
	>;

	/// Holds allowed mappings `MultiLocation` as trusted reserve locations
	/// (Config for transfer in)
	#[pallet::storage]
	#[pallet::getter(fn allowed_reserve_locations)]
	pub(super) type AllowedReserveLocations<T: Config> =
		StorageValue<_, BoundedBTreeSet<MultiLocation, T::ReserveLocationsLimit>, ValueQuery>;

	#[pallet::error]
	#[cfg_attr(test, derive(PartialEq))]
	pub enum Error<T> {
		InvalidConfiguration,
		UnavailableConfiguration,
		ConfigurationAlreadyExists,
		InvalidAssets,
		MaxAssetsLimitReached,
		UnsupportedDestination,
		UnsupportedXcmVersion,
		InvalidRemoteDestination,
		BridgeCallError,
		FailedToReserve,
		UnsupportedPing,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Transfer was successfully entered to the system (does not mean already delivered)
		TransferInitiated { message_hash: XcmHash, sender_cost: MultiAssets },

		/// Reserve asset passed
		ReserveAssetsDeposited { from: MultiLocation, to: MultiLocation, assets: MultiAssets },

		/// New bridge configuration was added
		BridgeAdded,
		/// Bridge configuration was removed
		BridgeRemoved,
		/// Bridge configuration was updated
		BridgeUpdated,

		/// New universal alias was added
		UniversalAliasAdded,
		/// New universal alias was removed
		UniversalAliasRemoved,

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
			let origin_location = T::TransferAssetOrigin::ensure_origin(origin)?;

			// Check remote destination + bridge_config
			let (_, bridge_config, remote_destination) =
				Self::ensure_remote_destination(*destination)?;

			// Check assets (lets leave others checks on `AssetTransactor`)
			let assets: MultiAssets =
				(*assets).try_into().map_err(|()| Error::<T>::InvalidAssets)?;
			ensure!(
				assets.len() <= T::MaxAssetsLimit::get() as usize,
				Error::<T>::MaxAssetsLimitReached
			);

			// Do this in transaction (explicitly), the rollback should occur in case of any error and no assets will be trapped or lost
			Self::do_reserve_and_send_in_transaction(
				origin_location,
				remote_destination,
				assets,
				bridge_config,
			)
		}

		/// Transfer `ping` via bridge to different global consensus.
		///
		/// - can be used for testing purposes that bridge transfer is working and configured for `destination`
		///
		/// Parameters:
		///
		/// * `destination`: Different consensus location, e.g. Polkadot's Statemint: `2, X2(GlobalConsensus(NetworkId::Polkadot), Parachain(1000))`
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::ping_via_bridge())]
		pub fn ping_via_bridge(
			origin: OriginFor<T>,
			destination: Box<VersionedMultiLocation>,
		) -> DispatchResult {
			let origin_location = T::TransferPingOrigin::ensure_origin(origin)?;

			// Check remote destination + bridge_config
			let (network, bridge_config, remote_destination) =
				Self::ensure_remote_destination(*destination)?;

			// Check reserve account - sovereign account of bridge
			let allowed_target_location = bridge_config.allowed_target_location;

			// Prepare `ping` message
			let xcm: Xcm<()> =
				T::PingMessageBuilder::try_build(&origin_location, &network, &remote_destination)
					.ok_or(Error::<T>::UnsupportedPing)?;

			// Initiate bridge transfer
			Self::initiate_bridge_transfer(allowed_target_location, xcm).map_err(Into::into)
		}

		/// Adds new bridge configuration, which allows transfer to this `bridged_network`.
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
			bridge_config: Box<BridgeConfig>,
		) -> DispatchResult {
			let _ = T::AdminOrigin::ensure_origin(origin)?;
			ensure!(
				!AllowedExporters::<T>::contains_key(bridged_network),
				Error::<T>::ConfigurationAlreadyExists
			);
			let allowed_target_location_network = bridge_config
				.allowed_target_location
				.interior()
				.global_consensus()
				.map_err(|_| Error::<T>::InvalidConfiguration)?;
			ensure!(
				bridged_network == allowed_target_location_network,
				Error::<T>::InvalidConfiguration
			);
			// bridged consensus must be different
			let local_network = T::UniversalLocation::get()
				.global_consensus()
				.map_err(|_| Error::<T>::InvalidConfiguration)?;
			ensure!(bridged_network != local_network, Error::<T>::InvalidConfiguration);

			AllowedExporters::<T>::insert(bridged_network, bridge_config);
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
		/// * `fee`: New fee to update
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::update_exporter_config())]
		pub fn update_exporter_config(
			origin: OriginFor<T>,
			bridged_network: NetworkId,
			bridge_location_fee: Option<Box<VersionedMultiAsset>>,
			target_location_fee: Option<Box<VersionedMultiAsset>>,
		) -> DispatchResult {
			let _ = T::AdminOrigin::ensure_origin(origin)?;

			AllowedExporters::<T>::try_mutate_exists(bridged_network, |maybe_bridge_config| {
				let bridge_config =
					maybe_bridge_config.as_mut().ok_or(Error::<T>::UnavailableConfiguration)?;
				bridge_config.bridge_location_fee = bridge_location_fee
					.map(|fee| MultiAsset::try_from(*fee))
					.transpose()
					.map_err(|_| Error::<T>::UnsupportedXcmVersion)?;
				bridge_config.max_target_location_fee = target_location_fee
					.map(|fee| MultiAsset::try_from(*fee))
					.transpose()
					.map_err(|_| Error::<T>::UnsupportedXcmVersion)?;
				Self::deposit_event(Event::BridgeUpdated);
				Ok(())
			})
		}

		/// Add `(MultiLocation, Junction)` mapping to `AllowedUniversalAliases`
		///
		/// Parameters:
		///
		/// * `location`: key
		/// * `junction`: value
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::add_universal_alias())]
		pub fn add_universal_alias(
			origin: OriginFor<T>,
			location: Box<VersionedMultiLocation>,
			junction: Junction,
		) -> DispatchResult {
			let _ = T::AdminOrigin::ensure_origin(origin)?;

			let location: MultiLocation =
				(*location).try_into().map_err(|_| Error::<T>::UnsupportedXcmVersion)?;
			let added = AllowedUniversalAliases::<T>::try_mutate(location, |junctions| {
				junctions.try_insert(junction)
			})
			.map_err(|_| Error::<T>::InvalidConfiguration)?;
			if added {
				Self::deposit_event(Event::UniversalAliasAdded);
			}
			Ok(())
		}

		/// Remove `(MultiLocation, Junction)` mapping from `AllowedUniversalAliases`
		///
		/// Parameters:
		///
		/// * `location`: key
		/// * `junction`: value
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::remove_universal_alias())]
		pub fn remove_universal_alias(
			origin: OriginFor<T>,
			location: Box<VersionedMultiLocation>,
			junctions_to_remove: sp_std::prelude::Vec<Junction>,
		) -> DispatchResult {
			let _ = T::AdminOrigin::ensure_origin(origin)?;

			let location: MultiLocation =
				(*location).try_into().map_err(|_| Error::<T>::UnsupportedXcmVersion)?;
			let removed = AllowedUniversalAliases::<T>::try_mutate(
				location,
				|junctions| -> Result<bool, Error<T>> {
					let mut removed = false;
					for jtr in junctions_to_remove {
						removed |= junctions.remove(&jtr);
					}
					Ok(removed)
				},
			)?;
			if removed {
				Self::deposit_event(Event::UniversalAliasRemoved);
			}
			Ok(())
		}

		/// Add `MultiLocation` mapping to `AllowedReserveLocations`
		///
		/// Parameters:
		///
		/// * `location`: as reserve `MultiLocation`
		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::add_reserve_location())]
		pub fn add_reserve_location(
			origin: OriginFor<T>,
			location: Box<VersionedMultiLocation>,
		) -> DispatchResult {
			let _ = T::AdminOrigin::ensure_origin(origin)?;

			let location: MultiLocation =
				(*location).try_into().map_err(|_| Error::<T>::UnsupportedXcmVersion)?;
			let added = AllowedReserveLocations::<T>::try_mutate(|locations| {
				locations.try_insert(location)
			})
			.map_err(|_| Error::<T>::InvalidConfiguration)?;
			if added {
				Self::deposit_event(Event::ReserveLocationAdded);
			}
			Ok(())
		}

		/// Remove `MultiLocation` mapping from `AllowedReserveLocations`
		///
		/// Parameters:
		///
		/// * `location`: as reserve `MultiLocation`
		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::remove_reserve_location())]
		pub fn remove_reserve_location(
			origin: OriginFor<T>,
			locations_to_remove: sp_std::prelude::Vec<VersionedMultiLocation>,
		) -> DispatchResult {
			let _ = T::AdminOrigin::ensure_origin(origin)?;

			let removed =
				AllowedReserveLocations::<T>::try_mutate(|locations| -> Result<bool, Error<T>> {
					let mut removed = false;
					for ltr in locations_to_remove {
						let ltr: MultiLocation =
							ltr.try_into().map_err(|_| Error::<T>::UnsupportedXcmVersion)?;
						removed |= locations.remove(&ltr);
					}
					Ok(removed)
				})?;
			if removed {
				Self::deposit_event(Event::ReserveLocationRemoved);
			}
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Validates destination and check if we support bridging to this remote global consensus
		///
		/// Returns: correct remote location, where we should be able to bridge
		pub(crate) fn ensure_remote_destination(
			remote_destination: VersionedMultiLocation,
		) -> Result<(NetworkId, BridgeConfig, MultiLocation), Error<T>> {
			match remote_destination {
				VersionedMultiLocation::V3(remote_location) => {
					ensure!(
						remote_location.parent_count() == 2,
						Error::<T>::UnsupportedDestination
					);
					let local_network = T::UniversalLocation::get()
						.global_consensus()
						.map_err(|_| Error::<T>::InvalidConfiguration)?;
					let remote_network = remote_location
						.interior()
						.global_consensus()
						.map_err(|_| Error::<T>::UnsupportedDestination)?;
					ensure!(local_network != remote_network, Error::<T>::UnsupportedDestination);
					match AllowedExporters::<T>::get(remote_network) {
						Some(bridge_config) => {
							ensure!(
								// TODO:check-parameter - verify and prepare test for ETH scenario - https://github.com/paritytech/cumulus/pull/2013#discussion_r1094909290
								remote_location.starts_with(&bridge_config.allowed_target_location),
								Error::<T>::UnsupportedDestination
							);
							Ok((remote_network, bridge_config, remote_location))
						},
						None => return Err(Error::<T>::UnsupportedDestination),
					}
				},
				_ => Err(Error::<T>::UnsupportedXcmVersion),
			}
		}

		#[transactional]
		fn do_reserve_and_send_in_transaction(
			origin_location: MultiLocation,
			remote_destination: MultiLocation,
			assets: MultiAssets,
			bridge_config: BridgeConfig,
		) -> Result<(), DispatchError> {
			// Resolve reserve account as sovereign account of bridge
			let reserve_account = bridge_config.bridge_location;

			let allowed_target_location = bridge_config.allowed_target_location;

			// UniversalLocation as sovereign account location on target_location (as target_location sees UniversalLocation)
			let universal_location_as_sovereign_account_on_target_location =
				T::UniversalLocation::get()
					.invert_target(&allowed_target_location)
					.map_err(|_| Error::<T>::InvalidConfiguration)?;

			// lets try to do a reserve for all assets
			let mut reserved_assets = xcm_executor::Assets::new();
			for asset in assets.into_inner() {
				// TODO:check-parameter - verify this Joe's text
				// Deposit assets into `AccountId` that corresponds to the bridge
				// hub. In this way, Statemine acts as a reserve location to the
				// bridge, such that it need not trust any consensus system from
				// `./Parent/Parent/...`. (It may trust Polkadot, but would
				// Polkadot trust Kusama with its DOT?)

				// Move asset to reserve account
				T::AssetTransactor::transfer_asset(
					&asset,
					&origin_location,
					&reserve_account,
					// We aren't able to track the XCM that initiated the fee deposit, so we create a
					// fake message hash here
					&XcmContext::with_message_hash([0; 32]),
				)
					.and_then(|reserved_asset| {
						Self::deposit_event(Event::ReserveAssetsDeposited {
							from: origin_location,
							to: reserve_account,
							assets: reserved_asset.clone().into(),
						});
						reserved_assets.subsume_assets(reserved_asset);
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
					})?;
			}

			// Prepare `ReserveAssetDeposited` msg to bridge to the other side.
			// Reanchor stuff - we need to convert local asset id/MultiLocation to format that could be understood by different consensus and from their point-of-view
			reserved_assets.reanchor(&allowed_target_location, T::UniversalLocation::get(), None);
			let remote_destination = remote_destination
				.reanchored(&allowed_target_location, T::UniversalLocation::get())
				.map_err(|errored_dest| {
				log::error!(
					target: LOG_TARGET,
					"Failed to reanchor remote_destination: {:?} for allowed_target_location: {:?} and universal_location: {:?}",
					errored_dest,
					allowed_target_location,
					T::UniversalLocation::get()
				);
				Error::<T>::InvalidRemoteDestination
			})?;

			// prepare xcm message
			// 1. buy execution (if needed) -> (we expect UniversalLocation's sovereign account should pay)
			let (mut xcm_instructions, maybe_buy_execution) = match bridge_config
				.max_target_location_fee
			{
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
			// 2. add deposit reserved asset to destination account
			xcm_instructions.extend(sp_std::vec![
				ReserveAssetDeposited(reserved_assets.clone().into()),
				DepositAsset {
					assets: MultiAssetFilter::from(MultiAssets::from(reserved_assets)),
					beneficiary: remote_destination
				},
			]);
			// 3. add return unspent weight/asset back to the UniversalLocation's sovereign account on target
			if let Some(target_location_fee) = maybe_buy_execution {
				xcm_instructions.extend(sp_std::vec![
					RefundSurplus,
					DepositAsset {
						assets: MultiAssetFilter::from(MultiAssets::from(target_location_fee)),
						beneficiary: universal_location_as_sovereign_account_on_target_location
					},
				]);
			}

			Self::initiate_bridge_transfer(allowed_target_location, xcm_instructions.into())
				.map_err(Into::into)
		}

		fn initiate_bridge_transfer(dest: MultiLocation, xcm: Xcm<()>) -> Result<(), Error<T>> {
			log::info!(
				target: LOG_TARGET,
				"[T::BridgeXcmSender] send to bridge, dest: {:?}, xcm: {:?}",
				dest,
				xcm,
			);
			// call bridge
			// TODO: check-parameter - should we handle `sender_cost` somehow ?
			let (message_hash, sender_cost) =
				send_xcm::<T::BridgeXcmSender>(dest, xcm).map_err(|e| {
					log::error!(
						target: LOG_TARGET,
						"[T::BridgeXcmSender] SendError occurred, error: {:?}",
						e
					);
					Error::<T>::BridgeCallError
				})?;

			// just fire event
			Self::deposit_event(Event::TransferInitiated { message_hash, sender_cost });
			Ok(())
		}
	}
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use crate as bridge_transfer;
	use frame_support::traits::{ConstU32, Contains, ContainsPair, Currency, Everything};

	use crate::impls::{AllowedUniversalAliasesOf, IsAllowedReserveOf};
	use frame_support::{
		assert_noop, assert_ok, dispatch::DispatchError, parameter_types, sp_io, sp_tracing,
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
		AccountId32Aliases, CurrencyAdapter, EnsureXcmOrigin, ExporterFor, IsConcrete,
		SiblingParachainConvertsVia, SignedToAccountId32, UnpaidRemoteExporter,
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
		type HoldIdentifier = ();
		type FreezeIdentifier = ();
		type MaxHolds = ConstU32<0>;
		type MaxFreezes = ConstU32<0>;
	}

	parameter_types! {
		// UniversalLocation as statemine
		pub const RelayNetwork: NetworkId = NetworkId::ByGenesis([9; 32]);
		pub UniversalLocation: InteriorMultiLocation = X2(GlobalConsensus(RelayNetwork::get()), Parachain(1000));
		// Test bridge cfg
		pub TestBridgeTable: sp_std::prelude::Vec<(NetworkId, MultiLocation, Option<MultiAsset>)> = sp_std::vec![
			(NetworkId::Wococo, (Parent, Parachain(1013)).into(), None),
			(NetworkId::Polkadot, (Parent, Parachain(1002)).into(), None),
		];
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
				Some(MultiLocation { interior: X1(Parachain(2222)), parents: 1 })
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

	/// Bridge configuration we use in our tests.
	fn test_bridge_config() -> (NetworkId, BridgeConfig) {
		(
			Wococo,
			BridgeConfig {
				bridge_location: (Parent, Parachain(1013)).into(),
				bridge_location_fee: None,
				allowed_target_location: MultiLocation::new(
					2,
					X2(GlobalConsensus(Wococo), Parachain(1000)),
				),
				max_target_location_fee: None,
			},
		)
	}

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

		fn prepare_ping_transfer() {
			unimplemented!("Not implemented here - not needed");
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
		type UniversalAliasesLimit = ConstU32<2>;
		type ReserveLocationsLimit = ConstU32<2>;
		type AssetTransactor = CurrencyTransactor;
		type BridgeXcmSender = TestBridgeXcmSender;
		type TransferAssetOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
		type MaxAssetsLimit = MaxAssetsLimit;
		type TransferPingOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
		type PingMessageBuilder = UnpaidTrapMessageBuilder<TrapCode>;
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
	fn test_ensure_remote_destination() {
		new_test_ext().execute_with(|| {
			// insert bridge config
			let bridge_network = Wococo;
			let bridge_config = test_bridge_config().1;
			assert_ok!(BridgeTransfer::add_exporter_config(
				RuntimeOrigin::root(),
				bridge_network,
				Box::new(bridge_config.clone()),
			));

			// v2 not supported
			assert_eq!(
				BridgeTransfer::ensure_remote_destination(VersionedMultiLocation::V2(
					xcm::v2::MultiLocation::default()
				)),
				Err(Error::<TestRuntime>::UnsupportedXcmVersion)
			);

			// v3 - "parent: 0" wrong
			assert_eq!(
				BridgeTransfer::ensure_remote_destination(VersionedMultiLocation::V3(
					MultiLocation::new(0, X2(GlobalConsensus(Wococo), Parachain(1000)))
				)),
				Err(Error::<TestRuntime>::UnsupportedDestination)
			);
			// v3 - "parent: 1" wrong
			assert_eq!(
				BridgeTransfer::ensure_remote_destination(VersionedMultiLocation::V3(
					MultiLocation::new(1, X2(GlobalConsensus(Wococo), Parachain(1000)))
				)),
				Err(Error::<TestRuntime>::UnsupportedDestination)
			);

			// v3 - Rococo is not supported
			assert_eq!(
				BridgeTransfer::ensure_remote_destination(VersionedMultiLocation::V3(
					MultiLocation::new(2, X2(GlobalConsensus(Rococo), Parachain(1000)))
				)),
				Err(Error::<TestRuntime>::UnsupportedDestination)
			);

			// v3 - remote_destination is not allowed
			assert_eq!(
				BridgeTransfer::ensure_remote_destination(VersionedMultiLocation::V3(
					MultiLocation::new(2, X2(GlobalConsensus(Wococo), Parachain(1234)))
				)),
				Err(Error::<TestRuntime>::UnsupportedDestination)
			);

			// v3 - ok (allowed)
			assert_eq!(
				BridgeTransfer::ensure_remote_destination(VersionedMultiLocation::V3(
					MultiLocation::new(2, X2(GlobalConsensus(Wococo), Parachain(1000)))
				)),
				Ok((
					bridge_network,
					bridge_config,
					MultiLocation::new(2, X2(GlobalConsensus(Wococo), Parachain(1000)))
				))
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
			let bridged_network = Wococo;
			assert_ok!(BridgeTransfer::add_exporter_config(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(test_bridge_config().1),
			));
			let bridge_location = AllowedExporters::<TestRuntime>::get(bridged_network)
				.expect("stored BridgeConfig for bridged_network")
				.bridge_location;

			// checks before
			assert!(ROUTED_MESSAGE.with(|r| r.borrow().is_none()));
			assert_eq!(Balances::free_balance(&user_account), user_account_init_balance);
			let bridge_location_as_sovereign_account =
				LocationToAccountId::convert_ref(bridge_location)
					.expect("converted bridge location as accountId");
			assert_eq!(Balances::free_balance(&bridge_location_as_sovereign_account), 0);

			// trigger transfer_asset_via_bridge - should trigger new ROUTED_MESSAGE
			let asset = MultiAsset {
				fun: Fungible(balance_to_transfer.into()),
				id: Concrete(RelayLocation::get()),
			};
			let assets = Box::new(VersionedMultiAssets::from(MultiAssets::from(asset)));

			// destination is account from different consensus
			let destination = Box::new(VersionedMultiLocation::from(MultiLocation::new(
				2,
				X3(GlobalConsensus(Wococo), Parachain(1000), consensus_account(Wococo, 2)),
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
			assert_eq!(Balances::free_balance(&bridge_location_as_sovereign_account), 15);

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
					ExportMessage { network: Wococo, destination: X1(Parachain(1000)), .. }
				)
			}) {
				assert!(xcm.0.iter().any(|instr| matches!(instr, UnpaidExecution { .. })));
				assert!(xcm.0.iter().any(|instr| matches!(instr, ReserveAssetDeposited(..))));
				assert!(xcm.0.iter().any(|instr| matches!(instr, ClearOrigin)));
				assert!(xcm.0.iter().any(|instr| matches!(instr, DepositAsset { .. })));
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
			let bridged_network = Wococo;
			assert_ok!(BridgeTransfer::add_exporter_config(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(BridgeConfig {
					bridge_location: MultiLocation::new(1, Parachain(2222)).into(),
					bridge_location_fee: None,
					allowed_target_location: MultiLocation::new(
						2,
						X2(GlobalConsensus(Wococo), Parachain(1000)),
					),
					max_target_location_fee: None,
				}),
			));

			let bridge_location = AllowedExporters::<TestRuntime>::get(bridged_network)
				.expect("stored BridgeConfig for bridged_network")
				.bridge_location;

			// checks before
			assert!(ROUTED_MESSAGE.with(|r| r.borrow().is_none()));
			let user_balance_before = Balances::free_balance(&user_account);
			assert_eq!(user_balance_before, user_account_init_balance);
			let bridge_location_as_sovereign_account =
				LocationToAccountId::convert_ref(bridge_location)
					.expect("converted bridge location as accountId");
			let reserve_account_before =
				Balances::free_balance(&bridge_location_as_sovereign_account);
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
				X3(GlobalConsensus(Wococo), Parachain(1000), consensus_account(Wococo, 2)),
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
					error: [8, 0, 0, 0],
					message: Some("BridgeCallError")
				})
			);

			// checks after
			// balances are untouched
			assert_eq!(Balances::free_balance(&user_account), user_balance_before);
			assert_eq!(
				Balances::free_balance(&bridge_location_as_sovereign_account),
				reserve_account_before
			);
			// no xcm messages fired
			assert!(ROUTED_MESSAGE.with(|r| r.borrow().is_none()));
			// check events (no events because of rollback)
			assert!(System::events().is_empty());
		});
	}

	#[test]
	fn test_ping_via_bridge_works() {
		new_test_ext().execute_with(|| {
			// insert bridge config
			let bridged_network = Wococo;
			assert_ok!(BridgeTransfer::add_exporter_config(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(test_bridge_config().1),
			));

			// checks before
			assert!(ROUTED_MESSAGE.with(|r| r.borrow().is_none()));

			// trigger ping_via_bridge - should trigger new ROUTED_MESSAGE
			// destination is account from different consensus
			let destination = Box::new(VersionedMultiLocation::V3(MultiLocation::new(
				2,
				X3(GlobalConsensus(Wococo), Parachain(1000), consensus_account(Wococo, 2)),
			)));

			// trigger asset transfer
			assert_ok!(BridgeTransfer::ping_via_bridge(
				RuntimeOrigin::signed(account(1)),
				destination,
			));

			// check events
			let events = System::events();
			assert!(!events.is_empty());

			// check TransferInitiated
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
					ExportMessage { network: Wococo, destination: X1(Parachain(1000)), .. }
				)
			}) {
				assert!(xcm.0.iter().any(|instr| instr.eq(&Trap(TrapCode::get()))));
			} else {
				assert!(false, "Does not contains [`ExportMessage`], fired_xcm: {:?}", fired_xcm);
			}
		});
	}

	#[test]
	fn allowed_exporters_management_works() {
		let bridged_network = Rococo;
		let bridged_config = Box::new(BridgeConfig {
			bridge_location: (Parent, Parachain(1013)).into(),
			bridge_location_fee: None,
			allowed_target_location: MultiLocation::new(
				2,
				X2(GlobalConsensus(bridged_network), Parachain(1000)),
			),
			max_target_location_fee: None,
		});
		let dummy_xcm = Xcm(vec![]);
		let dummy_remote_interior_multilocation = X1(Parachain(1234));

		{
			let mut asset = xcm_executor::Assets::from(MultiAsset {
				id: Concrete(MultiLocation::parent()),
				fun: Fungible(1_000),
			});
			println!("before: {:?}", asset);
			asset.reanchor(&bridged_config.allowed_target_location, UniversalLocation::get(), None);
			println!("after: {:?}", asset);
		}
		{
			let mut asset = xcm_executor::Assets::from(MultiAsset {
				id: Concrete(MultiLocation { parents: 1, interior: X1(Parachain(3000)) }),
				fun: Fungible(1_000),
			});
			println!("before: {:?}", asset);
			asset.reanchor(&bridged_config.allowed_target_location, UniversalLocation::get(), None);
			println!("after: {:?}", asset);
		}

		new_test_ext().execute_with(|| {
			assert_eq!(AllowedExporters::<TestRuntime>::iter().count(), 0);

			// should fail - just root is allowed
			assert_noop!(
				BridgeTransfer::add_exporter_config(
					RuntimeOrigin::signed(account(1)),
					bridged_network,
					bridged_config.clone(),
				),
				DispatchError::BadOrigin
			);

			// should fail - bridged_network should match allowed_target_location
			assert_noop!(
				BridgeTransfer::add_exporter_config(RuntimeOrigin::root(), bridged_network, {
					let remote_network = Westend;
					assert_ne!(bridged_network, remote_network);
					Box::new(test_bridge_config().1)
				}),
				DispatchError::Module(ModuleError {
					index: 52,
					error: [0, 0, 0, 0],
					message: Some("InvalidConfiguration")
				})
			);
			// should fail - bridged_network must be different global consensus than our `UniversalLocation`
			assert_noop!(
				BridgeTransfer::add_exporter_config(
					RuntimeOrigin::root(),
					UniversalLocation::get().global_consensus().expect("any `NetworkId`"),
					bridged_config.clone()
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
				bridged_config.clone(),
			));
			assert_eq!(AllowedExporters::<TestRuntime>::iter().count(), 1);
			assert_eq!(
				AllowedExporters::<TestRuntime>::get(bridged_network),
				Some(*bridged_config.clone())
			);
			assert_eq!(AllowedExporters::<TestRuntime>::get(Wococo), None);
			assert_eq!(
				BridgeTransfer::exporter_for(
					&bridged_network,
					&dummy_remote_interior_multilocation,
					&dummy_xcm
				),
				Some((bridged_config.bridge_location, bridged_config.bridge_location_fee))
			);
			assert_eq!(
				BridgeTransfer::exporter_for(
					&Wococo,
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
				Some(VersionedMultiAsset::V3((Parent, 300u128).into()).into()),
			));
			assert_eq!(AllowedExporters::<TestRuntime>::iter().count(), 1);
			assert_eq!(
				AllowedExporters::<TestRuntime>::get(bridged_network),
				Some(BridgeConfig {
					bridge_location: bridged_config.bridge_location.clone(),
					bridge_location_fee: Some((Parent, 200u128).into()),
					allowed_target_location: bridged_config.allowed_target_location.clone(),
					max_target_location_fee: Some((Parent, 300u128).into()),
				})
			);
			assert_eq!(
				BridgeTransfer::exporter_for(
					&bridged_network,
					&dummy_remote_interior_multilocation,
					&dummy_xcm
				),
				Some((bridged_config.bridge_location, Some((Parent, 200u128).into())))
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

			// remove ok
			assert_ok!(BridgeTransfer::remove_universal_alias(
				RuntimeOrigin::root(),
				Box::new(VersionedMultiLocation::V3(location1.clone())),
				vec![junction1.clone()],
			));
			assert!(!AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction1)));
			assert!(AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction2)));

			assert_ok!(BridgeTransfer::remove_universal_alias(
				RuntimeOrigin::root(),
				Box::new(VersionedMultiLocation::V3(location1.clone())),
				vec![junction2.clone()],
			));
			assert!(!AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction1)));
			assert!(!AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction2)));
		})
	}

	#[test]
	fn allowed_reserve_locations_management_works() {
		new_test_ext().execute_with(|| {
			assert!(AllowedReserveLocations::<TestRuntime>::get().is_empty());

			let location1 = MultiLocation::new(1, X1(Parachain(1014)));
			let location2 =
				MultiLocation::new(2, X2(GlobalConsensus(ByGenesis([1; 32])), Parachain(1014)));
			let asset: MultiAsset = (Parent, 200u128).into();

			// should fail - just root is allowed
			assert_noop!(
				BridgeTransfer::add_reserve_location(
					RuntimeOrigin::signed(account(1)),
					Box::new(VersionedMultiLocation::V3(location1.clone()))
				),
				DispatchError::BadOrigin
			);
			assert_eq!(AllowedReserveLocations::<TestRuntime>::get().len(), 0);
			assert!(!IsAllowedReserveOf::<TestRuntime, Everything>::contains(&asset, &location1));
			assert!(!IsAllowedReserveOf::<TestRuntime, Everything>::contains(&asset, &location2));

			// add ok
			assert_ok!(BridgeTransfer::add_reserve_location(
				RuntimeOrigin::root(),
				Box::new(VersionedMultiLocation::V3(location1.clone()))
			));
			assert_ok!(BridgeTransfer::add_reserve_location(
				RuntimeOrigin::root(),
				Box::new(VersionedMultiLocation::V3(location2.clone()))
			));
			assert_eq!(AllowedReserveLocations::<TestRuntime>::get().len(), 2);
			assert!(IsAllowedReserveOf::<TestRuntime, Everything>::contains(&asset, &location1));
			assert!(IsAllowedReserveOf::<TestRuntime, Everything>::contains(&asset, &location2));

			// remove ok
			assert_ok!(BridgeTransfer::remove_reserve_location(
				RuntimeOrigin::root(),
				vec![VersionedMultiLocation::V3(location1.clone())],
			));
			assert_eq!(AllowedReserveLocations::<TestRuntime>::get().len(), 1);
			assert!(!IsAllowedReserveOf::<TestRuntime, Everything>::contains(&asset, &location1));
			assert!(IsAllowedReserveOf::<TestRuntime, Everything>::contains(&asset, &location2));

			assert_ok!(BridgeTransfer::remove_reserve_location(
				RuntimeOrigin::root(),
				vec![VersionedMultiLocation::V3(location2.clone())],
			));
			assert!(AllowedReserveLocations::<TestRuntime>::get().is_empty());
			assert!(!IsAllowedReserveOf::<TestRuntime, Everything>::contains(&asset, &location1));
			assert!(!IsAllowedReserveOf::<TestRuntime, Everything>::contains(&asset, &location2));
		})
	}
}
