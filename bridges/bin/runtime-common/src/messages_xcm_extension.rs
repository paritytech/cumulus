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
	source_chain::MessagesBridge,
	target_chain::{DispatchMessage, MessageDispatch},
	LaneId, MessageNonce,
};
use bp_runtime::{messages::MessageDispatchResult, RangeInclusiveExt};
use bp_xcm_bridge_hub_router::LocalXcmChannel;
use codec::{Decode, Encode, FullCodec, MaxEncodedLen};
use frame_support::{
	dispatch::Weight,
	traits::{Get, ProcessMessage, ProcessMessageError, QueuePausedQuery},
	weights::WeightMeter,
	CloneNoBound, EqNoBound, PartialEqNoBound,
};
use pallet_bridge_messages::{
	Config as MessagesConfig, Pallet as MessagesPallet, WeightInfoExt as MessagesPalletWeights,
};
use scale_info::TypeInfo;
use sp_runtime::SaturatedConversion;
use sp_std::{fmt::Debug, marker::PhantomData};
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
pub struct XcmBlobMessageDispatch<DispatchBlob, Weights, Channel> {
	_marker: sp_std::marker::PhantomData<(DispatchBlob, Weights, Channel)>,
}

impl<BlobDispatcher: DispatchBlob, Weights: MessagesPalletWeights, Channel: LocalXcmChannel>
	MessageDispatch for XcmBlobMessageDispatch<BlobDispatcher, Weights, Channel>
{
	type DispatchPayload = XcmAsPlainPayload;
	type DispatchLevelResult = XcmBlobMessageDispatchResult;

	fn is_active() -> bool {
		!Channel::is_congested()
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

/// A pair of sending chain location and message lane, used by this chain to send messages
/// over the bridge.
pub struct SenderAndLane {
	/// Sending chain relative location.
	pub location: MultiLocation,
	/// Message lane, used by the sending chain.
	pub lane: LaneId,
}

impl SenderAndLane {
	/// Create new object using provided location and lane.
	pub fn new(location: MultiLocation, lane: LaneId) -> Self {
		SenderAndLane { location, lane }
	}
}

/// [`XcmBlobHauler`] is responsible for sending messages to the bridge "point-to-point link" from
/// one side, where on the other it can be dispatched by [`XcmBlobMessageDispatch`].
pub trait XcmBlobHauler {
	/// Runtime message sender adapter.
	type MessageSender: MessagesBridge<Self::MessageSenderOrigin, XcmAsPlainPayload>;
	/// Returns lane used by this hauler.
	type SenderAndLane: Get<SenderAndLane>;

	/// Runtime message sender origin, which is used by [`Self::MessageSender`].
	type MessageSenderOrigin;
	/// Runtime origin for our (i.e. this bridge hub) location within the Consensus Universe.
	fn message_sender_origin() -> Self::MessageSenderOrigin;
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
		let lane = H::SenderAndLane::get().lane;
		H::MessageSender::send_message(H::message_sender_origin(), lane, blob)
			.map(|artifacts| {
				log::info!(
					target: crate::LOG_TARGET_BRIDGE_DISPATCH,
					"haul_blob result - ok: {:?} on lane: {:?}. Enqueued messages: {}",
					artifacts.nonce,
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

/// Manager of local XCM queues (and indirectly - underlying transport channels) that
/// controls the queue state.
///
/// It needs to be used at the source bridge hub.
pub struct LocalXcmQueueManager;

/// Maximal number of messages in the outbound bridge queue. Once we reach this limit, we
/// stop processing XCM messages from the sending chain (asset hub) that "owns" the lane.
///
/// The value is a maximal number of messages that can be delivered in a single message
/// delivery transaction, used on initial bridge hubs.
const MAX_ENQUEUED_MESSAGES_AT_OUTBOUND_LANE: MessageNonce = 4096;

impl LocalXcmQueueManager {
	/// Returns true if XCM message queue with given location is currently suspended.
	pub fn is_inbound_queue_suspended<R: MessagesConfig<MI>, MI: 'static>(lane: LaneId) -> bool {
		let outbound_lane = MessagesPallet::<R, MI>::outbound_lane_data(lane);
		let enqueued_messages = outbound_lane.queued_messages().checked_len().unwrap_or(0);
		enqueued_messages > MAX_ENQUEUED_MESSAGES_AT_OUTBOUND_LANE
	}
}

/// A structure that implements [`frame_support:traits::messages::ProcessMessage`] and may
/// be used in the `pallet-message-queue` configuration to stop processing messages when the
/// bridge queue is congested.
///
/// It needs to be used at the source bridge hub.
pub struct LocalXcmQueueMessageProcessor<Origin, Inner, R, MI, SL>(
	PhantomData<(Origin, Inner, R, MI, SL)>,
);

impl<Origin, Inner, R, MI, SL> ProcessMessage
	for LocalXcmQueueMessageProcessor<Origin, Inner, R, MI, SL>
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
	R: MessagesConfig<MI>,
	MI: 'static,
	SL: Get<SenderAndLane>,
{
	type Origin = Origin;

	fn process_message(
		message: &[u8],
		origin: Self::Origin,
		meter: &mut WeightMeter,
		id: &mut [u8; 32],
	) -> Result<bool, ProcessMessageError> {
		// if the queue is suspended, yield immediately
		let sender_and_lane = SL::get();
		if origin.clone().into() == sender_and_lane.location {
			if LocalXcmQueueManager::is_inbound_queue_suspended::<R, MI>(sender_and_lane.lane) {
				return Err(ProcessMessageError::Yield)
			}
		}

		// else pass message to backed processor
		Inner::process_message(message, origin, meter, id)
	}
}

/// A structure that implements [`frame_support:traits::messages::QueuePausedQuery`] and may
/// be used in the `pallet-message-queue` configuration to stop processing messages when the
/// bridge queue is congested.
///
/// It needs to be used at the source bridge hub.
pub struct LocalXcmQueueSuspender<Origin, Inner, R, MI, SL>(
	PhantomData<(Origin, Inner, R, MI, SL)>,
);

impl<Origin, Inner, R, MI, SL> QueuePausedQuery<Origin>
	for LocalXcmQueueSuspender<Origin, Inner, R, MI, SL>
where
	Origin: Clone + Into<MultiLocation>,
	Inner: QueuePausedQuery<Origin>,
	R: MessagesConfig<MI>,
	MI: 'static,
	SL: Get<SenderAndLane>,
{
	fn is_paused(origin: &Origin) -> bool {
		// give priority to inner status
		if Inner::is_paused(origin) {
			return true
		}

		// if we have suspended the queue before, do not even start processing its messages
		let sender_and_lane = SL::get();
		if origin.clone().into() == sender_and_lane.location {
			if LocalXcmQueueManager::is_inbound_queue_suspended::<R, MI>(sender_and_lane.lane) {
				return true
			}
		}

		// else process message
		false
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::*;

	use frame_support::parameter_types;
	use sp_runtime::traits::{ConstBool, Get};

	parameter_types! {
		pub TestSenderAndLane: SenderAndLane = SenderAndLane::new(Here.into(), TEST_LANE_ID);
	}

	fn test_origin_location() -> MultiLocation {
		TestSenderAndLane::get().location
	}

	fn test_origin() -> MultiLocation {
		test_origin_location()
	}

	struct TestXcmBlobHauler;
	impl XcmBlobHauler for TestXcmBlobHauler {
		type MessageSender = BridgeMessages;
		type MessageSenderOrigin = RuntimeOrigin;
		type SenderAndLane = TestSenderAndLane;

		fn message_sender_origin() -> Self::MessageSenderOrigin {
			RuntimeOrigin::root()
		}
	}

	struct TestInnerXcmQueueMessageProcessor;
	impl ProcessMessage for TestInnerXcmQueueMessageProcessor {
		type Origin = MultiLocation;

		fn process_message(
			_message: &[u8],
			_origin: Self::Origin,
			_meter: &mut WeightMeter,
			_id: &mut [u8; 32],
		) -> Result<bool, ProcessMessageError> {
			Ok(true)
		}
	}

	struct TestInnerXcmQueueSuspender<IsSuspended>(PhantomData<IsSuspended>);
	impl<IsSuspended: Get<bool>> QueuePausedQuery<MultiLocation>
		for TestInnerXcmQueueSuspender<IsSuspended>
	{
		fn is_paused(_: &MultiLocation) -> bool {
			IsSuspended::get()
		}
	}

	type TestXcmBlobHaulerAdapter = XcmBlobHaulerAdapter<TestXcmBlobHauler>;
	type TestLocalXcmQueueMessageProcessor = LocalXcmQueueMessageProcessor<
		MultiLocation,
		TestInnerXcmQueueMessageProcessor,
		TestRuntime,
		(),
		TestSenderAndLane,
	>;
	type TestLocalXcmQueueSuspender = LocalXcmQueueSuspender<
		MultiLocation,
		TestInnerXcmQueueSuspender<ConstBool<false>>,
		TestRuntime,
		(),
		TestSenderAndLane,
	>;

	#[test]
	fn inbound_xcm_queue_with_sending_chain_is_managed_by_blob_hauler() {
		run_test(|| {
			// while we enqueue `MAX_ENQUEUED_MESSAGES_AT_OUTBOUND_LANE` messages to the bridge
			// queue, the inbound channel with the sending chain stays opened
			for _ in 0..MAX_ENQUEUED_MESSAGES_AT_OUTBOUND_LANE {
				TestXcmBlobHaulerAdapter::haul_blob(vec![42]).unwrap();
				assert!(!LocalXcmQueueManager::is_inbound_queue_suspended::<TestRuntime, ()>(
					TEST_LANE_ID
				));
			}

			// then when we enqueue more messages, we suspend inbound queue. Note that messages
			// are not dropped - they're enqueued at the bridge queue
			TestXcmBlobHaulerAdapter::haul_blob(vec![42]).unwrap();
			assert!(LocalXcmQueueManager::is_inbound_queue_suspended::<TestRuntime, ()>(
				TEST_LANE_ID
			));
		});
	}

	#[test]
	fn inbound_xcm_message_from_sibling_is_not_processed_when_bridge_queue_is_congested() {
		run_test(|| {
			for _ in 0..MAX_ENQUEUED_MESSAGES_AT_OUTBOUND_LANE + 1 {
				TestXcmBlobHaulerAdapter::haul_blob(vec![42]).unwrap();
			}

			assert_eq!(
				TestLocalXcmQueueMessageProcessor::process_message(
					&[42],
					test_origin(),
					&mut WeightMeter::max_limit(),
					&mut [0u8; 32],
				),
				Err(ProcessMessageError::Yield),
			);
		})
	}

	#[test]
	fn inbound_xcm_message_from_other_origin_is_processed_normally() {
		run_test(|| {
			for _ in 0..MAX_ENQUEUED_MESSAGES_AT_OUTBOUND_LANE + 1 {
				TestXcmBlobHaulerAdapter::haul_blob(vec![42]).unwrap();
			}

			assert_eq!(
				TestLocalXcmQueueMessageProcessor::process_message(
					&[42],
					ParentThen(X1(Parachain(1000))).into(),
					&mut WeightMeter::max_limit(),
					&mut [0u8; 32],
				),
				Ok(true),
			);
		})
	}

	#[test]
	fn inbound_xcm_message_from_sibling_is_processed_normally() {
		run_test(|| {
			assert_eq!(
				TestLocalXcmQueueMessageProcessor::process_message(
					&[42],
					test_origin(),
					&mut WeightMeter::max_limit(),
					&mut [0u8; 32],
				),
				Ok(true),
			);
		})
	}

	#[test]
	fn local_xcm_queue_is_paused_when_inner_suspender_returns_paused() {
		run_test(|| {
			assert!(LocalXcmQueueSuspender::<
				MultiLocation,
				TestInnerXcmQueueSuspender<ConstBool<true>>,
				TestRuntime,
				(),
				TestSenderAndLane,
			>::is_paused(&test_origin()))
		})
	}

	#[test]
	fn local_xcm_queue_is_paused_when_bridge_queue_is_congested() {
		run_test(|| {
			for _ in 0..MAX_ENQUEUED_MESSAGES_AT_OUTBOUND_LANE + 1 {
				TestXcmBlobHaulerAdapter::haul_blob(vec![42]).unwrap();
			}

			assert!(TestLocalXcmQueueSuspender::is_paused(&test_origin()))
		});
	}

	#[test]
	fn local_xcm_queue_with_other_origin_is_not_paused() {
		run_test(|| {
			assert!(!TestLocalXcmQueueSuspender::is_paused(&ParentThen(X1(Parachain(1000))).into()))
		});
	}

	#[test]
	fn local_xcm_queue_is_not_paused_normally() {
		run_test(|| assert!(!TestLocalXcmQueueSuspender::is_paused(&test_origin())));
	}
}
