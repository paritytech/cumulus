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
//! Module supports transfer assets over bridges between different consensus chain.
//! With fine-grained configuration you can control transferred assets (out/in) between different consensus chain.
//! "Transfer asset over bridge" recognize two kinds of transfer see [types::AssetTransferKind].

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub use pallet_bridge_transfer_primitives::MaybePaidLocation;

pub mod features;
mod impls;
mod types;
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

	use crate::types::ResolveAssetTransferKind;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use pallet_bridge_transfer_primitives::EnsureReachableDestination;
	use sp_std::boxed::Box;
	use xcm::prelude::*;
	use xcm_executor::traits::TransactAsset;

	#[cfg(feature = "runtime-benchmarks")]
	use pallet_bridge_transfer_primitives::MaybePaidLocation;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Everything we need to run benchmarks.
	#[cfg(feature = "runtime-benchmarks")]
	pub trait BenchmarkHelper<RuntimeOrigin> {
		/// Returns proper bridge location+fee  for NetworkId, supported by the runtime.
		///
		/// We expect that the XCM environment (`BridgeXcmSender`) has everything enabled
		/// to support transfer to this destination **after** `prepare_asset_transfer` call.
		fn bridge_location() -> Option<(NetworkId, MaybePaidLocation)> {
			None
		}

		/// Returns proper target_location location+fee supported by the runtime.
		///
		/// We expect that the XCM environment (`BridgeXcmSender`) has everything enabled
		/// to support transfer to this destination **after** `prepare_asset_transfer` call.
		fn allowed_bridged_target_location() -> Option<MaybePaidLocation> {
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

	#[cfg(feature = "runtime-benchmarks")]
	impl<RuntimeOrigin> BenchmarkHelper<RuntimeOrigin> for () {
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Runtime's universal location
		type UniversalLocation: Get<InteriorMultiLocation>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// How to withdraw and deposit an asset for reserve.
		/// (Config for transfer out)
		type AssetTransactor: TransactAsset;
		/// Transfer kind resolver for `asset` to `target_location`.
		type AssetTransferKindResolver: ResolveAssetTransferKind;
		/// Required origin for asset transfer. If successful, it resolves to `MultiLocation`.
		/// (Config for transfer out)
		type AssetTransferOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = MultiLocation>;
		/// Max count of assets in one call
		/// (Config for transfer out)
		type AssetsLimit: Get<u8>;

		/// Validates remote_destination if it is reachable from the point of configuration
		type BridgedDestinationValidator: EnsureReachableDestination;
		/// XCM sender which sends messages to the BridgeHub
		/// (Config for transfer out)
		type BridgeXcmSender: SendXcm;

		/// Benchmarks helper.
		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper: BenchmarkHelper<Self::RuntimeOrigin>;
	}

	#[pallet::error]
	#[cfg_attr(test, derive(PartialEq))]
	pub enum Error<T> {
		InvalidAssets,
		AssetsLimitReached,
		UnsupportedDestination,
		UnsupportedXcmVersion,
		InvalidTargetLocation,
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
			let destination = Self::ensure_reachable_remote_destination(
				(*destination).try_into().map_err(|()| Error::<T>::InvalidRemoteDestination)?,
			)?;

			// Check assets (lets leave others checks on `AssetTransactor`)
			let assets: MultiAssets =
				(*assets).try_into().map_err(|()| Error::<T>::InvalidAssets)?;
			ensure!(assets.len() <= T::AssetsLimit::get() as usize, Error::<T>::AssetsLimitReached);

			// Do this in transaction (explicitly), the rollback should occur in case of any error and no assets will be trapped or lost
			Self::initiate_transfer_asset_via_bridge_in_transaction(
				origin_location,
				destination,
				assets,
			)
		}
	}
}
