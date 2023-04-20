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

use bp_messages::target_chain::MessageDispatch;
use bp_runtime::messages::MessageDispatchResult;
pub use bridge_hub_rococo_runtime::{
	constants::fee::WeightToFee,
	xcm_config::{XcmConfig, XcmRouter},
	Balances, BridgeGrandpaRococoInstance, BridgeGrandpaWococoInstance, ExistentialDeposit,
	ParachainSystem, PolkadotXcm, Runtime, RuntimeCall, RuntimeEvent, SessionKeys,
};
use codec::{Decode, Encode};
use xcm::latest::prelude::*;

use bridge_hub_rococo_runtime::{
	bridge_hub_rococo_config, bridge_hub_wococo_config, WithBridgeHubRococoMessagesInstance,
	WithBridgeHubWococoMessagesInstance,
};

use bridge_hub_test_utils::{
	dummy_account, dummy_xcm, mock_open_hrmp_channel, new_test_ext, simulate_export_message,
	wrap_as_dispatch_message,
};
use bridge_runtime_common::messages_xcm_extension::XcmBlobMessageDispatchResult;
use frame_support::parameter_types;
use parachains_common::{AccountId, AuraId};
use xcm_builder::DispatchBlobError;

fn execute_on_runtime<R>(
	with_para_id: u32,
	open_hrmp_to_para_id: Option<u32>,
	execute: impl FnOnce() -> R,
) -> R {
	new_test_ext::<Runtime>(with_para_id.into(), 3).execute_with(|| {
		if let Some(open_hrmp_to_para_id) = open_hrmp_to_para_id {
			mock_open_hrmp_channel::<Runtime, ParachainSystem>(
				with_para_id.into(),
				open_hrmp_to_para_id.into(),
			);
		}
		execute()
	})
}

#[test]
fn dispatch_blob_and_xcm_routing_works_on_bridge_hub_wococo() {
	let universal_source_as_senders =
		vec![X1(GlobalConsensus(Rococo)), X2(GlobalConsensus(Rococo), Parachain(1000))];
	let runtime_para_id = bp_bridge_hub_wococo::BRIDGE_HUB_WOCOCO_PARACHAIN_ID;
	let destination_network_id = Wococo;
	let destination_para_id = 1000;

	for univeral_source_as_sender in universal_source_as_senders {
		// 1. message is sent to other global consensus - Wococo(Here)
		let bridging_message =
			simulate_export_message::<bridge_hub_rococo_config::WococoGlobalConsensusNetwork>(
				univeral_source_as_sender,
				destination_network_id,
				Here,
				dummy_xcm(),
			);
		let result: MessageDispatchResult<XcmBlobMessageDispatchResult> = execute_on_runtime(
			runtime_para_id,
			None,
			|| {
				<<Runtime as pallet_bridge_messages::Config<WithBridgeHubRococoMessagesInstance>>::MessageDispatch as MessageDispatch<_>>::dispatch(
					&dummy_account(),
					wrap_as_dispatch_message(bridging_message)
				)
			},
		);
		assert_eq!(result.dispatch_level_result, XcmBlobMessageDispatchResult::Dispatched);

		// 2. message is sent to other global consensus and its parachains - Wococo(Here)
		let bridging_message =
			simulate_export_message::<bridge_hub_rococo_config::WococoGlobalConsensusNetwork>(
				univeral_source_as_sender,
				destination_network_id,
				X1(Parachain(destination_para_id)),
				dummy_xcm(),
			);

		// 2.1. WITHOUT hrmp channel -> RoutingError
		let result: MessageDispatchResult<XcmBlobMessageDispatchResult> = execute_on_runtime(
			runtime_para_id,
			None,
			|| {
				<<Runtime as pallet_bridge_messages::Config<WithBridgeHubRococoMessagesInstance>>::MessageDispatch as MessageDispatch<_>>::dispatch(
					&dummy_account(),
					wrap_as_dispatch_message(bridging_message.clone())
				)
			},
		);
		assert_eq!(
			result.dispatch_level_result,
			XcmBlobMessageDispatchResult::NotDispatched(Some(DispatchBlobError::RoutingError))
		);

		// 2.1. WITH hrmp channel -> Ok
		let result: MessageDispatchResult<XcmBlobMessageDispatchResult> = execute_on_runtime(
			runtime_para_id,
			Some(destination_para_id),
			|| {
				<<Runtime as pallet_bridge_messages::Config<WithBridgeHubRococoMessagesInstance>>::MessageDispatch as MessageDispatch<_>>::dispatch(
					&dummy_account(),
					wrap_as_dispatch_message(bridging_message.clone())
				)
			},
		);
		assert_eq!(result.dispatch_level_result, XcmBlobMessageDispatchResult::Dispatched);
	}
}

#[test]
fn dispatch_blob_and_xcm_routing_works_on_bridge_hub_rococo() {
	let universal_source_as_senders =
		vec![X1(GlobalConsensus(Wococo)), X2(GlobalConsensus(Wococo), Parachain(1000))];
	let runtime_para_id = bp_bridge_hub_rococo::BRIDGE_HUB_ROCOCO_PARACHAIN_ID;
	let destination_network_id = Rococo;
	let destination_para_id = 1000;

	for univeral_source_as_sender in universal_source_as_senders {
		// 1. message is sent to other global consensus - Wococo(Here)
		let bridging_message =
			simulate_export_message::<bridge_hub_wococo_config::RococoGlobalConsensusNetwork>(
				univeral_source_as_sender,
				destination_network_id,
				Here,
				dummy_xcm(),
			);
		let result: MessageDispatchResult<XcmBlobMessageDispatchResult> = execute_on_runtime(
			runtime_para_id,
			None,
			|| {
				<<Runtime as pallet_bridge_messages::Config<WithBridgeHubWococoMessagesInstance>>::MessageDispatch as MessageDispatch<_>>::dispatch(
					&dummy_account(),
					wrap_as_dispatch_message(bridging_message)
				)
			},
		);
		assert_eq!(result.dispatch_level_result, XcmBlobMessageDispatchResult::Dispatched);

		// 2. message is sent to other global consensus and its parachains - Wococo(Here)
		let bridging_message =
			simulate_export_message::<bridge_hub_wococo_config::RococoGlobalConsensusNetwork>(
				univeral_source_as_sender,
				destination_network_id,
				X1(Parachain(destination_para_id)),
				dummy_xcm(),
			);

		// 2.1. WITHOUT hrmp channel -> RoutingError
		let result: MessageDispatchResult<XcmBlobMessageDispatchResult> = execute_on_runtime(
			runtime_para_id,
			None,
			|| {
				<<Runtime as pallet_bridge_messages::Config<WithBridgeHubWococoMessagesInstance>>::MessageDispatch as MessageDispatch<_>>::dispatch(
					&dummy_account(),
					wrap_as_dispatch_message(bridging_message.clone())
				)
			},
		);
		assert_eq!(
			result.dispatch_level_result,
			XcmBlobMessageDispatchResult::NotDispatched(Some(DispatchBlobError::RoutingError))
		);

		// 2.1. WITH hrmp channel -> Ok
		let result: MessageDispatchResult<XcmBlobMessageDispatchResult> = execute_on_runtime(
			runtime_para_id,
			Some(destination_para_id),
			|| {
				<<Runtime as pallet_bridge_messages::Config<WithBridgeHubWococoMessagesInstance>>::MessageDispatch as MessageDispatch<_>>::dispatch(
					&dummy_account(),
					wrap_as_dispatch_message(bridging_message.clone())
				)
			},
		);
		assert_eq!(result.dispatch_level_result, XcmBlobMessageDispatchResult::Dispatched);
	}
}

// TODO:check-parameter - add test for DeliveryConfirmationPayments when receive_messages_delivery_proof

const ALICE: [u8; 32] = [1u8; 32];

parameter_types! {
	pub CheckingAccount: AccountId = PolkadotXcm::check_account();
}

bridge_hub_test_utils::test_cases::include_teleports_for_native_asset_works!(
	Runtime,
	XcmConfig,
	CheckingAccount,
	WeightToFee,
	ParachainSystem,
	bridge_hub_test_utils::CollatorSessionKeys::new(
		AccountId::from(ALICE),
		AccountId::from(ALICE),
		SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) }
	),
	ExistentialDeposit::get(),
	Box::new(|runtime_event_encoded: Vec<u8>| {
		match RuntimeEvent::decode(&mut &runtime_event_encoded[..]) {
			Ok(RuntimeEvent::PolkadotXcm(event)) => Some(event),
			_ => None,
		}
	}),
	Box::new(|runtime_event_encoded: Vec<u8>| {
		match RuntimeEvent::decode(&mut &runtime_event_encoded[..]) {
			Ok(RuntimeEvent::XcmpQueue(event)) => Some(event),
			_ => None,
		}
	}),
	1013
);

bridge_hub_test_utils::include_initialize_bridge_by_governance_works!(
	initialize_bridge_to_wococo_by_governance_works,
	Runtime,
	XcmConfig,
	BridgeGrandpaWococoInstance,
	bridge_hub_test_utils::CollatorSessionKeys::new(
		AccountId::from(ALICE),
		AccountId::from(ALICE),
		SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) }
	),
	Box::new(|call| RuntimeCall::BridgeWococoGrandpa(call).encode()),
	1013
);

mod bridge_hub_wococo {
	use super::*;

	bridge_hub_test_utils::test_cases::include_teleports_for_native_asset_works!(
		Runtime,
		XcmConfig,
		CheckingAccount,
		WeightToFee,
		ParachainSystem,
		bridge_hub_test_utils::CollatorSessionKeys::new(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) }
		),
		ExistentialDeposit::get(),
		Box::new(|runtime_event_encoded: Vec<u8>| {
			match RuntimeEvent::decode(&mut &runtime_event_encoded[..]) {
				Ok(RuntimeEvent::PolkadotXcm(event)) => Some(event),
				_ => None,
			}
		}),
		Box::new(|runtime_event_encoded: Vec<u8>| {
			match RuntimeEvent::decode(&mut &runtime_event_encoded[..]) {
				Ok(RuntimeEvent::XcmpQueue(event)) => Some(event),
				_ => None,
			}
		}),
		1014
	);

	bridge_hub_test_utils::include_initialize_bridge_by_governance_works!(
		initialize_bridge_to_rococo_by_governance_works,
		Runtime,
		XcmConfig,
		BridgeGrandpaRococoInstance,
		bridge_hub_test_utils::CollatorSessionKeys::new(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) }
		),
		Box::new(|call| RuntimeCall::BridgeRococoGrandpa(call).encode()),
		1014
	);
}
