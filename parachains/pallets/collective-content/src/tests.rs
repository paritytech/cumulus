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

//! Tests.

use super::{mock::*, *};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};

#[test]
fn set_charter_works() {
	new_test_ext().execute_with(|| {
		// wrong origin.
		let origin = RuntimeOrigin::signed(OtherAccount::get());
		let cid: Cid = b"ipfs_hash_fail".to_vec().try_into().unwrap();

		assert_noop!(CollectiveContent::set_charter(origin, cid), BadOrigin);

		// success.
		let origin = RuntimeOrigin::signed(CharterManager::get());
		let cid: Cid = b"ipfs_hash_success".to_vec().try_into().unwrap();

		assert_ok!(CollectiveContent::set_charter(origin, cid.clone()));
		assert_eq!(CollectiveContent::charter(), Some(cid.clone()));
		System::assert_last_event(RuntimeEvent::CollectiveContent(Event::NewCharterSet { cid }));

		// reset. success.
		let origin = RuntimeOrigin::signed(CharterManager::get());
		let cid: Cid = b"ipfs_hash_reset_success".to_vec().try_into().unwrap();

		assert_ok!(CollectiveContent::set_charter(origin, cid.clone()));
		assert_eq!(CollectiveContent::charter(), Some(cid.clone()));
		System::assert_last_event(RuntimeEvent::CollectiveContent(Event::NewCharterSet { cid }));
	});
}

#[test]
fn announce_works() {
	new_test_ext().execute_with(|| {
		// wrong origin.
		let origin = RuntimeOrigin::signed(OtherAccount::get());
		let cid: Cid = b"ipfs_hash_fail".to_vec().try_into().unwrap();

		assert_noop!(CollectiveContent::announce(origin, cid), BadOrigin);

		// success.
		let origin = RuntimeOrigin::signed(AnnouncementManager::get());
		let cid: Cid = b"ipfs_hash_success".to_vec().try_into().unwrap();

		assert_ok!(CollectiveContent::announce(origin, cid.clone()));
		System::assert_last_event(RuntimeEvent::CollectiveContent(Event::AnnouncementAnnounced {
			cid,
		}));

		// one more. success.
		let origin = RuntimeOrigin::signed(AnnouncementManager::get());
		let cid: Cid = b"ipfs_hash_success_2".to_vec().try_into().unwrap();

		assert_ok!(CollectiveContent::announce(origin, cid.clone()));
		System::assert_last_event(RuntimeEvent::CollectiveContent(Event::AnnouncementAnnounced {
			cid,
		}));

		// too many announcements.
		let origin = RuntimeOrigin::signed(AnnouncementManager::get());
		let cid: Cid = b"ipfs_hash_success_2".to_vec().try_into().unwrap();

		assert_noop!(CollectiveContent::announce(origin, cid), Error::<Test>::TooManyAnnouncements);
	});
}

#[test]
fn remove_announcement_works() {
	new_test_ext().execute_with(|| {
		// wrong origin.
		let origin = RuntimeOrigin::signed(OtherAccount::get());
		let cid: Cid = b"ipfs_hash_fail".to_vec().try_into().unwrap();

		assert_noop!(CollectiveContent::remove_announcement(origin, cid), BadOrigin);

		// missing announcement.
		let origin = RuntimeOrigin::signed(AnnouncementManager::get());
		let cid: Cid = b"ipfs_hash_missing".to_vec().try_into().unwrap();

		assert_noop!(
			CollectiveContent::remove_announcement(origin, cid),
			Error::<Test>::MissingAnnouncement
		);

		// success.
		let origin = RuntimeOrigin::signed(AnnouncementManager::get());
		let cid: Cid = b"ipfs_hash_success".to_vec().try_into().unwrap();

		assert_ok!(CollectiveContent::announce(origin.clone(), cid.clone()));

		assert_ok!(CollectiveContent::remove_announcement(origin.clone(), cid.clone()));
		System::assert_last_event(RuntimeEvent::CollectiveContent(Event::AnnouncementRemoved {
			cid: cid.clone(),
		}));
		assert_noop!(
			CollectiveContent::remove_announcement(origin, cid),
			Error::<Test>::MissingAnnouncement
		);
	});
}
