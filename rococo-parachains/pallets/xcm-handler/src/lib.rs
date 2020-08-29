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
	relay_chain::DownwardMessage,
	xcmp::{XCMPMessageHandler, XCMPMessageSender},
	xcm::{v0::Xcm, VersionedXcm},
	DmpHandler, HmpHandler, HmpSender, UmpSender, ParaId
};
use cumulus_upward_message::BalancesMessage;
use polkadot_parachain::primitives::AccountIdConversion;
use frame_support::sp_std::result;
use polkadot_parachain::xcm::v0::MultiOrigin;

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

	// TODO: Configuration for how pallet instances map to Xcm concepts.
}

decl_event! {
	pub enum Event<T> where
		AccountId = <T as frame_system::Trait>::AccountId,
		Balance = BalanceOf<T>
	{
		/// Transferred tokens to the account on the relay chain.
		TransferredTokensToRelayChain(AccountId, Balance),
		/// Transferred tokens to the account on request from the relay chain.
		TransferredTokensFromRelayChain(AccountId, Balance),
		/// Transferred tokens to the account from the given parachain account.
		TransferredTokensViaXcmp(ParaId, AccountId, Balance, DispatchResult),
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;
	}
}

impl<T: Trait> DmpHandler for Module<T> {
	fn handle_downward(msg: VersionedXcm) {
		match msg.into() {
			Ok(Xcm::ReserveAssetCredit { asset, effect }) => {

			},
			Ok(Ok(Xcm::Transact{ origin_type, call })) => {
				if let Ok(message_call) = <T as Trait>::Call::decode(&mut &call[..]) {
					let origin: <T as Trait>::Origin = match origin_type {
						MultiOrigin::SovereignAccount => {
							// Unimplemented. Does the relay-chain have a sovereign account on the
							// parachain?
							Origin::RelayChain.into(),
						}
						MultiOrigin::Native =>
							Origin::RelayChain.into(),
						MultiOrigin::Superuser =>
							<T as Trait>::Origin::from(<T as system::Trait>::Origin::from(system::RawOrigin::Root)),
					};
					let _ok = message_call.dispatch(origin).is_ok();
					// Not much to do with the result as it is. It's up to the parachain to ensure that the
					// message makes sense.
				}
			}
		}
	}
}

impl<T: Trait> HmpHandler for Module<T> {
	fn handle_lateral(id: ParaId, msg: VersionedXcm) {

	}
}

/// Transfer `amount` of tokens on the relay chain from the Parachain account to
/// the given `dest` account.
#[weight = 10]
fn transfer_tokens_to_relay_chain(origin, dest: T::AccountId, amount: BalanceOf<T>) {
	let who = ensure_signed(origin)?;

	let _ = T::Currency::withdraw(
		&who,
		amount,
		WithdrawReason::Transfer.into(),
		ExistenceRequirement::AllowDeath,
	)?;

	let msg = <T as Trait>::UpwardMessage::transfer(dest.clone(), amount.clone());
	<T as Trait>::UpwardMessageSender::send_upward_message(&msg, UpwardMessageOrigin::Signed)
		.expect("Should not fail; qed");

	Self::deposit_event(Event::<T>::TransferredTokensToRelayChain(dest, amount));
}

/// Transfer `amount` of tokens to another parachain.
#[weight = 10]
fn transfer_tokens_to_parachain_chain(
	origin,
	para_id: u32,
	dest: T::AccountId,
	amount: BalanceOf<T>,
) {
	//TODO we don't make sure that the parachain has some tokens on the other parachain.
	let who = ensure_signed(origin)?;

	let _ = T::Currency::withdraw(
		&who,
		amount,
		WithdrawReason::Transfer.into(),
		ExistenceRequirement::AllowDeath,
	)?;

	T::XCMPMessageSender::send_xcmp_message(
		para_id.into(),
		&XCMPMessage::TransferToken(dest, amount),
	).expect("Should not fail; qed");
}

/// This is a hack to convert from one generic type to another where we are sure that both are the
/// same type/use the same encoding.
fn convert_hack<O: Decode>(input: &impl Encode) -> O {
	input.using_encoded(|e| Decode::decode(&mut &e[..]).expect("Must be compatible; qed"))
}

impl<T: Trait> DownwardMessageHandler for Module<T> {
	fn handle_downward_message(msg: &DownwardMessage) {
		match msg {
			DownwardMessage::TransferInto(dest, amount, _) => {
				let dest = convert_hack(&dest);
				let amount: BalanceOf<T> = convert_hack(amount);

				let _ = T::Currency::deposit_creating(&dest, amount.clone());

				Self::deposit_event(Event::<T>::TransferredTokensFromRelayChain(dest, amount));
			}
			_ => {}
		}
	}
}

impl<T: Trait> XCMPMessageHandler<XCMPMessage<T::AccountId, BalanceOf<T>>> for Module<T> {
	fn handle_xcmp_message(src: ParaId, msg: &XCMPMessage<T::AccountId, BalanceOf<T>>) {
		match msg {
			XCMPMessage::TransferToken(dest, amount) => {
				let para_account = src.clone().into_account();

				let res = T::Currency::transfer(
					&para_account,
					dest,
					amount.clone(),
					ExistenceRequirement::AllowDeath,
				);

				Self::deposit_event(Event::<T>::TransferredTokensViaXCMP(
					src,
					dest.clone(),
					amount.clone(),
					res,
				));
			}
		}
	}
}
