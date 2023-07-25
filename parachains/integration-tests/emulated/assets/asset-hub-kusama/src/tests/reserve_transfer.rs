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

type RelayToParaTest = Test<Kusama, AssetHubKusama>;
type ParaToRelayTest = Test<AssetHubKusama, Kusama>;

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

fn relay_origin_assertions(t: RelayToParaTest) {
	type RuntimeEvent = <Kusama as Chain>::RuntimeEvent;

	assert_expected_events!(
		Kusama,
		vec![
			// Amount to reserve transfer is transferred to System Parachain's account
			RuntimeEvent::Balances(pallet_balances::Event::Transfer { from, to, amount }) => {
				from: *from == t.sender.account_id,
				to: *to == Kusama::sovereign_account_id_of(
					Kusama::child_location_of(AssetHubKusama::para_id()).into()
				),
				amount:  *amount == t.args.amount,
			},
			// Dispatchable is properly executed and XCM message sent
			RuntimeEvent::XcmPallet(
				pallet_xcm::Event::Attempted { outcome: Outcome::Complete(weight) }
			) => {
				weight: weight_within_threshold(
					(REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD),
					Weight::from_parts(753_242_000, 0),
					*weight
				),
			},
		]
	);
}

fn para_dest_assertions(_t: RelayToParaTest) {
	type RuntimeEvent = <AssetHubKusama as Chain>::RuntimeEvent;

	assert_expected_events!(
		AssetHubKusama,
		vec![
			RuntimeEvent::DmpQueue(cumulus_pallet_dmp_queue::Event::ExecutedDownward {
				outcome: Outcome::Incomplete(weight, Error::UntrustedReserveLocation),
				..
			}) => {
				weight: weight_within_threshold(
					(REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD),
					Weight::from_parts(1_000_000_000, 0),
					*weight
				),
			},
		]
	);
}

fn para_origin_assertions(_t: ParaToRelayTest) {
	type RuntimeEvent = <AssetHubKusama as Chain>::RuntimeEvent;

	assert_expected_events!(
		AssetHubKusama,
		vec![
			// Execution fails in the origin with `Barrier`
			RuntimeEvent::PolkadotXcm(
				pallet_xcm::Event::Attempted { outcome: Outcome::Error(error) }
			) => {
				error: *error == XcmError::Barrier,
			},
		]
	);
}

#[test]
fn limited_reserve_transfer_native_asset_from_relay_to_system_para_fails() {
	// Init values for Relay Chain
	let amount_to_send: Balance = KUSAMA_ED * 1000;
	let test_args = TestArgs {
		sender: KusamaSender::get(),
		receiver: AssetHubKusamaReceiver::get(),
		args: get_relay_dispatch_args(amount_to_send),
	};

	let mut test = RelayToParaTest::new(test_args);

	let sender_balance_before = test.sender.balance;
	let receiver_balance_before = test.receiver.balance;

	let dispatchable = |t: RelayToParaTest| {
		<Kusama as KusamaPallet>::XcmPallet::limited_reserve_transfer_assets(
			t.signed_origin,
			bx!(t.args.dest),
			bx!(t.args.beneficiary),
			bx!(t.args.assets),
			t.args.fee_asset_item,
			t.args.weight_limit
		)
	};

	test.set_assertion::<Kusama>(relay_origin_assertions);
	test.set_assertion::<AssetHubKusama>(para_dest_assertions);
	test.dispatch::<Kusama>(dispatchable);
	test.assert();

	let sender_balance_after = test.sender.balance;
	let receiver_balance_after = test.receiver.balance;

	assert_eq!(sender_balance_before - amount_to_send, sender_balance_after);
	assert_eq!(receiver_balance_before, receiver_balance_after);
}

#[test]
fn limited_reserve_transfer_native_asset_from_system_para_to_relay_fails() {
	// Init values for Relay Chain
	let amount_to_send: Balance = ASSET_HUB_KUSAMA_ED * 1000;
	let test_args = TestArgs {
		sender: AssetHubKusamaSender::get(),
		receiver: KusamaReceiver::get(),
		args: get_para_dispatch_args(amount_to_send),
	};

	let mut test = ParaToRelayTest::new(test_args);

	let sender_balance_before = test.sender.balance;
	let receiver_balance_before = test.receiver.balance;

	let dispatchable = |t: ParaToRelayTest| {
		<AssetHubKusama as AssetHubKusamaPallet>::PolkadotXcm::limited_reserve_transfer_assets(
			t.signed_origin,
			bx!(t.args.dest),
			bx!(t.args.beneficiary),
			bx!(t.args.assets),
			t.args.fee_asset_item,
			t.args.weight_limit
		)
	};

	test.set_assertion::<AssetHubKusama>(para_origin_assertions);
	test.dispatch::<AssetHubKusama>(dispatchable);
	test.assert();

	let sender_balance_after = test.sender.balance;
	let receiver_balance_after = test.receiver.balance;

	assert_eq!(sender_balance_before, sender_balance_after);
	assert_eq!(receiver_balance_before, receiver_balance_after);
}

#[test]
fn reserve_transfer_native_asset_from_relay_to_system_para_fails() {
	// Init values for Relay Chain
	let amount_to_send: Balance = KUSAMA_ED * 1000;
	let test_args = TestArgs {
		sender: KusamaSender::get(),
		receiver: AssetHubKusamaReceiver::get(),
		args: get_relay_dispatch_args(amount_to_send),
	};

	let mut test = RelayToParaTest::new(test_args);

	let sender_balance_before = test.sender.balance;
	let receiver_balance_before = test.receiver.balance;

	let dispatchable = |t: RelayToParaTest| {
		<Kusama as KusamaPallet>::XcmPallet::reserve_transfer_assets(
			t.signed_origin,
			bx!(t.args.dest),
			bx!(t.args.beneficiary),
			bx!(t.args.assets),
			t.args.fee_asset_item,
		)
	};

	test.set_assertion::<Kusama>(relay_origin_assertions);
	test.set_assertion::<AssetHubKusama>(para_dest_assertions);
	test.dispatch::<Kusama>(dispatchable);
	test.assert();

	let sender_balance_after = test.sender.balance;
	let receiver_balance_after = test.receiver.balance;

	assert_eq!(sender_balance_before - amount_to_send, sender_balance_after);
	assert_eq!(receiver_balance_before, receiver_balance_after);
}

#[test]
fn reserve_transfer_native_asset_from_system_para_to_relay_fails() {
	// Init values for Relay Chain
	let amount_to_send: Balance = ASSET_HUB_KUSAMA_ED * 1000;
	let test_args = TestArgs {
		sender: AssetHubKusamaSender::get(),
		receiver: KusamaReceiver::get(),
		args: get_para_dispatch_args(amount_to_send),
	};

	let mut test = ParaToRelayTest::new(test_args);

	let sender_balance_before = test.sender.balance;
	let receiver_balance_before = test.receiver.balance;

	let dispatchable = |t: ParaToRelayTest| {
		<AssetHubKusama as AssetHubKusamaPallet>::PolkadotXcm::reserve_transfer_assets(
			t.signed_origin,
			bx!(t.args.dest),
			bx!(t.args.beneficiary),
			bx!(t.args.assets),
			t.args.fee_asset_item,
		)
	};

	test.set_assertion::<AssetHubKusama>(para_origin_assertions);
	test.dispatch::<AssetHubKusama>(dispatchable);
	test.assert();

	let sender_balance_after = test.sender.balance;
	let receiver_balance_after = test.receiver.balance;

	assert_eq!(sender_balance_before, sender_balance_after);
	assert_eq!(receiver_balance_before, receiver_balance_after);
}
