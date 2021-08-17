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

//! A pallet which uses the XCMP transport layer to handle both incoming and outgoing XCM message
//! sending and dispatch, queuing, signalling and backpressure. To do so, it implements:
//! * `XcmpMessageHandler`
//! * `XcmpMessageSource`
//!
//! Also provides an implementation of `SendXcm` which can be placed in a router tuple for relaying
//! XCM over XCMP if the destination is `Parent/Parachain`. It requires an implementation of
//! `XcmExecutor` for dispatching incoming XCM messages.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use codec::{Decode, Encode};
use cumulus_primitives_core::{
	relay_chain::BlockNumber as RelayBlockNumber, ChannelStatus, GetChannelInfo, MessageSendError,
	ParaId, XcmpMessageHandler, XcmpMessageSource, XcmpMessageFormat,
};
use frame_support::weights::Weight;
use rand_chacha::{
	rand_core::{RngCore, SeedableRng},
	ChaChaRng,
};
use sp_runtime::{traits::Hash, RuntimeDebug};
use sp_std::{prelude::*, convert::TryFrom};
use xcm::{latest::prelude::*, WrapVersion, VersionedXcm};

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Something to execute an XCM message. We need this to service the XCMoXCMP queue.
		type XcmExecutor: ExecuteXcm<Self::Call>;

		/// Information on the avaialble XCMP channels.
		type ChannelInfo: GetChannelInfo;

		/// Means of converting an `Xcm` into a `VersionedXcm`.
		type VersionWrapper: WrapVersion;
	}

	impl Default for QueueConfigData {
		fn default() -> Self {
			Self {
				suspend_threshold: 2,
				drop_threshold: 5,
				resume_threshold: 1,
				threshold_weight: 100_000,
				weight_restrict_decay: 2,
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_idle(_now: T::BlockNumber, max_weight: Weight) -> Weight {
			// on_idle processes additional messages with any remaining block weight.
			Self::service_xcmp_queue(max_weight)
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	#[pallet::metadata(Option<T::Hash> = "Option<Hash>")]
	pub enum Event<T: Config> {
		/// Some XCM was executed ok.
		Success(Option<T::Hash>),
		/// Some XCM failed.
		Fail(Option<T::Hash>, XcmError),
		/// Bad XCM version used.
		BadVersion(Option<T::Hash>),
		/// Bad XCM format used.
		BadFormat(Option<T::Hash>),
		/// An upward message was sent to the relay chain.
		UpwardMessageSent(Option<T::Hash>),
		/// An HRMP message was sent to a sibling parachain.
		XcmpMessageSent(Option<T::Hash>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Failed to send XCM message.
		FailedToSend,
		/// Bad XCM origin.
		BadXcmOrigin,
		/// Bad XCM data.
		BadXcm,
	}

	/// Status of the inbound XCMP channels.
	#[pallet::storage]
	pub(super) type InboundXcmpStatus<T: Config> = StorageValue<
		_,
		Vec<(
			ParaId,
			InboundStatus,
			Vec<(RelayBlockNumber, XcmpMessageFormat)>,
		)>,
		ValueQuery,
	>;

	/// Inbound aggregate XCMP messages. It can only be one per ParaId/block.
	#[pallet::storage]
	pub(super) type InboundXcmpMessages<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		ParaId,
		Twox64Concat,
		RelayBlockNumber,
		Vec<u8>,
		ValueQuery,
	>;

	/// The non-empty XCMP channels in order of becoming non-empty, and the index of the first
	/// and last outbound message. If the two indices are equal, then it indicates an empty
	/// queue and there must be a non-`Ok` `OutboundStatus`. We assume queues grow no greater
	/// than 65535 items. Queue indices for normal messages begin at one; zero is reserved in
	/// case of the need to send a high-priority signal message this block.
	/// The bool is true if there is a signal message waiting to be sent.
	#[pallet::storage]
	pub(super) type OutboundXcmpStatus<T: Config> =
		StorageValue<_, Vec<(ParaId, OutboundStatus, bool, u16, u16)>, ValueQuery>;

	// The new way of doing it:
	/// The messages outbound in a given XCMP channel.
	#[pallet::storage]
	pub(super) type OutboundXcmpMessages<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, ParaId, Twox64Concat, u16, Vec<u8>, ValueQuery>;

	/// Any signal messages waiting to be sent.
	#[pallet::storage]
	pub(super) type SignalMessages<T: Config> =
		StorageMap<_, Blake2_128Concat, ParaId, Vec<u8>, ValueQuery>;

	/// The configuration which controls the dynamics of the outbound queue.
	#[pallet::storage]
	pub(super) type QueueConfig<T: Config> = StorageValue<_, QueueConfigData, ValueQuery>;
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

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct QueueConfigData {
	/// The number of pages of messages which must be in the queue for the other side to be told to
	/// suspend their sending.
	suspend_threshold: u32,
	/// The number of pages of messages which must be in the queue after which we drop any further
	/// messages from the channel.
	drop_threshold: u32,
	/// The number of pages of messages which the queue must be reduced to before it signals that
	/// message sending may recommence after it has been suspended.
	resume_threshold: u32,
	// The amount of remaining weight under which we stop processing messages.
	threshold_weight: Weight,
	/// The speed to which the available weight approaches the maximum weight. A lower number
	/// results in a faster progression. A value of 1 makes the entire weight available initially.
	weight_restrict_decay: Weight,
}

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode)]
pub enum ChannelSignal {
	Suspend,
	Resume,
}

impl<T: Config> Pallet<T> {
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
	/// So each AGGREGATE is comprised of one or more concatenated SCALE-encoded `Vec<u8>`
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

		// Optimization note: `max_message_size` could potentially be stored in
		// `OutboundXcmpMessages` once known; that way it's only accessed when a new page is needed.

		let max_message_size =
			T::ChannelInfo::get_channel_max(recipient).ok_or(MessageSendError::NoChannel)?;
		if data.len() > max_message_size {
			return Err(MessageSendError::TooBig);
		}

		let mut s = <OutboundXcmpStatus<T>>::get();
		let index = s
			.iter()
			.position(|item| item.0 == recipient)
			.unwrap_or_else(|| {
				s.push((recipient, OutboundStatus::Ok, false, 0, 0));
				s.len() - 1
			});
		let have_active = s[index].4 > s[index].3;
		let appended = have_active
			&& <OutboundXcmpMessages<T>>::mutate(recipient, s[index].4 - 1, |s| {
				if XcmpMessageFormat::decode(&mut &s[..]) != Ok(format) {
					return false;
				}
				if s.len() + data.len() > max_message_size {
					return false;
				}
				s.extend_from_slice(&data[..]);
				return true;
			});
		if appended {
			Ok((s[index].4 - s[index].3 - 1) as u32)
		} else {
			// Need to add a new page.
			let page_index = s[index].4;
			s[index].4 += 1;
			let mut new_page = format.encode();
			new_page.extend_from_slice(&data[..]);
			<OutboundXcmpMessages<T>>::insert(recipient, page_index, new_page);
			let r = (s[index].4 - s[index].3 - 1) as u32;
			<OutboundXcmpStatus<T>>::put(s);
			Ok(r)
		}
	}

	/// Sends a signal to the `dest` chain over XCMP. This is guaranteed to be dispatched on this
	/// block.
	fn send_signal(dest: ParaId, signal: ChannelSignal) -> Result<(), ()> {
		let mut s = <OutboundXcmpStatus<T>>::get();
		if let Some(index) = s.iter().position(|item| item.0 == dest) {
			s[index].2 = true;
		} else {
			s.push((dest, OutboundStatus::Ok, true, 0, 0));
		}
		<SignalMessages<T>>::mutate(dest, |page| {
			if page.is_empty() {
				*page = (XcmpMessageFormat::Signals, signal).encode();
			} else {
				signal.using_encoded(|s| page.extend_from_slice(s));
			}
		});
		<OutboundXcmpStatus<T>>::put(s);

		Ok(())
	}

	pub fn send_blob_message(recipient: ParaId, blob: Vec<u8>) -> Result<u32, MessageSendError> {
		Self::send_fragment(recipient, XcmpMessageFormat::ConcatenatedEncodedBlob, blob)
	}

	pub fn send_xcm_message(
		recipient: ParaId,
		xcm: VersionedXcm<()>,
	) -> Result<u32, MessageSendError> {
		Self::send_fragment(recipient, XcmpMessageFormat::ConcatenatedVersionedXcm, xcm)
	}

	fn create_shuffle(len: usize) -> Vec<usize> {
		// Create a shuffled order for use to iterate through.
		// Not a great random seed, but good enough for our purposes.
		let seed = frame_system::Pallet::<T>::parent_hash();
		let seed = <[u8; 32]>::decode(&mut sp_runtime::traits::TrailingZeroInput::new(
			seed.as_ref(),
		))
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

	fn handle_blob_message(
		_sender: ParaId,
		_sent_at: RelayBlockNumber,
		_blob: Vec<u8>,
		_weight_limit: Weight,
	) -> Result<Weight, bool> {
		debug_assert!(false, "Blob messages not handled.");
		Err(false)
	}

	fn handle_xcm_message(
		sender: ParaId,
		_sent_at: RelayBlockNumber,
		xcm: VersionedXcm<T::Call>,
		max_weight: Weight,
	) -> Result<Weight, XcmError> {
		let hash = Encode::using_encoded(&xcm, T::Hashing::hash);
		log::debug!("Processing XCMP-XCM: {:?}", &hash);
		let (result, event) = match Xcm::<T::Call>::try_from(xcm) {
			Ok(xcm) => {
				let location = (1, Parachain(sender.into()));
				match T::XcmExecutor::execute_xcm(location.into(), xcm, max_weight) {
					Outcome::Error(e) => (Err(e.clone()), Event::Fail(Some(hash), e)),
					Outcome::Complete(w) => (Ok(w), Event::Success(Some(hash))),
					// As far as the caller is concerned, this was dispatched without error, so
					// we just report the weight used.
					Outcome::Incomplete(w, e) => (Ok(w), Event::Fail(Some(hash), e)),
				}
			}
			Err(()) => (
				Err(XcmError::UnhandledXcmVersion),
				Event::BadVersion(Some(hash)),
			),
		};
		Self::deposit_event(event);
		result
	}

	fn process_xcmp_message(
		sender: ParaId,
		(sent_at, format): (RelayBlockNumber, XcmpMessageFormat),
		max_weight: Weight,
	) -> (Weight, bool) {
		let data = <InboundXcmpMessages<T>>::get(sender, sent_at);
		let mut last_remaining_fragments;
		let mut remaining_fragments = &data[..];
		let mut weight_used = 0;
		match format {
			XcmpMessageFormat::ConcatenatedVersionedXcm => {
				while !remaining_fragments.is_empty() {
					last_remaining_fragments = remaining_fragments;
					if let Ok(xcm) = VersionedXcm::<T::Call>::decode(&mut remaining_fragments) {
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
			<InboundXcmpMessages<T>>::remove(sender, sent_at);
		} else {
			<InboundXcmpMessages<T>>::insert(sender, sent_at, remaining_fragments);
		}
		(weight_used, is_empty)
	}

	/// Service the incoming XCMP message queue attempting to execute up to `max_weight` execution
	/// weight of messages.
	///
	/// Channels are first shuffled and then processed in this random one page at a time, order over
	/// and over until either `max_weight` is exhausted or no channel has messages that can be
	/// processed any more.
	///
	/// There are two obvious "modes" that we could apportion `max_weight`: one would be to attempt
	/// to spend it all on the first channel's first page, then use the leftover (if any) for the
	/// second channel's first page and so on until finally we cycle back and the process messages
	/// on the first channel's second page &c. The other mode would be to apportion only `1/N` of
	/// `max_weight` for the first page (where `N` could be, perhaps, the number of channels to
	/// service, using the remainder plus the next `1/N` for the next channel's page &c.
	///
	/// Both modes have good qualities, the first ensures that a channel with a large message (over
	/// `1/N` does not get indefinitely blocked if other channels have continuous, light traffic.
	/// The second is fairer, and ensures that channels with continuous light messages don't suffer
	/// high latency.
	///
	/// The following code is a hybrid solution; we have a concept of `weight_available` which
	/// incrementally approaches `max_weight` as more channels are attempted to be processed. We use
	/// the parameter `weight_restrict_decay` to control the speed with which `weight_available`
	/// approaches `max_weight`, with `0` being strictly equivalent to the first aforementioned
	/// mode, and `N` approximating the second. A reasonable parameter may be `1`, which makes
	/// half of the `max_weight` available for the first page, then a quarter plus the remainder
	/// for the second &c. though empirical and or practical factors may give rise to adjusting it
	/// further.
	fn service_xcmp_queue(max_weight: Weight) -> Weight {
		let mut status = <InboundXcmpStatus<T>>::get(); // <- sorted.
		if status.len() == 0 {
			return 0;
		}

		let QueueConfigData {
			resume_threshold,
			threshold_weight,
			weight_restrict_decay,
			..
		} = <QueueConfig<T>>::get();

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
		while shuffle_index < shuffled.len()
			&& max_weight.saturating_sub(weight_used) >= threshold_weight
		{
			let index = shuffled[shuffle_index];
			let sender = status[index].0;

			if weight_available != max_weight {
				// Get incrementally closer to freeing up max_weight for message execution over the
				// first round. For the second round we unlock all weight. If we come close enough
				// on the first round to unlocking everything, then we do so.
				if shuffle_index < status.len() {
					weight_available +=
						(max_weight - weight_available) / (weight_restrict_decay + 1);
					if weight_available + threshold_weight > max_weight {
						weight_available = max_weight;
					}
				} else {
					weight_available = max_weight;
				}
			}

			let weight_processed = if status[index].2.is_empty() {
				debug_assert!(
					false,
					"channel exists in status; there must be messages; qed"
				);
				0
			} else {
				// Process up to one block's worth for now.
				let weight_remaining = weight_available.saturating_sub(weight_used);
				let (weight_processed, is_empty) =
					Self::process_xcmp_message(sender, status[index].2[0], weight_remaining);
				if is_empty {
					status[index].2.remove(0);
				}
				weight_processed
			};
			weight_used += weight_processed;

			if status[index].2.len() as u32 <= resume_threshold
				&& status[index].1 == InboundStatus::Suspended
			{
				// Resume
				let r = Self::send_signal(sender, ChannelSignal::Resume);
				debug_assert!(
					r.is_ok(),
					"WARNING: Failed sending resume into suspended channel"
				);
				status[index].1 = InboundStatus::Ok;
			}

			// If there are more and we're making progress, we process them after we've given the
			// other channels a look in. If we've still not unlocked all weight, then we set them
			// up for processing a second time anyway.
			if !status[index].2.is_empty() && (weight_processed > 0 || weight_available != max_weight)
			{
				if shuffle_index + 1 == shuffled.len() {
					// Only this queue left. Just run around this loop once more.
					continue;
				}
				shuffled.push(index);
			}
			shuffle_index += 1;
		}

		// Only retain the senders that have non-empty queues.
		status.retain(|item| !item.2.is_empty());

		<InboundXcmpStatus<T>>::put(status);
		weight_used
	}

	fn suspend_channel(target: ParaId) {
		<OutboundXcmpStatus<T>>::mutate(|s| {
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
		<OutboundXcmpStatus<T>>::mutate(|s| {
			if let Some(index) = s.iter().position(|item| item.0 == target) {
				let suspended = s[index].1 == OutboundStatus::Suspended;
				debug_assert!(
					suspended,
					"WARNING: Attempt to resume channel that was not suspended."
				);
				if s[index].3 == s[index].4 {
					s.remove(index);
				} else {
					s[index].1 = OutboundStatus::Ok;
				}
			} else {
				debug_assert!(
					false,
					"WARNING: Attempt to resume channel that was not suspended."
				);
			}
		});
	}
}

impl<T: Config> XcmpMessageHandler for Pallet<T> {
	fn handle_xcmp_messages<'a, I: Iterator<Item = (ParaId, RelayBlockNumber, &'a [u8])>>(
		iter: I,
		max_weight: Weight,
	) -> Weight {
		let mut status = <InboundXcmpStatus<T>>::get();

		let QueueConfigData {
			suspend_threshold,
			drop_threshold,
			..
		} = <QueueConfig<T>>::get();

		for (sender, sent_at, data) in iter {
			// Figure out the message format.
			let mut data_ref = data;
			let format = match XcmpMessageFormat::decode(&mut data_ref) {
				Ok(f) => f,
				Err(_) => {
					debug_assert!(
						false,
						"Unknown XCMP message format. Silently dropping message"
					);
					continue;
				}
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
						if count as u32 >= suspend_threshold && status[i].1 == InboundStatus::Ok {
							status[i].1 = InboundStatus::Suspended;
							let r = Self::send_signal(sender, ChannelSignal::Suspend);
							if r.is_err() {
								log::warn!(
									"Attempt to suspend channel failed. Messages may be dropped."
								);
							}
						}
						if (count as u32) < drop_threshold {
							status[i].2.push((sent_at, format));
						} else {
							debug_assert!(
								false,
								"XCMP channel queue full. Silently dropping message"
							);
						}
					}
					Err(_) => status.push((sender, InboundStatus::Ok, vec![(sent_at, format)])),
				}
				// Queue the payload for later execution.
				<InboundXcmpMessages<T>>::insert(sender, sent_at, data_ref);
			}

			// Optimization note; it would make sense to execute messages immediately if
			// `status.is_empty()` here.
		}
		status.sort();
		<InboundXcmpStatus<T>>::put(status);

		Self::service_xcmp_queue(max_weight)
	}
}

impl<T: Config> XcmpMessageSource for Pallet<T> {
	fn take_outbound_messages(maximum_channels: usize) -> Vec<(ParaId, Vec<u8>)> {
		let mut statuses = <OutboundXcmpStatus<T>>::get();
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
				continue;
			}
			let (max_size_now, max_size_ever) = match T::ChannelInfo::get_channel_status(para_id) {
				ChannelStatus::Closed => {
					// This means that there is no such channel anymore. Nothing to be done but
					// swallow the messages and discard the status.
					for i in begin..end {
						<OutboundXcmpMessages<T>>::remove(para_id, i);
					}
					if signalling {
						<SignalMessages<T>>::remove(para_id);
					}
					*status = (para_id, OutboundStatus::Ok, false, 0, 0);
					continue;
				}
				ChannelStatus::Full => continue,
				ChannelStatus::Ready(n, e) => (n, e),
			};

			let page = if signalling {
				let page = <SignalMessages<T>>::get(para_id);
				if page.len() < max_size_now {
					<SignalMessages<T>>::remove(para_id);
					signalling = false;
					page
				} else {
					continue;
				}
			} else if end > begin {
				let page = <OutboundXcmpMessages<T>>::get(para_id, begin);
				if page.len() < max_size_now {
					<OutboundXcmpMessages<T>>::remove(para_id, begin);
					begin += 1;
					page
				} else {
					continue;
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

		<OutboundXcmpStatus<T>>::put(statuses);

		result
	}
}

/// Xcm sender for sending to a sibling parachain.
impl<T: Config> SendXcm for Pallet<T> {
	fn send_xcm(dest: MultiLocation, msg: Xcm<()>) -> Result<(), XcmError> {
		match &dest {
			// An HRMP message for a sibling parachain.
			MultiLocation { parents: 1, interior: X1(Parachain(id)) } => {
				let versioned_xcm = T::VersionWrapper::wrap_version(&dest, msg)
					.map_err(|()| XcmError::DestinationUnsupported)?;
				let hash = T::Hashing::hash_of(&versioned_xcm);
				Self::send_fragment(
					(*id).into(),
					XcmpMessageFormat::ConcatenatedVersionedXcm,
					versioned_xcm,
				)
				.map_err(|e| XcmError::SendFailed(<&'static str>::from(e)))?;
				Self::deposit_event(Event::XcmpMessageSent(Some(hash)));
				Ok(())
			}
			// Anything else is unhandled. This includes a message this is meant for us.
			_ => Err(XcmError::CannotReachDestination(dest, msg)),
		}
	}
}
