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

//! Example Pallet that shows how to send upward messages and how to receive
//! downward messages.

use frame_support::{
	decl_event, decl_module, decl_storage,
	traits::{Currency, ExistenceRequirement, WithdrawReason},
};
use frame_system::ensure_signed;

use codec::{Decode, Encode};
use cumulus_primitives::{
	relay_chain::DownwardMessage, DownwardMessageHandler, ParaId, UpwardMessageOrigin,
	UpwardMessageSender,
};
use cumulus_upward_message::BalancesMessage;

type BalanceOf<T> =
	<<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

/// Configuration trait of this pallet.
pub trait Trait: frame_system::Trait {
	/// Event type used by the runtime.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// The sender of upward messages.
	type UpwardMessageSender: UpwardMessageSender<Self::UpwardMessage>;

	/// The upward message type used by the Parachain runtime.
	type UpwardMessage: codec::Codec + BalancesMessage<Self::AccountId, BalanceOf<Self>>;

	/// Currency of the runtime.
	type Currency: Currency<Self::AccountId>;
}

// This pallet's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as ParachainUpgrade {}
	add_extra_genesis {
		config(parachain_id): ParaId;
		build(|config: &Self| {
			// This is basically a hack to make the parachain id easily configurable.
			// Could also be done differently, but yeah..
			crate::ParachainId::set(&config.parachain_id);
		});
	}
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
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin, system = frame_system {
		/// Transfer `amount` of tokens on the relay chain from the Parachain account to
		/// the given `dest` account.
		#[weight = 10]
		fn transfer_tokens(origin, dest: T::AccountId, amount: BalanceOf<T>) {
			//TODO: Remove the tokens from the given account
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

		fn deposit_event() = default;
	}
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
