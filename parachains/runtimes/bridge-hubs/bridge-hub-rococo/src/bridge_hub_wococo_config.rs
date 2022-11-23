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

use crate::{
	BridgeParachainRococoInstance, ParachainInfo, Runtime, WithBridgeHubRococoMessagesInstance,
	XcmAsPlainPayload, XcmBlobHauler, XcmBlobHaulerAdapter, XcmRouter,
};
use bp_messages::{
	source_chain::TargetHeaderChain,
	target_chain::{ProvedMessages, SourceHeaderChain},
	InboundLaneData, LaneId, Message, MessageNonce,
};
use bp_runtime::ChainId;
use bridge_runtime_common::{
	messages,
	messages::{
		target::FromBridgedChainMessagesProof, MessageBridge, ThisChainWithMessages,
		UnderlyingChainProvider,
	},
};
use frame_support::{parameter_types, RuntimeDebug};
use xcm::{
	latest::prelude::*,
	prelude::{InteriorMultiLocation, NetworkId},
};
use xcm_builder::{BridgeBlobDispatcher, HaulBlobExporter};

// TODO:check-parameter
parameter_types! {
	pub const MaxUnrewardedRelayerEntriesAtInboundLane: bp_messages::MessageNonce =
		bp_bridge_hub_wococo::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	pub const MaxUnconfirmedMessagesAtInboundLane: bp_messages::MessageNonce =
		bp_bridge_hub_wococo::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
	pub const BridgeHubRococoChainId: bp_runtime::ChainId = bp_runtime::BRIDGE_HUB_ROCOCO_CHAIN_ID;
	pub BridgeHubWococoUniversalLocation: InteriorMultiLocation = X2(GlobalConsensus(Wococo), Parachain(ParachainInfo::parachain_id().into()));
	pub RococoGlobalConsensusNetwork: NetworkId = NetworkId::Rococo;
	pub ActiveOutboundLanesToBridgeHubRococo: &'static [bp_messages::LaneId] = &[DEFAULT_XCM_LANE_TO_BRIDGE_HUB_ROCOCO];
}

/// Dispatches received XCM messages from other bridge
pub type OnBridgeHubWococoBlobDispatcher =
	BridgeBlobDispatcher<XcmRouter, BridgeHubWococoUniversalLocation>;

/// Export XCM messages to be relayed to the otherside
pub type ToBridgeHubRococoHaulBlobExporter = HaulBlobExporter<
	XcmBlobHaulerAdapter<ToBridgeHubRococoXcmBlobHauler>,
	RococoGlobalConsensusNetwork,
	(),
>;
pub struct ToBridgeHubRococoXcmBlobHauler;
pub const DEFAULT_XCM_LANE_TO_BRIDGE_HUB_ROCOCO: LaneId = [0, 0, 0, 1];
impl XcmBlobHauler for ToBridgeHubRococoXcmBlobHauler {
	type MessageSender =
		pallet_bridge_messages::Pallet<Runtime, WithBridgeHubRococoMessagesInstance>;

	fn message_sender_origin() -> InteriorMultiLocation {
		crate::xcm_config::UniversalLocation::get()
	}

	fn xcm_lane() -> LaneId {
		DEFAULT_XCM_LANE_TO_BRIDGE_HUB_ROCOCO
	}
}

/// Messaging Bridge configuration for BridgeHubWococo -> BridgeHubRococo
pub struct WithBridgeHubRococoMessageBridge;
impl MessageBridge for WithBridgeHubRococoMessageBridge {
	const THIS_CHAIN_ID: ChainId = bp_runtime::BRIDGE_HUB_WOCOCO_CHAIN_ID;
	const BRIDGED_CHAIN_ID: ChainId = bp_runtime::BRIDGE_HUB_ROCOCO_CHAIN_ID;
	const BRIDGED_MESSAGES_PALLET_NAME: &'static str =
		bp_bridge_hub_wococo::WITH_BRIDGE_HUB_WOCOCO_MESSAGES_PALLET_NAME;
	type ThisChain = BridgeHubWococo;
	type BridgedChain = BridgeHubRococo;
	type BridgedHeaderChain = pallet_bridge_parachains::ParachainHeaders<
		Runtime,
		BridgeParachainRococoInstance,
		bp_bridge_hub_rococo::BridgeHubRococo,
	>;
}

/// Message verifier for BridgeHubRococo messages sent from BridgeHubWococo
pub type ToBridgeHubRococoMessageVerifier =
	messages::source::FromThisChainMessageVerifier<WithBridgeHubRococoMessageBridge>;

/// Maximal outbound payload size of BridgeHubWococo -> BridgeHubRococo messages.
pub type ToBridgeHubRococoMaximalOutboundPayloadSize =
	messages::source::FromThisChainMaximalOutboundPayloadSize<WithBridgeHubRococoMessageBridge>;

/// BridgeHubRococo chain from message lane point of view.
#[derive(RuntimeDebug, Clone, Copy)]
pub struct BridgeHubRococo;

impl UnderlyingChainProvider for BridgeHubRococo {
	type Chain = bp_bridge_hub_rococo::BridgeHubRococo;
}

impl SourceHeaderChain for BridgeHubRococo {
	type Error = &'static str;
	type MessagesProof = FromBridgedChainMessagesProof<crate::Hash>;

	fn verify_messages_proof(
		proof: Self::MessagesProof,
		messages_count: u32,
	) -> Result<ProvedMessages<Message>, Self::Error> {
		bridge_runtime_common::messages::target::verify_messages_proof::<
			WithBridgeHubRococoMessageBridge,
		>(proof, messages_count)
		.map_err(Into::into)
	}
}

impl TargetHeaderChain<XcmAsPlainPayload, crate::AccountId> for BridgeHubRococo {
	type Error = &'static str;
	type MessagesDeliveryProof =
		messages::source::FromBridgedChainMessagesDeliveryProof<bp_bridge_hub_rococo::Hash>;

	fn verify_message(payload: &XcmAsPlainPayload) -> Result<(), Self::Error> {
		messages::source::verify_chain_message::<WithBridgeHubRococoMessageBridge>(payload)
	}

	fn verify_messages_delivery_proof(
		proof: Self::MessagesDeliveryProof,
	) -> Result<(LaneId, InboundLaneData<bp_bridge_hub_wococo::AccountId>), Self::Error> {
		messages::source::verify_messages_delivery_proof::<WithBridgeHubRococoMessageBridge>(proof)
	}
}

impl messages::BridgedChainWithMessages for BridgeHubRococo {
	fn verify_dispatch_weight(_message_payload: &[u8]) -> bool {
		true
	}
}

/// BridgeHubWococo chain from message lane point of view.
#[derive(RuntimeDebug, Clone, Copy)]
pub struct BridgeHubWococo;

impl UnderlyingChainProvider for BridgeHubWococo {
	type Chain = bp_bridge_hub_wococo::BridgeHubWococo;
}

impl ThisChainWithMessages for BridgeHubWococo {
	type RuntimeOrigin = crate::RuntimeOrigin;
	type RuntimeCall = crate::RuntimeCall;

	fn is_message_accepted(origin: &Self::RuntimeOrigin, lane: &LaneId) -> bool {
		log::info!(target: crate::LOG_TARGET, "[BridgeHubWococo::ThisChainWithMessages] is_message_accepted - origin: {:?}, lane: {:?}", origin, lane);
		lane == &DEFAULT_XCM_LANE_TO_BRIDGE_HUB_ROCOCO
	}

	fn maximal_pending_messages_at_outbound_lane() -> MessageNonce {
		log::info!(
			target: crate::LOG_TARGET,
			"[BridgeHubWococo::ThisChainWithMessages] maximal_pending_messages_at_outbound_lane"
		);
		MessageNonce::MAX / 2
	}
}
