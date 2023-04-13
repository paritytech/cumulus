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
	Balances, ExistentialDeposit, ParachainSystem, PolkadotXcm, Runtime, RuntimeEvent, SessionKeys,
};
use codec::{Decode, Encode};
use xcm::latest::prelude::*;

use bridge_hub_test_utils::*;
use bridge_runtime_common::messages_xcm_extension::XcmBlobMessageDispatchResult;
use frame_support::{parameter_types, weights::Weight};
use parachains_common::{AccountId, AuraId};
use xcm_builder::DispatchBlobError;
use xcm_executor::XcmExecutor;

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

#[test]
fn can_govornance_call_xcm_transact_with_initialize_on_bridge_hub_rococo() {
	// prepare xcm as govornance will do
	let initialize_call: RuntimeCall =
		RuntimeCall::BridgeRococoGrandpa(pallet_bridge_grandpa::Call::<
			Runtime,
			BridgeGrandpaRococoInstance,
		>::initialize {
			init_data: mock_initialiation_data(),
		});
	let xcm = Xcm(vec![
		UnpaidExecution { weight_limit: Unlimited, check_origin: None },
		Transact {
			origin_kind: OriginKind::Superuser,
			require_weight_at_most: Weight::from_parts(1000000000, 0),
			call: initialize_call.encode().into(),
		},
	]);
	// origin as relay chain
	let origin = MultiLocation { parents: 1, interior: Here };

	execute_on_runtime(bp_bridge_hub_rococo::BRIDGE_HUB_ROCOCO_PARACHAIN_ID, None, || {
		// check mode before
		assert_eq!(
			pallet_bridge_grandpa::PalletOperatingMode::<Runtime, BridgeGrandpaRococoInstance>::try_get(),
			Err(())
		);

		// initialize bridge through governance-like
		let hash = xcm.using_encoded(sp_io::hashing::blake2_256);
		let weight_limit = Weight::from_parts(41666666666, 0);
		let outcome = XcmExecutor::<XcmConfig>::execute_xcm(origin, xcm, hash, weight_limit);

		// check mode after
		assert_eq!(outcome.ensure_complete(), Ok(()));
		assert_eq!(
			pallet_bridge_grandpa::PalletOperatingMode::<Runtime, BridgeGrandpaRococoInstance>::try_get(),
			Ok(bp_runtime::BasicOperatingMode::Normal)
		);
	})
}

#[test]
fn can_govornance_call_xcm_transact_with_initialize_bridge_on_bridge_hub_wococo() {
	// prepare xcm as govornance will do
	let initialize_call: RuntimeCall =
		RuntimeCall::BridgeWococoGrandpa(pallet_bridge_grandpa::Call::<
			Runtime,
			BridgeGrandpaWococoInstance,
		>::initialize {
			init_data: mock_initialiation_data(),
		});
	let xcm = Xcm(vec![
		UnpaidExecution { weight_limit: Unlimited, check_origin: None },
		Transact {
			origin_kind: OriginKind::Superuser,
			require_weight_at_most: Weight::from_parts(1000000000, 0),
			call: initialize_call.encode().into(),
		},
	]);
	// origin as relay chain
	let origin = MultiLocation { parents: 1, interior: Here };

	execute_on_runtime(bp_bridge_hub_wococo::BRIDGE_HUB_WOCOCO_PARACHAIN_ID, None, || {
		// check mode before
		assert_eq!(
			pallet_bridge_grandpa::PalletOperatingMode::<Runtime, BridgeGrandpaWococoInstance>::try_get(),
			Err(())
		);

		// initialize bridge through governance-like
		let hash = xcm.using_encoded(sp_io::hashing::blake2_256);
		let weight_limit = Weight::from_parts(41666666666, 0);
		let outcome = XcmExecutor::<XcmConfig>::execute_xcm(origin, xcm, hash, weight_limit);

		// check mode after
		assert_eq!(outcome.ensure_complete(), Ok(()));
		assert_eq!(
			pallet_bridge_grandpa::PalletOperatingMode::<Runtime, BridgeGrandpaWococoInstance>::try_get(),
			Ok(bp_runtime::BasicOperatingMode::Normal)
		);
	})
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
