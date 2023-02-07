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

use super::*;
use XcmpMessageFormat::*;

use cumulus_primitives_core::XcmpMessageHandler;
use frame_support::{assert_noop, assert_ok, assert_storage_noop};
use mock::{new_test_ext, RuntimeOrigin, Test, XcmpQueue};
use sp_runtime::traits::BadOrigin;
use std::iter::once;

#[test]
fn empty_concatenated_works() {
	new_test_ext().execute_with(|| {
		let data = ConcatenatedVersionedXcm.encode();

		assert_storage_noop!(XcmpQueue::handle_xcmp_messages(
			once((1000.into(), 1, data.as_slice())),
			Weight::MAX,
		));
	})
}

#[test]
#[should_panic = "Blob messages not handled"]
#[cfg(debug_assertions)]
fn bad_blob_message_is_ignored() {
	new_test_ext().execute_with(|| {
		let data = [ConcatenatedEncodedBlob.encode(), vec![1]].concat();

		XcmpQueue::handle_xcmp_messages(once((1000.into(), 1, data.as_slice())), Weight::MAX);
	});
}

/// Message blobs are not supported and panic in debug mode.
#[test]
#[should_panic = "Blob messages not handled"]
#[cfg(debug_assertions)]
fn handle_blob_message() {
	new_test_ext().execute_with(|| {
		let data = [ConcatenatedEncodedBlob.encode(), vec![1].encode()].concat();

		XcmpQueue::handle_xcmp_messages(once((1000.into(), 1, data.as_slice())), Weight::MAX);
	});
}

/// Message blobs do not panic in release mode but are just a No-OP.
#[test]
#[cfg(not(debug_assertions))]
fn handle_blob_message() {
	new_test_ext().execute_with(|| {
		let data = [ConcatenatedEncodedBlob.encode(), vec![1].encode()].concat();

		assert_storage_noop!(XcmpQueue::handle_xcmp_messages(
			once((1000.into(), 1, data.as_slice())),
			Weight::MAX,
		));
	});
}

/// Invalid concatenated XCMs panic in debug mode.
#[test]
#[should_panic = "Invalid incoming XCMP message data"]
#[cfg(debug_assertions)]
fn handle_invalid_data_panics() {
	new_test_ext().execute_with(|| {
		let data = [ConcatenatedVersionedXcm.encode(), Xcm::<Test>(vec![]).encode()].concat();

		XcmpQueue::handle_xcmp_messages(once((1000.into(), 1, data.as_slice())), Weight::MAX);
	});
}

/// Invalid concatenated XCMs do not panic in release mode but are just a No-OP.
#[test]
#[cfg(not(debug_assertions))]
fn handle_invalid_data_no_panic() {
	new_test_ext().execute_with(|| {
		let data = [ConcatenatedVersionedXcm.encode(), Xcm::<Test>(vec![]).encode()].concat();

		assert_storage_noop!(XcmpQueue::handle_xcmp_messages(
			once((1000.into(), 1, data.as_slice())),
			Weight::MAX,
		));
	});
}

#[test]
fn service_overweight_unknown() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			XcmpQueue::service_overweight(RuntimeOrigin::root(), 0, Weight::MAX),
			Error::<Test>::BadOverweightIndex,
		);
	});
}

#[test]
fn service_overweight_bad_xcm_format() {
	new_test_ext().execute_with(|| {
		let bad_xcm = vec![255];
		Overweight::<Test>::insert(0, (ParaId::from(1000), 0, bad_xcm));

		assert_noop!(
			XcmpQueue::service_overweight(RuntimeOrigin::root(), 0, Weight::MAX),
			Error::<Test>::BadXcm
		);
	});
}

// FAIL-CI add queue-suspension
//#[test]
//fn suspend_xcm_execution_works() {
//	new_test_ext().execute_with(|| {
//		QueueSuspended::<Test>::put(true);
//
//		let xcm =
//			VersionedXcm::from(Xcm::<RuntimeCall>(vec![Instruction::<RuntimeCall>::ClearOrigin]))
//				.encode();
//
//		let data = [ConcatenatedVersionedXcm.encode(), xcm.encode()].concat();
//		// This should have executed the incoming XCM, because it came from a system parachain
//		XcmpQueue::handle_xcmp_messages(
//			once((999.into(), 1, data.as_slice())),
//			Weight::MAX
//		);
//
//		// This should have queue instead of executing since it comes from a sibling.
//		XcmpQueue::handle_xcmp_messages(
//			once((2000.into(), 1, data.as_slice())),
//			Weight::MAX
//		);
//
//		let queued_xcm = mock::EnqueueToLocalStorage::get(AggregateMessageOrigin::Sibling(ParaId::from(2000)));
//		assert_eq!(queued_xcm, vec![xcm]);
//	});
//}

// FAIL-CI reimplement and test back-pressure
#[test]
fn update_suspend_threshold_works() {
	new_test_ext().execute_with(|| {
		let data: QueueConfigData = <QueueConfig<Test>>::get();
		assert_eq!(data.suspend_threshold, 2);
		assert_ok!(XcmpQueue::update_suspend_threshold(RuntimeOrigin::root(), 3));
		assert_noop!(XcmpQueue::update_suspend_threshold(RuntimeOrigin::signed(2), 5), BadOrigin);
		let data: QueueConfigData = <QueueConfig<Test>>::get();

		assert_eq!(data.suspend_threshold, 3);
	});
}

#[test]
fn update_drop_threshold_works() {
	new_test_ext().execute_with(|| {
		let data: QueueConfigData = <QueueConfig<Test>>::get();
		assert_eq!(data.drop_threshold, 5);
		assert_ok!(XcmpQueue::update_drop_threshold(RuntimeOrigin::root(), 6));
		assert_noop!(XcmpQueue::update_drop_threshold(RuntimeOrigin::signed(2), 7), BadOrigin);
		let data: QueueConfigData = <QueueConfig<Test>>::get();

		assert_eq!(data.drop_threshold, 6);
	});
}

#[test]
fn update_resume_threshold_works() {
	new_test_ext().execute_with(|| {
		let data: QueueConfigData = <QueueConfig<Test>>::get();
		assert_eq!(data.resume_threshold, 1);
		assert_ok!(XcmpQueue::update_resume_threshold(RuntimeOrigin::root(), 2));
		assert_noop!(XcmpQueue::update_resume_threshold(RuntimeOrigin::signed(7), 3), BadOrigin);
		let data: QueueConfigData = <QueueConfig<Test>>::get();

		assert_eq!(data.resume_threshold, 2);
	});
}

#[test]
fn update_threshold_weight_works() {
	new_test_ext().execute_with(|| {
		let data: QueueConfigData = <QueueConfig<Test>>::get();
		assert_eq!(data.threshold_weight, Weight::from_ref_time(100_000));
		assert_ok!(XcmpQueue::update_threshold_weight(
			RuntimeOrigin::root(),
			Weight::from_ref_time(10_000)
		));
		assert_noop!(
			XcmpQueue::update_threshold_weight(
				RuntimeOrigin::signed(5),
				Weight::from_ref_time(10_000_000),
			),
			BadOrigin
		);
		let data: QueueConfigData = <QueueConfig<Test>>::get();

		assert_eq!(data.threshold_weight, Weight::from_ref_time(10_000));
	});
}

#[test]
fn update_weight_restrict_decay_works() {
	new_test_ext().execute_with(|| {
		let data: QueueConfigData = <QueueConfig<Test>>::get();
		assert_eq!(data.weight_restrict_decay, Weight::from_ref_time(2));
		assert_ok!(XcmpQueue::update_weight_restrict_decay(
			RuntimeOrigin::root(),
			Weight::from_ref_time(5)
		));
		assert_noop!(
			XcmpQueue::update_weight_restrict_decay(
				RuntimeOrigin::signed(6),
				Weight::from_ref_time(4),
			),
			BadOrigin
		);
		let data: QueueConfigData = <QueueConfig<Test>>::get();

		assert_eq!(data.weight_restrict_decay, Weight::from_ref_time(5));
	});
}

#[test]
fn update_xcmp_max_individual_weight() {
	new_test_ext().execute_with(|| {
		let data: QueueConfigData = <QueueConfig<Test>>::get();
		assert_eq!(
			data.xcmp_max_individual_weight,
			Weight::from_parts(20u64 * WEIGHT_REF_TIME_PER_MILLIS, DEFAULT_POV_SIZE),
		);
		assert_ok!(XcmpQueue::update_xcmp_max_individual_weight(
			RuntimeOrigin::root(),
			Weight::from_ref_time(30u64 * WEIGHT_REF_TIME_PER_MILLIS)
		));
		assert_noop!(
			XcmpQueue::update_xcmp_max_individual_weight(
				RuntimeOrigin::signed(3),
				Weight::from_ref_time(10u64 * WEIGHT_REF_TIME_PER_MILLIS)
			),
			BadOrigin
		);
		let data: QueueConfigData = <QueueConfig<Test>>::get();

		assert_eq!(
			data.xcmp_max_individual_weight,
			Weight::from_ref_time(30u64 * WEIGHT_REF_TIME_PER_MILLIS)
		);
	});
}

/// Validates [`validate`] for required Some(destination) and Some(message)
struct OkFixedXcmHashWithAssertingRequiredInputsSender;
impl OkFixedXcmHashWithAssertingRequiredInputsSender {
	const FIXED_XCM_HASH: [u8; 32] = [9; 32];

	fn fixed_delivery_asset() -> MultiAssets {
		MultiAssets::new()
	}

	fn expected_delivery_result() -> Result<(XcmHash, MultiAssets), SendError> {
		Ok((Self::FIXED_XCM_HASH, Self::fixed_delivery_asset()))
	}
}
impl SendXcm for OkFixedXcmHashWithAssertingRequiredInputsSender {
	type Ticket = ();

	fn validate(
		destination: &mut Option<MultiLocation>,
		message: &mut Option<Xcm<()>>,
	) -> SendResult<Self::Ticket> {
		assert!(destination.is_some());
		assert!(message.is_some());
		Ok(((), OkFixedXcmHashWithAssertingRequiredInputsSender::fixed_delivery_asset()))
	}

	fn deliver(_: Self::Ticket) -> Result<XcmHash, SendError> {
		Ok(Self::FIXED_XCM_HASH)
	}
}

#[test]
fn xcmp_queue_does_not_consume_dest_or_msg_on_not_applicable() {
	// dummy message
	let message = Xcm(vec![Trap(5)]);

	// XcmpQueue - check dest is really not applicable
	let dest = (Parent, Parent, Parent);
	let mut dest_wrapper = Some(dest.clone().into());
	let mut msg_wrapper = Some(message.clone());
	assert_eq!(
		Err(SendError::NotApplicable),
		<XcmpQueue as SendXcm>::validate(&mut dest_wrapper, &mut msg_wrapper)
	);

	// check wrapper were not consumed
	assert_eq!(Some(dest.clone().into()), dest_wrapper.take());
	assert_eq!(Some(message.clone()), msg_wrapper.take());

	// another try with router chain with asserting sender
	assert_eq!(
		OkFixedXcmHashWithAssertingRequiredInputsSender::expected_delivery_result(),
		send_xcm::<(XcmpQueue, OkFixedXcmHashWithAssertingRequiredInputsSender)>(
			dest.into(),
			message
		)
	);
}

#[test]
fn xcmp_queue_consumes_dest_and_msg_on_ok_validate() {
	// dummy message
	let message = Xcm(vec![Trap(5)]);

	// XcmpQueue - check dest/msg is valid
	let dest = (Parent, X1(Parachain(5555)));
	let mut dest_wrapper = Some(dest.clone().into());
	let mut msg_wrapper = Some(message.clone());
	assert!(<XcmpQueue as SendXcm>::validate(&mut dest_wrapper, &mut msg_wrapper).is_ok());

	// check wrapper were consumed
	assert_eq!(None, dest_wrapper.take());
	assert_eq!(None, msg_wrapper.take());

	new_test_ext().execute_with(|| {
		// another try with router chain with asserting sender
		assert_eq!(
			Err(SendError::Transport("NoChannel")),
			send_xcm::<(XcmpQueue, OkFixedXcmHashWithAssertingRequiredInputsSender)>(
				dest.into(),
				message
			)
		);
	});
}
