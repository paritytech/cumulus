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
	source_chain::{LaneMessageVerifier, TargetHeaderChain},
	target_chain::{DispatchMessage, MessageDispatch, ProvedMessages, SourceHeaderChain},
	InboundLaneData, LaneId, Message, OutboundLaneData,
};
use bp_polkadot_core::Balance;
use bp_runtime::{messages::MessageDispatchResult, AccountIdOf, BalanceOf, Chain};
use codec::Decode;
use frame_support::{dispatch::Weight, parameter_types, RuntimeDebug};

parameter_types! {
	// TODO:check-parameter
	pub const BridgeHubRococoMaxMessagesToPruneAtOnce: bp_messages::MessageNonce = 8;
	pub const BridgeHubWococoMaxMessagesToPruneAtOnce: bp_messages::MessageNonce = 8;
	// TODO:check-parameter
	pub const BridgeHubRococoMaxUnrewardedRelayerEntriesAtInboundLane: bp_messages::MessageNonce =
		bp_bridge_hub_rococo::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	pub const BridgeHubWococoMaxUnrewardedRelayerEntriesAtInboundLane: bp_messages::MessageNonce =
		bp_bridge_hub_wococo::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	// TODO:check-parameter
	pub const BridgeHubRococoMaxUnconfirmedMessagesAtInboundLane: bp_messages::MessageNonce =
		bp_bridge_hub_rococo::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
	pub const BridgeHubWococoMaxUnconfirmedMessagesAtInboundLane: bp_messages::MessageNonce =
		bp_bridge_hub_wococo::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;

	pub const BridgeHubRococoChainId: bp_runtime::ChainId = bp_runtime::BRIDGE_HUB_ROCOCO_CHAIN_ID;
	pub const BridgeHubWococoChainId: bp_runtime::ChainId = bp_runtime::BRIDGE_HUB_WOCOCO_CHAIN_ID;
}

// TODO:check-parameter - when integration XCMv3 change this to struct
pub type PlainXcmPayload = sp_std::prelude::Vec<u8>;

// TODO:check-parameter - when integration XCMv3 change this to struct
pub type FromBridgeHubRococoMessagePayload = PlainXcmPayload;
pub type FromBridgeHubWococoMessagePayload = PlainXcmPayload;
pub type ToBridgeHubRococoMessagePayload = PlainXcmPayload;
pub type ToBridgeHubWococoMessagePayload = PlainXcmPayload;

// TODO:check-parameter - when integrating XCMv3 change this to FromBridgedChainMessagePayload
pub struct FromBridgeHubRococoMessageDispatch<SourceBridgeHubChain, TargetBridgeHubChain> {
	_marker: sp_std::marker::PhantomData<(SourceBridgeHubChain, TargetBridgeHubChain)>,
}
pub struct FromBridgeHubWococoMessageDispatch<SourceBridgeHubChain, TargetBridgeHubChain> {
	_marker: sp_std::marker::PhantomData<(SourceBridgeHubChain, TargetBridgeHubChain)>,
}

impl<SourceBridgeHubChain: Chain, TargetBridgeHubChain: Chain>
	MessageDispatch<AccountIdOf<SourceBridgeHubChain>, BalanceOf<TargetBridgeHubChain>>
	for FromBridgeHubRococoMessageDispatch<SourceBridgeHubChain, TargetBridgeHubChain>
{
	type DispatchPayload = FromBridgeHubRococoMessagePayload;

	fn dispatch_weight(
		message: &mut DispatchMessage<Self::DispatchPayload, BalanceOf<TargetBridgeHubChain>>,
	) -> Weight {
		log::error!("[FromBridgeHubRococoMessageDispatch] TODO: change here to XCMv3 dispatch_weight with XcmExecutor");
		0
	}

	fn dispatch(
		relayer_account: &AccountIdOf<SourceBridgeHubChain>,
		message: DispatchMessage<Self::DispatchPayload, BalanceOf<TargetBridgeHubChain>>,
	) -> MessageDispatchResult {
		log::error!("[FromBridgeHubRococoMessageDispatch] TODO: change here to XCMv3 dispatch with XcmExecutor");
		todo!("TODO: implement XCMv3 dispatch")
	}
}

impl<SourceBridgeHubChain: Chain, TargetBridgeHubChain: Chain>
	MessageDispatch<AccountIdOf<SourceBridgeHubChain>, BalanceOf<TargetBridgeHubChain>>
	for FromBridgeHubWococoMessageDispatch<SourceBridgeHubChain, TargetBridgeHubChain>
{
	type DispatchPayload = FromBridgeHubWococoMessagePayload;

	fn dispatch_weight(
		message: &mut DispatchMessage<Self::DispatchPayload, BalanceOf<TargetBridgeHubChain>>,
	) -> Weight {
		log::error!("[FromBridgeHubWococoMessageDispatch] TODO: change here to XCMv3 dispatch_weight with XcmExecutor");
		0
	}

	fn dispatch(
		relayer_account: &AccountIdOf<SourceBridgeHubChain>,
		message: DispatchMessage<Self::DispatchPayload, BalanceOf<TargetBridgeHubChain>>,
	) -> MessageDispatchResult {
		log::error!("[FromBridgeHubWococoMessageDispatch] TODO: change here to XCMv3 dispatch with XcmExecutor");
		todo!("TODO: implement XCMv3 dispatch")
	}
}

pub struct ToBridgeHubRococoMessageVerifier<Origin, Sender> {
	_marker: sp_std::marker::PhantomData<(Origin, Sender)>,
}
pub struct ToBridgeHubWococoMessageVerifier<Origin, Sender> {
	_marker: sp_std::marker::PhantomData<(Origin, Sender)>,
}

impl<Origin: Clone, Sender: Chain>
	LaneMessageVerifier<
		Origin,
		AccountIdOf<Sender>,
		ToBridgeHubRococoMessagePayload,
		BalanceOf<Sender>,
	> for ToBridgeHubRococoMessageVerifier<Origin, Sender>
{
	type Error = &'static str;

	fn verify_message(
		submitter: &Origin,
		delivery_and_dispatch_fee: &BalanceOf<Sender>,
		lane: &LaneId,
		outbound_data: &OutboundLaneData,
		payload: &ToBridgeHubRococoMessagePayload,
	) -> Result<(), Self::Error> {
		todo!("TODO: ToBridgeHubRococoMessageVerifier - fix verify_message - at the begining to allow all")
	}
}

impl<Origin: Clone, Sender: Chain>
	LaneMessageVerifier<
		Origin,
		AccountIdOf<Sender>,
		ToBridgeHubWococoMessagePayload,
		BalanceOf<Sender>,
	> for ToBridgeHubWococoMessageVerifier<Origin, Sender>
{
	type Error = &'static str;

	fn verify_message(
		submitter: &Origin,
		delivery_and_dispatch_fee: &BalanceOf<Sender>,
		lane: &LaneId,
		outbound_data: &OutboundLaneData,
		payload: &ToBridgeHubWococoMessagePayload,
	) -> Result<(), Self::Error> {
		todo!("TODO: ToBridgeHubWococoMessageVerifier - fix verify_message - at the begining to allow all")
	}
}

/// BridgeHubRococo chain from message lane point of view.
#[derive(RuntimeDebug, Clone, Copy)]
pub struct BridgeHubRococoMessagingSupport;
/// BridgeHubWococo chain from message lane point of view.
#[derive(RuntimeDebug, Clone, Copy)]
pub struct BridgeHubWococoMessagingSupport;

impl SourceHeaderChain<crate::Balance /* bp_bridge_hub_rococo::Balance */>
	for BridgeHubRococoMessagingSupport
{
	type Error = &'static str;
	type MessagesProof = ();

	fn verify_messages_proof(
		proof: Self::MessagesProof,
		messages_count: u32,
	) -> Result<ProvedMessages<Message<crate::Balance>>, Self::Error> {
		// TODO: need to add, bridges-runtime-common and refactor out of bin
		// messages::target::verify_messages_proof_from_parachain::<
		// 	WithRialtoParachainMessageBridge,
		// 	bp_bridge_hub_rococo::Header,
		// 	crate::Runtime,
		// 	crate::WithRialtoParachainsInstance,
		// >(ParaId(bp_rialto_parachain::RIALTO_PARACHAIN_ID), proof, messages_count)
		todo!("TODO: fix and implement SourceHeaderChain::verify_messages_proof")
	}
}

impl
	TargetHeaderChain<
		ToBridgeHubRococoMessagePayload,
		crate::AccountId, /* bp_bridge_hub_wococo::AccountId */
	> for BridgeHubRococoMessagingSupport
{
	type Error = &'static str;
	type MessagesDeliveryProof = ();

	fn verify_message(payload: &ToBridgeHubRococoMessagePayload) -> Result<(), Self::Error> {
		// messages::source::verify_chain_message::<WithRialtoParachainMessageBridge>(payload)
		todo!("TODO: fix implementation: TargetHeaderChain::verify_message")
	}

	fn verify_messages_delivery_proof(
		proof: Self::MessagesDeliveryProof,
	) -> Result<
		(LaneId, InboundLaneData<crate::AccountId /* bp_bridge_hub_wococo::AccountId */>),
		Self::Error,
	> {
		// messages::source::verify_messages_delivery_proof_from_parachain::<
		// 	WithRialtoParachainMessageBridge,
		// 	bp_rialto_parachain::Header,
		// 	Runtime,
		// 	crate::WithRialtoParachainsInstance,
		// >(ParaId(bp_rialto_parachain::RIALTO_PARACHAIN_ID), proof)
		todo!("TODO: fix implementation: TargetHeaderChain::verify_messages_delivery_proof")
	}
}

impl SourceHeaderChain<crate::Balance /* bp_bridge_hub_wococo::Balance */>
	for BridgeHubWococoMessagingSupport
{
	type Error = &'static str;
	type MessagesProof = ();

	fn verify_messages_proof(
		proof: Self::MessagesProof,
		messages_count: u32,
	) -> Result<ProvedMessages<Message<crate::Balance>>, Self::Error> {
		// TODO: need to add, bridges-runtime-common and refactor out of bin
		// messages::target::verify_messages_proof_from_parachain::<
		// 	WithMIllauParachainMessageBridge,
		// 	bp_bridge_hub_wococo::Header,
		// 	crate::Runtime,
		// 	crate::WithRialtoParachainsInstance,
		// >(ParaId(bp_rialto_parachain::RIALTO_PARACHAIN_ID), proof, messages_count)
		todo!("TODO: fix and implement SourceHeaderChain::verify_messages_proof")
	}
}

impl
	TargetHeaderChain<
		ToBridgeHubWococoMessagePayload,
		crate::AccountId, /* bp_bridge_hub_rococo::AccountId */
	> for BridgeHubWococoMessagingSupport
{
	type Error = &'static str;
	type MessagesDeliveryProof = ();

	fn verify_message(payload: &ToBridgeHubWococoMessagePayload) -> Result<(), Self::Error> {
		// messages::source::verify_chain_message::<WithRialtoParachainMessageBridge>(payload)
		todo!("TODO: fix implementation: TargetHeaderChain::verify_message")
	}

	fn verify_messages_delivery_proof(
		proof: Self::MessagesDeliveryProof,
	) -> Result<
		(LaneId, InboundLaneData<crate::AccountId /* bp_bridge_hub_rococo::AccountId */>),
		Self::Error,
	> {
		// messages::source::verify_messages_delivery_proof_from_parachain::<
		// 	WithRialtoParachainMessageBridge,
		// 	bp_rialto_parachain::Header,
		// 	Runtime,
		// 	crate::WithRialtoParachainsInstance,
		// >(ParaId(bp_rialto_parachain::RIALTO_PARACHAIN_ID), proof)
		todo!("TODO: fix implementation: TargetHeaderChain::verify_messages_delivery_proof")
	}
}
