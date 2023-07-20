// Copyright Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Cumulus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

use crate::*;

fn get_relay_dispatch_args(amount: Balance) -> DispatchArgs {
	DispatchArgs {
		dest: Kusama::child_location_of(AssetHubKusama::para_id()).into(),
		beneficiary: AccountId32Junction {
			network: None,
			id: AssetHubKusamaReceiver::get().into()
		}.into(),
		amount,
		assets: (Here, amount).into(),
		fee_asset_item: 0,
		weight_limit: WeightLimit::Unlimited,
	}
}

fn get_para_dispatch_args(amount: Balance) -> DispatchArgs {
	DispatchArgs {
		dest: AssetHubKusama::parent_location().into(),
		beneficiary: AccountId32Junction {
			network: None,
			id: KusamaReceiver::get().into()
		}.into(),
		amount,
		assets: (Parent, amount).into(),
		fee_asset_item: 0,
		weight_limit: WeightLimit::Unlimited,
	}
}

fn relay_origin_assertions(t: Test<Kusama, AssetHubKusama>) {
	type RuntimeEvent = <Kusama as Chain>::RuntimeEvent;

	assert_expected_events!(
		Kusama,
		vec![
			// Amount to teleport is withdrawn from Sender
			RuntimeEvent::Balances(pallet_balances::Event::Withdraw { who, amount }) => {
				who: *who == t.sender.account_id,
				amount: *amount == t.args.amount,
			},
			// Amount to teleport is deposited in Relay's `CheckAccount`
			RuntimeEvent::Balances(pallet_balances::Event::Deposit { who, amount }) => {
				who: *who == <Kusama as KusamaPallet>::XcmPallet::check_account(),
				amount:  *amount == t.args.amount,
			},
			// Dispatchable is properly executed and XCM message sent
			RuntimeEvent::XcmPallet(
				pallet_xcm::Event::Attempted { outcome: Outcome::Complete(weight) }
			) => {
				weight: weight_within_threshold(
					(REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD),
					Weight::from_parts(763_770_000, 0), *weight
				),
			},
		]
	);
}

fn para_dest_assertions(t: Test<Kusama, AssetHubKusama>) {
	type RuntimeEvent = <AssetHubKusama as Chain>::RuntimeEvent;

	assert_expected_events!(
		AssetHubKusama,
		vec![
			// XCM message is succesfully executed in destination
			RuntimeEvent::DmpQueue(cumulus_pallet_dmp_queue::Event::ExecutedDownward {
				outcome: Outcome::Complete(weight), ..
			}) => {
				weight: weight_within_threshold(
					(REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD),
					Weight::from_parts(165_592_000, 0), *weight
				),
			},
			// Amount minus fees are deposited in Receiver's account
			RuntimeEvent::Balances(pallet_balances::Event::Deposit { who, .. }) => {
				who: *who == t.receiver.account_id,
			},
		]
	);
}

fn para_origin_assertions(t: Test<AssetHubKusama, Kusama>) {
	type RuntimeEvent = <AssetHubKusama as Chain>::RuntimeEvent;

	assert_expected_events!(
		AssetHubKusama,
		vec![
			// Dispatchable is properly executed
			RuntimeEvent::PolkadotXcm(
				pallet_xcm::Event::Attempted { outcome: Outcome::Complete(weight) }
			) => {
				weight: weight_within_threshold(
					(REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD),
					Weight::from_parts(534_872_000, 7_133), *weight
				),
			},
			// XCM message is sent
			RuntimeEvent::ParachainSystem(cumulus_pallet_parachain_system::Event::UpwardMessageSent { .. }) => {},
			// Amount is withdrawn from Sender's account
			RuntimeEvent::Balances(pallet_balances::Event::Withdraw { who, amount }) => {
				who: *who == t.sender.account_id,
				amount: *amount == t.args.amount,
			},
		]
	);
}

fn relay_dest_assertions(t: Test<AssetHubKusama, Kusama>) {
	type RuntimeEvent = <Kusama as Chain>::RuntimeEvent;

	assert_expected_events!(
		Kusama,
		vec![
			// XCM is succesfully received and proccessed
			RuntimeEvent::MessageQueue(pallet_message_queue::Event::Processed {
				origin: AggregateMessageOrigin::Ump(UmpQueueId::Para(id)),
				weight_used,
				success,
				..
			}) => {
				id: *id == AssetHubKusama::para_id(),
				weight_used: weight_within_threshold(
					(REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD),
					Weight::from_parts(299_246_000, 0), *weight_used
				),
				success: *success == true,
			},
			// Amount is witdrawn from Relay Chain's `CheckAccount`
			RuntimeEvent::Balances(pallet_balances::Event::Withdraw { who, amount }) => {
				who: *who == <Kusama as KusamaPallet>::XcmPallet::check_account(),
				amount: *amount == t.args.amount,
			},
			// Amount minus fees are deposited in Receiver's account
			RuntimeEvent::Balances(pallet_balances::Event::Deposit { who, .. }) => {
				who: *who == t.receiver.account_id,
			},
		]
	);
}

fn relay_dest_assertions_fail(_t: Test<AssetHubKusama, Kusama>) {
	type RuntimeEvent = <Kusama as Chain>::RuntimeEvent;

	assert_expected_events!(
		Kusama,
		vec![
			// XCM is succesfully received but fails execution
			RuntimeEvent::MessageQueue(pallet_message_queue::Event::Processed {
				origin: AggregateMessageOrigin::Ump(UmpQueueId::Para(id)),
				weight_used,
				success,
				..
			}) => {
				id: *id == AssetHubKusama::para_id(),
				weight_used: weight_within_threshold(
					(REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD),
					Weight::from_parts(144_595_000, 0), *weight_used
				),
				success: *success == false,
			},
		]
	);
}

/// Limited Teleport of native asset from Relay Chain to the System Parachain works
// #[test]
// fn limited_teleport_native_assets_from_relay_to_system_para_works() {
// 	// Init values for Relay Chain
// 	let amount_to_send: Balance = KUSAMA_ED * 1000;
// 	let test_args = TestArgs {
// 		sender: KusamaSender::get(),
// 		receiver: AssetHubKusamaReceiver::get(),
// 		args: get_relay_dispatch_args(amount_to_send),
// 	};

// 	let mut test = Test::<Kusama, AssetHubKusama>::new(test_args);

// 	let sender_balance_before = test.sender.balance;
// 	let receiver_balance_before = test.receiver.balance;

// 	let dispatchable = |t: Test<Kusama, AssetHubKusama>| {
// 		<Kusama as KusamaPallet>::XcmPallet::limited_teleport_assets(
// 			t.signed_origin,
// 			bx!(t.args.dest),
// 			bx!(t.args.beneficiary),
// 			bx!(t.args.assets),
// 			t.args.fee_asset_item,
// 			t.args.weight_limit
// 		)
// 	};

// 	test.set_assertions(
// 		relay_origin_assertions,
// 		para_dest_assertions
// 	);

// 	test.dispatch(dispatchable);

// 	let sender_balance_after = test.sender.balance;
// 	let receiver_balance_after = test.receiver.balance;

// 	// Sender's balance is reduced
// 	assert_eq!(sender_balance_before - amount_to_send, sender_balance_after);
// 	// Receiver's balance is increased
// 	assert!(receiver_balance_after > receiver_balance_before);
// }

// /// Limited Teleport of native asset from System Parachain to Relay Chain
// /// works when there is enough balance in Relay Chain's `CheckAccount`
// #[test]
// fn limited_teleport_native_assets_back_from_system_para_to_relay_works() {
// 	// Dependency - Relay Chain's `CheckAccount` should have enough balance
// 	limited_teleport_native_assets_from_relay_to_system_para_works();

// 	// Init values for Relay Chain
// 	let amount_to_send: Balance = ASSET_HUB_KUSAMA_ED * 1000;
// 	let test_args = TestArgs {
// 		sender: AssetHubKusamaSender::get(),
// 		receiver: KusamaReceiver::get(),
// 		args: get_para_dispatch_args(amount_to_send),
// 	};

// 	let mut test = Test::<AssetHubKusama, Kusama>::new(test_args);

// 	let sender_balance_before = test.sender.balance;
// 	let receiver_balance_before = test.receiver.balance;

// 	let dispatchable = |t: Test<AssetHubKusama, Kusama>| {
// 		<AssetHubKusama as AssetHubKusamaPallet>::PolkadotXcm::limited_teleport_assets(
// 			t.signed_origin,
// 			bx!(t.args.dest),
// 			bx!(t.args.beneficiary),
// 			bx!(t.args.assets),
// 			t.args.fee_asset_item,
// 			t.args.weight_limit
// 		)
// 	};

// 	test.set_assertions(
// 		para_origin_assertions,
// 		relay_dest_assertions
// 	);

// 	test.dispatch(dispatchable);

// 	let sender_balance_after = test.sender.balance;
// 	let receiver_balance_after = test.receiver.balance;

// 	// Sender's balance is reduced
// 	assert_eq!(sender_balance_before - amount_to_send, sender_balance_after);
// 	// Receiver's balance is increased
// 	assert!(receiver_balance_after > receiver_balance_before);
// }

// /// Limited Teleport of native asset from System Parachain to Relay Chain
// /// fails when there is not enough balance in Relay Chain's `CheckAccount`
// #[test]
// fn limited_teleport_native_assets_from_system_para_to_relay_fails() {
// 	// Init values for Relay Chain
// 	let amount_to_send: Balance = ASSET_HUB_KUSAMA_ED * 1000;
// 	let test_args = TestArgs {
// 		sender: AssetHubKusamaSender::get(),
// 		receiver: KusamaReceiver::get(),
// 		args: get_para_dispatch_args(amount_to_send),
// 	};

// 	let mut test = Test::<AssetHubKusama, Kusama>::new(test_args);

// 	let sender_balance_before = test.sender.balance;
// 	let receiver_balance_before = test.receiver.balance;

// 	let dispatchable = |t: Test<AssetHubKusama, Kusama>| {
// 		<AssetHubKusama as AssetHubKusamaPallet>::PolkadotXcm::limited_teleport_assets(
// 			t.signed_origin,
// 			bx!(t.args.dest),
// 			bx!(t.args.beneficiary),
// 			bx!(t.args.assets),
// 			t.args.fee_asset_item,
// 			t.args.weight_limit
// 		)
// 	};

// 	test.set_assertions(
// 		para_origin_assertions,
// 		relay_dest_assertions_fail
// 	);

// 	test.dispatch(dispatchable);

// 	let sender_balance_after = test.sender.balance;
// 	let receiver_balance_after = test.receiver.balance;

// 	// Sender's balance is reduced
// 	assert_eq!(sender_balance_before - amount_to_send, sender_balance_after);
// 	// Receiver's balance does not change
// 	assert_eq!(receiver_balance_after, receiver_balance_before);
// }

// /// Teleport of native asset from Relay Chain to the System Parachain works
// #[test]
// fn teleport_native_assets_from_relay_to_system_para_works() {
// 	// Init values for Relay Chain
// 	let amount_to_send: Balance = KUSAMA_ED * 1000;
// 	let test_args = TestArgs {
// 		sender: KusamaSender::get(),
// 		receiver: AssetHubKusamaReceiver::get(),
// 		args: get_relay_dispatch_args(amount_to_send),
// 	};

// 	let mut test = Test::<Kusama, AssetHubKusama>::new(test_args);

// 	let sender_balance_before = test.sender.balance;
// 	let receiver_balance_before = test.receiver.balance;

// 	let dispatchable = |t: Test<Kusama, AssetHubKusama>| {
// 		<Kusama as KusamaPallet>::XcmPallet::teleport_assets(
// 			t.signed_origin,
// 			bx!(t.args.dest),
// 			bx!(t.args.beneficiary),
// 			bx!(t.args.assets),
// 			t.args.fee_asset_item
// 		)
// 	};

// 	test.set_assertions(
// 		relay_origin_assertions,
// 		para_dest_assertions
// 	);

// 	test.dispatch(dispatchable);

// 	let sender_balance_after = test.sender.balance;
// 	let receiver_balance_after = test.receiver.balance;

// 	// Sender's balance is reduced
// 	assert_eq!(sender_balance_before - amount_to_send, sender_balance_after);
// 	// Receiver's balance is increased
// 	assert!(receiver_balance_after > receiver_balance_before);
// }

// /// Teleport of native asset from System Parachains to the Relay Chain
// /// works when there is enough balance in Relay Chain's `CheckAccount`
// #[test]
// fn teleport_native_assets_back_from_system_para_to_relay_works() {
// 	// Dependency - Relay Chain's `CheckAccount` should have enough balance
// 	teleport_native_assets_from_relay_to_system_para_works();

// 	// Init values for Relay Chain
// 	let amount_to_send: Balance = ASSET_HUB_KUSAMA_ED * 1000;
// 	let test_args = TestArgs {
// 		sender: AssetHubKusamaSender::get(),
// 		receiver: KusamaReceiver::get(),
// 		args: get_para_dispatch_args(amount_to_send),
// 	};

// 	let mut test = Test::<AssetHubKusama, Kusama>::new(test_args);

// 	let sender_balance_before = test.sender.balance;
// 	let receiver_balance_before = test.receiver.balance;

// 	let dispatchable = |t: Test<AssetHubKusama, Kusama>| {
// 		<AssetHubKusama as AssetHubKusamaPallet>::PolkadotXcm::teleport_assets(
// 			t.signed_origin,
// 			bx!(t.args.dest),
// 			bx!(t.args.beneficiary),
// 			bx!(t.args.assets),
// 			t.args.fee_asset_item
// 		)
// 	};

// 	test.set_assertions(
// 		para_origin_assertions,
// 		relay_dest_assertions
// 	);

// 	test.dispatch(dispatchable);

// 	let sender_balance_after = test.sender.balance;
// 	let receiver_balance_after = test.receiver.balance;

// 	// Sender's balance is reduced
// 	assert_eq!(sender_balance_before - amount_to_send, sender_balance_after);
// 	// Receiver's balance is increased
// 	assert!(receiver_balance_after > receiver_balance_before);
// }

/// Teleport of native asset from System Parachain to Relay Chain
/// fails when there is not enough balance in Relay Chain's `CheckAccount`
#[test]
fn teleport_native_assets_from_system_para_to_relay_fails() {
	// Init values for Relay Chain
	let amount_to_send: Balance = ASSET_HUB_KUSAMA_ED * 1000;
	let test_args = TestArgs {
		sender: AssetHubKusamaSender::get(),
		receiver: KusamaReceiver::get(),
		args: get_para_dispatch_args(amount_to_send),
	};

	let mut test = Test::<AssetHubKusama, Kusama>::new(test_args);

	let sender_balance_before = test.sender.balance;
	let receiver_balance_before = test.receiver.balance;

	let dispatchable = |t: Test<AssetHubKusama, Kusama>| {
		<AssetHubKusama as AssetHubKusamaPallet>::PolkadotXcm::teleport_assets(
			t.signed_origin,
			bx!(t.args.dest),
			bx!(t.args.beneficiary),
			bx!(t.args.assets),
			t.args.fee_asset_item
		)
	};

	test.set_assertions(
		para_origin_assertions,
		relay_dest_assertions_fail
	);

	test.dispatch(dispatchable);

	let sender_balance_after = test.sender.balance;
	let receiver_balance_after = test.receiver.balance;

	// Sender's balance is reduced
	assert_eq!(sender_balance_before - amount_to_send, sender_balance_after);
	// Receiver's balance does not change
	assert_eq!(receiver_balance_after, receiver_balance_before);
}
