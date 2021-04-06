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
//! * `XcmpMessageHandler`
//!
//! Also provides an implementation of `SendXcm` to handle outgoing XCM.

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{prelude::*, convert::TryFrom};
use rand_chacha::{rand_core::{RngCore, SeedableRng}, ChaChaRng};
use codec::{Decode, Encode};
use sp_runtime::{RuntimeDebug, traits::Hash};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::{DispatchError, Weight}, traits::{EnsureOrigin, Get}, error::BadOrigin,
};
use xcm::{
	VersionedXcm, VersionedXcmGeneric, v0::{
		Error as XcmError, ExecuteXcm, Junction, MultiLocation, SendXcm, Xcm, Outcome, XcmGeneric,
	},
};
use cumulus_primitives_core::{
	DownwardMessageHandler, XcmpMessageHandler, InboundDownwardMessage,
	ParaId, UpwardMessageSender, XcmpMessageSource, ChannelStatus,
	relay_chain::BlockNumber as RelayBlockNumber, MessageSendError,
	GetChannelInfo,
};
use xcm_executor::traits::LocationConversion;

pub trait Config: frame_system::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

	/// Something to execute an XCM message.
	type XcmExecutor: ExecuteXcm<Self::Call>;
	/// Something to send an upward message.
	type UpwardMessageSender: UpwardMessageSender;
	/// Something to send an HRMP message.
	type ChannelInfo: GetChannelInfo;

	/// Required origin for sending XCM messages. Typically Root or parachain
	/// council majority.
	type SendXcmOrigin: EnsureOrigin<Self::Origin>;
	/// Utility for converting from the signed origin (of type `Self::AccountId`) into a sensible
	/// `MultiLocation` ready for passing to the XCM interpreter.
	type AccountIdConverter: LocationConversion<Self::AccountId>;

	/// The maximum amount of weight we will give to the execution of a downward message.
	// TODO: ditch this and queue up downward messages just like XCMP messages.
	type MaxDownwardMessageWeight: Get<Weight>;
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug)]
pub enum InboundStatus {
	Ok,
	Suspended,
}

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum OutboundStatus {
	Ok,
	Suspended,
}

decl_storage! {
	trait Store for Module<T: Config> as XcmHandler {
		/// Status of the inbound XCMP channels.
		InboundXcmpStatus: Vec<(ParaId, InboundStatus, Vec<(RelayBlockNumber, XcmpMessageFormat)>)>;

		/// Inbound aggregate XCMP messages. It can only be one per ParaId/block.
		InboundXcmpMessages: double_map hasher(blake2_128_concat) ParaId,
			hasher(twox_64_concat) RelayBlockNumber
			=> Vec<u8>;

		/// The non-empty XCMP channels in order of becoming non-empty, and the index of the first
		/// and last outbound message. If the two indices are equal, then it indicates an empty
		/// queue and there must be a non-`Ok` `OutboundStatus`. We assume queues grow no greater
		/// than 65535 items. Queue indices for normal messages begin at one; zero is reserved in
		/// case of the need to send a high-priority signal message this block.
		/// The bool is true if there is a signal message waiting to be sent.
		OutboundXcmpStatus: Vec<(ParaId, OutboundStatus, bool, u16, u16)>;

		// The new way of doing it:
		/// The messages outbound in a given XCMP channel.
		OutboundXcmpMessages: double_map hasher(blake2_128_concat) ParaId,
			hasher(twox_64_concat) u16 => Vec<u8>;

		/// Any signal messages waiting to be sent.
		SignalMessages: map hasher(blake2_128_concat) ParaId => Vec<u8>;
	}
}

decl_event! {
	pub enum Event<T> where Hash = <T as frame_system::Config>::Hash {
		/// Some XCM was executed ok.
		Success(Option<Hash>),
		/// Some XCM failed.
		Fail(Option<Hash>, XcmError),
		/// Bad XCM version used.
		BadVersion(Option<Hash>),
		/// Bad XCM format used.
		BadFormat(Option<Hash>),
		/// An upward message was sent to the relay chain.
		UpwardMessageSent(Option<Hash>),
		/// An HRMP message was sent to a sibling parachain.
		XcmpMessageSent(Option<Hash>),
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
		/// Failed to send XCM message.
		FailedToSend,
		/// Bad XCM origin.
		BadXcmOrigin,
		/// Bad XCM data.
		BadXcm,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		fn on_idle(_now: T::BlockNumber, max_weight: Weight) -> Weight {
			// on_idle processes additional messages with any remaining block weight.
			Self::service_xcmp_queue(max_weight)
		}

		#[weight = 1_000]
		fn send_xcm(origin, dest: MultiLocation, message: Xcm) {
			T::SendXcmOrigin::ensure_origin(origin)?;
			<Self as SendXcm>::send_xcm(dest, message).map_err(|_| Error::<T>::FailedToSend)?;
		}

		#[weight = 1_000]
		fn send_upward_xcm(origin, message: VersionedXcm) {
			T::SendXcmOrigin::ensure_origin(origin)?;
			let data = message.encode();
			T::UpwardMessageSender::send_upward_message(data).map_err(|_| Error::<T>::FailedToSend)?;
		}

		#[weight = 1_000]
		fn send_xcmp_xcm(origin, recipient: ParaId, message: VersionedXcmGeneric<()>) {
			T::SendXcmOrigin::ensure_origin(origin)?;
			Self::send_fragment(recipient, XcmpMessageFormat::ConcatenatedVersionedXcm, message)
				.map_err(|_| Error::<T>::FailedToSend)?;
		}
	}
}

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode)]
pub enum ChannelSignal {
	Suspend,
	Resume,
}

/// The aggregate XCMP message format.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode)]
pub enum XcmpMessageFormat {
	/// Encoded `VersionedXcm` messages, all concatenated.
	ConcatenatedVersionedXcm,
	/// Encoded `Vec<u8>` messages, all concatenated.
	ConcatenatedEncodedBlob,
	/// One or more channel control signals; these should be interpreted immediately upon receipt
	/// from the relay-chain.
	Signals,
}

impl<T: Config> Module<T> {
	/// Execute an XCM message from a local, signed, origin.
	///
	/// Returns `DispatchError` if execution was not possible. Otherwise returns the amount of
	/// weight that was used in its attempted execution.
	///
	/// An event is deposited indicating whether `msg` could be executed completely or only
	/// partially.
	///
	/// No more than `max_weight` will be used in its attempted execution. If this is less than the
	/// maximum amount of weight that the message could take to be executed, then no execution
	/// attempt will be made.
	///
	/// NOTE: A successful return to this does *not* imply that the `msg` was executed successfully
	/// to completion; only that *some* of it was executed.
	pub fn execute_xcm(
		origin: T::AccountId,
		id: impl Encode,
		xcm: XcmGeneric<T::Call>,
		max_weight: Weight,
	)
		-> Result<Weight, DispatchError>
	{
		let xcm_origin = T::AccountIdConverter::try_into_location(origin)
			.map_err(|_| Error::<T>::BadXcmOrigin)?;
		let maybe_hash = id.using_encoded(|s| if s.is_empty() { None } else { Some(T::Hashing::hash(s)) });
		let (event, weight) = match T::XcmExecutor::execute_xcm(xcm_origin, xcm, max_weight) {
			Outcome::Complete(weight) => (Event::<T>::Success(maybe_hash), weight),
			Outcome::Incomplete(weight, error) => (Event::<T>::Fail(maybe_hash, error), weight),
			Outcome::Error(_error) => Err(Error::<T>::BadXcm)?,
		};
		Self::deposit_event(event);
		Ok(weight)
	}

	/// Place a message `fragment` on the outgoing XCMP queue for `recipient`.
	///
	/// Format is the type of aggregate message that the `fragment` may be safely encoded and
	/// appended onto. Whether earlier unused space is used for the fragment at the risk of sending
	/// it out of order is determined with `qos`. NOTE: For any two messages to be guaranteed to be
	/// dispatched in order, then both must be sent with `ServiceQuality::Ordered`.
	///
	/// ## Background
	///
	/// For our purposes, one HRMP "message" is actually an aggregated block of XCM "messages".
	///
	/// For the sake of clarity, we distinguish between them as message AGGREGATEs versus
	/// message FRAGMENTs.
	///
	/// So each AGGREGATE is comprised af one or more concatenated SCALE-encoded `Vec<u8>`
	/// FRAGMENTs. Though each fragment is already probably a SCALE-encoded Xcm, we can't be
	/// certain, so we SCALE encode each `Vec<u8>` fragment in order to ensure we have the
	/// length prefixed and can thus decode each fragment from the aggregate stream. With this,
	/// we can concatenate them into a single aggregate blob without needing to be concerned
	/// about encoding fragment boundaries.
	fn send_fragment<Fragment: Encode>(
		recipient: ParaId,
		format: XcmpMessageFormat,
		fragment: Fragment,
	) -> Result<u32, MessageSendError> {
		let data = fragment.encode();

		// TODO: Cache max_message_size in `OutboundXcmpMessages` once known; that way it's only
		//  accessed when a new page is needed.

		let max_message_size = T::ChannelInfo::get_channel_max(recipient)
			.ok_or(MessageSendError::NoChannel)?;
		if data.len() > max_message_size {
			return Err(MessageSendError::TooBig);
		}

		let mut s = OutboundXcmpStatus::get();
		let index = s.iter().position(|item| item.0 == recipient)
			.unwrap_or_else(|| {
				s.push((recipient, OutboundStatus::Ok, false, 0, 0));
				s.len() - 1
			});
		let have_active = s[index].4 > s[index].3;
		let appended = have_active && OutboundXcmpMessages::mutate(recipient, s[index].4 - 1, |s| {
			if XcmpMessageFormat::decode(&mut &s[..]) != Ok(format) { return false }
			if s.len() + data.len() > max_message_size { return false }
			s.extend_from_slice(&data[..]);
			return true
		});
		if appended {
			Ok((s[index].4 - s[index].3 - 1) as u32)
		} else {
			// Need to add a new page.
			let page_index = s[index].4;
			s[index].4 += 1;
			let mut new_page = format.encode();
			new_page.extend_from_slice(&data[..]);
			OutboundXcmpMessages::insert(recipient, page_index, new_page);
			let r = (s[index].4 - s[index].3 - 1) as u32;
			OutboundXcmpStatus::put(s);
			Ok(r)
		}
	}

	/// Sends a signal to the `dest` chain over XCMP. This is guaranteed to be dispatched on this
	/// block.
	fn send_signal(dest: ParaId, signal: ChannelSignal) -> Result<(), ()> {
		let mut s = OutboundXcmpStatus::get();
		if let Some(index) = s.iter().position(|item| item.0 == dest) {
			s[index].2 = true;
		} else {
			s.push((dest, OutboundStatus::Ok, true, 0, 0));
		}
		SignalMessages::mutate(dest, |page| if page.is_empty() {
			*page = (XcmpMessageFormat::Signals, signal).encode();
		} else {
			signal.using_encoded(|s| page.extend_from_slice(s));
		});
		OutboundXcmpStatus::put(s);

		Ok(())
	}

	fn create_shuffle(len: usize) -> Vec<usize> {
		// Create a shuffled order for use to iterate through.
		// Not a great random seed, but good enough for our purposes.
		let seed = frame_system::Pallet::<T>::parent_hash();
		let seed = <[u8; 32]>::decode(&mut sp_runtime::traits::TrailingZeroInput::new(seed.as_ref()))
			.expect("input is padded with zeroes; qed");
		let mut rng = ChaChaRng::from_seed(seed);
		let mut shuffled = (0..len).collect::<Vec<_>>();
		for i in 0..len {
			let j = (rng.next_u32() as usize) % len;
			let a = shuffled[i];
			shuffled[i] = shuffled[j];
			shuffled[j] = a;
		}
		shuffled
	}

	fn handle_blob_message(_sender: ParaId, _sent_at: RelayBlockNumber, _blob: Vec<u8>, _weight_limit: Weight) -> Result<Weight, bool> {
		debug_assert!(false, "Blob messages not handled.");
		Err(false)
	}

	fn handle_xcm_message(
		sender: ParaId,
		_sent_at: RelayBlockNumber,
		xcm: VersionedXcmGeneric<T::Call>,
		max_weight: Weight,
	) -> Result<Weight, XcmError> {
		let hash = Encode::using_encoded(&xcm, T::Hashing::hash);
		log::debug!("Processing XCMP-XCM: {:?}", &hash);
		let (result, event) = match XcmGeneric::<T::Call>::try_from(xcm) {
			Ok(xcm) => {
				let location = (
					Junction::Parent,
					Junction::Parachain { id: sender.into() },
				);
				match T::XcmExecutor::execute_xcm(
					location.into(),
					xcm,
					max_weight,
				) {
					Outcome::Error(e) => (Err(e), RawEvent::Fail(Some(hash), e)),
					Outcome::Complete(w) => (Ok(w), RawEvent::Success(Some(hash))),
					// As far as the caller is concerned, this was dispatched without error, so
					// we just report the weight used.
					Outcome::Incomplete(w, e) => (Ok(w), RawEvent::Fail(Some(hash), e)),
				}
			}
			Err(()) => (Err(XcmError::UnhandledXcmVersion), RawEvent::BadVersion(Some(hash))),
		};
		Self::deposit_event(event);
		result
	}

	fn process_xcmp_message(
		sender: ParaId,
		(sent_at, format): (RelayBlockNumber, XcmpMessageFormat),
		max_weight: Weight,
	) -> (Weight, bool) {
		let data = InboundXcmpMessages::get(sender, sent_at);
		let mut last_remaining_fragments;
		let mut remaining_fragments = &data[..];
		let mut weight_used = 0;
		// TODO: Handle whether it is in order or not in the fragment. For that we'll need a new
		//  XcmpMessageFormat type.
		match format {
			XcmpMessageFormat::ConcatenatedVersionedXcm => {
				while !remaining_fragments.is_empty() {
					last_remaining_fragments = remaining_fragments;
					if let Ok(xcm) = VersionedXcmGeneric::<T::Call>::decode(&mut remaining_fragments) {
						let weight = max_weight - weight_used;
						match Self::handle_xcm_message(sender, sent_at, xcm, weight) {
							Ok(used) => weight_used = weight_used.saturating_add(used),
							Err(XcmError::TooMuchWeightRequired) => {
								// That message didn't get processed this time because of being
								// too heavy. We leave it around for next time and bail.
								remaining_fragments = last_remaining_fragments;
								break;
							}
							Err(_) => {
								// Message looks invalid; don't attempt to retry
							}
						}
					} else {
						debug_assert!(false, "Invalid incoming XCMP message data");
						remaining_fragments = &b""[..];
					}
				}
			}
			XcmpMessageFormat::ConcatenatedEncodedBlob => {
				while !remaining_fragments.is_empty() {
					last_remaining_fragments = remaining_fragments;
					if let Ok(blob) = <Vec<u8>>::decode(&mut remaining_fragments) {
						let weight = max_weight - weight_used;
						match Self::handle_blob_message(sender, sent_at, blob, weight) {
							Ok(used) => weight_used = weight_used.saturating_add(used),
							Err(true) => {
								// That message didn't get processed this time because of being
								// too heavy. We leave it around for next time and bail.
								remaining_fragments = last_remaining_fragments;
								break;
							}
							Err(false) => {
								// Message invalid; don't attempt to retry
							}
						}
					} else {
						debug_assert!(false, "Invalid incoming blob message data");
						remaining_fragments = &b""[..];
					}
				}
			}
			XcmpMessageFormat::Signals => {
				debug_assert!(false, "All signals are handled immediately; qed");
				remaining_fragments = &b""[..];
			}
		}
		let is_empty = remaining_fragments.is_empty();
		if is_empty {
			InboundXcmpMessages::remove(sender, sent_at);
		} else {
			InboundXcmpMessages::insert(sender, sent_at, remaining_fragments);
		}
		(weight_used, is_empty)
	}

	/// Service the incoming XCMP message queue attempting to execute up to `max_weight` execution
	/// weight of messages.
	fn service_xcmp_queue(max_weight: Weight) -> Weight {
		// TODO: Move to Config trait.
		let resume_threshold = 1;
		// The amount of remaining weight under which we stop processing messages.
		// TODO: Move to Config trait.
		let threshold_weight = 100_000;
		// TODO: Move to Config trait.
		let weight_restrict_decay = 2;

		// sorted.
		let mut status = InboundXcmpStatus::get();
		if status.len() == 0 {
			return 0
		}

		let mut shuffled = Self::create_shuffle(status.len());
		let mut weight_used = 0;
		let mut weight_available = 0;

		// We don't want the possibility of a chain sending a series of really heavy messages and
		// tying up the block's execution time from other chains. Therefore we execute any remaining
		// messages in a random order.
		// Order within a single channel will always be preserved, however this does mean that
		// relative order between channels may not. The result is that chains which tend to send
		// fewer, lighter messages will generally have a lower latency than chains which tend to
		// send more, heavier messages.

		let mut shuffle_index = 0;
		while shuffle_index < shuffled.len() && max_weight.saturating_sub(weight_used) < threshold_weight {
			let index = shuffled[shuffle_index];
			let sender = status[index].0;

			if weight_available != max_weight {
				// Get incrementally closer to freeing up max_weight for message execution over the
				// first round. For the second round we unlock all weight. If we come close enough
				// on the first round to unlocking everything, then we do so.
				if shuffle_index < status.len() {
					weight_available += (max_weight - weight_available) / weight_restrict_decay;
					if weight_available + threshold_weight > max_weight {
						weight_available = max_weight;
					}
				} else {
					weight_available = max_weight;
				}
			}

			let weight_processed = if status[index].2.is_empty() {
				debug_assert!(false, "channel exists in status; there must be messages; qed");
				0
			} else {
				// Process up to one block's worth for now.
				let weight_remaining = weight_available.saturating_sub(weight_used);
				let (weight_processed, is_empty) = Self::process_xcmp_message(
					sender,
					status[index].2[0],
					weight_remaining,
				);
				if is_empty {
					status[index].2.remove(0);
				}
				weight_processed
			};
			weight_used += weight_processed;

			if status[index].2.len() <= resume_threshold && status[index].1 == InboundStatus::Suspended {
				// Resume
				let r = Self::send_signal(sender, ChannelSignal::Resume);
				debug_assert!(r.is_ok(), "WARNING: Failed sending resume into suspended channel");
				status[index].1 = InboundStatus::Ok;
			}

			// If there are more and we're making progress, we process them after we've given the
			// other channels a look in. If we've still not unlocked all weight, then we set them
			// up for processing a second time anyway.
			if !status[index].2.is_empty() && weight_processed > 0 || weight_available != max_weight {
				if shuffle_index + 1 == shuffled.len() {
					// Only this queue left. Just run around this loop once more.
					continue
				}
				shuffled.push(index);
			}
			shuffle_index += 1;
		}

		// Only retain the senders that have non-empty queues.
		status.retain(|item| !item.2.is_empty());

		InboundXcmpStatus::put(status);
		weight_used
	}

	fn suspend_channel(target: ParaId) {
		OutboundXcmpStatus::mutate(|s| {
			if let Some(index) = s.iter().position(|item| item.0 == target) {
				let ok = s[index].1 == OutboundStatus::Ok;
				debug_assert!(ok, "WARNING: Attempt to suspend channel that was not Ok.");
				s[index].1 = OutboundStatus::Suspended;
			} else {
				s.push((target, OutboundStatus::Suspended, false, 0, 0));
			}
		});
	}

	fn resume_channel(target: ParaId) {
		OutboundXcmpStatus::mutate(|s| {
			if let Some(index) = s.iter().position(|item| item.0 == target) {
				let suspended = s[index].1 == OutboundStatus::Suspended;
				debug_assert!(suspended, "WARNING: Attempt to resume channel that was not suspended.");
				if s[index].3 == s[index].4 {
					s.remove(index);
				} else {
					s[index].1 = OutboundStatus::Ok;
				}
			} else {
				debug_assert!(false, "WARNING: Attempt to resume channel that was not suspended.");
			}
		});
	}
}

impl<T: Config> XcmpMessageHandler for Module<T> {
	fn handle_xcmp_messages(
		iter: impl Iterator<Item=(ParaId, RelayBlockNumber, Vec<u8>)>,
		max_weight: Weight,
	) -> Weight {
		let mut status = InboundXcmpStatus::get();

		// TODO: Move to Config trait.
		let suspend_threshold = 2;
		// TODO: Move to Config trait.
		let hard_limit = 5;

		for (sender, sent_at, data) in iter {

			// Figure out the message format.
			let mut data_ref = &data[..];
			let format = match XcmpMessageFormat::decode(&mut data_ref) {
				Ok(f) => f,
				Err(_) => {
					debug_assert!(false, "Unknown XCMP message format. Silently dropping message");
					continue
				},
			};
			if format == XcmpMessageFormat::Signals {
				while !data_ref.is_empty() {
					use ChannelSignal::*;
					match ChannelSignal::decode(&mut data_ref) {
						Ok(Suspend) => Self::suspend_channel(sender),
						Ok(Resume) => Self::resume_channel(sender),
						Err(_) => break,
					}
				}
			} else {
				// Record the fact we received it.
				match status.binary_search_by_key(&sender, |item| item.0) {
					Ok(i) => {
						let count = status[i].2.len();
						if count >= suspend_threshold && status[i].1 == InboundStatus::Ok {
							status[i].1 = InboundStatus::Suspended;
							let r = Self::send_signal(sender, ChannelSignal::Suspend);
							if r.is_err() {
								log::warn!("Attempt to suspend channel failed. Messages may be dropped.");
							}
						}
						if count < hard_limit {
							status[i].2.push((sent_at, format));
						} else {
							debug_assert!(false, "XCMP channel queue full. Silently dropping message");
						}
					},
					Err(_) => status.push((sender, InboundStatus::Ok, vec![(sent_at, format)])),
				}
				// Queue the payload for later execution.
				InboundXcmpMessages::insert(sender, sent_at, data_ref);
			}

			// TODO: Execute messages immediately if `status.is_empty()`.
		}
		status.sort();
		InboundXcmpStatus::put(status);

		Self::service_xcmp_queue(max_weight)
	}
}

impl<T: Config> XcmpMessageSource for Module<T> {
	fn take_outbound_messages(maximum_channels: usize) -> Vec<(ParaId, Vec<u8>)> {
		let mut statuses = OutboundXcmpStatus::get();
		let old_statuses_len = statuses.len();
		let max_message_count = statuses.len().min(maximum_channels);
		let mut result = Vec::with_capacity(max_message_count);

		for status in statuses.iter_mut() {
			let (para_id, outbound_status, mut signalling, mut begin, mut end) = *status;

			if result.len() == max_message_count {
				// We check this condition in the beginning of the loop so that we don't include
				// a message where the limit is 0.
				break;
			}
			if outbound_status == OutboundStatus::Suspended {
				continue
			}
			let (max_size_now, max_size_ever) = match T::ChannelInfo::get_channel_status(para_id) {
				ChannelStatus::Closed => {
					// This means that there is no such channel anymore. Nothing to be done but
					// swallow the messages and discard the status.
					for i in begin..end {
						OutboundXcmpMessages::remove(para_id, i);
					}
					if signalling {
						SignalMessages::remove(para_id);
					}
					*status = (para_id, OutboundStatus::Ok, false, 0, 0);
					continue
				}
				ChannelStatus::Full => continue,
				ChannelStatus::Ready(n, e) => (n, e),
			};

			let page = if signalling {
				let page = SignalMessages::get(para_id);
				if page.len() < max_size_now {
					SignalMessages::remove(para_id);
					signalling = false;
					page
				} else {
					continue
				}
			} else if end > begin {
				let page = OutboundXcmpMessages::get(para_id, begin);
				if page.len() < max_size_now {
					OutboundXcmpMessages::remove(para_id, begin);
					begin += 1;
					page
				} else {
					continue
				}
			} else {
				continue;
			};
			if begin == end {
				begin = 0;
				end = 0;
			}

			if page.len() > max_size_ever {
				// TODO: #274 This means that the channel's max message size has changed since
				//   the message was sent. We should parse it and split into smaller mesasges but
				//   since it's so unlikely then for now we just drop it.
				log::warn!("WARNING: oversize message in queue. silently dropping.");
			} else {
				result.push((para_id, page));
			}

			*status = (para_id, outbound_status, signalling, begin, end);
		}

		// Sort the outbound messages by ascending recipient para id to satisfy the acceptance
		// criteria requirement.
		result.sort_by_key(|m| m.0);

		// Prune hrmp channels that became empty. Additionally, because it may so happen that we
		// only gave attention to some channels in `non_empty_hrmp_channels` it's important to
		// change the order. Otherwise, the next `on_finalize` we will again give attention
		// only to those channels that happen to be in the beginning, until they are emptied.
		// This leads to "starvation" of the channels near to the end.
		//
		// To mitigate this we shift all processed elements towards the end of the vector using
		// `rotate_left`. To get intuition how it works see the examples in its rustdoc.
		statuses.retain(|x| x.1 == OutboundStatus::Suspended || x.2 || x.3 < x.4);

		// old_status_len must be >= status.len() since we never add anything to status.
		let pruned = old_statuses_len - statuses.len();
		// removing an item from status implies a message being sent, so the result messages must
		// be no less than the pruned channels.
		statuses.rotate_left(result.len() - pruned);

		OutboundXcmpStatus::put(statuses);

		result
		// END
	}
}

impl<T: Config> DownwardMessageHandler for Module<T> {
	fn handle_downward_message(msg: InboundDownwardMessage) -> Weight {
		let hash = msg.using_encoded(T::Hashing::hash);
		log::debug!("Processing Downward XCM: {:?}", &hash);
		let msg = VersionedXcmGeneric::<T::Call>::decode(&mut &msg.msg[..])
			.map(XcmGeneric::<T::Call>::try_from);
		let (event, weight_used) = match msg {
			Ok(Ok(xcm)) => {
				let weight_limit = T::MaxDownwardMessageWeight::get();
				match T::XcmExecutor::execute_xcm(Junction::Parent.into(), xcm, weight_limit) {
					Outcome::Complete(w) => (RawEvent::Success(Some(hash)), w),
					Outcome::Incomplete(w, e) => (RawEvent::Fail(Some(hash), e), w),
					Outcome::Error(e) => (RawEvent::Fail(Some(hash), e), 0),
				}
			}
			Ok(Err(..)) => (RawEvent::BadVersion(Some(hash)), 0),
			Err(..) => (RawEvent::BadFormat(Some(hash)), 0),
		};
		Self::deposit_event(event);
		weight_used
	}
}

impl<T: Config> SendXcm for Module<T> {
	fn send_xcm(dest: MultiLocation, msg: XcmGeneric<()>) -> Result<(), XcmError> {
		match dest.first() {
			// A message for us. We can't send it anywhere.
			None => Err(XcmError::DestinationIsLocal),
			// An upward message for the relay chain.
			Some(Junction::Parent) if dest.len() == 1 => {
				let data = VersionedXcmGeneric::<()>::from(msg).encode();
				let hash = T::Hashing::hash(&data);

				T::UpwardMessageSender::send_upward_message(data)
					.map_err(|e| XcmError::CannotReachDestination(e.into()))?;

				Self::deposit_event(RawEvent::UpwardMessageSent(Some(hash)));
				Ok(())
			}
			// An HRMP message for a sibling parachain.
			Some(Junction::Parent) if dest.len() == 2 => {
				let msg = VersionedXcmGeneric::<()>::from(msg);
				if let Some(Junction::Parachain { id }) = dest.at(1) {
					let hash = T::Hashing::hash_of(&msg);
					Self::send_fragment((*id).into(), XcmpMessageFormat::ConcatenatedVersionedXcm, msg)
						.map_err(|e| XcmError::CannotReachDestination(<&'static str>::from(e)))?;
					Self::deposit_event(RawEvent::XcmpMessageSent(Some(hash)));
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

/// Ensure that the origin `o` represents a sibling parachain.
/// Returns `Ok` with the parachain ID of the sibling or an `Err` otherwise.
pub fn ensure_sibling_para<OuterOrigin>(o: OuterOrigin) -> Result<ParaId, BadOrigin>
	where OuterOrigin: Into<Result<Origin, OuterOrigin>>
{
	match o.into() {
		Ok(Origin::SiblingParachain(id)) => Ok(id),
		_ => Err(BadOrigin),
	}
}

/// Ensure that the origin `o` represents is the relay chain.
/// Returns `Ok` if it does or an `Err` otherwise.
pub fn ensure_relay<OuterOrigin>(o: OuterOrigin) -> Result<(), BadOrigin>
	where OuterOrigin: Into<Result<Origin, OuterOrigin>>
{
	match o.into() {
		Ok(Origin::Relay) => Ok(()),
		_ => Err(BadOrigin),
	}
}
