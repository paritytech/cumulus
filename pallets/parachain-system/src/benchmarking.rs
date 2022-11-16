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

//! Benchmarks for the `parachain-system` pallet.

use crate::*;

use codec::Decode;
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;

benchmarks! {
	authorize_upgrade {}: _(RawOrigin::Root, Default::default())

	sudo_send_upward_message {
		let l in 0 .. host_config().max_upward_message_size;
		HostConfiguration::<T>::put(host_config());

		// Populate the queue and leave space for one more message.
		for _ in 1..host_config().max_upward_queue_count {
			let msg = vec![255u8; host_config().max_upward_message_size as usize];
			PendingUpwardMessages::<T>::append(msg);
		}

	}: _(RawOrigin::Root, vec![255u8; l as usize])

	enact_authorized_upgrade {
		let s in 0 .. host_config().max_code_size;
		HostConfiguration::<T>::put(host_config());

		let code = vec![255u8; s as usize];
		let code_hash = T::Hashing::hash(&code);
		AuthorizedUpgrade::<T>::put(code_hash);

		// Set the needed storage values.
		RelaychainBlockNumberProvider::<T>::set_block_number(1);
	}: _(RawOrigin::Root, code)

	set_validation_data {
		// Copy the constant before taking `&mut` to make Clippy happy.
		let mut raw_inherent_data = PARA_INHERENT_DATA;
		let para_inherent_data = ParachainInherentData::decode(&mut raw_inherent_data).unwrap();
	}: _(RawOrigin::None, para_inherent_data)

	impl_benchmark_test_suite!(Pallet, crate::tests::new_test_ext(), crate::tests::Test);
}

// TODO does this need to be injected by the runtime?
// It needs to be compile-time available for the benchmarking.
const fn host_config() -> cumulus_primitives_core::AbridgedHostConfiguration {
	cumulus_primitives_core::AbridgedHostConfiguration {
		max_code_size: 2 * 1024 * 1024,
		max_head_data_size: 1024 * 1024,
		max_upward_queue_count: 8,
		max_upward_queue_size: 1024,
		max_upward_message_size: 256,
		max_upward_message_num_per_candidate: 5,
		hrmp_max_message_num_per_candidate: 5,
		validation_upgrade_cooldown: 6,
		validation_upgrade_delay: 6,
	}
}

/// Tests that `PARA_INHERENT_DATA` is good.
#[test]
fn para_inherent_data_is_good() {
	let data_provider =
		cumulus_primitives_parachain_inherent::MockValidationDataInherentDataProvider::<()> {
			current_para_block: 2,
			relay_offset: 1,
			relay_blocks_per_para_block: 2,
			para_blocks_per_relay_epoch: 4,
			relay_randomness_config: (),
			xcm_config: Default::default(),
			raw_downward_messages: Default::default(),
			raw_horizontal_messages: Default::default(),
		};
	let para_inherent_data = data_provider.provide_para_inherent_data();
	// NOTE: If this test fails, just modify the `PARA_INHERENT_DATA` constant.
	assert_eq!(para_inherent_data.encode(), PARA_INHERENT_DATA, "PARA_INHERENT_DATA is invalid");
}
/// Contains an encoded and known-good `ParachainInherentData` for benchmarking purposes.
///
/// Proof generation requires std; `sp_trie::StateMachine::prove_read` in particular.
/// We therefore use a hard-coded value. This value is checked in a test.
const PARA_INHERENT_DATA: &'static [u8] = &hex_literal::hex!("0005000000f9c8346fc133e6b5479cdb5976ba53e8a735d5dc6c022257a3b93b36ef63c1e200000000189000002000000010000800000000040000000100000500000005000000060000000600000009013f2006de3d8a54d27e44a9d5ce189618f22db4b49d95320d9021994c850f25b8e3857a5f5394a5cbec57fd0b3c52245fc7783616e4e36ee7cc535eb39a08c8b459ebc85f0ce678799d3eff024253b90e84927cc68000000000000000020000000000000000000000000000000000000000000000003d017f1803f78c98723ddc9073523ef3beefda0c4d7fefc408aac59dbfe80a72ac8e3ce5b4def25cfda6ef3a00000000800000000000000000000000000000000000000000000000000000000000000000990180430080f2eec8c7aa03e907fd0ba1cc1c6dcf6e74f069b52640766f03de37b8b751b53d80548f64860878eb240e1c677f2f0527ebf6a146a3fc206249039ae093aa3737b880f09aa7114b573120db89867240a7262f85ee6592fb5d9c89368310e989de031da9019f0cb6f36e027abb2091cfb5110ab5087f8900685f06155b3cd9a8c9e5e9a23fd5dc13a5ed200000000000000000685f08316cbf8fa0da822a20ac1c55bf1be32000000000000000008062dd33c055efc9c5f6d66689e59d44d34be68536e63b4ce401c7b4dc3b8d8b300000");
