// Copyright 2023 Parity Technologies (UK) Ltd.
// This file is part of Parity Bridges Common.

// Parity Bridges Common is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Bridges Common is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Bridges Common.  If not, see <http://www.gnu.org/licenses/>.

//! Module provides utilities for easier XCM handling, e.g:
//! `XcmExecutor` -> `MessageSender` -> `OutboundMessageQueue`
//!                                             |
//!                                          `Relayer`
//!                                             |
//! `XcmRouter` <- `MessageDispatch` <- `InboundMessageQueue`

use bp_messages::{
	source_chain::{MessagesBridge, OnMessagesDelivered},
	target_chain::{DispatchMessage, MessageDispatch},
	LaneId, MessageNonce,
};
use bp_runtime::messages::MessageDispatchResult;
use codec::{Decode, Encode, FullCodec, MaxEncodedLen};
use frame_support::{
	dispatch::Weight,
	traits::{Get, ProcessMessage, ProcessMessageError, QueuePausedQuery},
	weights::WeightMeter,
	CloneNoBound, EqNoBound, PartialEqNoBound,
};
use pallet_bridge_messages::WeightInfoExt as MessagesPalletWeights;
use scale_info::TypeInfo;
use sp_io::hashing::blake2_256;
use sp_runtime::SaturatedConversion;
use sp_std::{boxed::Box, fmt::Debug, marker::PhantomData, vec::Vec};
use xcm::prelude::*;
use xcm_builder::{DispatchBlob, DispatchBlobError, HaulBlob, HaulBlobError};

/// Plain "XCM" payload, which we transfer through bridge
pub type XcmAsPlainPayload = sp_std::prelude::Vec<u8>;

/// Message dispatch result type for single message
#[derive(CloneNoBound, EqNoBound, PartialEqNoBound, Encode, Decode, Debug, TypeInfo)]
pub enum XcmBlobMessageDispatchResult {
	InvalidPayload,
	Dispatched,
	NotDispatched(#[codec(skip)] Option<DispatchBlobError>),
}

/// [`XcmBlobMessageDispatch`] is responsible for dispatching received messages
///
/// It needs to be used at the target bridge hub.
pub struct XcmBlobMessageDispatch<DispatchBlob, Weights, IsChannelActive> {
	_marker: sp_std::marker::PhantomData<(DispatchBlob, Weights, IsChannelActive)>,
}

impl<BlobDispatcher: DispatchBlob, Weights: MessagesPalletWeights, IsChannelActive: Get<bool>>
	MessageDispatch for XcmBlobMessageDispatch<BlobDispatcher, Weights, IsChannelActive>
{
	type DispatchPayload = XcmAsPlainPayload;
	type DispatchLevelResult = XcmBlobMessageDispatchResult;

	fn is_active() -> bool {
		// TODO: we can only implement `IsChannelActive` in Cumulus. Assuming that all messages
		// will only be sent to taget asset hub:
		//
		// ```rust
		// pub struct IsChannelWithAssetHubActive;
		//
		// impl Get<bool> for IsChannelWithAssetHubActive {
		//     fn get() -> bool {
		//         !cumulus_pallet_xcmp_queue::InboundXcmpSuspended::get().contains(&SIBLNG_ASSET_HUB_PARA_ID)
		//     }
		// }
		// ```
		let is_active = IsChannelActive::get();
		if !is_active {
			log::info!(target: "runtime::bridge-xcm-queues", "Target.BH -> TargetAH: overloaded. Failing delivery");
		} else {
			log::info!(target: "runtime::bridge-xcm-queues", "Target.BH -> TargetAH: not overloaded. Accepting delivery");
		}
		is_active
	}

	fn dispatch_weight(message: &mut DispatchMessage<Self::DispatchPayload>) -> Weight {
		match message.data.payload {
			Ok(ref payload) => {
				let payload_size = payload.encoded_size().saturated_into();
				Weights::message_dispatch_weight(payload_size)
			},
			Err(_) => Weight::zero(),
		}
	}

	fn dispatch(
		message: DispatchMessage<Self::DispatchPayload>,
	) -> MessageDispatchResult<Self::DispatchLevelResult> {
		let payload = match message.data.payload {
			Ok(payload) => payload,
			Err(e) => {
				log::error!(
					target: crate::LOG_TARGET_BRIDGE_DISPATCH,
					"[XcmBlobMessageDispatch] payload error: {:?} - message_nonce: {:?}",
					e,
					message.key.nonce
				);
				return MessageDispatchResult {
					unspent_weight: Weight::zero(),
					dispatch_level_result: XcmBlobMessageDispatchResult::InvalidPayload,
				}
			},
		};
		let dispatch_level_result = match BlobDispatcher::dispatch_blob(payload) {
			Ok(_) => {
				log::debug!(
					target: crate::LOG_TARGET_BRIDGE_DISPATCH,
					"[XcmBlobMessageDispatch] DispatchBlob::dispatch_blob was ok - message_nonce: {:?}",
					message.key.nonce
				);
				XcmBlobMessageDispatchResult::Dispatched
			},
			Err(e) => {
				log::error!(
					target: crate::LOG_TARGET_BRIDGE_DISPATCH,
					"[XcmBlobMessageDispatch] DispatchBlob::dispatch_blob failed, error: {:?} - message_nonce: {:?}",
					e, message.key.nonce
				);
				XcmBlobMessageDispatchResult::NotDispatched(Some(e))
			},
		};
		MessageDispatchResult { unspent_weight: Weight::zero(), dispatch_level_result }
	}
}

/// [`XcmBlobHauler`] is responsible for sending messages to the bridge "point-to-point link" from
/// one side, where on the other it can be dispatched by [`XcmBlobMessageDispatch`].
pub trait XcmBlobHauler {
	/// Runtime message sender adapter.
	type MessageSender: MessagesBridge<Self::MessageSenderOrigin, XcmAsPlainPayload>;

	/// Runtime message sender origin, which is used by [`Self::MessageSender`].
	type MessageSenderOrigin;
	/// Runtime origin for our (i.e. this bridge hub) location within the Consensus Universe.
	fn message_sender_origin() -> Self::MessageSenderOrigin;
	/// Location of the sending chain (i.e. sibling asset hub) within the Consensus universe.
	fn sending_chain_location() -> MultiLocation;
	/// Return message lane (as "point-to-point link") used to deliver XCM messages.
	fn xcm_lane() -> LaneId;
}

/// XCM bridge adapter which connects [`XcmBlobHauler`] with [`XcmBlobHauler::MessageSender`] and
/// makes sure that XCM blob is sent to the [`pallet_bridge_messages`] queue to be relayed.
///
/// It needs to be used at the source bridge hub.
pub struct XcmBlobHaulerAdapter<XcmBlobHauler>(sp_std::marker::PhantomData<XcmBlobHauler>);

impl<HaulerOrigin, H: XcmBlobHauler<MessageSenderOrigin = HaulerOrigin>> HaulBlob
	for XcmBlobHaulerAdapter<H>
{
	fn haul_blob(blob: sp_std::prelude::Vec<u8>) -> Result<(), HaulBlobError> {
		let lane = H::xcm_lane();
		H::MessageSender::send_message(H::message_sender_origin(), lane, blob)
			.map(|artifacts| {
				log::info!(
					target: crate::LOG_TARGET_BRIDGE_DISPATCH,
					"haul_blob result - ok: {:?} on lane: {:?}",
					artifacts.nonce,
					lane
				);

				// notify XCM queue manager about updated lane state
				LocalXcmQueueManager::on_bridge_message_enqueued(
					Box::new(H::sending_chain_location()),
					lane,
					artifacts.enqueued_messages,
				);
			})
			.map_err(|error| {
				log::error!(
					target: crate::LOG_TARGET_BRIDGE_DISPATCH,
					"haul_blob result - error: {:?} on lane: {:?}",
					error,
					lane
				);
				HaulBlobError::Transport("MessageSenderError")
			})
	}
}

impl<HaulerOrigin, H: XcmBlobHauler<MessageSenderOrigin = HaulerOrigin>> OnMessagesDelivered
	for XcmBlobHaulerAdapter<H>
{
	fn on_messages_delivered(lane: LaneId, enqueued_messages: MessageNonce) {
		// notify XCM queue manager about updated lane state
		LocalXcmQueueManager::on_bridge_messages_delivered(
			Box::new(H::sending_chain_location()),
			lane,
			enqueued_messages,
		);
	}
}

/// Manager of local XCM queues (and indirectly - underlying transport channels) that
/// controls the queue state.
///
/// It needs to be used at the source bridge hub.
pub struct LocalXcmQueueManager;

/// Prefix for storage keys, written by the `LocalXcmQueueManager`.
///
/// We don't have a separate pallet with available storage entries for managing XCM queues
/// in this (internediate) version of dynamic fees implementation. So we write to the runtime
/// storage directly with this prefix.
const LOCAL_XCM_QUEUE_MANAGER_STORAGE_PREFIX: &[u8] = b"LocalXcmQueueManager";

/// Name of "virtual" storage map that holds entries for every suspended queue.
const SUSPENDED_QUEUE_MAP_STORAGE_PREFIX: &[u8] = b"SuspendedQueues";

/// Maximal number of messages in the outbound bridge queue. Once we reach this limit, we
/// stop processing XCM messages from the sending chain (asset hub) that "owns" the lane.
// TODO: should be some factor of `MaxUnconfirmedMessagesAtInboundLane` at bridged side?
const MAX_ENQUEUED_MESSAGES_AT_OUTBOUND_LANE: MessageNonce = 300;

impl LocalXcmQueueManager {
	/// Must be called whenever we push a message to the bridge lane.
	pub fn on_bridge_message_enqueued(
		sending_chain_location: Box<MultiLocation>,
		lane: LaneId,
		enqueued_messages: MessageNonce,
	) {
		// suspend the inbound XCM queue with the sender to avoid queueing more messages
		// at the outbound bridge queue AND turn on internal backpressure mechanism of the
		// XCM queue
		let is_overloaded = enqueued_messages > MAX_ENQUEUED_MESSAGES_AT_OUTBOUND_LANE;
		if !is_overloaded {
			log::info!(target: "runtime::bridge-xcm-queues", "Source.BH -> Target.BH: message sent, not overloaded ({})", enqueued_messages);
			return
		}

		log::info!(
			target: crate::LOG_TARGET_BRIDGE_DISPATCH,
			"Suspending inbound XCM queue with {:?} to avoid overloading lane {:?}: there are\
			{} messages queued at the bridge queue",
			sending_chain_location,
			lane,
			enqueued_messages,
		);

		log::info!(target: "runtime::bridge-xcm-queues", "Source.BH -> Target.BH: message sent, overloaded ({}). Suspending Source.AH -> Source.BH queue", enqueued_messages);

		Self::suspend_inbound_queue(sending_chain_location);
	}

	/// Must be called whenever we receive a message delivery confirmation.
	pub fn on_bridge_messages_delivered(
		sending_chain_location: Box<MultiLocation>,
		lane: LaneId,
		enqueued_messages: MessageNonce,
	) {
		// the queue before this call may be either suspended or not. If the lane is still
		// overloaded, we win't need to do anything
		let is_overloaded = enqueued_messages > MAX_ENQUEUED_MESSAGES_AT_OUTBOUND_LANE;
		if is_overloaded {
			return
		}

		// else - resume the inbound queue
		if !Self::is_inbound_queue_suspended(sending_chain_location.clone()) {
			return
		}

		log::info!(target: "runtime::bridge-xcm-queues", "Source.BH -> Target.BH: not overloaded ({}). Resuming Source.AH -> Source.BH queue", enqueued_messages);

		log::info!(
			target: crate::LOG_TARGET_BRIDGE_DISPATCH,
			"Resuming inbound XCM queue with {:?} using lane {:?}: there are\
			{} messages queued at the bridge queue",
			sending_chain_location,
			lane,
			enqueued_messages,
		);

		Self::resume_inbound_queue(sending_chain_location);
	}

	/// Returns true if XCM message queue with given location is currently suspended.
	pub fn is_inbound_queue_suspended(with: Box<MultiLocation>) -> bool {
		frame_support::storage::unhashed::get_or_default(&Self::suspended_queue_map_storage_key(
			with,
		))
	}

	fn suspend_inbound_queue(with: Box<MultiLocation>) {
		frame_support::storage::unhashed::put(&Self::suspended_queue_map_storage_key(with), &true);
	}

	fn resume_inbound_queue(with: Box<MultiLocation>) {
		frame_support::storage::unhashed::kill(&Self::suspended_queue_map_storage_key(with));
	}

	fn suspended_queue_map_storage_key(with: Box<MultiLocation>) -> Vec<u8> {
		let with: VersionedMultiLocation = (*with).into();
		let with_hashed = with.using_encoded(blake2_256);

		// let's emulate real map here - it'd be easier to kill all entries later
		let mut final_key = Vec::with_capacity(
			LOCAL_XCM_QUEUE_MANAGER_STORAGE_PREFIX.len() +
				SUSPENDED_QUEUE_MAP_STORAGE_PREFIX.len() +
				with_hashed.len(),
		);

		final_key.extend_from_slice(LOCAL_XCM_QUEUE_MANAGER_STORAGE_PREFIX);
		final_key.extend_from_slice(SUSPENDED_QUEUE_MAP_STORAGE_PREFIX);
		final_key.extend_from_slice(&with_hashed);

		final_key
	}
}

/// A structure that implements [`frame_support:traits::messages::ProcessMessage`] and may
/// be used in the `pallet-message-queue` configuration to stop processing messages when the
/// bridge queue is overloaded.
///
/// It needs to be used at the source bridge hub.
pub struct LocalXcmQueueMessageProcessor<Origin, Inner>(PhantomData<(Origin, Inner)>);

impl<Origin, Inner> ProcessMessage for LocalXcmQueueMessageProcessor<Origin, Inner>
where
	Origin: Clone
		+ Into<MultiLocation>
		+ FullCodec
		+ MaxEncodedLen
		+ Clone
		+ Eq
		+ PartialEq
		+ TypeInfo
		+ Debug,
	Inner: ProcessMessage<Origin = Origin>,
{
	type Origin = Origin;

	fn process_message(
		message: &[u8],
		origin: Self::Origin,
		meter: &mut WeightMeter,
		id: &mut [u8; 32],
	) -> Result<bool, ProcessMessageError> {
		// if the queue is suspended, yield immediately
		if LocalXcmQueueManager::is_inbound_queue_suspended(Box::new(origin.clone().into())) {
			return Err(ProcessMessageError::Yield)
		}

		// else pass message to backed processor
		Inner::process_message(message, origin, meter, id)
	}
}

/// A structure that implements [`frame_support:traits::messages::QueuePausedQuery`] and may
/// be used in the `pallet-message-queue` configuration to stop processing messages when the
/// bridge queue is overloaded.
///
/// It needs to be used at the source bridge hub.
pub struct LocalXcmQueueSuspender<Origin, Inner>(PhantomData<(Origin, Inner)>);

impl<Origin, Inner> QueuePausedQuery<Origin> for LocalXcmQueueSuspender<Origin, Inner>
where
	Origin: Clone + Into<MultiLocation>,
	Inner: QueuePausedQuery<Origin>,
{
	fn is_paused(origin: &Origin) -> bool {
		// give priority to inner status
		if Inner::is_paused(origin) {
			return true
		}

		// if we have suspended the queue before, do not even start processing its messages
		if LocalXcmQueueManager::is_inbound_queue_suspended(Box::new(origin.clone().into())) {
			return true
		}

		// else process message
		false
	}
}
