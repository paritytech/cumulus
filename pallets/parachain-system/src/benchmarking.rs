// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Benchmarking setup for pallet-collator-selection

#![allow(unused)]

use super::*;

use crate::{Pallet, ValidationData};
use cumulus_primitives_core::relay_chain::v2::HeadData;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{
	assert_ok,
	codec::Decode,
	traits::{Currency, EnsureOrigin, Get},
};
use frame_system::{EnsureNone, EnsureSigned, EventRecord, RawOrigin};
use sp_core::H256;
use sp_std::{collections::btree_set::BTreeSet, prelude::*};
use sp_trie::StorageProof;

const SEED: u32 = 0;

benchmarks! {
	set_validation_data_no_messages {
		// root and proof are obtained via `RelayStateSproofBuilder::default().into_state_root_and_proof()`
		let root = [87, 247, 132, 202, 60, 197, 129, 248, 29, 30, 75, 209, 87, 82, 217, 27, 193, 91, 133, 158, 57, 219, 5, 125, 31, 120, 73, 108, 83, 161, 122, 141];
		let proof = vec![
			vec![0, 0, 32, 0, 0, 0, 16, 0, 8, 0, 0, 0, 0, 4, 0, 0, 0, 1, 0, 0, 5, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 6, 0, 0, 0],
			vec![63, 32, 6, 222, 61, 138, 84, 210, 126, 68, 169, 213, 206, 24, 150, 24, 242, 45, 180, 180, 157, 149, 50, 13, 144, 33, 153, 76, 133, 15, 37, 184, 227, 133, 122, 95, 83, 148, 165, 203, 236, 87, 253, 11, 60, 82, 36, 95, 199, 120, 54, 22, 228, 227, 110, 231, 204, 83, 94, 179, 154, 8, 200, 180, 89, 235],
			vec![127, 0, 12, 182, 243, 110, 2, 122, 187, 32, 145, 207, 181, 17, 10, 181, 8, 127, 6, 21, 91, 60, 217, 168, 201, 229, 233, 162, 63, 213, 220, 19, 165, 237, 32, 0, 0, 0, 0, 0, 0, 0, 0],
			vec![128, 3, 0, 128, 242, 238, 200, 199, 170, 3, 233, 7, 253, 11, 161, 204, 28, 109, 207, 110, 116, 240, 105, 181, 38, 64, 118, 111, 3, 222, 55, 184, 183, 81, 181, 61, 128, 30, 47, 17, 234, 172, 118, 214, 115, 102, 113, 126, 126, 111, 149, 151, 252, 216, 113, 234, 7, 17, 48, 53, 220, 0, 90, 170, 247, 242, 1, 69, 8],
		];

		let data = ParachainInherentData {
			validation_data: PersistedValidationData {
				parent_head: vec![].into(),
				relay_parent_number: 1,
				relay_parent_storage_root: H256::from_slice(&root),
				max_pov_size: Default::default(),
			},
			relay_chain_state: StorageProof::new(proof),
			downward_messages: vec![],
			horizontal_messages: BTreeMap::new(),
		};
	}: {
		assert_ok!(
			<Pallet<T>>::set_validation_data(EnsureNone::successful_origin(), data)
		);
	}
	verify {
		assert_eq!(<ValidationData<T>>::get().unwrap().relay_parent_number, 1);
	}
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test,);
