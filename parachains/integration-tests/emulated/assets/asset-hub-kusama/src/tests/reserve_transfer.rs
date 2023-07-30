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

const ASSET_ID: u32 = 1;
const ASSET_MIN_BALANCE: u128 = 1;
const ASSETS_PALLET_ID: u8 = 50;

fn relay_origin_assertions(t: RelayToSystemParaTest) {
	type RuntimeEvent = <Kusama as Chain>::RuntimeEvent;

	assert_expected_events!(
		Kusama,
		vec![
			// Amount to reserve transfer is transferred to System Parachain's Sovereign account
			RuntimeEvent::Balances(pallet_balances::Event::Transfer { from, to, amount }) => {
				from: *from == t.sender.account_id,
				to: *to == Kusama::sovereign_account_id_of(
					t.args.dest
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

fn system_para_dest_assertions_incomplete(_t: RelayToSystemParaTest) {
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

fn system_para_to_relay_assertions(_t: SystemParaToRelayTest) {
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

fn system_para_to_para_assertions(t: SystemParaToParaTest) {
	type RuntimeEvent = <AssetHubKusama as Chain>::RuntimeEvent;

	assert_expected_events!(
		AssetHubKusama,
		vec![
			// Dispatchable is properly executed and XCM message sent
			RuntimeEvent::PolkadotXcm(
				pallet_xcm::Event::Attempted { outcome: Outcome::Complete(weight) }
			) => {
				weight: weight_within_threshold(
					(REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD),
					Weight::from_parts(676_119_000, 6196),
					*weight
				),
			},
			// Amount to reserve transfer is transferred to Parachain's Sovereing account
			RuntimeEvent::Balances(
				pallet_balances::Event::Transfer { from, to, amount }
			) => {
				from: *from == t.sender.account_id,
				to: *to == AssetHubKusama::sovereign_account_id_of(
					t.args.dest
				),
				amount: *amount == t.args.amount,
			},
		]
	);
}

fn system_para_to_para_assets_assertions(t: SystemParaToParaTest) {
	type RuntimeEvent = <AssetHubKusama as Chain>::RuntimeEvent;

	assert_expected_events!(
		AssetHubKusama,
		vec![
			// Dispatchable is properly executed and XCM message sent
			RuntimeEvent::PolkadotXcm(
				pallet_xcm::Event::Attempted { outcome: Outcome::Complete(weight) }
			) => {
				weight: weight_within_threshold(
					(REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD),
					Weight::from_parts(676_119_000, 6196),
					*weight
				),
			},
			// Amount to reserve transfer is transferred to Parachain's Sovereing account
			RuntimeEvent::Assets(
				pallet_assets::Event::Transferred { asset_id, from, to, amount }
			) => {
				asset_id: *asset_id == ASSET_ID,
				from: *from == t.sender.account_id,
				to: *to == AssetHubKusama::sovereign_account_id_of(
					t.args.dest
				),
				amount: *amount == t.args.amount,
			},
		]
	);
}

fn relay_limited_reserve_transfer_assets(t: RelayToSystemParaTest) -> DispatchResult {
	<Kusama as KusamaPallet>::XcmPallet::limited_reserve_transfer_assets(
		t.signed_origin,
		bx!(t.args.dest.into()),
		bx!(t.args.beneficiary.into()),
		bx!(t.args.assets.into()),
		t.args.fee_asset_item,
		t.args.weight_limit
	)
}

fn relay_reserve_transfer_assets(t: RelayToSystemParaTest) -> DispatchResult {
	<Kusama as KusamaPallet>::XcmPallet::reserve_transfer_assets(
		t.signed_origin,
		bx!(t.args.dest.into()),
		bx!(t.args.beneficiary.into()),
		bx!(t.args.assets.into()),
		t.args.fee_asset_item
	)
}

fn system_para_limited_reserve_transfer_assets(t: SystemParaToRelayTest) -> DispatchResult {
	<AssetHubKusama as AssetHubKusamaPallet>::PolkadotXcm::limited_reserve_transfer_assets(
		t.signed_origin,
		bx!(t.args.dest.into()),
		bx!(t.args.beneficiary.into()),
		bx!(t.args.assets.into()),
		t.args.fee_asset_item,
		t.args.weight_limit
	)
}

fn system_para_reserve_transfer_assets(t: SystemParaToRelayTest) -> DispatchResult {
	<AssetHubKusama as AssetHubKusamaPallet>::PolkadotXcm::reserve_transfer_assets(
		t.signed_origin,
		bx!(t.args.dest.into()),
		bx!(t.args.beneficiary.into()),
		bx!(t.args.assets.into()),
		t.args.fee_asset_item
	)
}

fn system_para_to_para_limited_reserve_transfer_assets(t: SystemParaToParaTest) -> DispatchResult {
	<AssetHubKusama as AssetHubKusamaPallet>::PolkadotXcm::limited_reserve_transfer_assets(
		t.signed_origin,
		bx!(t.args.dest.into()),
		bx!(t.args.beneficiary.into()),
		bx!(t.args.assets.into()),
		t.args.fee_asset_item,
		t.args.weight_limit
	)
}

fn system_para_to_para_reserve_transfer_assets(t: SystemParaToParaTest) -> DispatchResult {
	<AssetHubKusama as AssetHubKusamaPallet>::PolkadotXcm::reserve_transfer_assets(
		t.signed_origin,
		bx!(t.args.dest.into()),
		bx!(t.args.beneficiary.into()),
		bx!(t.args.assets.into()),
		t.args.fee_asset_item,
	)
}

/// Limited Reserve Transfers of native asset from Relay Chain to the System Parachain shouldn't work
#[test]
fn limited_reserve_transfer_native_asset_from_relay_to_system_para_fails() {
	// Init values for Relay Chain
	let amount_to_send: Balance = KUSAMA_ED * 1000;
	let test_args = TestContext {
		sender: KusamaSender::get(),
		receiver: AssetHubKusamaReceiver::get(),
		args: get_relay_test_args(amount_to_send),
	};

	let mut test = RelayToSystemParaTest::new(test_args);

	let sender_balance_before = test.sender.balance;
	let receiver_balance_before = test.receiver.balance;

	test.set_assertion::<Kusama>(relay_origin_assertions);
	test.set_assertion::<AssetHubKusama>(system_para_dest_assertions_incomplete);
	test.set_dispatchable::<Kusama>(relay_limited_reserve_transfer_assets);
	test.assert();

	let sender_balance_after = test.sender.balance;
	let receiver_balance_after = test.receiver.balance;

	assert_eq!(sender_balance_before - amount_to_send, sender_balance_after);
	assert_eq!(receiver_balance_before, receiver_balance_after);
}

/// Limited Reserve Transfers of native asset from System Parachain to Relay Chain shoudln't work
#[test]
fn limited_reserve_transfer_native_asset_from_system_para_to_relay_fails() {
	// Init values for System Parachain
	let destination = AssetHubKusama::parent_location();
	let beneficiary_id = KusamaReceiver::get();
	let amount_to_send: Balance = ASSET_HUB_KUSAMA_ED * 1000;
	let assets = (Parent, amount_to_send).into();

	let test_args = TestContext {
		sender: AssetHubKusamaSender::get(),
		receiver: KusamaReceiver::get(),
		args: get_system_para_test_args(
			destination,
			beneficiary_id,
			amount_to_send,
			assets,
			None
		),
	};

	let mut test = SystemParaToRelayTest::new(test_args);

	let sender_balance_before = test.sender.balance;
	let receiver_balance_before = test.receiver.balance;

	test.set_assertion::<AssetHubKusama>(system_para_to_relay_assertions);
	test.set_dispatchable::<AssetHubKusama>(system_para_limited_reserve_transfer_assets);
	test.assert();

	let sender_balance_after = test.sender.balance;
	let receiver_balance_after = test.receiver.balance;

	assert_eq!(sender_balance_before, sender_balance_after);
	assert_eq!(receiver_balance_before, receiver_balance_after);
}

/// Reserve Transfers of native asset from Relay Chain to the System Parachain shouldn't work
#[test]
fn reserve_transfer_native_asset_from_relay_to_system_para_fails() {
	// Init values for Relay Chain
	let amount_to_send: Balance = KUSAMA_ED * 1000;
	let test_args = TestContext {
		sender: KusamaSender::get(),
		receiver: AssetHubKusamaReceiver::get(),
		args: get_relay_test_args(amount_to_send),
	};

	let mut test = RelayToSystemParaTest::new(test_args);

	let sender_balance_before = test.sender.balance;
	let receiver_balance_before = test.receiver.balance;

	test.set_assertion::<Kusama>(relay_origin_assertions);
	test.set_assertion::<AssetHubKusama>(system_para_dest_assertions_incomplete);
	test.set_dispatchable::<Kusama>(relay_reserve_transfer_assets);
	test.assert();

	let sender_balance_after = test.sender.balance;
	let receiver_balance_after = test.receiver.balance;

	assert_eq!(sender_balance_before - amount_to_send, sender_balance_after);
	assert_eq!(receiver_balance_before, receiver_balance_after);
}

/// Reserve Transfers of native asset from System Parachain to Relay Chain shouldn't work
#[test]
fn reserve_transfer_native_asset_from_system_para_to_relay_fails() {
	// Init values for System Parachain
	let destination = AssetHubKusama::parent_location();
	let beneficiary_id = KusamaReceiver::get();
	let amount_to_send: Balance = ASSET_HUB_KUSAMA_ED * 1000;
	let assets = (Parent, amount_to_send).into();

	let test_args = TestContext {
		sender: AssetHubKusamaSender::get(),
		receiver: KusamaReceiver::get(),
		args: get_system_para_test_args(
			destination,
			beneficiary_id,
			amount_to_send,
			assets,
			None
		),
	};

	let mut test = SystemParaToRelayTest::new(test_args);

	let sender_balance_before = test.sender.balance;
	let receiver_balance_before = test.receiver.balance;

	test.set_assertion::<AssetHubKusama>(system_para_to_relay_assertions);
	test.set_dispatchable::<AssetHubKusama>(system_para_reserve_transfer_assets);
	test.assert();

	let sender_balance_after = test.sender.balance;
	let receiver_balance_after = test.receiver.balance;

	assert_eq!(sender_balance_before, sender_balance_after);
	assert_eq!(receiver_balance_before, receiver_balance_after);
}

/// Limited Reserve Transfers of native asset from System Parachain to Parachain should work
#[test]
fn limited_reserve_transfer_native_asset_from_system_para_to_para() {
	// Init values for System Parachain
	let destination = AssetHubKusama::sibling_location_of(
		PenpalKusamaA::para_id()
	);
	let beneficiary_id = PenpalKusamaAReceiver::get();
	let amount_to_send: Balance = ASSET_HUB_KUSAMA_ED * 1000;
	let assets = (Parent, amount_to_send).into();

	let test_args = TestContext {
		sender: AssetHubKusamaSender::get(),
		receiver: PenpalKusamaAReceiver::get(),
		args: get_system_para_test_args(
			destination,
			beneficiary_id,
			amount_to_send,
			assets,
			None
		),
	};

	let mut test = SystemParaToParaTest::new(test_args);

	let sender_balance_before = test.sender.balance;

	test.set_assertion::<AssetHubKusama>(system_para_to_para_assertions);
	// TODO: Add assertion for Penpal runtime. Right now message is failing with `UntrustedReserveLocation`
	test.set_dispatchable::<AssetHubKusama>(system_para_to_para_limited_reserve_transfer_assets);
	test.assert();

	let sender_balance_after = test.sender.balance;

	assert_eq!(sender_balance_before - amount_to_send, sender_balance_after);
	// TODO: Check receiver balance when Penpal runtime is improved to propery handle reserve transfers
}

/// Reserve Transfers of native asset from System Parachain to Parachain should work
#[test]
fn reserve_transfer_native_asset_from_system_para_to_para() {
	// Init values for System Parachain
	let destination = AssetHubKusama::sibling_location_of(
		PenpalKusamaA::para_id()
	);
	let beneficiary_id = PenpalKusamaAReceiver::get();
	let amount_to_send: Balance = ASSET_HUB_KUSAMA_ED * 1000;
	let assets = (Parent, amount_to_send).into();

	let test_args = TestContext {
		sender: AssetHubKusamaSender::get(),
		receiver: PenpalKusamaAReceiver::get(),
		args: get_system_para_test_args(
			destination,
			beneficiary_id,
			amount_to_send,
			assets,
			None
		),
	};

	let mut test = SystemParaToParaTest::new(test_args);

	let sender_balance_before = test.sender.balance;

	test.set_assertion::<AssetHubKusama>(system_para_to_para_assertions);
	// TODO: Add assertion for Penpal runtime. Right now message is failing with `UntrustedReserveLocation`
	test.set_dispatchable::<AssetHubKusama>(system_para_to_para_reserve_transfer_assets);
	test.assert();

	let sender_balance_after = test.sender.balance;

	assert_eq!(sender_balance_before - amount_to_send, sender_balance_after);
	// TODO: Check receiver balance when Penpal runtime is improved to propery handle reserve transfers
}

/// Limited Reserve Transfers of a local asset from System Parachain to Parachain should work
#[test]
fn limited_reserve_transfer_asset_from_system_para_to_para() {
	// Force create asset from Relay Chain and mint assets for System Parachain's sender account
	force_create_and_mint_asset(
		ASSET_ID,
		ASSET_MIN_BALANCE,
		true,
		AssetHubKusamaSender::get()
	);

	// Init values for System Parachain
	let destination = AssetHubKusama::sibling_location_of(
		PenpalKusamaA::para_id()
	);
	let beneficiary_id = PenpalKusamaAReceiver::get();
	let amount_to_send = ASSET_MIN_BALANCE * 1000;
	let assets = (
		X2(PalletInstance(ASSETS_PALLET_ID), GeneralIndex(ASSET_ID.into())),
		amount_to_send
	).into();

	let system_para_test_args = TestContext {
		sender: AssetHubKusamaSender::get(),
		receiver: PenpalKusamaAReceiver::get(),
		args: get_system_para_test_args(
			destination,
			beneficiary_id,
			amount_to_send,
			assets,
			None
		),
	};

	let mut system_para_test
		= SystemParaToParaTest::new(system_para_test_args);

	system_para_test.set_assertion::<AssetHubKusama>(system_para_to_para_assets_assertions);
	// TODO: Add assertions when Penpal is able to manage assets
	system_para_test.set_dispatchable::<AssetHubKusama>(system_para_to_para_limited_reserve_transfer_assets);
	system_para_test.assert();
}

/// Reserve Transfers of a local asset from System Parachain to Parachain should work
#[test]
fn reserve_transfer_asset_from_system_para_to_para() {
	// Force create asset from Relay Chain and mint assets for System Parachain's sender account
	force_create_and_mint_asset(
		ASSET_ID,
		ASSET_MIN_BALANCE,
		true,
		AssetHubKusamaSender::get()
	);

	// Init values for System Parachain
	let destination = AssetHubKusama::sibling_location_of(
		PenpalKusamaA::para_id()
	);
	let beneficiary_id = PenpalKusamaAReceiver::get();
	let amount_to_send = ASSET_MIN_BALANCE * 1000;
	let assets = (
		X2(PalletInstance(ASSETS_PALLET_ID), GeneralIndex(ASSET_ID.into())),
		amount_to_send
	).into();

	let system_para_test_args = TestContext {
		sender: AssetHubKusamaSender::get(),
		receiver: PenpalKusamaAReceiver::get(),
		args: get_system_para_test_args(
			destination,
			beneficiary_id,
			amount_to_send,
			assets,
			None
		),
	};

	let mut system_para_test
		= SystemParaToParaTest::new(system_para_test_args);

	system_para_test.set_assertion::<AssetHubKusama>(system_para_to_para_assets_assertions);
	// TODO: Add assertions when Penpal is able to manage assets
	system_para_test.set_dispatchable::<AssetHubKusama>(system_para_to_para_reserve_transfer_assets);
	system_para_test.assert();
}
