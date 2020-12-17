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

//! A pallet which implements the message-broker APIs for handling XCM:
//! * `DownwardMessageHandler`
//! * `HrmpMessageHandler`

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use sp_std::convert::{TryFrom, TryInto};
use frame_support::{
	decl_module, decl_event,
	sp_runtime::traits::Hash,
};
use cumulus_primitives::{
	ParaId, InboundHrmpMessage, InboundDownwardMessage, OutboundHrmpMessage,
	DownwardMessageHandler, HrmpMessageHandler, UpwardMessageSender, HrmpMessageSender,
};
use xcm::{
	VersionedXcm,
	v0::{Xcm, MultiLocation, Error as XcmError, Junction, SendXcm, ExecuteXcm}
};

pub trait Config: frame_system::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	/// Something to execute an XCM message.
	type XcmExecutor: ExecuteXcm;
	/// Something to send an upward message.
	type UpwardMessageSender: UpwardMessageSender;
	/// Something to send an HRMP message.
	type HrmpMessageSender: HrmpMessageSender;
}

decl_event! {
	pub enum Event<T> where Hash = <T as frame_system::Config>::Hash {
		/// Some XCM was executed ok.
		Success(Hash),
		/// Some XCM failed.
		Fail(Hash, XcmError),
		/// Bad XCM version used.
		BadVersion(Hash),
		/// Bad XCM format used.
		BadFormat(Hash),
		/// An upward message was sent to the relay chain.
		UpwardMessageSent(Hash),
		/// An HRMP message was sent to a sibling parachainchain.
		HrmpMessageSent(Hash),
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		fn deposit_event() = default;
	}
}

impl<T: Config> DownwardMessageHandler for Module<T> {
	fn handle_downward_message(msg: InboundDownwardMessage) {
		let hash = msg.using_encoded(T::Hashing::hash);
		frame_support::debug::print!("Processing Downward XCM: {:?}", &hash);
		match VersionedXcm::decode(&mut &msg.msg[..]).map(Xcm::try_from) {
			Ok(Ok(xcm)) => {
				match T::XcmExecutor::execute_xcm(Junction::Parent.into(), xcm) {
					Ok(..) => RawEvent::Success(hash),
					Err(e) => RawEvent::Fail(hash, e),
				};
			}
			Ok(Err(..)) => Self::deposit_event(RawEvent::BadVersion(hash)),
			Err(..) => Self::deposit_event(RawEvent::BadFormat(hash)),
		}
	}
}

impl<T: Config> HrmpMessageHandler for Module<T> {
	fn handle_hrmp_message(sender: ParaId, msg: InboundHrmpMessage) {
		let hash = msg.using_encoded(T::Hashing::hash);
		frame_support::debug::print!("Processing HRMP XCM: {:?}", &hash);
		match VersionedXcm::decode(&mut &msg.data[..]).map(Xcm::try_from) {
			Ok(Ok(xcm)) => {
				match T::XcmExecutor::execute_xcm(Junction::Parachain { id: sender.into() }.into(), xcm) {
					Ok(..) => RawEvent::Success(hash),
					Err(e) => RawEvent::Fail(hash, e),
				};
			}
			Ok(Err(..)) => Self::deposit_event(RawEvent::BadVersion(hash)),
			Err(..) => Self::deposit_event(RawEvent::BadFormat(hash)),
		}
	}
}

impl<T: Config> SendXcm for Module<T> {
	fn send_xcm(dest: MultiLocation, msg: Xcm) -> Result<(), XcmError> {
		let msg: VersionedXcm = msg.into();
		match dest.first() {
			// A message for us. Execute directly.
			None => {
				let msg = msg.try_into().map_err(|_| XcmError::UnhandledXcmVersion)?;
				let res = T::XcmExecutor::execute_xcm(MultiLocation::Null, msg);
				res
			}
			// An upward message for the relay chain.
			Some(Junction::Parent) if dest.len() == 1 => {
				let data = msg.encode();
				let hash = T::Hashing::hash(&data);

				T::UpwardMessageSender::send_upward_message(data);
				Self::deposit_event(RawEvent::UpwardMessageSent(hash));

				Ok(())
			}
			// An HRMP message for a sibling parachain.
			Some(Junction::Parachain { id }) => {
				let data = msg.encode();
				let hash = T::Hashing::hash(&data);
				let message = OutboundHrmpMessage {
					recipient: (*id).into(),
					data,
				};
				T::HrmpMessageSender::send_hrmp_message(message);
				Self::deposit_event(RawEvent::HrmpMessageSent(hash));
				Ok(())
			}
			_ => {
				/* TODO: Handle other cases, like downward message */
				Err(XcmError::UnhandledXcmMessage)
			}
		}
	}
}
