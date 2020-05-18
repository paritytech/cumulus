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

use frame_support::{decl_event, decl_module};
use frame_system::ensure_signed;

use cumulus_primitives::{
	relay_chain::{AccountId as RAccountId, Balance as RBalance},
	UpwardMessageOrigin, UpwardMessageSender,
};
use cumulus_upward_message::BalancesMessage;

/// Configuration trait of this pallet.
pub trait Trait: frame_system::Trait {
	/// Event type used by the runtime.
	type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;

	/// The sender of upward messages.
	type UpwardMessageSender: UpwardMessageSender<Self::UpwardMessage>;

	/// The upward message type used by the Parachain runtime.
	type UpwardMessage: codec::Codec + BalancesMessage;
}

decl_event! {
	pub enum Event {
		/// Transferred tokens to the account on the relay chain.
		TransferredTokens(RAccountId, RBalance),
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin, system = frame_system {
		/// Transfer `amount` of tokens on the relay chain from the Parachain account to
		/// the given `dest` account.
		#[weight = 10]
		fn transfer_tokens(origin, dest: RAccountId, amount: RBalance) {
			//TODO: Remove the tokens from the given account
			ensure_signed(origin)?;

			let msg = <T as Trait>::UpwardMessage::transfer(dest.clone(), amount);
			<T as Trait>::UpwardMessageSender::send_upward_message(&msg, UpwardMessageOrigin::Signed)
				.expect("Should not fail; qed");

			Self::deposit_event(Event::TransferredTokens(dest, amount));
		}

		fn deposit_event() = default;
	}
}
