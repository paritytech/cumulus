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

use frame_support::{
	decl_event, decl_module,
	dispatch::DispatchResult,
	traits::{Currency, ExistenceRequirement, WithdrawReason},
};
use frame_system::ensure_signed;

use codec::{Encode, Decode, Input, Output, compact};
use cumulus_primitives::{
	relay_chain::DownwardMessage,
	xcmp::{XcmpMessageHandler, XcmpMessageSender},
	DownwardMessageHandler, ParaId, UpwardMessageOrigin, UpwardMessageSender,
};
use cumulus_upward_message::BalancesMessage;
use polkadot_parachain::primitives::AccountIdConversion;

/// An envelope for an XCM. This is only really useful if you're not integrating into the runtime's
/// `Call` system.
pub struct XcmEnvelope(VersionedXcm);

impl Encode for VersionedXcm {
	fn encode_to<O: Output>(&self, dest: &mut O) {
		// Just insert 0xff, 0x00 before the
		dest.push_byte(0xff);
		dest.push_byte(0x00);
		dest.push(self.0);
	}
}

impl Decode for VersionedXcm {
	fn decode<I: Input>(input: &mut I) -> Result<Self, codec::Error> {
		if input.read_byte()? != 0xff || input.read_byte()? != 0x00 {
			return None
		}
		Ok(Self(Decode::decode(input)))
	}
}

/// A straight forward XCM, together with its version code.
#[derive(Clone, Eq, PartialEq, Encode, Decode)]
pub enum VersionedXcm {
	V0(v0::Xcm),
}

#[derive(Clone, Eq, PartialEq, Encode, Decode)]
pub enum VersionedMultiLocation {
	V0(v0::MultiLocation),
}

#[derive(Clone, Eq, PartialEq, Encode, Decode)]
pub enum VersionedMultiNetwork {
	V0(v0::MultiNetwork),
}

#[derive(Clone, Eq, PartialEq, Encode, Decode)]
pub enum VersionedMultiAsset {
	V0(v0::MultiAsset),
}

pub mod v0 {
	use super::*;

	#[derive(Clone, Eq, PartialEq, Encode, Decode)]
	pub enum MultiNetwork {
		Wildcard,
		Identified(Vec<u8>),
	}

	#[derive(Clone, Eq, PartialEq, Encode, Decode)]
	pub enum MultiLocation {
		Null,
		Parent,
		ChildOf { primary: Box<MultiLocation>, subordinate: Box<MultiLocation> },
		SiblingOf(Box<MultiLocation>),
		Reserved4,
		Reserved5,
		Reserved6,
		OpaqueRemark(Vec<u8>),
		AccountId32 { network: MultiNetwork, id: [u8; 32] },
		AccountIndex64 { network: MultiNetwork, #[compact] index: u64 },
		ParachainPrimaryAccount { network: MultiNetwork, #[compact] id: u32 },
		AccountKey20 { network: MultiNetwork, key: [u8; 20] },
	}

	#[derive(Clone, Eq)]
	pub enum AssetInstance {
		Undefined,
		Index8(u8),
		Index16(#[compact] u16),
		Index32(#[compact] u32),
		Index64(#[compact] u64),
		Index128(#[compact] u128),
		Array4([u8; 4]),
		Array8([u8; 8]),
		Array16([u8; 16]),
		Array32([u8; 32]),
		Blob(Vec<u8>),
	}

	#[derive(Clone, Eq, PartialEq, Encode, Decode)]
	pub enum MultiAsset {
		Wild,
		WildFungible,
		WildNonFungible,
		WildAbstractFungible { id: Vec<u8> },
		WildAbstractNonFungible { class: Vec<u8> },
		WildConcreteFungible { id: MultiLocation },
		WildConcreteNonFungible { class: MultiLocation },
		AbstractFungible { id: Vec<u8>, #[compact] amount: u128 },
		AbstractNonFungible { class: Vec<u8>, instance: AssetInstance },
		ConcreteFungible { id: MultiLocation, #[compact] amount: u128 },
		ConcreteNonFungible { class: MultiLocation, instance: AssetInstance },
		Each(Vec<MultiAsset>),
	}

	#[derive(Clone, Eq, PartialEq, Encode, Decode)]
	pub enum Ai {
		Each(Vec<Ai>),
		DepositAsset { asset: MultiAsset, dest: MultiLocation },
		ExchangeAsset { give: MultiAsset, receive: MultiAsset },
		InitiateReserveTransfer { asset: MultiAsset, dest: MultiLocation, effect: Ai },
		InitiateTeleport { asset: MultiAsset, dest: MultiLocation, effect: Ai },
		QueryHolding { #[compact] query_id: u64, dest: MultiLocation, assets: Vec<MultiAsset> },
	}

	#[derive(Clone, Eq, PartialEq, Encode, Decode)]
	pub enum Xcm {
		WithdrawAsset { asset: MultiAsset, effect: Ai },
		ReserveAssetTransfer { asset: MultiAsset, dest: MultiLocation, effect: Ai },
		ReserveAssetCredit { asset: MultiAsset, effect: Ai },
		TeleportAsset { asset: MultiAsset, effect: Ai },
		Balances { query_id: Vec<u8>, assets: Vec<MultiAsset> },
	}
}


#[derive(Encode, Decode)]
pub enum XcmpMessage<XAccountId, XBalance> {
	/// Transfer tokens to the given account from the Parachain account.
	TransferToken(XAccountId, XBalance),
}

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

	/// The sender of XCMP messages.
	type XcmpMessageSender: XcmpMessageSender<XcmpMessage<Self::AccountId, BalanceOf<Self>>>;
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
		TransferredTokensViaXCMP(ParaId, AccountId, Balance, DispatchResult),
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
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

			T::XcmpMessageSender::send_xcmp_message(
				para_id.into(),
				&XcmpMessage::TransferToken(dest, amount),
			).expect("Should not fail; qed");
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

impl<T: Trait> XcmpMessageHandler<XcmpMessage<T::AccountId, BalanceOf<T>>> for Module<T> {
	fn handle_xcmp_message(src: ParaId, msg: &XcmpMessage<T::AccountId, BalanceOf<T>>) {
		match msg {
			XcmpMessage::TransferToken(dest, amount) => {
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
