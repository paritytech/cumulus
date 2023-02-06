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

fn assert_last_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

/// returns CID hash of 68 bytes of given `i`.
fn create_cid(i: u8) -> OpaqueCid {
	let cid: OpaqueCid = [i; 68].to_vec().try_into().unwrap();
	cid
}

benchmarks_instance_pallet! {
	set_charter {
		let cid: OpaqueCid = b"bafkreif2mywzuu2b2uwehi6c6fojgd7dhcqngus5gzy23uangvpa2kc5si"
			.to_vec().try_into().unwrap();
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
		let cid: OpaqueCid = create_cid(1);
		let call = Call::<T, I>::announce {
			cid: cid.clone(),
			maybe_expire: maybe_expire.clone(),
		};
		let origin = T::AnnouncementOrigin::successful_origin();
	}: { call.dispatch_bypass_filter(origin)? }
	verify {
		assert_eq!(CollectiveContent::<T, I>::announcements().len(), 1);
		assert_eq!(NextAnnouncementExpireAt::<T, I>::get().map_or(0, |_| 1), x);
		assert_last_event::<T, I>(Event::AnnouncementAnnounced {
			cid,
			maybe_expire_at: maybe_expire.map_or(None, |e| Some(e.evaluate(now))),
		}.into());
	}

	remove_announcement {
		let cid: OpaqueCid = create_cid(1);
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

	cleanup_announcements {
		let origin = T::AnnouncementOrigin::successful_origin();
		let max_count = T::MaxAnnouncementsCount::get() as usize;

		for i in 0..max_count {
			let cid: OpaqueCid = create_cid(i as u8);
			CollectiveContent::<T, I>::announce(
				origin.clone(),
				cid,
				Some(DispatchTimeFor::<T>::At(5u32.into())),
			).expect("could not publish an announcement");
		}
		assert_eq!(CollectiveContent::<T, I>::announcements().len(), max_count);
		frame_system::Pallet::<T>::set_block_number(10u32.into());
	}: {
		CollectiveContent::<T, I>::cleanup_announcements(10u32.into());
	} verify {
		assert_eq!(CollectiveContent::<T, I>::announcements().len(), 0);
		assert_eq!(frame_system::Pallet::<T>::events().len(), max_count)
	}

	impl_benchmark_test_suite!(CollectiveContent, super::mock::new_bench_ext(), super::mock::Test);
}
