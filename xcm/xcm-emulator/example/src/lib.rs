// Copyright 2023 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

use frame_support::{pallet_prelude::Weight, traits::GenesisBuild};
use sp_runtime::AccountId32;

use xcm_emulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};

// Setup relay chain.
decl_test_relay_chain! {
	pub struct KusamaRelay {
		Runtime = kusama_runtime::Runtime,
		XcmConfig = kusama_runtime::xcm_config::XcmConfig,
		new_ext = kusama_ext(),
	}
}

// Setup ParachainA.
decl_test_parachain! {
	pub struct ParachainA {
		Runtime = test_runtime::Runtime,
		RuntimeOrigin = test_runtime::RuntimeOrigin,
		XcmpMessageHandler = test_runtime::XcmpQueue,
		DmpMessageHandler = test_runtime::DmpQueue,
		new_ext = parachain_ext(1),
	}
}

// Setup ParachainB.
decl_test_parachain! {
	pub struct ParachainB {
		Runtime = test_runtime::Runtime,
		RuntimeOrigin = test_runtime::RuntimeOrigin,
		XcmpMessageHandler = test_runtime::XcmpQueue,
		DmpMessageHandler = test_runtime::DmpQueue,
		new_ext = parachain_ext(2),
	}
}

// Setup ParachainC.
decl_test_parachain! {
	pub struct ParachainC {
		Runtime = test_runtime::Runtime,
		RuntimeOrigin = test_runtime::RuntimeOrigin,
		XcmpMessageHandler = test_runtime::XcmpQueue,
		DmpMessageHandler = test_runtime::DmpQueue,
		new_ext = parachain_ext(3),
	}
}

// Setup a network with all the declared parachains and a relay chain.
decl_test_network! {
	pub struct Network {
		relay_chain = KusamaRelay,
		parachains = vec![
			(1, ParachainA),
			(2, ParachainB),
			(3, ParachainC),
		],
	}
}

pub const ALICE: AccountId32 = AccountId32::new([0u8; 32]);
pub const INITIAL_BALANCE: u128 = 1_000_000_000_000;

// Parachain TextExternalities setup.
pub fn parachain_ext(para_id: u32) -> sp_io::TestExternalities {
	use test_runtime::{Runtime, System};

	let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

	let parachain_info_config = parachain_info::GenesisConfig { parachain_id: para_id.into() };

	<parachain_info::GenesisConfig as GenesisBuild<Runtime, _>>::assimilate_storage(
		&parachain_info_config,
		&mut t,
	)
	.unwrap();

	pallet_balances::GenesisConfig::<Runtime> { balances: vec![(ALICE, INITIAL_BALANCE)] }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

fn default_parachains_host_configuration(
) -> polkadot_runtime_parachains::configuration::HostConfiguration<
	polkadot_primitives::v4::BlockNumber,
> {
	use polkadot_primitives::v4::{MAX_CODE_SIZE, MAX_POV_SIZE};

	polkadot_runtime_parachains::configuration::HostConfiguration {
		minimum_validation_upgrade_delay: 5,
		validation_upgrade_cooldown: 10u32,
		validation_upgrade_delay: 10,
		code_retention_period: 1200,
		max_code_size: MAX_CODE_SIZE,
		max_pov_size: MAX_POV_SIZE,
		max_head_data_size: 32 * 1024,
		group_rotation_frequency: 20,
		chain_availability_period: 4,
		thread_availability_period: 4,
		max_upward_queue_count: 8,
		max_upward_queue_size: 1024 * 1024,
		max_downward_message_size: 1024,
		ump_service_total_weight: Weight::from_parts(4 * 1_000_000_000, 0),
		max_upward_message_size: 50 * 1024,
		max_upward_message_num_per_candidate: 5,
		hrmp_sender_deposit: 0,
		hrmp_recipient_deposit: 0,
		hrmp_channel_max_capacity: 8,
		hrmp_channel_max_total_size: 8 * 1024,
		hrmp_max_parachain_inbound_channels: 4,
		hrmp_max_parathread_inbound_channels: 4,
		hrmp_channel_max_message_size: 1024 * 1024,
		hrmp_max_parachain_outbound_channels: 4,
		hrmp_max_parathread_outbound_channels: 4,
		hrmp_max_message_num_per_candidate: 5,
		dispute_period: 6,
		no_show_slots: 2,
		n_delay_tranches: 25,
		needed_approvals: 2,
		relay_vrf_modulo_samples: 2,
		zeroth_delay_tranche_width: 0,
		..Default::default()
	}
}

// Relay chain TestExternalities setup.
pub fn kusama_ext() -> sp_io::TestExternalities {
	use kusama_runtime::{Runtime, System};

	let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

	pallet_balances::GenesisConfig::<Runtime> { balances: vec![(ALICE, INITIAL_BALANCE)] }
		.assimilate_storage(&mut t)
		.unwrap();

	polkadot_runtime_parachains::configuration::GenesisConfig::<Runtime> {
		config: default_parachains_host_configuration(),
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

#[cfg(test)]
mod tests {
	use super::*;
	use codec::Encode;

	use cumulus_primitives_core::ParaId;
	use frame_support::{assert_ok, dispatch::GetDispatchInfo, traits::Currency};
	use sp_runtime::traits::AccountIdConversion;
	use xcm::{latest::Error::BadOrigin, v3::prelude::*, VersionedMultiLocation, VersionedXcm};
	use xcm_emulator::TestExt;

	#[test]
	fn dmp() {
		Network::reset();

		let remark = test_runtime::RuntimeCall::System(
			frame_system::Call::<test_runtime::Runtime>::remark_with_event {
				remark: "Hello from Kusama!".as_bytes().to_vec(),
			},
		);
		KusamaRelay::execute_with(|| {
			assert_ok!(kusama_runtime::XcmPallet::force_default_xcm_version(
				kusama_runtime::RuntimeOrigin::root(),
				Some(3)
			));
			assert_ok!(kusama_runtime::XcmPallet::send_xcm(
				Here,
				Parachain(1),
				Xcm(vec![Transact {
					origin_kind: OriginKind::SovereignAccount,
					require_weight_at_most: Weight::from_parts(INITIAL_BALANCE as u64, 1024 * 1024),
					call: remark.encode().into(),
				}]),
			));
		});

		ParachainA::execute_with(|| {
			use test_runtime::{RuntimeEvent, System};
			System::events().iter().for_each(|r| println!(">>> {:?}", r.event));

			assert!(System::events().iter().any(|r| matches!(
				r.event,
				RuntimeEvent::System(frame_system::Event::Remarked { sender: _, hash: _ })
			)));
		});
	}

	#[test]
	fn ump() {
		Network::reset();

		KusamaRelay::execute_with(|| {
			assert_ok!(kusama_runtime::XcmPallet::force_default_xcm_version(
				kusama_runtime::RuntimeOrigin::root(),
				Some(3)
			));
			let _ = kusama_runtime::Balances::deposit_creating(
				&ParaId::from(1).into_account_truncating(),
				1_000_000_000_000,
			);
		});

		let remark = kusama_runtime::RuntimeCall::System(frame_system::Call::<
			kusama_runtime::Runtime,
		>::remark_with_event {
			remark: "Hello from ParachainA!".as_bytes().to_vec(),
		});
		ParachainA::execute_with(|| {
			assert_ok!(test_runtime::PolkadotXcm::force_default_xcm_version(
				test_runtime::RuntimeOrigin::root(),
				Some(3)
			));
			assert_ok!(test_runtime::PolkadotXcm::send_xcm(
				Here,
				Parent,
				Xcm(vec![
					UnpaidExecution { weight_limit: Unlimited, check_origin: None },
					Transact {
						origin_kind: OriginKind::SovereignAccount,
						require_weight_at_most: Weight::from_parts(
							INITIAL_BALANCE as u64,
							1024 * 1024
						),
						call: remark.encode().into(),
					}
				]),
			));
		});

		KusamaRelay::execute_with(|| {
			use kusama_runtime::{RuntimeEvent, System};
			// TODO: https://github.com/paritytech/polkadot/pull/6824 or change this call to
			// force_create_assets like we do in cumulus integration tests.
			// assert!(System::events().iter().any(|r| matches!(
			// 	r.event,
			// 	RuntimeEvent::System(frame_system::Event::Remarked { sender: _, hash: _ })
			// )));
			assert!(System::events().iter().any(|r| matches!(
				r.event,
				RuntimeEvent::Ump(polkadot_runtime_parachains::ump::Event::ExecutedUpward(
					_,
					Outcome::Incomplete(_, XcmError::NoPermission)
				))
			)));
		});
	}

	#[test]
	fn para_to_para() {
		Network::reset();

		let remark = test_runtime::RuntimeCall::System(
			frame_system::Call::<test_runtime::Runtime>::remark_with_event {
				remark: "Hello from ParachainA!".as_bytes().to_vec(),
			},
		);
		ParachainA::execute_with(|| {
			assert_ok!(test_runtime::PolkadotXcm::send_xcm(
				Here,
				MultiLocation::new(1, X1(Parachain(2))),
				Xcm(vec![Transact {
					origin_kind: OriginKind::SovereignAccount,
					require_weight_at_most: Weight::from_parts(9_000_000, 0),
					call: remark.encode().into(),
				}]),
			));
		});

		ParachainB::execute_with(|| {
			use test_runtime::{RuntimeEvent, System};
			System::events().iter().for_each(|r| println!(">>> {:?}", r.event));

			assert!(System::events().iter().any(|r| matches!(
				r.event,
				RuntimeEvent::System(frame_system::Event::Remarked { sender: _, hash: _ })
			)));
		});
	}

	#[test]
	fn xcmp_through_a_parachain() {
		use test_runtime::{PolkadotXcm, Runtime, RuntimeCall};

		Network::reset();

		// The message goes through: ParachainA --> ParachainB --> ParachainC
		let remark = RuntimeCall::System(frame_system::Call::<Runtime>::remark_with_event {
			remark: "Hello from ParachainA!".as_bytes().to_vec(),
		});
		let send_xcm_to_parachain_c = RuntimeCall::PolkadotXcm(pallet_xcm::Call::<Runtime>::send {
			dest: Box::new(VersionedMultiLocation::V3(MultiLocation::new(1, X1(Parachain(3))))),
			message: Box::new(VersionedXcm::V3(Xcm(vec![Transact {
				origin_kind: OriginKind::SovereignAccount,
				// The weight here does not matter, as this is going to fail with BadOrigin once it
				// reaches ParachainC, as it's not going to satisfy AccountId32Aliases conversion
				// rules.
				require_weight_at_most: Weight::from_parts(0, 0),
				call: remark.encode().into(),
			}]))),
		});
		assert_eq!(
			send_xcm_to_parachain_c.get_dispatch_info().weight,
			Weight::from_parts(100000010, 10)
		);
		ParachainA::execute_with(|| {
			assert_ok!(PolkadotXcm::send_xcm(
				Here,
				MultiLocation::new(1, X1(Parachain(2))),
				Xcm(vec![Transact {
					origin_kind: OriginKind::SovereignAccount,
					require_weight_at_most: Weight::from_parts(110_000_010, 20_000),
					call: send_xcm_to_parachain_c.encode().into(),
				}]),
			));
		});

		ParachainB::execute_with(|| {
			use test_runtime::{RuntimeEvent, System};
			System::events().iter().for_each(|r| println!(">>> {:?}", r.event));

			assert!(System::events().iter().any(|r| matches!(
				r.event,
				RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent(_, _, _))
			)));
		});

		// ParachainC: The origin should not satisfy AccountId32Aliases conversion rules and thus fail.
		ParachainC::execute_with(|| {
			use test_runtime::{RuntimeEvent, System};
			System::events().iter().for_each(|r| println!(">>> {:?}", r.event));

			assert!(System::events().iter().any(|r| matches!(
				r.event,
				RuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Fail { error, .. }) if error == BadOrigin
			)));
		});
	}

	#[test]
	fn deduplicate_dmp() {
		Network::reset();
		KusamaRelay::execute_with(|| {
			assert_ok!(kusama_runtime::XcmPallet::force_default_xcm_version(
				kusama_runtime::RuntimeOrigin::root(),
				Some(3)
			));
		});

		send_remark_with_event_to_child_parachain_a("Kusama", 2);
		parachain_a_verify_remark_event_and_reset_all_events(true);

		// a different dmp message in same relay-parent-block allow execution.
		send_remark_with_event_to_child_parachain_a("Polkadot", 1);
		parachain_a_verify_remark_event_and_reset_all_events(true);

		// same dmp message with same relay-parent-block wouldn't execution
		send_remark_with_event_to_child_parachain_a("Kusama", 1);
		parachain_a_verify_remark_event_and_reset_all_events(false);

		// different relay-parent-block allow dmp message execution
		KusamaRelay::execute_with(|| kusama_runtime::System::set_block_number(2));

		send_remark_with_event_to_child_parachain_a("Kusama", 1);
		parachain_a_verify_remark_event_and_reset_all_events(true);

		// reset can send same dmp message again
		Network::reset();
		KusamaRelay::execute_with(|| {
			assert_ok!(kusama_runtime::XcmPallet::force_default_xcm_version(
				kusama_runtime::RuntimeOrigin::root(),
				Some(3)
			));
		});

		send_remark_with_event_to_child_parachain_a("Kusama", 1);
		parachain_a_verify_remark_event_and_reset_all_events(true);
	}

	fn send_remark_with_event_to_child_parachain_a(msg: &str, count: u32) {
		let remark = test_runtime::RuntimeCall::System(
			frame_system::Call::<test_runtime::Runtime>::remark_with_event {
				remark: msg.as_bytes().to_vec(),
			},
		);
		KusamaRelay::execute_with(|| {
			for _ in 0..count {
				assert_ok!(kusama_runtime::XcmPallet::send_xcm(
					Here,
					Parachain(1),
					Xcm(vec![Transact {
						origin_kind: OriginKind::SovereignAccount,
						require_weight_at_most: Weight::from_parts(
							INITIAL_BALANCE as u64,
							1024 * 1024
						),
						call: remark.encode().into(),
					}]),
				));
			}
		});
	}

	fn parachain_a_verify_remark_event_and_reset_all_events(received: bool) {
		ParachainA::execute_with(|| {
			use test_runtime::{RuntimeEvent, System};
			System::events().iter().for_each(|r| println!(">>> {:?}", r.event));

			if received {
				assert!(System::events().iter().any(|r| matches!(
					r.event,
					RuntimeEvent::System(frame_system::Event::Remarked { sender: _, hash: _ })
				)));

				System::reset_events();
			} else {
				assert!(System::events().iter().all(|r| !matches!(
					r.event,
					RuntimeEvent::System(frame_system::Event::Remarked { sender: _, hash: _ })
				)));
			}
		});
	}
}
