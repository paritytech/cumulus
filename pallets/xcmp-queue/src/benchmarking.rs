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

//! Benchmarking setup for cumulus-pallet-xcmp-queue

use crate::*;

use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	/// Modify any of the `QueueConfig` fields with a new `u32` value.
	///
	/// Used as weight for:
	/// - update_suspend_threshold
	/// - update_drop_threshold
	/// - update_resume_threshold
	#[benchmark]
	fn set_config_with_u32() {
		#[extrinsic_call]
		Pallet::<T>::update_resume_threshold(RawOrigin::Root, 100);
	}

	#[benchmark]
	fn enqueue_xcmp_messages(n: Linear<0, 1000> /* FAIL-CI */) {
		let msg = BoundedVec::<u8, XcmOverHrmpMaxLenOf<T>>::default();
		let msgs = vec![msg; n as usize];

		QueueConfig::<T>::mutate(|data| {
			data.suspend_threshold = 1000;
			data.drop_threshold = data.suspend_threshold * 2;
			data.validate::<T>().unwrap();
		});

		#[block]
		{
			Pallet::<T>::enqueue_xcmp_messages(0.into(), msgs, &mut WeightMeter::max_limit());
		}
	}

	/// Benchmark `process_message` without the `XcmpProcessor` callback.
	#[benchmark]
	fn process_message() {
		let msg = vec![0u8; 1024];

		#[block]
		{
			Pallet::<T>::process_message(msg.as_slice(), 0.into(), &mut WeightMeter::max_limit())
				.unwrap();
		}
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
