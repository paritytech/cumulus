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
	BridgeParachainWococoInstance, ParachainInfo, Runtime, WithBridgeHubWococoMessagesInstance,
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
		bp_bridge_hub_rococo::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	pub const MaxUnconfirmedMessagesAtInboundLane: bp_messages::MessageNonce =
		bp_bridge_hub_rococo::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
	pub const BridgeHubWococoChainId: bp_runtime::ChainId = bp_runtime::BRIDGE_HUB_WOCOCO_CHAIN_ID;
	pub BridgeHubRococoUniversalLocation: InteriorMultiLocation = X2(GlobalConsensus(Rococo), Parachain(ParachainInfo::parachain_id().into()));
	pub WococoGlobalConsensusNetwork: NetworkId = NetworkId::Wococo;
	pub ActiveOutboundLanesToBridgeHubWococo: &'static [bp_messages::LaneId] = &[DEFAULT_XCM_LANE_TO_BRIDGE_HUB_WOCOCO];
}

/// Dispatches received XCM messages from other bridge
pub type OnBridgeHubRococoBlobDispatcher =
	BridgeBlobDispatcher<XcmRouter, BridgeHubRococoUniversalLocation>;

/// Export XCM messages to be relayed to the otherside
pub type ToBridgeHubWococoHaulBlobExporter = HaulBlobExporter<
	XcmBlobHaulerAdapter<ToBridgeHubWococoXcmBlobHauler>,
	WococoGlobalConsensusNetwork,
	(),
>;
pub struct ToBridgeHubWococoXcmBlobHauler;
pub const DEFAULT_XCM_LANE_TO_BRIDGE_HUB_WOCOCO: LaneId = [0, 0, 0, 2];
impl XcmBlobHauler for ToBridgeHubWococoXcmBlobHauler {
	type MessageSender =
		pallet_bridge_messages::Pallet<Runtime, WithBridgeHubWococoMessagesInstance>;

	fn message_sender_origin() -> InteriorMultiLocation {
		crate::xcm_config::UniversalLocation::get()
	}

	fn xcm_lane() -> LaneId {
		DEFAULT_XCM_LANE_TO_BRIDGE_HUB_WOCOCO
	}
}

/// Messaging Bridge configuration for BridgeHubRococo -> BridgeHubWococo
pub struct WithBridgeHubWococoMessageBridge;
impl MessageBridge for WithBridgeHubWococoMessageBridge {
	const THIS_CHAIN_ID: ChainId = bp_runtime::BRIDGE_HUB_ROCOCO_CHAIN_ID;
	const BRIDGED_CHAIN_ID: ChainId = bp_runtime::BRIDGE_HUB_WOCOCO_CHAIN_ID;
	const BRIDGED_MESSAGES_PALLET_NAME: &'static str =
		bp_bridge_hub_rococo::WITH_BRIDGE_HUB_ROCOCO_MESSAGES_PALLET_NAME;
	type ThisChain = BridgeHubRococo;
	type BridgedChain = BridgeHubWococo;
	type BridgedHeaderChain = pallet_bridge_parachains::ParachainHeaders<
		Runtime,
		BridgeParachainWococoInstance,
		bp_bridge_hub_wococo::BridgeHubWococo,
	>;
}

/// Message verifier for BridgeHubWococo messages sent from BridgeHubRococo
pub type ToBridgeHubWococoMessageVerifier =
	messages::source::FromThisChainMessageVerifier<WithBridgeHubWococoMessageBridge>;

/// Maximal outbound payload size of BridgeHubRococo -> BridgeHubWococo messages.
pub type ToBridgeHubWococoMaximalOutboundPayloadSize =
	messages::source::FromThisChainMaximalOutboundPayloadSize<WithBridgeHubWococoMessageBridge>;

/// BridgeHubWococo chain from message lane point of view.
#[derive(RuntimeDebug, Clone, Copy)]
pub struct BridgeHubWococo;

impl UnderlyingChainProvider for BridgeHubWococo {
	type Chain = bp_bridge_hub_wococo::BridgeHubWococo;
}

impl SourceHeaderChain for BridgeHubWococo {
	type Error = &'static str;
	type MessagesProof = FromBridgedChainMessagesProof<crate::Hash>;

	fn verify_messages_proof(
		proof: Self::MessagesProof,
		messages_count: u32,
	) -> Result<ProvedMessages<Message>, Self::Error> {
		bridge_runtime_common::messages::target::verify_messages_proof::<
			WithBridgeHubWococoMessageBridge,
		>(proof, messages_count)
		.map_err(Into::into)
	}
}

impl TargetHeaderChain<XcmAsPlainPayload, crate::AccountId> for BridgeHubWococo {
	type Error = &'static str;
	type MessagesDeliveryProof =
		messages::source::FromBridgedChainMessagesDeliveryProof<bp_bridge_hub_wococo::Hash>;

	fn verify_message(payload: &XcmAsPlainPayload) -> Result<(), Self::Error> {
		messages::source::verify_chain_message::<WithBridgeHubWococoMessageBridge>(payload)
	}

	fn verify_messages_delivery_proof(
		proof: Self::MessagesDeliveryProof,
	) -> Result<(LaneId, InboundLaneData<bp_bridge_hub_rococo::AccountId>), Self::Error> {
		messages::source::verify_messages_delivery_proof::<WithBridgeHubWococoMessageBridge>(proof)
	}
}

impl messages::BridgedChainWithMessages for BridgeHubWococo {
	fn verify_dispatch_weight(_message_payload: &[u8]) -> bool {
		true
	}
}

/// BridgeHubRococo chain from message lane point of view.
#[derive(RuntimeDebug, Clone, Copy)]
pub struct BridgeHubRococo;

impl UnderlyingChainProvider for BridgeHubRococo {
	type Chain = bp_bridge_hub_rococo::BridgeHubRococo;
}

impl ThisChainWithMessages for BridgeHubRococo {
	type RuntimeOrigin = crate::RuntimeOrigin;
	type RuntimeCall = crate::RuntimeCall;

	fn is_message_accepted(origin: &Self::RuntimeOrigin, lane: &LaneId) -> bool {
		log::info!(target: crate::LOG_TARGET, "[BridgeHubRococo::ThisChainWithMessages] is_message_accepted - origin: {:?}, lane: {:?}", origin, lane);
		lane == &DEFAULT_XCM_LANE_TO_BRIDGE_HUB_WOCOCO
	}

	fn maximal_pending_messages_at_outbound_lane() -> MessageNonce {
		log::info!(
			target: crate::LOG_TARGET,
			"[BridgeHubRococo::ThisChainWithMessages] maximal_pending_messages_at_outbound_lane"
		);
		MessageNonce::MAX / 2
	}
}
