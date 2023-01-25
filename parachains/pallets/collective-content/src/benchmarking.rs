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

use super::{DispatchTimeFor, Pallet as CollectiveContent, *};
use frame_benchmarking::benchmarks_instance_pallet;
use frame_support::traits::{EnsureOrigin, UnfilteredDispatchable};
use sp_core::Get;
use sp_std::vec;

fn assert_last_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks_instance_pallet! {
	set_charter {
		let cid: Cid = b"ipfs_hash".to_vec().try_into().unwrap();
		let call = Call::<T, I>::set_charter { cid: cid.clone() };
		let origin = T::CharterOrigin::successful_origin();
	}: { call.dispatch_bypass_filter(origin)? }
	verify {
		assert_eq!(CollectiveContent::<T, I>::charter(), Some(cid.clone()));
		assert_last_event::<T, I>(Event::NewCharterSet { cid }.into());
	}

	announce {
		let x in 0 .. 1;

		let mut maybe_expire = None;
		if x == 1 {
			maybe_expire = Some(DispatchTimeFor::<T>::At(10u32.into()));
		}
		let now = frame_system::Pallet::<T>::block_number();
		let cid: Cid = b"ipfs_hash".to_vec().try_into().unwrap();
		let call = Call::<T, I>::announce {
			cid: cid.clone(),
			maybe_expire: maybe_expire.clone(),
		};
		let origin = T::AnnouncementOrigin::successful_origin();
	}: { call.dispatch_bypass_filter(origin)? }
	verify {
		assert_eq!(CollectiveContent::<T, I>::announcements().len(), 1);
		assert_eq!(NextAnnouncementExpire::<T, I>::get().map_or(0, |_| 1), x);
		assert_last_event::<T, I>(Event::AnnouncementAnnounced {
			cid,
			maybe_expire_at: maybe_expire.map_or(None, |e| Some(e.evaluate(now))),
		}.into());
	}

	remove_announcement {
		let cid: Cid = b"ipfs_hash".to_vec().try_into().unwrap();
		let origin = T::AnnouncementOrigin::successful_origin();
		let max_count = T::MaxAnnouncementsCount::get() as usize;

		// fill the announcements vec for the worst case.
		let announcements = vec![(cid.clone(), None); max_count];
		let announcements: BoundedVec<_, T::MaxAnnouncementsCount> = BoundedVec::try_from(announcements).unwrap();
		Announcements::<T, I>::put(announcements);
		assert_eq!(CollectiveContent::<T, I>::announcements().len(), max_count);

		let call = Call::<T, I>::remove_announcement { cid: cid.clone() };
	}: { call.dispatch_bypass_filter(origin)? }
	verify {
		assert_eq!(CollectiveContent::<T, I>::announcements().len(), max_count - 1);
		assert_last_event::<T, I>(Event::AnnouncementRemoved { cid }.into());
	}

	impl_benchmark_test_suite!(CollectiveContent, super::mock::new_bench_ext(), super::mock::Test);
}
