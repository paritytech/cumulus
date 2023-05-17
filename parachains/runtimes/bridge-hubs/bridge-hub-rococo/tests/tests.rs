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

#![cfg(test)]

use bp_messages::LaneId;
use bp_polkadot_core::Signature;
use bridge_hub_rococo_runtime::{
	bridge_hub_rococo_config, bridge_hub_wococo_config,
	constants::fee::WeightToFee,
	xcm_config::{RelayNetwork, XcmConfig},
	Balances, BridgeRejectObsoleteHeadersAndMessages, Executive, ExistentialDeposit,
	ParachainSystem, PolkadotXcm, Runtime, RuntimeCall, RuntimeEvent, SessionKeys, SignedExtra,
	UncheckedExtrinsic,
};
use codec::{Decode, Encode};
use frame_support::parameter_types;
use parachains_common::{AccountId, AuraId};
use sp_keyring::AccountKeyring::Alice;
use sp_runtime::{
	generic::{Era, SignedPayload},
	AccountId32,
};
use xcm::latest::prelude::*;

parameter_types! {
	pub CheckingAccount: AccountId = PolkadotXcm::check_account();
}

fn construct_extrinsic(
	sender: sp_keyring::AccountKeyring,
	call: RuntimeCall,
) -> UncheckedExtrinsic {
	let extra: SignedExtra = (
		frame_system::CheckNonZeroSender::<Runtime>::new(),
		frame_system::CheckSpecVersion::<Runtime>::new(),
		frame_system::CheckTxVersion::<Runtime>::new(),
		frame_system::CheckGenesis::<Runtime>::new(),
		frame_system::CheckEra::<Runtime>::from(Era::immortal()),
		frame_system::CheckNonce::<Runtime>::from(0),
		frame_system::CheckWeight::<Runtime>::new(),
		pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(0),
		BridgeRejectObsoleteHeadersAndMessages {},
		(
			bridge_hub_wococo_config::BridgeRefundBridgeHubRococoMessages::default(),
			bridge_hub_rococo_config::BridgeRefundBridgeHubWococoMessages::default(),
		),
	);
	let payload = SignedPayload::new(call.clone(), extra.clone()).unwrap();
	let signature = payload.using_encoded(|e| sender.sign(e));
	UncheckedExtrinsic::new_signed(
		call,
		AccountId32::from(sender.public()).into(),
		Signature::Sr25519(signature.clone()),
		extra,
	)
}

fn construct_and_apply_extrinsic(
	relayer_at_target: sp_keyring::AccountKeyring,
	batch: pallet_utility::Call<Runtime>,
) -> sp_runtime::DispatchOutcome {
	let batch_call = RuntimeCall::Utility(batch);
	let xt = construct_extrinsic(relayer_at_target, batch_call);
	let r = Executive::apply_extrinsic(xt);
	r.unwrap()
}

fn executive_init_block(header: &<Runtime as frame_system::Config>::Header) {
	Executive::initialize_block(header)
}

fn drip_some_balance(account: &<Runtime as frame_system::Config>::AccountId) {
	use frame_support::traits::fungible::Mutate;
	let some_currency = ExistentialDeposit::get() * 100000;
	Balances::mint_into(account, some_currency).unwrap();
}

fn collator_session_keys() -> parachains_runtimes_test_utils::CollatorSessionKeys<Runtime> {
	parachains_runtimes_test_utils::CollatorSessionKeys::new(
		AccountId::from(Alice),
		AccountId::from(Alice),
		SessionKeys { aura: AuraId::from(Alice.public()) },
	)
}

mod bridge_hub_rococo_tests {
	use super::*;
	use bridge_hub_rococo_config::WithBridgeHubWococoMessageBridge;
	use bridge_hub_rococo_runtime::{
		BridgeGrandpaWococoInstance, BridgeParachainWococoInstance,
		WithBridgeHubWococoMessagesInstance,
	};

	bridge_hub_test_utils::test_cases::include_teleports_for_native_asset_works!(
		Runtime,
		XcmConfig,
		CheckingAccount,
		WeightToFee,
		ParachainSystem,
		collator_session_keys(),
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
		bp_bridge_hub_rococo::BRIDGE_HUB_ROCOCO_PARACHAIN_ID
	);

	#[test]
	fn initialize_bridge_by_governance_works() {
		bridge_hub_test_utils::test_cases::initialize_bridge_by_governance_works::<
			Runtime,
			BridgeGrandpaWococoInstance,
		>(
			collator_session_keys(),
			bp_bridge_hub_rococo::BRIDGE_HUB_ROCOCO_PARACHAIN_ID,
			Box::new(|call| RuntimeCall::BridgeWococoGrandpa(call).encode()),
		)
	}

	#[test]
	fn handle_export_message_from_system_parachain_add_to_outbound_queue_works() {
		bridge_hub_test_utils::test_cases::handle_export_message_from_system_parachain_to_outbound_queue_works::<
			Runtime,
			XcmConfig,
			WithBridgeHubWococoMessagesInstance,
		>(
			collator_session_keys(),
			bp_bridge_hub_rococo::BRIDGE_HUB_ROCOCO_PARACHAIN_ID,
			1000,
			Box::new(|runtime_event_encoded: Vec<u8>| {
				match RuntimeEvent::decode(&mut &runtime_event_encoded[..]) {
					Ok(RuntimeEvent::BridgeWococoMessages(event)) => Some(event),
					_ => None,
				}
			}),
			|| ExportMessage { network: Wococo, destination: X1(Parachain(1234)), xcm: Xcm(vec![]) },
			bridge_hub_rococo_config::DEFAULT_XCM_LANE_TO_BRIDGE_HUB_WOCOCO
		)
	}

	#[test]
	fn message_dispatch_routing_works() {
		bridge_hub_test_utils::test_cases::message_dispatch_routing_works::<
			Runtime,
			XcmConfig,
			ParachainSystem,
			WithBridgeHubWococoMessagesInstance,
			RelayNetwork,
			bridge_hub_rococo_config::WococoGlobalConsensusNetwork,
		>(
			collator_session_keys(),
			bp_bridge_hub_rococo::BRIDGE_HUB_ROCOCO_PARACHAIN_ID,
			1000,
			Box::new(|runtime_event_encoded: Vec<u8>| {
				match RuntimeEvent::decode(&mut &runtime_event_encoded[..]) {
					Ok(RuntimeEvent::ParachainSystem(event)) => Some(event),
					_ => None,
				}
			}),
			Box::new(|runtime_event_encoded: Vec<u8>| {
				match RuntimeEvent::decode(&mut &runtime_event_encoded[..]) {
					Ok(RuntimeEvent::XcmpQueue(event)) => Some(event),
					_ => None,
				}
			}),
			bridge_hub_rococo_config::DEFAULT_XCM_LANE_TO_BRIDGE_HUB_WOCOCO,
		)
	}

	#[test]
	fn relayed_incoming_message_works() {
		bridge_hub_test_utils::test_cases::relayed_incoming_message_works::<
			Runtime,
			XcmConfig,
			ParachainSystem,
			BridgeGrandpaWococoInstance,
			BridgeParachainWococoInstance,
			WithBridgeHubWococoMessagesInstance,
			WithBridgeHubWococoMessageBridge,
		>(
			collator_session_keys(),
			bp_bridge_hub_rococo::BRIDGE_HUB_ROCOCO_PARACHAIN_ID,
			bp_bridge_hub_wococo::BRIDGE_HUB_WOCOCO_PARACHAIN_ID,
			1000,
			Rococo,
			LaneId([0, 0, 0, 1]),
		)
	}

	#[test]
	pub fn complex_relay_extrinsic_works() {
		bridge_hub_test_utils::test_cases::complex_relay_extrinsic_works::<
			Runtime,
			XcmConfig,
			ParachainSystem,
			BridgeGrandpaWococoInstance,
			BridgeParachainWococoInstance,
			WithBridgeHubWococoMessagesInstance,
			WithBridgeHubWococoMessageBridge,
		>(
			collator_session_keys(),
			bp_bridge_hub_rococo::BRIDGE_HUB_ROCOCO_PARACHAIN_ID,
			bp_bridge_hub_wococo::BRIDGE_HUB_WOCOCO_PARACHAIN_ID,
			1000,
			bridge_hub_rococo_config::BridgeHubWococoChainId::get(),
			Rococo,
			LaneId([0, 0, 0, 1]),
			executive_init_block,
			drip_some_balance,
			construct_and_apply_extrinsic,
		);
	}
}

mod bridge_hub_wococo_tests {
	use super::*;
	use bridge_hub_rococo_runtime::{
		BridgeGrandpaRococoInstance, BridgeParachainRococoInstance,
		WithBridgeHubRococoMessagesInstance,
	};
	use bridge_hub_wococo_config::WithBridgeHubRococoMessageBridge;

	bridge_hub_test_utils::test_cases::include_teleports_for_native_asset_works!(
		Runtime,
		XcmConfig,
		CheckingAccount,
		WeightToFee,
		ParachainSystem,
		collator_session_keys(),
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
		bp_bridge_hub_wococo::BRIDGE_HUB_WOCOCO_PARACHAIN_ID
	);

	#[test]
	fn initialize_bridge_by_governance_works() {
		bridge_hub_test_utils::test_cases::initialize_bridge_by_governance_works::<
			Runtime,
			BridgeGrandpaRococoInstance,
		>(
			collator_session_keys(),
			bp_bridge_hub_wococo::BRIDGE_HUB_WOCOCO_PARACHAIN_ID,
			Box::new(|call| RuntimeCall::BridgeRococoGrandpa(call).encode()),
		)
	}

	#[test]
	fn handle_export_message_from_system_parachain_add_to_outbound_queue_works() {
		bridge_hub_test_utils::test_cases::handle_export_message_from_system_parachain_to_outbound_queue_works::<
			Runtime,
			XcmConfig,
			WithBridgeHubRococoMessagesInstance,
		>(
			collator_session_keys(),
			bp_bridge_hub_wococo::BRIDGE_HUB_WOCOCO_PARACHAIN_ID,
			1000,
			Box::new(|runtime_event_encoded: Vec<u8>| {
				match RuntimeEvent::decode(&mut &runtime_event_encoded[..]) {
					Ok(RuntimeEvent::BridgeRococoMessages(event)) => Some(event),
					_ => None,
				}
			}),
			|| ExportMessage { network: Rococo, destination: X1(Parachain(4321)), xcm: Xcm(vec![]) },
			bridge_hub_wococo_config::DEFAULT_XCM_LANE_TO_BRIDGE_HUB_ROCOCO
		)
	}

	#[test]
	fn message_dispatch_routing_works() {
		bridge_hub_test_utils::test_cases::message_dispatch_routing_works::<
			Runtime,
			XcmConfig,
			ParachainSystem,
			WithBridgeHubRococoMessagesInstance,
			RelayNetwork,
			bridge_hub_wococo_config::RococoGlobalConsensusNetwork,
		>(
			collator_session_keys(),
			bp_bridge_hub_wococo::BRIDGE_HUB_WOCOCO_PARACHAIN_ID,
			1000,
			Box::new(|runtime_event_encoded: Vec<u8>| {
				match RuntimeEvent::decode(&mut &runtime_event_encoded[..]) {
					Ok(RuntimeEvent::ParachainSystem(event)) => Some(event),
					_ => None,
				}
			}),
			Box::new(|runtime_event_encoded: Vec<u8>| {
				match RuntimeEvent::decode(&mut &runtime_event_encoded[..]) {
					Ok(RuntimeEvent::XcmpQueue(event)) => Some(event),
					_ => None,
				}
			}),
			bridge_hub_wococo_config::DEFAULT_XCM_LANE_TO_BRIDGE_HUB_ROCOCO,
		)
	}

	#[test]
	fn relayed_incoming_message_works() {
		bridge_hub_test_utils::test_cases::relayed_incoming_message_works::<
			Runtime,
			XcmConfig,
			ParachainSystem,
			BridgeGrandpaRococoInstance,
			BridgeParachainRococoInstance,
			WithBridgeHubRococoMessagesInstance,
			WithBridgeHubRococoMessageBridge,
		>(
			collator_session_keys(),
			bp_bridge_hub_wococo::BRIDGE_HUB_WOCOCO_PARACHAIN_ID,
			bp_bridge_hub_rococo::BRIDGE_HUB_ROCOCO_PARACHAIN_ID,
			1000,
			Wococo,
			LaneId([0, 0, 0, 1]),
		)
	}

	#[test]
	pub fn complex_relay_extrinsic_works() {
		bridge_hub_test_utils::test_cases::complex_relay_extrinsic_works::<
			Runtime,
			XcmConfig,
			ParachainSystem,
			BridgeGrandpaRococoInstance,
			BridgeParachainRococoInstance,
			WithBridgeHubRococoMessagesInstance,
			WithBridgeHubRococoMessageBridge,
		>(
			collator_session_keys(),
			bp_bridge_hub_wococo::BRIDGE_HUB_WOCOCO_PARACHAIN_ID,
			bp_bridge_hub_rococo::BRIDGE_HUB_ROCOCO_PARACHAIN_ID,
			1000,
			bridge_hub_wococo_config::BridgeHubRococoChainId::get(),
			Wococo,
			LaneId([0, 0, 0, 1]),
			executive_init_block,
			drip_some_balance,
			construct_and_apply_extrinsic,
		);
	}
}
