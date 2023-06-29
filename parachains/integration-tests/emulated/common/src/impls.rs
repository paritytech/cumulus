// use std::marker::PhantomData;
use codec::{Encode, Decode};
use xcm_emulator::{BridgeMessage, BridgeMessageDispatchError, BridgeMessageHandler, NetworkComponent, Parachain};
use pallet_bridge_messages::{Config, Instance1, Instance2, OutboundLanes, Pallet};
use bp_messages::{LaneId, MessageKey, OutboundLaneData, target_chain::{DispatchMessage, MessageDispatch, DispatchMessageData}};
use bridge_runtime_common::messages_xcm_extension::XcmBlobMessageDispatchResult;
pub use cumulus_primitives_core::{DmpMessageHandler, XcmpMessageHandler};
use sp_core::Get;
use super::{BridgeHubRococo, BridgeHubWococo, BridgeHubPolkadot, BridgeHubKusama};

// pub struct AssetHubMessageHandler<S, T, I>(PhantomData<S>, PhantomData<T>, PhantomData<I>);
pub struct AssetHubMessageHandler<S, T, I> {
	_marker: std::marker::PhantomData<(S, T, I)>
}

type BridgeHubRococoRuntime = <BridgeHubRococo as Parachain>::Runtime;
type BridgeHubWococoRuntime = <BridgeHubWococo as Parachain>::Runtime;

// TODO: uncomment when https://github.com/paritytech/cumulus/pull/2528 is merged
type BridgeHubPolkadotRuntime = <BridgeHubPolkadot as Parachain>::Runtime;
type BridgeHubKusamaRuntime = <BridgeHubKusama as Parachain>::Runtime;

pub type RococoWococoMessageHandler
	= AssetHubMessageHandler<BridgeHubRococoRuntime, BridgeHubWococoRuntime, Instance2>;
pub type WococoRococoMessageHandler
	= AssetHubMessageHandler<BridgeHubWococoRuntime, BridgeHubRococoRuntime, Instance2>;

// TODO: uncomment when https://github.com/paritytech/cumulus/pull/2528 is merged
// pub type PolkadotKusamaMessageHandler
//	= AssetHubMessageHandler<BridgeHubPolkadotRuntime, BridgeHubKusamaRuntime, Instance1>;
// pub type KusamaPolkadotMessageHandler
//	= AssetHubMessageHandler<BridgeHubKusamaRuntime, BridgeHubPolkadoRuntime, Instance1>;


impl <S, T, I> BridgeMessageHandler for AssetHubMessageHandler<S, T, I>
where
	S: Config<Instance1>,
	T: Config<I>,
	I: 'static,
{
	fn get_source_outbound_messages() -> Vec<BridgeMessage> {
		// get the source active outbound lanes
		let active_lanes = S::ActiveOutboundLanes::get();

		let mut messages: Vec<BridgeMessage> = Default::default();

		// collect messages from `OutboundMessages` for each active outbound lane in the source
		for lane in active_lanes {
			let latest_generated_nonce = OutboundLanes::<S, Instance1>::get(lane).latest_generated_nonce;
			let latest_received_nonce = OutboundLanes::<S, Instance1>::get(lane).latest_received_nonce;

			(latest_received_nonce + 1..=latest_generated_nonce).for_each(|nonce| {
				let mut encoded_payload: Vec<u8> = Pallet::<S, Instance1>::outbound_message_data(*lane, nonce).expect("Bridge message does not exist").into();
				let payload = Vec::<u8>::decode(&mut &encoded_payload[..]).expect("Decodign XCM message failed");
				let id: u32 = (*lane).into();
				let message = BridgeMessage { id, nonce, payload };

				messages.push(message);
			});
		}
		messages
	}

	fn dispatch_target_inbound_message(message: BridgeMessage) -> Result<(), BridgeMessageDispatchError> {
		type TargetMessageDispatch<T, I> = <T as Config<I>>::MessageDispatch;
		type InboundPayload<T, I> = <T as Config<I>>::InboundPayload;

		let lane_id = message.id.into();
		let nonce = message.nonce;
		let payload = Ok(message.payload);

		// Directly dispatch outbound messages assuming everything is correct
		// and bypassing the `InboundLane` logic
		let dispatch_result = TargetMessageDispatch::<T, I>::dispatch(DispatchMessage {
			key: MessageKey { lane_id, nonce },
			// data: DispatchMessageData::<InboundPayload<T, I>> { payload },
			data: DispatchMessageData::<InboundPayload<T, I>> { payload },
		});

		let result = match dispatch_result.dispatch_level_result {
			XcmBlobMessageDispatchResult::Dispatched => {
				Ok(())
			},
			XcmBlobMessageDispatchResult::InvalidPayload => {
				Err(BridgeMessageDispatchError(Box::new(XcmBlobMessageDispatchResult::InvalidPayload)))
			},
			XcmBlobMessageDispatchResult::NotDispatched(e) => {
				Err(BridgeMessageDispatchError(Box::new(XcmBlobMessageDispatchResult::NotDispatched(e))))
			},
			_ => Err(BridgeMessageDispatchError(Box::new("Unknown Error"))),
		};

		// result
		Ok(())
	}

	fn notify_source_message_delivery(lane_id: u32) {
		let data = OutboundLanes::<S, Instance1>::get(LaneId::from(lane_id));
		let new_data = OutboundLaneData {
			oldest_unpruned_nonce: data.oldest_unpruned_nonce + 1,
			latest_received_nonce: data.latest_received_nonce + 1,
			..data
		};

		OutboundLanes::<S, Instance1>::insert(LaneId::from(lane_id), new_data);
	}
}
