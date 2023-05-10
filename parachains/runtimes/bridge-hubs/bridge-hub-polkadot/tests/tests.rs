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

pub use bridge_hub_polkadot_runtime::{
	bridge_hub_config,
	constants::fee::WeightToFee,
	xcm_config::{RelayNetwork, XcmConfig},
	Balances, BridgeGrandpaKusamaInstance, ExistentialDeposit, ParachainSystem, PolkadotXcm,
	Runtime, RuntimeCall, RuntimeEvent, SessionKeys, WithBridgeHubKusamaMessagesInstance,
};
use codec::{Decode, Encode};
use frame_support::parameter_types;
use parachains_common::{AccountId, AuraId};
use xcm::latest::prelude::*;

const ALICE: [u8; 32] = [1u8; 32];

parameter_types! {
	pub CheckingAccount: AccountId = PolkadotXcm::check_account();
	pub RuntimeNetwork: NetworkId = RelayNetwork::get().unwrap();
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
	1002
);

bridge_hub_test_utils::include_initialize_bridge_by_governance_works!(
	Runtime,
	BridgeGrandpaKusamaInstance,
	bridge_hub_test_utils::CollatorSessionKeys::new(
		AccountId::from(ALICE),
		AccountId::from(ALICE),
		SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) }
	),
	bp_bridge_hub_polkadot::BRIDGE_HUB_POLKADOT_PARACHAIN_ID,
	Box::new(|call| RuntimeCall::BridgeKusamaGrandpa(call).encode())
);

bridge_hub_test_utils::include_handle_export_message_from_system_parachain_to_outbound_queue_works!(
	Runtime,
	XcmConfig,
	WithBridgeHubKusamaMessagesInstance,
	bridge_hub_test_utils::CollatorSessionKeys::new(
		AccountId::from(ALICE),
		AccountId::from(ALICE),
		SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) }
	),
	bp_bridge_hub_polkadot::BRIDGE_HUB_POLKADOT_PARACHAIN_ID,
	1000,
	Box::new(|runtime_event_encoded: Vec<u8>| {
		match RuntimeEvent::decode(&mut &runtime_event_encoded[..]) {
			Ok(RuntimeEvent::BridgeKusamaMessages(event)) => Some(event),
			_ => None,
		}
	}),
	|| ExportMessage { network: Kusama, destination: X1(Parachain(1234)), xcm: Xcm(vec![]) },
	bridge_hub_config::DEFAULT_XCM_LANE_TO_BRIDGE_HUB_KUSAMA
);

bridge_hub_test_utils::include_message_dispatch_routing_works!(
	Runtime,
	XcmConfig,
	ParachainSystem,
	WithBridgeHubKusamaMessagesInstance,
	RuntimeNetwork,
	bridge_hub_config::KusamaGlobalConsensusNetwork,
	bridge_hub_test_utils::CollatorSessionKeys::new(
		AccountId::from(ALICE),
		AccountId::from(ALICE),
		SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) }
	),
	bp_bridge_hub_polkadot::BRIDGE_HUB_POLKADOT_PARACHAIN_ID,
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
	bridge_hub_config::DEFAULT_XCM_LANE_TO_BRIDGE_HUB_KUSAMA
);
