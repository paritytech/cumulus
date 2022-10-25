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

use bp_messages::{source_chain::{LaneMessageVerifier, TargetHeaderChain}, target_chain::{ProvedMessages, SourceHeaderChain}, InboundLaneData, LaneId, Message, OutboundLaneData, MessageKey, MessageData};
use bp_runtime::{BalanceOf, Chain};
use frame_support::{parameter_types, RuntimeDebug};
use xcm::prelude::{InteriorMultiLocation, NetworkId};
use bp_messages::target_chain::ProvedLaneMessages;
use bridge_runtime_common::messages::target::FromBridgedChainMessagesProof;
use crate::universal_exports::{BridgeBlobDispatcher, HaulBlobExporter};
use crate::{WithBridgeHubWococoMessagesInstance, XcmAsPlainPayload, XcmBlobHauler, XcmRouter};
use crate::Runtime;
use crate::ParachainInfo;
use xcm::latest::prelude::*;

// TODO:check-parameter
parameter_types! {
	pub const MaxUnrewardedRelayerEntriesAtInboundLane: bp_messages::MessageNonce =
		bp_bridge_hub_rococo::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	pub const MaxUnconfirmedMessagesAtInboundLane: bp_messages::MessageNonce =
		bp_bridge_hub_rococo::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
	pub const BridgeHubWococoChainId: bp_runtime::ChainId = bp_runtime::BRIDGE_HUB_WOCOCO_CHAIN_ID;
	pub BridgeHubRococoUniversalLocation: InteriorMultiLocation = X2(GlobalConsensus(Rococo), Parachain(ParachainInfo::parachain_id().into()));
	pub WococoGlobalConsensusNetwork: NetworkId = NetworkId::Wococo;
}

/// Dispatches received XCM messages from other bridge
pub type OnBridgeHubRococoBlobDispatcher = BridgeBlobDispatcher<XcmRouter, BridgeHubRococoUniversalLocation>;

/// Export XCM messages to be relayed to the otherside
pub type ToBridgeHubWococoHaulBlobExporter = HaulBlobExporter<ToBridgeHubWococoXcmBlobHauler, WococoGlobalConsensusNetwork, ()>;
pub struct ToBridgeHubWococoXcmBlobHauler;
pub const DEFAULT_XCM_LANE_TO_BRIDGE_HUB_WOCOCO: LaneId = [0, 0, 0, 1];
impl XcmBlobHauler for ToBridgeHubWococoXcmBlobHauler {
	type SenderChain = bp_bridge_hub_rococo::BridgeHubRococo;
	type MessageSender = pallet_bridge_messages::Pallet<Runtime, WithBridgeHubWococoMessagesInstance>;

	fn message_sender_origin() -> InteriorMultiLocation {
		crate::xcm_config::UniversalLocation::get()
	}

	fn xcm_lane() -> LaneId {
		DEFAULT_XCM_LANE_TO_BRIDGE_HUB_WOCOCO
	}
}
