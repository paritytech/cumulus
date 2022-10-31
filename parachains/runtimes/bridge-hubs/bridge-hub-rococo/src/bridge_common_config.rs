// Copyright 2022 Parity Technologies (UK) Ltd.
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

use crate::universal_exports::HaulBlob;
use bp_messages::{
	source_chain::MessagesBridge,
	target_chain::{DispatchMessage, MessageDispatch},
	LaneId,
};
use bp_runtime::{messages::MessageDispatchResult, AccountIdOf, BalanceOf, Chain};
use codec::Encode;
use frame_support::{dispatch::Weight, parameter_types};
use xcm::latest::prelude::*;

// TODO:check-parameter - we could possibly use BridgeMessage from xcm:v3 stuff
/// PLain "XCM" payload, which we transfer through bridge
pub type XcmAsPlainPayload = sp_std::prelude::Vec<u8>;

// TODO:check-parameter
parameter_types! {
	pub const MaxMessagesToPruneAtOnce: bp_messages::MessageNonce = 8;
	pub const MaxRequests: u32 = 64;
	pub const HeadersToKeep: u32 = 1024;
}

/// [`XcmBlobMessageDispatch`] is responsible for dispatching received messages from other BridgeHub
pub struct XcmBlobMessageDispatch<SourceBridgeHubChain, TargetBridgeHubChain, DispatchBlob> {
	_marker:
		sp_std::marker::PhantomData<(SourceBridgeHubChain, TargetBridgeHubChain, DispatchBlob)>,
}

impl<
		SourceBridgeHubChain: Chain,
		TargetBridgeHubChain: Chain,
		DispatchBlob: crate::universal_exports::DispatchBlob,
	> MessageDispatch<AccountIdOf<SourceBridgeHubChain>, BalanceOf<TargetBridgeHubChain>>
	for XcmBlobMessageDispatch<SourceBridgeHubChain, TargetBridgeHubChain, DispatchBlob>
{
	type DispatchPayload = XcmAsPlainPayload;

	fn dispatch_weight(
		message: &mut DispatchMessage<Self::DispatchPayload, BalanceOf<TargetBridgeHubChain>>,
	) -> Weight {
		log::error!(
			"[XcmBlobMessageDispatch] TODO: change here to XCMv3 dispatch_weight with XcmExecutor - message: ?...?",
		);
		0
	}

	fn dispatch(
		_relayer_account: &AccountIdOf<SourceBridgeHubChain>,
		message: DispatchMessage<Self::DispatchPayload, BalanceOf<TargetBridgeHubChain>>,
	) -> MessageDispatchResult {
		log::warn!("[XcmBlobMessageDispatch] DispatchBlob::dispatch_blob triggering");
		let payload = match message.data.payload {
			Ok(payload) => payload,
			Err(e) => {
				log::error!("[XcmBlobMessageDispatch] payload error: {:?}", e);
				return MessageDispatchResult {
					dispatch_result: false,
					unspent_weight: 0,
					dispatch_fee_paid_during_dispatch: false,
				}
			},
		};
		let dispatch_result = match DispatchBlob::dispatch_blob(payload) {
			Ok(_) => true,
			Err(e) => {
				log::error!(
					"[XcmBlobMessageDispatch] DispatchBlob::dispatch_blob failed, error: {:?}",
					e
				);
				false
			},
		};
		MessageDispatchResult {
			dispatch_result,
			dispatch_fee_paid_during_dispatch: false,
			unspent_weight: 0,
		}
	}
}

/// [`XcmBlobHauler`] is responsible for sending messages to the bridge "point-to-point link" from one side,
/// where on the other it can be dispatched by [`XcmBlobMessageDispatch`].
pub trait XcmBlobHauler {
	/// Which chain is sending
	type SenderChain: Chain;

	/// Runtime message sender adapter.
	type MessageSender: MessagesBridge<
		super::Origin,
		AccountIdOf<Self::SenderChain>,
		BalanceOf<Self::SenderChain>,
		XcmAsPlainPayload,
	>;

	/// Our location within the Consensus Universe.
	fn message_sender_origin() -> InteriorMultiLocation;

	/// Return message lane (as "point-to-point link") used to deliver XCM messages.
	fn xcm_lane() -> LaneId;
}

impl<T: XcmBlobHauler> HaulBlob for T {
	fn haul_blob(blob: sp_std::prelude::Vec<u8>) {
		let lane = T::xcm_lane();
		// TODO:check-parameter - fee could be taken from BridgeMessage - or add as optional fo send_message
		// TODO:check-parameter - or add here something like PriceForSiblingDelivery
		let fee = <T::SenderChain as Chain>::Balance::from(0u8);

		let result = T::MessageSender::send_message(
			pallet_xcm::Origin::from(MultiLocation::from(T::message_sender_origin())).into(),
			lane,
			blob,
			fee,
		);
		let result = result
			.map(|artifacts| {
				let hash = (lane, artifacts.nonce).using_encoded(sp_io::hashing::blake2_256);
				hash
			})
			.map_err(|e| {
				e
			});
		log::info!(target: "runtime::bridge-hub", "haul_blob result: {:?} on lane: {:?}", result, lane);
		result.expect("failed to process: TODO:check-parameter - wait for origin/gav-xcm-v3, there is a comment about handliing errors for HaulBlob");
	}
}
