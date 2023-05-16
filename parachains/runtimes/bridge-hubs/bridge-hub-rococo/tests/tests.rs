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
use bridge_hub_rococo_runtime::{
	bridge_hub_rococo_config, bridge_hub_wococo_config, BridgeRelayers,
	WithBridgeHubRococoMessagesInstance, WithBridgeHubWococoMessagesInstance,
};
pub use bridge_hub_rococo_runtime::{
	constants::fee::WeightToFee,
	xcm_config::{RelayNetwork, XcmConfig, XcmRouter},
	Balances, BridgeGrandpaRococoInstance, BridgeGrandpaWococoInstance,
	BridgeParachainWococoInstance, BridgeWococoMessages, ExistentialDeposit, ParachainSystem,
	PolkadotXcm, Runtime, RuntimeCall, RuntimeEvent, SessionKeys,
};
use codec::{Decode, Encode};
use pallet_bridge_grandpa::BridgedHeader;
use sp_keyring::AccountKeyring::*;
use sp_runtime::AccountId32;
use xcm::latest::prelude::*;

use frame_support::{dispatch::Dispatchable, parameter_types};
use parachains_common::{AccountId, AuraId};

const ALICE: [u8; 32] = [1u8; 32];

parameter_types! {
	pub CheckingAccount: AccountId = PolkadotXcm::check_account();
}

mod bridge_hub_rococo_tests {
	use super::*;
	use bp_messages::Weight;
	use bp_parachains::{BestParaHeadHash, ParaInfo};
	use bp_polkadot_core::{parachains::ParaId, Header, Signature};
	use bp_relayers::{RewardsAccountOwner, RewardsAccountParams};
	use bridge_hub_rococo_config::WithBridgeHubWococoMessageBridge;
	use bridge_hub_rococo_runtime::{
		bridge_hub_rococo_config::BridgeRefundBridgeHubWococoMessages,
		bridge_hub_wococo_config::BridgeRefundBridgeHubRococoMessages,
		BridgeRejectObsoleteHeadersAndMessages, Executive, SignedExtra, UncheckedExtrinsic,
	};
	use bridge_hub_test_utils::test_cases::test_data;
	use frame_support::{assert_ok, dispatch::RawOrigin};
	use sp_core::{Pair, H256};
	use sp_runtime::{
		generic::{Era, SignedPayload},
		traits::Header as HeaderT,
	};

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
		bp_bridge_hub_rococo::BRIDGE_HUB_ROCOCO_PARACHAIN_ID
	);

	bridge_hub_test_utils::include_initialize_bridge_by_governance_works!(
		Runtime,
		BridgeGrandpaWococoInstance,
		bridge_hub_test_utils::CollatorSessionKeys::new(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) }
		),
		bp_bridge_hub_rococo::BRIDGE_HUB_ROCOCO_PARACHAIN_ID,
		Box::new(|call| RuntimeCall::BridgeWococoGrandpa(call).encode())
	);

	bridge_hub_test_utils::include_handle_export_message_from_system_parachain_to_outbound_queue_works!(
		Runtime,
		XcmConfig,
		WithBridgeHubWococoMessagesInstance,
		bridge_hub_test_utils::CollatorSessionKeys::new(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) }
		),
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
	);

	bridge_hub_test_utils::include_message_dispatch_routing_works!(
		Runtime,
		XcmConfig,
		ParachainSystem,
		WithBridgeHubWococoMessagesInstance,
		RelayNetwork,
		bridge_hub_rococo_config::WococoGlobalConsensusNetwork,
		bridge_hub_test_utils::CollatorSessionKeys::new(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) }
		),
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
		bridge_hub_rococo_config::DEFAULT_XCM_LANE_TO_BRIDGE_HUB_WOCOCO
	);

	bridge_hub_test_utils::include_relayed_incoming_message_works!(
		Runtime,
		XcmConfig,
		ParachainSystem,
		BridgeGrandpaWococoInstance,
		BridgeParachainWococoInstance,
		WithBridgeHubWococoMessagesInstance,
		WithBridgeHubWococoMessageBridge,
		bridge_hub_test_utils::CollatorSessionKeys::new(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) }
		),
		bp_bridge_hub_rococo::BRIDGE_HUB_ROCOCO_PARACHAIN_ID,
		bp_bridge_hub_wococo::BRIDGE_HUB_WOCOCO_PARACHAIN_ID,
		1000,
		LaneId([0, 0, 0, 0]),
		Rococo,
	);

	pub fn construct_extrinsic(
		sender: sp_core::sr25519::Pair,
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
				BridgeRefundBridgeHubRococoMessages::default(),
				BridgeRefundBridgeHubWococoMessages::default(),
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

	#[test]
	pub fn complex_relay_extrinsic_works() {
		use asset_test_utils::{mock_open_hrmp_channel, ExtBuilder};

		test(
			bridge_hub_test_utils::CollatorSessionKeys::new(
				AccountId::from(ALICE),
				AccountId::from(ALICE),
				SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) },
			),
			bp_bridge_hub_rococo::BRIDGE_HUB_ROCOCO_PARACHAIN_ID,
			bp_bridge_hub_wococo::BRIDGE_HUB_WOCOCO_PARACHAIN_ID,
			1000,
			LaneId([0, 0, 0, 1]),
			Rococo,
		);

		fn test(
			collator_session_key: bridge_hub_test_utils::CollatorSessionKeys<Runtime>,
			runtime_para_id: u32,
			bridged_para_id: u32,
			sibling_parachain_id: u32,
			lane_id: LaneId,
			local_relay_chain_id: NetworkId,
		) {
			assert_ne!(runtime_para_id, sibling_parachain_id);
			assert_ne!(runtime_para_id, bridged_para_id);

			ExtBuilder::<Runtime>::default()
				.with_collators(collator_session_key.collators())
				.with_session_keys(collator_session_key.session_keys())
				.with_safe_xcm_version(XCM_VERSION)
				.with_para_id(runtime_para_id.into())
				.with_tracing()
				.build()
				.execute_with(|| {
					let genesis_hash = frame_system::Pallet::<Runtime>::block_hash(0u32);
					let header = Header::new(
						1,
						H256::default(),
						H256::default(),
						genesis_hash,
						Default::default(),
					);
					Executive::initialize_block(&header);

					mock_open_hrmp_channel::<Runtime, ParachainSystem>(
						runtime_para_id.into(),
						sibling_parachain_id.into(),
					);

					// start with bridged chain block#0
					let init_data =
						test_data::initialization_data::<Runtime, BridgeGrandpaWococoInstance>(0);
					pallet_bridge_grandpa::Pallet::<Runtime, BridgeGrandpaWococoInstance>::initialize(
						<Runtime as frame_system::Config>::RuntimeOrigin::root(),
						init_data,
					)
					.unwrap();

					// set up relayer details and proofs

					let message_destination =
						X2(GlobalConsensus(local_relay_chain_id), Parachain(sibling_parachain_id));
					// some random numbers (checked by test)
					let message_nonce = 1;
					let para_header_number = 5;
					let relay_header_number = 1;

					let relayer_at_target = Bob.pair();
					let relayer_id_on_target: <Runtime as frame_system::Config>::AccountId =
						relayer_at_target.public().into();
					let relayer_at_source = Dave.pair();
					let relayer_id_on_source: sp_runtime::AccountId32 =
						relayer_at_source.public().into();

					// Drip some balance
					use frame_support::traits::fungible::Mutate;
					let some_currency = ExistentialDeposit::get() * 100000;
					Balances::mint_into(&relayer_id_on_target, some_currency).unwrap();

					let xcm = vec![xcm::v3::Instruction::<()>::ClearOrigin; 42];
					let expected_dispatch = xcm::VersionedXcm::<()>::V3(xcm.clone().into());
					// generate bridged relay chain finality, parachain heads and message proofs,
					// to be submitted by relayer to this chain.
					let (
						relay_chain_header,
						grandpa_justification,
						bridged_para_head,
						parachain_heads,
						para_heads_proof,
						message_proof,
					) = test_data::make_complex_relayer_proofs::<
						BridgedHeader<Runtime, BridgeGrandpaWococoInstance>,
						WithBridgeHubWococoMessageBridge,
						(),
					>(
						lane_id,
						xcm.into(),
						message_nonce,
						message_destination,
						para_header_number,
						relay_header_number,
						bridged_para_id,
					);

					let submit_grandpa = pallet_bridge_grandpa::Call::<
						Runtime,
						BridgeGrandpaWococoInstance,
					>::submit_finality_proof {
						finality_target: Box::new(relay_chain_header.clone()),
						justification: grandpa_justification,
					};
					let submit_para_head = pallet_bridge_parachains::Call::<
						Runtime,
						BridgeParachainWococoInstance,
					>::submit_parachain_heads {
						at_relay_block: (relay_header_number, relay_chain_header.hash().into()),
						parachains: parachain_heads,
						parachain_heads_proof: para_heads_proof,
					};
					let submit_message = pallet_bridge_messages::Call::<
						Runtime,
						WithBridgeHubWococoMessagesInstance,
					>::receive_messages_proof {
						relayer_id_at_bridged_chain: relayer_id_on_source,
						proof: message_proof.into(),
						messages_count: 1,
						dispatch_weight: Weight::from_parts(1000000000, 0),
					};
					let batch_call =
						RuntimeCall::Utility(pallet_utility::Call::<Runtime>::batch_all {
							calls: vec![
								submit_grandpa.into(),
								submit_para_head.into(),
								submit_message.into(),
							],
						});

					// sanity checks - before relayer extrinsic
					use cumulus_primitives_core::XcmpMessageSource;
					assert!(cumulus_pallet_xcmp_queue::Pallet::<Runtime>::take_outbound_messages(
						usize::MAX
					)
					.is_empty());
					assert_eq!(
						pallet_bridge_messages::InboundLanes::<
							Runtime,
							WithBridgeHubWococoMessagesInstance,
						>::get(lane_id)
						.last_delivered_nonce(),
						0,
					);
					let msg_proofs_rewards_account = RewardsAccountParams::new(
						lane_id,
						bridge_hub_rococo_config::BridgeHubWococoChainId::get(),
						RewardsAccountOwner::ThisChain,
					);
					assert_eq!(
						BridgeRelayers::relayer_reward(
							relayer_id_on_target.clone(),
							msg_proofs_rewards_account
						),
						None,
					);

					// construct and apply extrinsic containing batch calls:
					//   bridged relay chain finality proof
					//   + parachain heads proof
					//   + submit message proof
					let xt = construct_extrinsic(relayer_at_target, batch_call);
					let r = Executive::apply_extrinsic(xt);
					let dispatch_res = r.unwrap();

					// verify finality proof correctly imported
					assert_ok!(dispatch_res);
					assert_eq!(
						<pallet_bridge_grandpa::BestFinalized<Runtime, BridgeGrandpaWococoInstance>>::get().unwrap().1,
						relay_chain_header.hash()
					);
					assert!(<pallet_bridge_grandpa::ImportedHeaders<
						Runtime,
						BridgeGrandpaWococoInstance,
					>>::contains_key(relay_chain_header.hash()));
					// verify parachain head proof correctly imported
					assert_eq!(
						pallet_bridge_parachains::ParasInfo::<Runtime, BridgeParachainWococoInstance>::get(ParaId(
							bridged_para_id
						)),
						Some(ParaInfo {
							best_head_hash: BestParaHeadHash {
								at_relay_block_number: relay_header_number,
								head_hash: bridged_para_head.hash()
							},
							next_imported_hash_position: 1,
						})
					);
					// verify message correctly imported and dispatched
					assert_eq!(
						pallet_bridge_messages::InboundLanes::<
							Runtime,
							WithBridgeHubWococoMessagesInstance,
						>::get(lane_id)
						.last_delivered_nonce(),
						1,
					);
					// verify relayer is refunded
					assert!(BridgeRelayers::relayer_reward(
						relayer_id_on_target,
						msg_proofs_rewards_account
					)
					.is_some());
					// verify relayed bridged XCM message is dispatched to destination sibling para
					let dispatched = test_data::take_outbound_message::<
						cumulus_pallet_xcmp_queue::Pallet<Runtime>,
					>(sibling_parachain_id.into())
					.unwrap();
					assert_eq!(dispatched, expected_dispatch);
				})
		}
	}

	#[test]
	fn relayed_batch_works() {
		let batch_call = Box::new(
			|relayer_id_on_target: <Runtime as frame_system::Config>::AccountId,
			 submit_grandpa: pallet_bridge_grandpa::Call<Runtime, BridgeGrandpaWococoInstance>,
			 submit_para_head: pallet_bridge_parachains::Call<
				Runtime,
				BridgeParachainWococoInstance,
			>,
			 submit_message: pallet_bridge_messages::Call<
				Runtime,
				WithBridgeHubWococoMessagesInstance,
			>| {
				RuntimeCall::Utility(pallet_utility::Call::<Runtime>::batch_all {
					calls: vec![
						submit_grandpa.into(),
						submit_para_head.into(),
						submit_message.into(),
					],
				})
				.dispatch(RawOrigin::Signed(relayer_id_on_target).into())
				.unwrap();
			},
		);

		bridge_hub_test_utils::test_cases::relayed_batch_works::<
			Runtime,
			XcmConfig,
			ParachainSystem,
			BridgeGrandpaWococoInstance,
			BridgeParachainWococoInstance,
			WithBridgeHubWococoMessagesInstance,
			WithBridgeHubWococoMessageBridge,
		>(
			bridge_hub_test_utils::CollatorSessionKeys::new(
				AccountId::from(ALICE),
				AccountId::from(ALICE),
				SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) },
			),
			bp_bridge_hub_rococo::BRIDGE_HUB_ROCOCO_PARACHAIN_ID,
			bp_bridge_hub_wococo::BRIDGE_HUB_WOCOCO_PARACHAIN_ID,
			1000,
			LaneId([0, 0, 0, 0]),
			Rococo,
			batch_call,
		);
	}
}

mod bridge_hub_wococo_tests {
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
		bp_bridge_hub_wococo::BRIDGE_HUB_WOCOCO_PARACHAIN_ID
	);

	bridge_hub_test_utils::include_initialize_bridge_by_governance_works!(
		Runtime,
		BridgeGrandpaRococoInstance,
		bridge_hub_test_utils::CollatorSessionKeys::new(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) }
		),
		bp_bridge_hub_wococo::BRIDGE_HUB_WOCOCO_PARACHAIN_ID,
		Box::new(|call| RuntimeCall::BridgeRococoGrandpa(call).encode())
	);

	bridge_hub_test_utils::include_handle_export_message_from_system_parachain_to_outbound_queue_works!(
		Runtime,
		XcmConfig,
		WithBridgeHubRococoMessagesInstance,
		bridge_hub_test_utils::CollatorSessionKeys::new(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) }
		),
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
	);

	bridge_hub_test_utils::include_message_dispatch_routing_works!(
		Runtime,
		XcmConfig,
		ParachainSystem,
		WithBridgeHubRococoMessagesInstance,
		RelayNetwork,
		bridge_hub_wococo_config::RococoGlobalConsensusNetwork,
		bridge_hub_test_utils::CollatorSessionKeys::new(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) }
		),
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
		bridge_hub_wococo_config::DEFAULT_XCM_LANE_TO_BRIDGE_HUB_ROCOCO
	);
}
