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

use bp_messages::{
	target_chain::{DispatchMessage, DispatchMessageData},
	LaneId, MessageKey,
};
use cumulus_primitives_core::{AbridgedHrmpChannel, ParaId, PersistedValidationData};
use cumulus_primitives_parachain_inherent::ParachainInherentData;
use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;
use frame_support::{
	dispatch::{RawOrigin, UnfilteredDispatchable},
	inherent::{InherentData, ProvideInherent},
	sp_io,
	traits::Get,
};
use parachains_common::AccountId;
use polkadot_parachain::primitives::{HrmpChannelId, RelayChainBlockNumber};
use xcm::{latest::prelude::*, prelude::XcmVersion};
use xcm_builder::{HaulBlob, HaulBlobError, HaulBlobExporter};
use xcm_executor::traits::{validate_export, ExportXcm};

pub use bp_test_utils::test_header;
pub mod test_cases;
pub use test_cases::CollatorSessionKeys;

/// Dummy xcm
pub fn dummy_xcm() -> Xcm<()> {
	vec![Trap(42)].into()
}

pub fn wrap_as_dispatch_message(payload: Vec<u8>) -> DispatchMessage<Vec<u8>> {
	DispatchMessage {
		key: MessageKey { lane_id: LaneId([0, 0, 0, 0]), nonce: 1 },
		data: DispatchMessageData { payload: Ok(payload) },
	}
}

/// Dummy account
pub fn dummy_account() -> AccountId {
	AccountId::from([0u8; 32])
}

/// Macro used for simulate_export_message and capturing bytes
macro_rules! grab_haul_blob (
	($name:ident, $grabbed_payload:ident) => {
		std::thread_local! {
			static $grabbed_payload: std::cell::RefCell<Option<Vec<u8>>> = std::cell::RefCell::new(None);
		}

		struct $name;
		impl HaulBlob for $name {
			fn haul_blob(blob: Vec<u8>) -> Result<(), HaulBlobError>{
				$grabbed_payload.with(|rm| *rm.borrow_mut() = Some(blob));
				Ok(())
			}
		}
	}
);

/// Simulates HaulBlobExporter and all its wrapping and captures generated plain bytes
pub fn simulate_export_message<BridgedNetwork: Get<NetworkId>>(
	sender: Junctions,
	destination_network: NetworkId,
	destination: Junctions,
	xcm: xcm::v3::Xcm<()>,
) -> Vec<u8> {
	grab_haul_blob!(GrabbingHaulBlob, GRABBED_HAUL_BLOB_PAYLOAD);

	let channel = 1_u32;
	let universal_source = sender;

	// simulate XCM message export
	let (ticket, fee) = validate_export::<HaulBlobExporter<GrabbingHaulBlob, BridgedNetwork, ()>>(
		destination_network,
		channel,
		universal_source,
		destination,
		xcm,
	)
	.expect("validate_export error");
	println!("[MessageExporter::fee] {:?}", fee);
	let result = HaulBlobExporter::<GrabbingHaulBlob, BridgedNetwork, ()>::deliver(ticket)
		.expect("deliver error");
	println!("[MessageExporter::deliver] {:?}", result);

	GRABBED_HAUL_BLOB_PAYLOAD.with(|r| r.take().expect("xcm::ExportMessage should be here"))
}

/// Initialize runtime/externalities
pub fn new_test_ext<T: frame_system::Config + pallet_xcm::Config + parachain_info::Config>(
	para_id: ParaId,
	xcm_version: XcmVersion,
) -> sp_io::TestExternalities {
	frame_support::sp_tracing::try_init_simple();

	let mut t = frame_system::GenesisConfig::default().build_storage::<T>().unwrap();
	<pallet_xcm::GenesisConfig as frame_support::traits::GenesisBuild<T>>::assimilate_storage(
		&pallet_xcm::GenesisConfig { safe_xcm_version: Some(xcm_version) },
		&mut t,
	)
	.unwrap();
	<parachain_info::GenesisConfig as frame_support::traits::GenesisBuild<T>>::assimilate_storage(
		&parachain_info::GenesisConfig { parachain_id: para_id },
		&mut t,
	)
	.unwrap();

	sp_io::TestExternalities::new(t)
}

/// Helper function which emulates opening HRMP channel which is needed for XcmpQueue xcm router to pass
pub fn mock_open_hrmp_channel<
	C: cumulus_pallet_parachain_system::Config,
	T: ProvideInherent<Call = cumulus_pallet_parachain_system::Call<C>>,
>(
	sender: ParaId,
	recipient: ParaId,
) {
	let n = 1_u32;
	let mut sproof_builder = RelayStateSproofBuilder::default();
	sproof_builder.para_id = sender;
	sproof_builder.hrmp_channels.insert(
		HrmpChannelId { sender, recipient },
		AbridgedHrmpChannel {
			max_capacity: 10,
			max_total_size: 10_000_000_u32,
			max_message_size: 10_000_000_u32,
			msg_count: 10,
			total_size: 10_000_000_u32,
			mqc_head: None,
		},
	);
	sproof_builder.hrmp_egress_channel_index = Some(vec![recipient]);

	let (relay_parent_storage_root, relay_chain_state) = sproof_builder.into_state_root_and_proof();
	let vfp = PersistedValidationData {
		relay_parent_number: n as RelayChainBlockNumber,
		relay_parent_storage_root,
		..Default::default()
	};
	// It is insufficient to push the validation function params
	// to storage; they must also be included in the inherent data.
	let inherent_data = {
		let mut inherent_data = InherentData::default();
		let system_inherent_data = ParachainInherentData {
			validation_data: vfp.clone(),
			relay_chain_state,
			downward_messages: Default::default(),
			horizontal_messages: Default::default(),
		};
		inherent_data
			.put_data(
				cumulus_primitives_parachain_inherent::INHERENT_IDENTIFIER,
				&system_inherent_data,
			)
			.expect("failed to put VFP inherent");
		inherent_data
	};

	// execute the block
	T::create_inherent(&inherent_data)
		.expect("got an inherent")
		.dispatch_bypass_filter(RawOrigin::None.into())
		.expect("dispatch succeeded");
}

pub type RelayBlockNumber = bp_polkadot_core::BlockNumber;
pub type RelayBlockHasher = bp_polkadot_core::Hasher;
pub type RelayBlockHeader = sp_runtime::generic::Header<RelayBlockNumber, RelayBlockHasher>;

/// Helper that creates InitializationData mock data, that can be used to initialize bridge GRANDPA pallet
pub fn mock_initialiation_data() -> bp_header_chain::InitializationData<RelayBlockHeader> {
	use sp_runtime::traits::Header;
	use std::str::FromStr;

	let header = RelayBlockHeader::new(
		75,
		bp_polkadot_core::Hash::from_str(
			"0xd2c0afaab32de0cb8f7f0d89217e37c5ea302c1ffb5a7a83e10d20f12c32874d",
		)
		.expect("invalid value"),
		bp_polkadot_core::Hash::from_str(
			"0x92b965f0656a4e0e5fc0167da2d4b5ee72b3be2c1583c4c1e5236c8c12aa141b",
		)
		.expect("invalid value"),
		bp_polkadot_core::Hash::from_str(
			"0xae4a25acf250d72ed02c149ecc7dd3c9ee976d41a2888fc551de8064521dc01d",
		)
		.expect("invalid value"),
		Default::default(),
	);
	bp_header_chain::InitializationData {
		header: Box::new(header),
		authority_list: Default::default(),
		set_id: 6,
		operating_mode: bp_runtime::BasicOperatingMode::Normal,
	}
}
