// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Cumulus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_event, decl_module, dispatch::{Dispatchable, DispatchResult}, traits::{Currency, ExistenceRequirement, WithdrawReason, OriginTrait}, Parameter};
use frame_system::{RawOrigin, ensure_signed};

use codec::{Codec, Encode, Decode, Input, Output};
use cumulus_primitives::{
	xcm::{v0::Xcm, VersionedXcm},
	DmpHandler, HmpHandler, HmpSender, UmpSender, ParaId
};
use cumulus_upward_message::BalancesMessage;
use polkadot_parachain::primitives::AccountIdConversion;
use frame_support::sp_std::result;
use polkadot_parachain::xcm::{
	VersionedMultiAsset, VersionedMultiLocation,
	v0::{MultiOrigin, MultiAsset, MultiLocation, Junction, Ai}
};

/// Origin for the parachains module.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub enum Origin {
	/// It comes from the relay-chain.
	RelayChain,
	/// It comes from a parachain.
	Parachain(ParaId),
}

/// Configuration trait of this pallet.
pub trait Trait: frame_system::Trait {
	/// Event type used by the runtime.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// The outer origin type.
	type Origin: From<Origin>
		+ From<<Self as system::Trait>::Origin>
		+ Into<result::Result<Origin, <Self as Trait>::Origin>>;

	/// The outer call dispatch type.
	type Call: Parameter + Dispatchable<Origin=<Self as Trait>::Origin> + From<Call<Self>>;

	/// The sender of upward messages.
	type UmpSender: UmpSender;

	/// The sender of horizontal/lateral messages.
	type HmpSender: HmpSender;

	// TODO: Configuration for how pallets map to MultiAssets.
}

decl_event! {
	pub enum Event<T> where
		AccountId = <T as frame_system::Trait>::AccountId,
		Balance = BalanceOf<T>
	{
		/// Transferred tokens to the account on the relay chain.
		TransferredToRelayChain(VersionedMultiLocation, VersionedMultiAsset),
		/// Soem assets have been received.
		ReceivedAssets(AccountId, VersionedMultiAsset),
		/// Transferred tokens to the account from the given parachain account.
		TransferredToParachainViaReserve(ParaId, VersionedMultiLocation, VersionedMultiAsset),
	}
}

decl_error! {
	pub enum Error<T> {
		/// A version of a data format is unsupported.
		UnsupportedVersion,
		/// Asset given was invalid or unsupported.
		BadAsset,
		/// Location given was invalid or unsupported.
		BadLocation,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		/// Transfer some `asset` from the Parachain to the given `dest`.
		#[weight = 10]
		fn transfer(origin, dest: VersionedMultiLocation, versioned_asset: VersionedMultiAsset) {
			let who = ensure_signed(origin)?;

			let asset = MultiAsset::try_from(versioned_asset).map_err(|_| Error::<T>::UnsupportedVersion)?;
			let dest = MultiLocation::try_from(dest).map_err(|_| Error::<T>::UnsupportedVersion)?;

			let (amount, asset) = match asset {
				// The only asset whose reserve we recognise for now is native tokens from the
				// relay chain, identified as the singular asset of the Relay-chain. From our
				// context (i.e. a parachain) , this is `Parent`. From the Relay-chain's context,
				// it is `Null`.
				MultiAsset::ConcreteFungible { id: MultiLocation::X1(Junction::Parent), amount } =>
					(MultiAsset::ConcreteFungible { id: MultiLocation::Null, amount }, amount),
				_ => Err(Error::<T>::BadAsset)?,	// Asset not recognised.
			};

			match dest {
				MultiLocation::X3(Junction::Parent, Junction::Parachain(dest_id), dest_loc) => {
					// Reserve transfer using the Relay-chain.
					let _ = T::Currency::withdraw(
						&who,
						amount,
						WithdrawReason::Transfer.into(),
						ExistenceRequirement::AllowDeath,
					)?;

					let dest_loc = MultiLocation::from(dest_loc);
					let msg = Xcm::ReserveAssetTransfer(
						asset,
						Junction::Parachain(dest_id).into(),
						Ai::DepositAsset(MultiAsset::Wild, dest_loc.clone())
					);
					// TODO: Check that this will work prior to withdraw.
					let _ = T::UmpSender::send_upward(msg);

					Self::deposit_event(Event::<T>::TransferredToParachainViaReserve(
						dest_id,
						dest_loc.into(),
						versioned_asset,
					));
				}
				MultiLocation::X2(Junction::Parent, dest_loc) => {
					// Direct withdraw/deposit on the Relay-chain
					let _ = T::Currency::withdraw(
						&who,
						amount,
						WithdrawReason::Transfer.into(),
						ExistenceRequirement::AllowDeath,
					)?;

					let dest_loc = MultiLocation::from(dest_loc);
					let msg = Xcm::WithdrawAsset(
						asset,
						Ai::DepositAsset(MultiAsset::Wild, dest_loc.clone())
					);
					let _ = T::UmpSender::send_upward(msg);

					Self::deposit_event(Event::<T>::TransferredToRelayChain(
						dest_loc.into(),
						versioned_asset,
					));
				}
				_ => Err(Error::<T>::BadLocation)?,	// Invalid location.
			}
		}
	}
}

impl<T: Trait> Module<T> {
	fn handle_message(origin: Origin, msg: VersionedXcm) {
		match (origin, msg.into()) {
			(Origin::RelayChain, Ok(Xcm::ReserveAssetCredit { asset, effect })) => {
				let amount = match asset {
					// The only asset whose reserve we recognise for now is native tokens from the
					// relay chain, identified as the singular asset of the Relay-chain, `Parent`.
					MultiAsset::ConcreteFungible { id: MultiLocation::Parent, ref amount } => *amount,
					_ => return,	// Asset not recognised.
				};
				match effect {
					// For now we only support wildcard asset here.
					Ai::DepositAsset { asset: MultiAsset::Wild, dest_: MultiLocation::AccountId32 { id, .. } } => {
						// deposit the holding account's contents into account `id`. holding
						// account is just amount of DOT. We assume that `Currency` maps to this
						// parachain's reserve-backed local derivative of the relay-chain's
						// currency.
						let _ = T::Currency::deposit_creating(id.into(), amount.into());
						Self::deposit_event(Event::<T>::ReceivedAssets(dest, asset.into()));
					},
					_ => return,	// Assets are lost, since we don't support any other `Ai`s right now.
				}
			},
			(origin, Ok(Ok(Xcm::Transact{ origin_type, call }))) => {
				// We assume that the Relay-chain is allowed to use transact on this parachain.
				// TODO: allow this to be configurable in the trait.
				// TODO: allow the trait to issue filters for the relay-chain
				if let Ok(message_call) = <T as Trait>::Call::decode(&mut &call[..]) {
					let origin: <T as Trait>::Origin = match origin_type {
						MultiOrigin::SovereignAccount => {
							match origin {
								// Unimplemented. Relay-chain doesn't yet have a sovereign account
								// on the parachain.
								Origin::RelayChain => return,
								Origin::Parachain(id) => RawOrigin::Signed(id.into_account()).into(),
							}
						}
						MultiOrigin::Native => origin.into(),
						MultiOrigin::Superuser => match origin {
							Origin::RelayChain =>
								// We assume that the relay-chain is allowed to execute with superuser
								// privileges if it wants.
								// TODO: allow this to be configurable in the trait.
								RawOrigin::Root.into(),
							Origin::Parachain(_) =>
								return,
						}
					};
					let _ok = message_call.dispatch(origin).is_ok();
					// Not much to do with the result as it is. It's up to the parachain to ensure that the
					// message makes sense.
				}
			}
		}
	}
}

impl<T: Trait> DmpHandler for Module<T> {
	fn handle_downward(msg: VersionedXcm) {
		Self::handle_message(Origin::RelayChain, msg);
	}
}

impl<T: Trait> HmpHandler for Module<T> {
	fn handle_lateral(id: ParaId, msg: VersionedXcm) {
		Self::handle_message(Origin::Parachain(id), msg);
	}
}
