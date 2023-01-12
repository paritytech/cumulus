// Copyright (C) 2023 Parity Technologies (UK) Ltd.
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

//! The pallet benchmarks.

use super::{Pallet as CollectiveContent, *};
use frame_benchmarking::benchmarks;
use frame_support::traits::{EnsureOrigin, UnfilteredDispatchable};
use sp_core::Get;
use sp_std::vec;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
	set_charter {
		let cid: Cid = b"ipfs_hash".to_vec().try_into().unwrap();
		let call = Call::<T>::set_charter { cid: cid.clone() };
		let origin = T::CharterOrigin::successful_origin();
	}: { call.dispatch_bypass_filter(origin)? }
	verify {
		assert_eq!(CollectiveContent::<T>::charter(), Some(cid.clone()));
		assert_last_event::<T>(Event::NewCharterSet { cid }.into());
	}

	announce {
		let cid: Cid = b"ipfs_hash".to_vec().try_into().unwrap();
		let call = Call::<T>::announce { cid: cid.clone() };
		let origin = T::AnnouncementOrigin::successful_origin();
	}: { call.dispatch_bypass_filter(origin)? }
	verify {
		assert_eq!(CollectiveContent::<T>::announcements().len(), 1);
		assert_last_event::<T>(Event::AnnouncementAnnounced { cid }.into());
	}

	remove_announcement {
		let cid: Cid = b"ipfs_hash".to_vec().try_into().unwrap();
		let origin = T::AnnouncementOrigin::successful_origin();
		let max_count = T::MaxAnnouncementsCount::get() as usize;

		// fill the announcements vec for the worst case.
		let announcements = vec![cid.clone(); max_count];
		let announcements: BoundedVec<_, T::MaxAnnouncementsCount> = BoundedVec::try_from(announcements).unwrap();
		Announcements::<T>::put(announcements);
		assert_eq!(CollectiveContent::<T>::announcements().len(), max_count);

		let call = Call::<T>::remove_announcement { cid: cid.clone() };
	}: { call.dispatch_bypass_filter(origin)? }
	verify {
		assert_eq!(CollectiveContent::<T>::announcements().len(), max_count - 1);
		assert_last_event::<T>(Event::AnnouncementRemoved { cid }.into());
	}

	impl_benchmark_test_suite!(CollectiveContent, super::mock::new_bench_ext(), super::mock::Test);
}
