// This file is part of Substrate.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
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

//! Benchmarks for the Session Pallet.
// This is separated into its own crate due to cyclic dependency issues.

#![cfg_attr(not(feature = "std"), no_std)]

mod mock;

use sp_std::prelude::*;
use sp_std::vec;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{
	traits::{Currency},
};
use frame_system::RawOrigin;
use pallet_session::*;
use pallet_collator_selection;
pub struct Pallet<T: Config>(pallet_session::Pallet<T>);
pub trait Config: pallet_session::Config + pallet_collator_selection::Config {}


benchmarks! {
	set_keys {
		let caller = whitelisted_caller();
		T::Currency::make_free_balance_be(&caller, T::Currency::minimum_balance() * 10u32.into());
		let _r = pallet_collator_selection::Pallet::<T>::set_desired_candidates(RawOrigin::Signed(caller.clone()).into(), 10);
		let _s = pallet_collator_selection::Pallet::<T>::register_as_candidate(RawOrigin::Signed(caller.clone()).into());
		let keys = T::Keys::default();
		let proof: Vec<u8> = vec![0,1,2,3];

	}: _(RawOrigin::Signed(caller), keys, proof)

	purge_keys {
		let caller = whitelisted_caller();
		let keys = T::Keys::default();
		let proof: Vec<u8> = vec![0,1,2,3];
		T::Currency::make_free_balance_be(&caller, T::Currency::minimum_balance() * 10u32.into());
		let _r = pallet_collator_selection::Pallet::<T>::set_desired_candidates(RawOrigin::Signed(caller.clone()).into(), 10);
		let _s = pallet_collator_selection::Pallet::<T>::register_as_candidate(RawOrigin::Signed(caller.clone()).into());
		let _t = pallet_session::Pallet::<T>::set_keys(RawOrigin::Signed(caller.clone()).into(), keys, proof);
	}: _(RawOrigin::Signed(caller))

}
impl_benchmark_test_suite!(
	Pallet,
	crate::mock::new_test_ext(),
	crate::mock::Test,
	extra = false,
);
