// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

//! This pallet deals with the low-level details of parachain message passing.
//!
//! Specifically, this pallet serves as a glue layer between cumulus collation pipeline and the
//! runtime that hosts this pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{
	decl_module, decl_storage, decl_event, storage,
	traits::Get,
	weights::{DispatchClass, Weight},
	StorageValue,
	sp_runtime::traits::Hash,
};
use frame_system::ensure_none;
use sp_inherents::{InherentData, InherentIdentifier, MakeFatalError, ProvideInherent};
use sp_std::{cmp, prelude::*, convert::{TryFrom, TryInto}};

use cumulus_primitives::{
	inherents::{DownwardMessagesType, DOWNWARD_MESSAGES_IDENTIFIER},
	well_known_keys, DownwardMessageHandler, InboundDownwardMessage, UpwardMessage, ParaId,
};

use xcm::{
	VersionedXcm,
	v0::{Xcm, MultiLocation, Error as XcmError, Junction, SendXcm, ExecuteXcm}
};

/// Origin for the parachains module.
#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum Origin {
	/// It comes from the (parent) relay chain.
	Relay,
	/// It comes from a (sibling) parachain.
	SiblingParachain(ParaId),
}

impl From<ParaId> for Origin {
	fn from(id: ParaId) -> Origin {
		Origin::SiblingParachain(id)
	}
}
impl From<u32> for Origin {
	fn from(id: u32) -> Origin {
		Origin::SiblingParachain(id.into())
	}
}

/// Configuration trait of the message broker pallet.
pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	/// The downward message handlers that will be informed when a message is received.
	type DownwardMessageHandlers: DownwardMessageHandler;
	/// Something to execute an XCM message.
	type XcmExecutor: ExecuteXcm;
	/// Some way of sending a message downward.
	type SendDownward: SendDownward;
}

decl_event! {
	pub enum Event<T> where Hash = <T as frame_system::Trait>::Hash {
		/// An upward message was sent to the relay chain.
		///
		/// The hash corresponds to the hash of the encoded upward message.
		UmpMessageSent(Hash),
		/// Some downward message was executed ok.
		DmpSuccess(Hash),
		/// Some downward message failed.
		DmpFail(Hash, XcmError),
		/// DMP message had a bad XCM version.
		DmpBadVersion(Hash),
		/// DMP message was of an invalid format.
		DmpBadFormat(Hash),
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as MessageBroker {
		PendingUpwardMessages: Vec<UpwardMessage>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		/// An entrypoint for an inherent to deposit downward messages into the runtime. It accepts
		/// and processes the list of downward messages.
		#[weight = (10, DispatchClass::Mandatory)]
		fn receive_downward_messages(origin, messages: Vec<InboundDownwardMessage>) {
			ensure_none(origin)?;

			let messages_len = messages.len() as u32;
			for message in messages {
				T::DownwardMessageHandlers::handle_downward_message(message);
			}

			// Store the processed_downward_messages here so that it's will be accessible from
			// PVF's `validate_block` wrapper and collation pipeline.
			storage::unhashed::put(
				well_known_keys::PROCESSED_DOWNWARD_MESSAGES,
				&messages_len,
			);
		}

		/// Executes the given downward messages by calling the message handlers.
		///
		/// The origin of this call needs to be `None` as this is an inherent.
		#[weight = (10, DispatchClass::Mandatory)]
		fn execute_downward_messages(origin, messages: Vec<InboundDownwardMessage>) {
			ensure_none(origin)?;

			// TODO: max messages should not be hardcoded. It should be determined based on the
			//   weight used by the handlers.
			let max_messages = 10;
			messages.iter().take(max_messages).for_each(|msg| {
				let hash = msg.using_encoded(T::Hashing::hash);
				frame_support::debug::print!("Processing downward message: {:?}", &hash);
				match VersionedXcm::decode(&mut &msg.msg[..]).map(Xcm::try_from) {
					Ok(Ok(xcm)) => {
						let event = match T::XcmExecutor::execute_xcm(Junction::Parent.into(), xcm) {
							Ok(..) => RawEvent::DmpSuccess(hash),
							Err(e) => RawEvent::DmpFail(hash, e),
						};
						Self::deposit_event(event);
					}
					Ok(Err(..)) => Self::deposit_event(RawEvent::DmpBadVersion(hash)),
					Err(..) => Self::deposit_event(RawEvent::DmpBadFormat(hash)),
				}
			});

			let processed = sp_std::cmp::min(messages.len(), max_messages) as u32;
			storage::unhashed::put(
				well_known_keys::PROCESSED_DOWNWARD_MESSAGES,
				&processed
			);
		}

		fn on_initialize() -> Weight {
			// Reads and writes performed by `on_finalize`.
			T::DbWeight::get().reads_writes(1, 2)
		}

		fn on_finalize() {
			// TODO: this should be not a constant, but sourced from the relay-chain configuration.
			const UMP_MSG_NUM_PER_CANDIDATE: usize = 5;

			<Self as Store>::PendingUpwardMessages::mutate(|up| {
				let num = cmp::min(UMP_MSG_NUM_PER_CANDIDATE, up.len());
				storage::unhashed::put(
					well_known_keys::UPWARD_MESSAGES,
					&up[0..num],
				);
				*up = up.split_off(num);
			});
		}
	}
}

impl<T: Trait> SendXcm for Module<T> {
	fn send_xcm(dest: MultiLocation, msg: Xcm) -> Result<(), XcmError> {
		let msg: VersionedXcm = msg.into();
		match dest.first() {
			// A message for us. Execute directly.
			None => {
				let msg = msg.try_into().map_err(|_| XcmError::UnhandledXcmVersion)?;
				T::XcmExecutor::execute_xcm(MultiLocation::Null, msg)
			}
			// An upward message - just send to the relay-chain.
			Some(Junction::Parent) if dest.len() == 1 => {
				//TODO: check fee schedule
				let data = msg.encode();
				let hash = T::Hashing::hash(&data);

				sp_io::storage::append(well_known_keys::UPWARD_MESSAGES, data);
				Self::deposit_event(RawEvent::UmpMessageSent(hash));

				Ok(())
			}
			// A message which needs routing via the relay-chain.
			Some(Junction::Parent) =>
				Self::send_xcm(
					MultiLocation::X1(Junction::Parent),
					Xcm::RelayTo { dest: dest.split_first().0, inner: Box::new(msg) }.into()
				),
			// A downward message
			_ => {
				T::SendDownward::send_downward(dest, msg)
			},
		}
	}
}

pub trait SendDownward {
	fn send_downward(dest: MultiLocation, msg: VersionedXcm) -> Result<(), XcmError>;
}

impl SendDownward for () {
	fn send_downward(_dest: MultiLocation, _msg: VersionedXcm) -> Result<(), XcmError> {
		Err(XcmError::Unimplemented)
	}
}

/// An error that can be raised upon sending an upward message.
pub enum SendUpErr {
	/// The message sent is too big.
	TooBig,
}

impl<T: Trait> Module<T> {
	pub fn send_upward_message(message: UpwardMessage) -> Result<(), SendUpErr> {
		// TODO: check the message against the limit. The limit should be sourced from the
		// relay-chain configuration.
		<Self as Store>::PendingUpwardMessages::append(message);
		Ok(())
	}
}

impl<T: Trait> ProvideInherent for Module<T> {
	type Call = Call<T>;
	type Error = MakeFatalError<()>;
	const INHERENT_IDENTIFIER: InherentIdentifier = DOWNWARD_MESSAGES_IDENTIFIER;

	fn create_inherent(data: &InherentData) -> Option<Self::Call> {
		data.get_data::<DownwardMessagesType>(&DOWNWARD_MESSAGES_IDENTIFIER)
			.expect("Downward messages inherent data failed to decode")
			.map(|msgs| Call::execute_downward_messages(msgs))
	}
}
