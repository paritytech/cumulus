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
	universal_exports::{BridgeBlobDispatcher, HaulBlobExporter},
	ParachainInfo, Runtime, WithBridgeHubWococoMessagesInstance, XcmAsPlainPayload, XcmBlobHauler,
	XcmRouter,
};
use bp_messages::{
	source_chain::TargetHeaderChain,
	target_chain::{ProvedMessages, SourceHeaderChain},
	InboundLaneData, LaneId, Message, MessageNonce,
};
use bp_polkadot_core::parachains::ParaId;
use bp_runtime::{Chain, ChainId};
use bridge_runtime_common::{
	messages,
	messages::{
		target::FromBridgedChainMessagesProof, BasicConfirmationTransactionEstimation,
		BridgedChain, ChainWithMessages, MessageBridge, MessageTransaction, ThisChain,
		ThisChainWithMessages, WeightOf,
	},
};
use frame_support::{dispatch::Weight, parameter_types, RuntimeDebug};
use sp_runtime::FixedU128;
use xcm::{
	latest::prelude::*,
	prelude::{InteriorMultiLocation, NetworkId},
};

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
pub type OnBridgeHubRococoBlobDispatcher =
	BridgeBlobDispatcher<XcmRouter, BridgeHubRococoUniversalLocation>;

/// Export XCM messages to be relayed to the otherside
pub type ToBridgeHubWococoHaulBlobExporter =
	HaulBlobExporter<ToBridgeHubWococoXcmBlobHauler, WococoGlobalConsensusNetwork, ()>;
pub struct ToBridgeHubWococoXcmBlobHauler;
pub const DEFAULT_XCM_LANE_TO_BRIDGE_HUB_WOCOCO: LaneId = [0, 0, 0, 2];
impl XcmBlobHauler for ToBridgeHubWococoXcmBlobHauler {
	type SenderChain = bp_bridge_hub_rococo::BridgeHubRococo;
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
	// TODO:check-parameter - relayers rewards
	const RELAYER_FEE_PERCENT: u32 = 0;
	const THIS_CHAIN_ID: ChainId = bp_runtime::BRIDGE_HUB_ROCOCO_CHAIN_ID;
	const BRIDGED_CHAIN_ID: ChainId = bp_runtime::BRIDGE_HUB_WOCOCO_CHAIN_ID;
	const BRIDGED_MESSAGES_PALLET_NAME: &'static str =
		bp_bridge_hub_rococo::WITH_BRIDGE_HUB_ROCOCO_MESSAGES_PALLET_NAME;
	type ThisChain = BridgeHubRococo;
	type BridgedChain = BridgeHubWococo;

	fn bridged_balance_to_this_balance(
		bridged_balance: bridge_runtime_common::messages::BalanceOf<BridgedChain<Self>>,
		bridged_to_this_conversion_rate_override: Option<FixedU128>,
	) -> bridge_runtime_common::messages::BalanceOf<ThisChain<Self>> {
		log::info!("[WithBridgeHubWococoMessageBridge] bridged_balance_to_this_balance - bridged_balance: {:?}, bridged_to_this_conversion_rate_override: {:?}", bridged_balance, bridged_to_this_conversion_rate_override);
		unimplemented!("TODO: WithBridgeHubWococoMessageBridge - bridged_balance_to_this_balance")
	}
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

impl ChainWithMessages for BridgeHubWococo {
	type Hash = bp_bridge_hub_wococo::Hash;
	type AccountId = bp_bridge_hub_wococo::AccountId;
	type Signer = bp_bridge_hub_wococo::AccountSigner;
	type Signature = bp_bridge_hub_wococo::Signature;
	type Weight = Weight;
	type Balance = bp_bridge_hub_wococo::Balance;
}

impl SourceHeaderChain<crate::Balance> for BridgeHubWococo {
	type Error = &'static str;
	type MessagesProof = FromBridgedChainMessagesProof<crate::Hash>;

	fn verify_messages_proof(
		proof: Self::MessagesProof,
		messages_count: u32,
	) -> Result<ProvedMessages<Message<crate::Balance>>, Self::Error> {
		bridge_runtime_common::messages::target::verify_messages_proof_from_parachain::<
			WithBridgeHubWococoMessageBridge,
			bp_bridge_hub_wococo::Header,
			crate::Runtime,
			crate::BridgeParachainWococoInstance,
		>(ParaId(bp_bridge_hub_wococo::BRIDGE_HUB_WOCOCO_PARACHAIN_ID), proof, messages_count)
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
		messages::source::verify_messages_delivery_proof_from_parachain::<
			WithBridgeHubWococoMessageBridge,
			bp_bridge_hub_wococo::Header,
			crate::Runtime,
			crate::BridgeParachainWococoInstance,
		>(ParaId(bp_bridge_hub_wococo::BRIDGE_HUB_WOCOCO_PARACHAIN_ID), proof)
	}
}

impl messages::BridgedChainWithMessages for BridgeHubWococo {
	fn maximal_extrinsic_size() -> u32 {
		bp_bridge_hub_wococo::BridgeHubWococo::max_extrinsic_size()
	}

	fn verify_dispatch_weight(_message_payload: &[u8]) -> bool {
		true
	}

	fn estimate_delivery_transaction(
		message_payload: &[u8],
		include_pay_dispatch_fee_cost: bool,
		message_dispatch_weight: WeightOf<Self>,
	) -> MessageTransaction<WeightOf<Self>> {
		let message_payload_len = u32::try_from(message_payload.len()).unwrap_or(u32::MAX);
		let extra_bytes_in_payload = Weight::from(message_payload_len)
			.saturating_sub(pallet_bridge_messages::EXPECTED_DEFAULT_MESSAGE_LENGTH.into());

		MessageTransaction {
			dispatch_weight: extra_bytes_in_payload
				.saturating_mul(bp_bridge_hub_wococo::ADDITIONAL_MESSAGE_BYTE_DELIVERY_WEIGHT)
				.saturating_add(bp_bridge_hub_wococo::DEFAULT_MESSAGE_DELIVERY_TX_WEIGHT)
				.saturating_sub(if include_pay_dispatch_fee_cost {
					0
				} else {
					bp_bridge_hub_wococo::PAY_INBOUND_DISPATCH_FEE_WEIGHT
				})
				.saturating_add(message_dispatch_weight),
			size: message_payload_len
				.saturating_add(bp_bridge_hub_rococo::EXTRA_STORAGE_PROOF_SIZE)
				.saturating_add(bp_bridge_hub_wococo::TX_EXTRA_BYTES),
		}
	}

	fn transaction_payment(
		transaction: MessageTransaction<WeightOf<Self>>,
	) -> messages::BalanceOf<Self> {
		log::info!(
			"[BridgeHubWococo::BridgedChainWithMessages] transaction_payment - transaction: {:?}",
			transaction
		);
		// TODO:check-parameter - any payment? from sovereign account?
		unimplemented!(
			"[BridgeHubWococo/BridgedChainWithMessages] transaction_payment - transaction: {:?}",
			transaction
		)
	}
}

/// BridgeHubRococo chain from message lane point of view.
#[derive(RuntimeDebug, Clone, Copy)]
pub struct BridgeHubRococo;

impl ChainWithMessages for BridgeHubRococo {
	type Hash = bp_bridge_hub_rococo::Hash;
	type AccountId = bp_bridge_hub_rococo::AccountId;
	type Signer = bp_bridge_hub_rococo::AccountSigner;
	type Signature = bp_bridge_hub_rococo::Signature;
	type Weight = Weight;
	type Balance = bp_bridge_hub_rococo::Balance;
}

impl ThisChainWithMessages for BridgeHubRococo {
	type Origin = crate::Origin;
	type Call = crate::Call;
	type ConfirmationTransactionEstimation = BasicConfirmationTransactionEstimation<
		Self::AccountId,
		{ bp_bridge_hub_rococo::MAX_SINGLE_MESSAGE_DELIVERY_CONFIRMATION_TX_WEIGHT },
		{ bp_bridge_hub_wococo::EXTRA_STORAGE_PROOF_SIZE },
		{ bp_bridge_hub_rococo::TX_EXTRA_BYTES },
	>;

	fn is_message_accepted(origin: &Self::Origin, lane: &LaneId) -> bool {
		log::info!("[BridgeHubRococo::ThisChainWithMessages] is_message_accepted - origin: {:?}, lane: {:?}", origin, lane);
		lane == &DEFAULT_XCM_LANE_TO_BRIDGE_HUB_WOCOCO
	}

	fn maximal_pending_messages_at_outbound_lane() -> MessageNonce {
		log::info!(
			"[BridgeHubRococo::ThisChainWithMessages] maximal_pending_messages_at_outbound_lane"
		);
		MessageNonce::MAX / 2
	}

	fn transaction_payment(
		transaction: MessageTransaction<WeightOf<Self>>,
	) -> messages::BalanceOf<Self> {
		log::info!(
			"[BridgeHubRococo::ThisChainWithMessages] transaction_payment - transaction: {:?}",
			transaction
		);
		// TODO:check-parameter - any payment? from sovereign account?
		unimplemented!(
			"[BridgeHubRococo/ThisChainWithMessages] transaction_payment - transaction: {:?}",
			transaction
		)
	}
}
