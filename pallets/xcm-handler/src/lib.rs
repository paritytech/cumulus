// Copyright 2020-2021 Parity Technologies (UK) Ltd.
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

//! A pallet which implements the message handling APIs for handling incoming XCM:
//! * `DownwardMessageHandler`
//! * `HrmpMessageHandler`
//!
//! Also provides an implementation of `SendXcm` to handle outgoing XCM.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use codec::{Decode, Encode};
	use cumulus_primitives_core::{
		DownwardMessageHandler, HrmpMessageHandler, HrmpMessageSender, InboundDownwardMessage,
		InboundHrmpMessage, OutboundHrmpMessage, ParaId, UpwardMessageSender,
	};
	use frame_support::{pallet_prelude::*, sp_runtime::traits::Hash};
	use frame_system::pallet_prelude::*;
	use sp_std::convert::{TryFrom, TryInto};
	use xcm::{
		v0::{Error as XcmError, ExecuteXcm, Junction, MultiLocation, SendXcm, Xcm},
		VersionedXcm,
	};
	use xcm_executor::traits::Convert;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Something to execute an XCM message.
		type XcmExecutor: ExecuteXcm;
		/// Something to send an upward message.
		type UpwardMessageSender: UpwardMessageSender;
		/// Something to send an HRMP message.
		type HrmpMessageSender: HrmpMessageSender;
		/// Required origin for sending XCM messages. Typically Root or parachain
		/// council majority.
		type SendXcmOrigin: EnsureOrigin<Self::Origin>;
		/// Utility for converting from the signed origin (of type `Self::AccountId`) into a sensible
		/// `MultiLocation` ready for passing to the XCM interpreter.
		type AccountIdConverter: Convert<MultiLocation, Self::AccountId>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000)]
		fn send_xcm(
			origin: OriginFor<T>,
			dest: MultiLocation,
			message: Xcm,
		) -> DispatchResultWithPostInfo {
			T::SendXcmOrigin::ensure_origin(origin)?;
			<Self as SendXcm>::send_xcm(dest, message).map_err(|_| Error::<T>::FailedToSend)?;
			Ok(().into())
		}

		#[pallet::weight(1_000)]
		fn send_upward_xcm(
			origin: OriginFor<T>,
			message: VersionedXcm,
		) -> DispatchResultWithPostInfo {
			T::SendXcmOrigin::ensure_origin(origin)?;
			let data = message.encode();
			T::UpwardMessageSender::send_upward_message(data)
				.map_err(|_| Error::<T>::FailedToSend)?;
			Ok(().into())
		}

		#[pallet::weight(1_000)]
		fn send_hrmp_xcm(
			origin: OriginFor<T>,
			recipient: ParaId,
			message: VersionedXcm,
		) -> DispatchResultWithPostInfo {
			T::SendXcmOrigin::ensure_origin(origin)?;
			let data = message.encode();
			let outbound_message = OutboundHrmpMessage { recipient, data };
			T::HrmpMessageSender::send_hrmp_message(outbound_message)
				.map_err(|_| Error::<T>::FailedToSend)?;
			Ok(().into())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	#[pallet::metadata(T::Hash = "Hash")]
	pub enum Event<T: Config> {
		/// Some XCM was executed ok.
		Success(T::Hash),
		/// Some XCM failed.
		Fail(T::Hash, XcmError),
		/// Bad XCM version used.
		BadVersion(T::Hash),
		/// Bad XCM format used.
		BadFormat(T::Hash),
		/// An upward message was sent to the relay chain.
		UpwardMessageSent(T::Hash),
		/// An HRMP message was sent to a sibling parachain.
		HrmpMessageSent(T::Hash),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Failed to send XCM message.
		FailedToSend,
		/// Bad XCM origin.
		BadXcmOrigin,
	}

	impl<T: Config> Pallet<T> {
		/// Execute an XCM message locally. Returns `DispatchError` if failed.
		pub fn execute_xcm(origin: T::AccountId, xcm: Xcm) -> DispatchResult {
			let xcm_origin = T::AccountIdConverter::reverse(origin)
				.map_err(|_| Error::<T>::BadXcmOrigin)?;
			let hash = T::Hashing::hash(&xcm.encode());
			let event = match T::XcmExecutor::execute_xcm(xcm_origin, xcm) {
				Ok(_) => Event::<T>::Success(hash),
				Err(e) => Event::<T>::Fail(hash, e),
			};
			Self::deposit_event(event);
			Ok(())
		}
	}

	impl<T: Config> DownwardMessageHandler for Pallet<T> {
		fn handle_downward_message(msg: InboundDownwardMessage) {
			let hash = msg.using_encoded(T::Hashing::hash);
			log::debug!("Processing Downward XCM: {:?}", &hash);
			let event = match VersionedXcm::decode(&mut &msg.msg[..]).map(Xcm::try_from) {
				Ok(Ok(xcm)) => match T::XcmExecutor::execute_xcm(Junction::Parent.into(), xcm) {
					Ok(..) => Event::Success(hash),
					Err(e) => Event::Fail(hash, e),
				},
				Ok(Err(..)) => Event::BadVersion(hash),
				Err(..) => Event::BadFormat(hash),
			};
			Self::deposit_event(event);
		}
	}

	impl<T: Config> HrmpMessageHandler for Pallet<T> {
		fn handle_hrmp_message(sender: ParaId, msg: InboundHrmpMessage) {
			let hash = msg.using_encoded(T::Hashing::hash);
			log::debug!("Processing HRMP XCM: {:?}", &hash);
			let event = match VersionedXcm::decode(&mut &msg.data[..]).map(Xcm::try_from) {
				Ok(Ok(xcm)) => {
					let location = (Junction::Parent, Junction::Parachain { id: sender.into() });
					match T::XcmExecutor::execute_xcm(location.into(), xcm) {
						Ok(..) => Event::Success(hash),
						Err(e) => Event::Fail(hash, e),
					}
				}
				Ok(Err(..)) => Event::BadVersion(hash),
				Err(..) => Event::BadFormat(hash),
			};
			Self::deposit_event(event);
		}
	}

	impl<T: Config> SendXcm for Pallet<T> {
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

					T::UpwardMessageSender::send_upward_message(data)
						.map_err(|_| XcmError::CannotReachDestination)?;
					Self::deposit_event(Event::UpwardMessageSent(hash));

					Ok(())
				}
				// An HRMP message for a sibling parachain.
				Some(Junction::Parent) if dest.len() == 2 => {
					if let Some(Junction::Parachain { id }) = dest.at(1) {
						let data = msg.encode();
						let hash = T::Hashing::hash(&data);
						let message = OutboundHrmpMessage {
							recipient: (*id).into(),
							data,
						};
						T::HrmpMessageSender::send_hrmp_message(message)
							.map_err(|_| XcmError::CannotReachDestination)?;
						Self::deposit_event(Event::HrmpMessageSent(hash));
						Ok(())
					} else {
						Err(XcmError::UnhandledXcmMessage)
					}
				}
				_ => {
					/* TODO: Handle other cases, like downward message */
					Err(XcmError::UnhandledXcmMessage)
				}
			}
		}
	}

	/// Origin for the parachains pallet.
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
}
