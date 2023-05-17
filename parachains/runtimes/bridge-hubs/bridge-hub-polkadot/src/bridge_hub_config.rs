// Copyright 2023 Parity Technologies (UK) Ltd.
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

//! Bridge definitions.

use crate::{
	BridgeParachainKusamaInstance, Runtime, WithBridgeHubKusamaMessagesInstance, XcmRouter,
};
use bp_messages::LaneId;
use bridge_runtime_common::{
	messages,
	messages::{
		source::FromBridgedChainMessagesDeliveryProof, target::FromBridgedChainMessagesProof,
		MessageBridge, ThisChainWithMessages, UnderlyingChainProvider,
	},
	messages_xcm_extension::{XcmBlobHauler, XcmBlobHaulerAdapter},
	refund_relayer_extension::{
		ActualFeeRefund, RefundBridgedParachainMessages, RefundableMessagesLane,
		RefundableParachain,
	},
};
use frame_support::{parameter_types, RuntimeDebug};
use xcm::{latest::prelude::*, prelude::NetworkId};
use xcm_builder::{BridgeBlobDispatcher, HaulBlobExporter};

parameter_types! {
	pub const MaxUnrewardedRelayerEntriesAtInboundLane: bp_messages::MessageNonce =
		bp_bridge_hub_polkadot::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	pub const MaxUnconfirmedMessagesAtInboundLane: bp_messages::MessageNonce =
		bp_bridge_hub_polkadot::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
	pub const BridgeHubKusamaChainId: bp_runtime::ChainId = bp_runtime::BRIDGE_HUB_KUSAMA_CHAIN_ID;
	pub KusamaGlobalConsensusNetwork: NetworkId = NetworkId::Kusama;
	pub ActiveOutboundLanesToBridgeHubKusama: &'static [bp_messages::LaneId] = &[DEFAULT_XCM_LANE_TO_BRIDGE_HUB_KUSAMA];
	pub PriorityBoostPerMessage: u64 = 921_900_294;
	pub const BridgeHubKusamaMessagesLane: bp_messages::LaneId = DEFAULT_XCM_LANE_TO_BRIDGE_HUB_KUSAMA;
}

/// Proof of messages, coming from BridgeHubKusama.
pub type FromBridgeHubKusamaMessagesProof =
	FromBridgedChainMessagesProof<bp_bridge_hub_kusama::Hash>;
/// Messages delivery proof for BridgeHubKusama for BridgeHubKusama messages.
pub type ToBridgeHubKusamaMessagesDeliveryProof =
	FromBridgedChainMessagesDeliveryProof<bp_bridge_hub_kusama::Hash>;

/// Dispatches received XCM messages from other bridge
pub type OnThisChainBlobDispatcher<UniversalLocation> =
	BridgeBlobDispatcher<XcmRouter, UniversalLocation>;

/// Export XCM messages to be relayed to the otherside
pub type ToBridgeHubKusamaHaulBlobExporter = HaulBlobExporter<
	XcmBlobHaulerAdapter<ToBridgeHubKusamaXcmBlobHauler>,
	KusamaGlobalConsensusNetwork,
	(),
>;
pub struct ToBridgeHubKusamaXcmBlobHauler;
impl XcmBlobHauler for ToBridgeHubKusamaXcmBlobHauler {
	type MessageSender =
		pallet_bridge_messages::Pallet<Runtime, WithBridgeHubKusamaMessagesInstance>;

	type MessageSenderOrigin = super::RuntimeOrigin;

	fn message_sender_origin() -> Self::MessageSenderOrigin {
		// TODO:check-parameter - maybe Here.into() is enought?
		pallet_xcm::Origin::from(MultiLocation::new(1, crate::xcm_config::UniversalLocation::get()))
			.into()
	}

	fn xcm_lane() -> LaneId {
		DEFAULT_XCM_LANE_TO_BRIDGE_HUB_KUSAMA
	}
}
pub const DEFAULT_XCM_LANE_TO_BRIDGE_HUB_KUSAMA: LaneId = LaneId([0, 0, 0, 1]);

/// Messaging Bridge configuration for ThisChain -> BridgeHubKusama
pub struct WithBridgeHubKusamaMessageBridge;
impl MessageBridge for WithBridgeHubKusamaMessageBridge {
	const BRIDGED_MESSAGES_PALLET_NAME: &'static str =
		bp_bridge_hub_polkadot::WITH_BRIDGE_HUB_POLKADOT_MESSAGES_PALLET_NAME;
	type ThisChain = ThisChain;
	type BridgedChain = BridgeHubKusama;
	type BridgedHeaderChain = pallet_bridge_parachains::ParachainHeaders<
		Runtime,
		BridgeParachainKusamaInstance,
		bp_bridge_hub_kusama::BridgeHubKusama,
	>;
}

/// Message verifier for BridgeHubKusama messages sent from ThisChain
pub type ToBridgeHubKusamaMessageVerifier =
	messages::source::FromThisChainMessageVerifier<WithBridgeHubKusamaMessageBridge>;

/// Maximal outbound payload size of ThisChain -> BridgeHubKusama messages.
pub type ToBridgeHubKusamaMaximalOutboundPayloadSize =
	messages::source::FromThisChainMaximalOutboundPayloadSize<WithBridgeHubKusamaMessageBridge>;

/// BridgeHubKusama chain from message lane point of view.
#[derive(RuntimeDebug, Clone, Copy)]
pub struct BridgeHubKusama;

impl UnderlyingChainProvider for BridgeHubKusama {
	type Chain = bp_bridge_hub_kusama::BridgeHubKusama;
}

impl messages::BridgedChainWithMessages for BridgeHubKusama {}

/// ThisChain chain from message lane point of view.
#[derive(RuntimeDebug, Clone, Copy)]
pub struct ThisChain;

impl UnderlyingChainProvider for ThisChain {
	type Chain = bp_bridge_hub_polkadot::BridgeHubPolkadot;
}

impl ThisChainWithMessages for ThisChain {
	type RuntimeOrigin = crate::RuntimeOrigin;
}

/// Signed extension that refunds relayers that are delivering messages from the kusama BridgeHub.
pub type BridgeRefundBridgeHubKusamaMessages = RefundBridgedParachainMessages<
	Runtime,
	RefundableParachain<BridgeParachainKusamaInstance, BridgeHubKusama>,
	RefundableMessagesLane<WithBridgeHubKusamaMessagesInstance, BridgeHubKusamaMessagesLane>,
	ActualFeeRefund<Runtime>,
	PriorityBoostPerMessage,
	StrBridgeRefundBridgeHubKusamaMessages,
>;
bp_runtime::generate_static_str_provider!(BridgeRefundBridgeHubKusamaMessages);

#[cfg(test)]
mod tests {
	use super::*;
	use crate::BridgeGrandpaKusamaInstance;
	use bridge_runtime_common::{
		assert_complete_bridge_types,
		integrity::{
			assert_complete_bridge_constants, check_message_lane_weights,
			AssertBridgeMessagesPalletConstants, AssertBridgePalletNames, AssertChainConstants,
			AssertCompleteBridgeConstants,
		},
	};

	#[test]
	fn ensure_lane_weights_are_correct() {
		check_message_lane_weights::<
			bp_bridge_hub_polkadot::BridgeHubPolkadot,
			Runtime,
			WithBridgeHubKusamaMessagesInstance,
		>(
			bp_bridge_hub_kusama::EXTRA_STORAGE_PROOF_SIZE,
			bp_bridge_hub_polkadot::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX,
			bp_bridge_hub_polkadot::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX,
			true,
		);
	}

	#[test]
	fn ensure_bridge_integrity() {
		assert_complete_bridge_types!(
			runtime: Runtime,
			with_bridged_chain_grandpa_instance: BridgeGrandpaKusamaInstance,
			with_bridged_chain_messages_instance: WithBridgeHubKusamaMessagesInstance,
			bridge: WithBridgeHubKusamaMessageBridge,
			this_chain: bp_polkadot::Polkadot,
			bridged_chain: bp_kusama::Kusama,
		);

		assert_complete_bridge_constants::<
			Runtime,
			BridgeGrandpaKusamaInstance,
			WithBridgeHubKusamaMessagesInstance,
			WithBridgeHubKusamaMessageBridge,
		>(AssertCompleteBridgeConstants {
			this_chain_constants: AssertChainConstants {
				block_length: bp_bridge_hub_polkadot::BlockLength::get(),
				block_weights: bp_bridge_hub_polkadot::BlockWeights::get(),
			},
			messages_pallet_constants: AssertBridgeMessagesPalletConstants {
				max_unrewarded_relayers_in_bridged_confirmation_tx:
					bp_bridge_hub_kusama::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX,
				max_unconfirmed_messages_in_bridged_confirmation_tx:
					bp_bridge_hub_kusama::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX,
				bridged_chain_id: bp_runtime::BRIDGE_HUB_KUSAMA_CHAIN_ID,
			},
			pallet_names: AssertBridgePalletNames {
				with_this_chain_messages_pallet_name:
					bp_bridge_hub_polkadot::WITH_BRIDGE_HUB_POLKADOT_MESSAGES_PALLET_NAME,
				with_bridged_chain_grandpa_pallet_name: bp_kusama::WITH_KUSAMA_GRANDPA_PALLET_NAME,
				with_bridged_chain_messages_pallet_name:
					bp_bridge_hub_kusama::WITH_BRIDGE_HUB_KUSAMA_MESSAGES_PALLET_NAME,
			},
		});
	}
}
