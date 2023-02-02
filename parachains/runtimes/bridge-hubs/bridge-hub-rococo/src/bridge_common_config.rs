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

use bp_messages::{
	source_chain::MessagesBridge,
	target_chain::{DispatchMessage, MessageDispatch},
	LaneId,
};
use bp_runtime::{messages::MessageDispatchResult, AccountIdOf, Chain};
use codec::{Decode, Encode};
use frame_support::{
	dispatch::Weight, ensure, traits::Get, CloneNoBound, EqNoBound, PartialEqNoBound,
};
use xcm_builder::{DispatchBlob, DispatchBlobError, HaulBlob, HaulBlobError};

use rustc_hex::ToHexIter;

use scale_info::{
	prelude::{format, string::String},
	TypeInfo,
};

use sp_std::{marker::PhantomData, vec::Vec};

use xcm::{latest::prelude::*, prelude::*};

pub fn bytes_to_hex(bytes: &[u8]) -> String {
	let iter = ToHexIter::new(bytes.iter());
	format!("0x{}", iter.collect::<String>())
}

const LOG_TARGET: &str = "bridge-hub-debug";

/// PLain "XCM" payload, which we transfer through bridge
pub type XcmAsPlainPayload = sp_std::prelude::Vec<u8>;

/// Message dispatch result type for single message
#[derive(CloneNoBound, EqNoBound, PartialEqNoBound, Encode, Decode, Debug, TypeInfo)]
pub enum XcmBlobMessageDispatchResult {
	InvalidPayload,
	Dispatched,
	NotDispatched(#[codec(skip)] &'static str),
}

#[derive(Clone, Encode, Decode)]
pub struct BridgedMessage {
	/// The message destination as a *Universal Location*. This means it begins with a
	/// `GlobalConsensus` junction describing the network under which global consensus happens.
	/// If this does not match our global consensus then it's a fatal error.
	universal_dest: VersionedInteriorMultiLocation,
	message: VersionedXcm<()>,
}

pub struct DebugBridgeBlobDispatcher<Router, OurPlace>(PhantomData<(Router, OurPlace)>);
impl<Router: SendXcm, OurPlace: Get<InteriorMultiLocation>> DispatchBlob
	for DebugBridgeBlobDispatcher<Router, OurPlace>
{
	fn dispatch_blob(blob: Vec<u8>) -> Result<(), DispatchBlobError> {
		let our_universal = OurPlace::get();
		let our_global =
			our_universal.global_consensus().map_err(|()| DispatchBlobError::Unbridgable)?;
		log::error!(target: LOG_TARGET, "Encoded message: {:?}", bytes_to_hex(&blob));
		log::error!(target: LOG_TARGET, "Our global: {:?}", our_global);
		let BridgedMessage { universal_dest, message } =
			Decode::decode(&mut &blob[..]).map_err(|_| DispatchBlobError::InvalidEncoding)?;
		log::error!(
			target: LOG_TARGET,
			"Universal dest: {:?}, message: {:?}",
			universal_dest,
			message
		);
		let universal_dest: InteriorMultiLocation = universal_dest
			.try_into()
			.map_err(|_| DispatchBlobError::UnsupportedLocationVersion)?;
		// `universal_dest` is the desired destination within the universe: first we need to check
		// we're in the right global consensus.
		let intended_global = universal_dest
			.global_consensus()
			.map_err(|()| DispatchBlobError::NonUniversalDestination)?;
		ensure!(intended_global == our_global, DispatchBlobError::WrongGlobal);

		let dest = universal_dest.relative_to(&our_universal);
		log::error!(target: LOG_TARGET, "Relative dest: {:?}", dest);

		let message: Xcm<()> =
			message.try_into().map_err(|_| DispatchBlobError::UnsupportedXcmVersion)?;
		log::error!(target: LOG_TARGET, "XCM message: {:?}", message);

		send_xcm::<Router>(dest, message).map_err(|_| DispatchBlobError::RoutingError)?;
		Ok(())
	}
}

/// [`XcmBlobMessageDispatch`] is responsible for dispatching received messages from other BridgeHub
pub struct XcmBlobMessageDispatch<SourceBridgeHubChain, TargetBridgeHubChain, DispatchBlob> {
	_marker:
		sp_std::marker::PhantomData<(SourceBridgeHubChain, TargetBridgeHubChain, DispatchBlob)>,
}

impl<SourceBridgeHubChain: Chain, TargetBridgeHubChain: Chain, BlobDispatcher: DispatchBlob>
	MessageDispatch<AccountIdOf<SourceBridgeHubChain>>
	for XcmBlobMessageDispatch<SourceBridgeHubChain, TargetBridgeHubChain, BlobDispatcher>
{
	type DispatchPayload = XcmAsPlainPayload;
	type DispatchLevelResult = XcmBlobMessageDispatchResult;

	fn dispatch_weight(_message: &mut DispatchMessage<Self::DispatchPayload>) -> Weight {
		log::error!(
			target: crate::LOG_TARGET,
			"[XcmBlobMessageDispatch] TODO: change here to XCMv3 dispatch_weight with XcmExecutor - message: ?...?",
		);
		// TODO:check-parameter - setup weight?
		Weight::zero()
	}

	fn dispatch(
		_relayer_account: &AccountIdOf<SourceBridgeHubChain>,
		message: DispatchMessage<Self::DispatchPayload>,
	) -> MessageDispatchResult<Self::DispatchLevelResult> {
		let payload = match message.data.payload {
			Ok(payload) => payload,
			Err(e) => {
				log::error!(
					target: crate::LOG_TARGET,
					"[XcmBlobMessageDispatch] payload error: {:?} - message_nonce: {:?}",
					e,
					message.key.nonce
				);
				return MessageDispatchResult {
					// TODO:check-parameter - setup uspent_weight?
					unspent_weight: Weight::zero(),
					dispatch_level_result: XcmBlobMessageDispatchResult::InvalidPayload,
				}
			},
		};
		let dispatch_level_result = match BlobDispatcher::dispatch_blob(payload) {
			Ok(_) => {
				log::debug!(
					target: crate::LOG_TARGET,
					"[XcmBlobMessageDispatch] DispatchBlob::dispatch_blob was ok - message_nonce: {:?}",
					message.key.nonce
				);
				XcmBlobMessageDispatchResult::Dispatched
			},
			Err(e) => {
				let e = match e {
					DispatchBlobError::Unbridgable => "DispatchBlobError::Unbridgable",
					DispatchBlobError::InvalidEncoding => "DispatchBlobError::InvalidEncoding",
					DispatchBlobError::UnsupportedLocationVersion =>
						"DispatchBlobError::UnsupportedLocationVersion",
					DispatchBlobError::UnsupportedXcmVersion =>
						"DispatchBlobError::UnsupportedXcmVersion",
					DispatchBlobError::RoutingError => "DispatchBlobError::RoutingError",
					DispatchBlobError::NonUniversalDestination =>
						"DispatchBlobError::NonUniversalDestination",
					DispatchBlobError::WrongGlobal => "DispatchBlobError::WrongGlobal",
				};
				log::error!(
					target: crate::LOG_TARGET,
					"[XcmBlobMessageDispatch] DispatchBlob::dispatch_blob failed, error: {:?} - message_nonce: {:?}",
					e, message.key.nonce
				);
				XcmBlobMessageDispatchResult::NotDispatched(e)
			},
		};
		MessageDispatchResult {
			// TODO:check-parameter - setup uspent_weight?
			unspent_weight: Weight::zero(),
			dispatch_level_result,
		}
	}
}

/// [`XcmBlobHauler`] is responsible for sending messages to the bridge "point-to-point link" from one side,
/// where on the other it can be dispatched by [`XcmBlobMessageDispatch`].
pub trait XcmBlobHauler {
	/// Runtime message sender adapter.
	type MessageSender: MessagesBridge<Self::MessageSenderOrigin, XcmAsPlainPayload>;

	/// Runtime message sender origin, which is used by MessageSender.
	type MessageSenderOrigin;
	/// Our location within the Consensus Universe.
	fn message_sender_origin() -> Self::MessageSenderOrigin;

	/// Return message lane (as "point-to-point link") used to deliver XCM messages.
	fn xcm_lane() -> LaneId;
}

pub struct XcmBlobHaulerAdapter<XcmBlobHauler>(sp_std::marker::PhantomData<XcmBlobHauler>);
impl<HaulerOrigin, H: XcmBlobHauler<MessageSenderOrigin = HaulerOrigin>> HaulBlob
	for XcmBlobHaulerAdapter<H>
{
	fn haul_blob(blob: sp_std::prelude::Vec<u8>) -> Result<(), HaulBlobError> {
		let lane = H::xcm_lane();
		let result = H::MessageSender::send_message(H::message_sender_origin(), lane, blob);
		let result = result.map(|artifacts| {
			let hash = (lane, artifacts.nonce).using_encoded(sp_io::hashing::blake2_256);
			hash
		});
		match &result {
			Ok(result) => log::info!(
				target: crate::LOG_TARGET,
				"haul_blob result - ok: {:?} on lane: {:?}",
				result,
				lane
			),
			Err(error) => log::error!(
				target: crate::LOG_TARGET,
				"haul_blob result - error: {:?} on lane: {:?}",
				error,
				lane
			),
		};
		result.map(|_| ()).map_err(|_| HaulBlobError::Transport("MessageSenderError"))
	}
}
