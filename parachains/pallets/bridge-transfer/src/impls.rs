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
	types::{AssetTransferKind, ResolveAssetTransferKind, UsingVersioned},
	Config, Error, Event, Pallet, LOG_TARGET,
};
use frame_support::{pallet_prelude::Get, transactional};
use frame_system::unique;
use pallet_bridge_transfer_primitives::{
	EnsureReachableDestination, ReachableDestination, ReachableDestinationError,
};
use sp_runtime::DispatchError;
use xcm::prelude::*;
use xcm_builder::ensure_is_remote;
use xcm_executor::traits::TransactAsset;

impl<T: Config> From<ReachableDestinationError> for Error<T> {
	fn from(value: ReachableDestinationError) -> Self {
		match value {
			ReachableDestinationError::UnsupportedDestination => Error::<T>::UnsupportedDestination,
			ReachableDestinationError::UnsupportedXcmVersion => Error::<T>::UnsupportedXcmVersion,
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Validates destination and check if we support bridging to this remote global consensus
	///
	/// Returns: correct remote location, where we should be able to bridge
	pub(crate) fn ensure_reachable_remote_destination(
		remote_destination: VersionedMultiLocation,
	) -> Result<ReachableDestination, Error<T>> {
		let remote_destination: MultiLocation = remote_destination
			.to_versioned()
			.map_err(|_| Error::<T>::UnsupportedXcmVersion)?;

		let devolved = ensure_is_remote(T::UniversalLocation::get(), remote_destination)
			.map_err(|_| Error::<T>::UnsupportedDestination)?;
		let (remote_network, _) = devolved;

		T::BridgedDestinationValidator::ensure_destination(remote_network, remote_destination)
			.map_err(Into::into)
	}

	/// Tries to initiate transfer assets over bridge.
	#[transactional]
	pub(crate) fn initiate_transfer_asset_via_bridge_in_transaction(
		origin_location: MultiLocation,
		destination: ReachableDestination,
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

	/// Tries to send xcm message over bridge
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
	fn resolve_reserve_account(destination: &ReachableDestination) -> MultiLocation {
		destination.target.location
	}
}
