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

use bp_messages::target_chain::MessageDispatch;
use bp_runtime::messages::MessageDispatchResult;
use bridge_hub_rococo_runtime::bridge_common_config::XcmBlobMessageDispatchResult;
pub use bridge_hub_rococo_runtime::{
	runtime_api,
	xcm_config::{XcmConfig, XcmRouter},
	Runtime, *,
};
use xcm::latest::prelude::*;

use bridge_hub_test_utils::*;

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
#[serial_test::serial]
fn test_bridge_hub_wococo_dispatch_blob_and_xcm_routing_works() {
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
			XcmBlobMessageDispatchResult::NotDispatched("DispatchBlobError::RoutingError")
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
#[serial_test::serial]
fn test_bridge_hub_rococo_dispatch_blob_and_xcm_routing_works() {
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
			XcmBlobMessageDispatchResult::NotDispatched("DispatchBlobError::RoutingError")
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
