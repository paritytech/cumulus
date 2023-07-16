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

use xcm_emulator::NetworkComponent;
use xcm_emulator::Network;

use crate::*;

const LIMITED_TELEPORT_ID: &'static str = "limited_teleport";
const TELEPORT_ID: &'static str = "teleport";

fn get_relay_dispatch_args(amount: Balance) -> DispatchArgs {
	DispatchArgs {
		dest: Kusama::child_location_of(AssetHubKusama::para_id()).into(),
		beneficiary: AccountId32Junction {
			network: None,
			id: AssetHubKusamaReceiver::get().into()
		}.into(),
		assets: (Here, amount).into(),
		fee_asset_item: 0,
		weight_limit: WeightLimit::Unlimited,
	}
}

fn send_xcm_message(dispatchable: impl FnOnce() -> DispatchResult) {
		// Send XCM message from Relay Chain
		Kusama::execute_with(|| {
			assert_ok!(dispatchable());

			type RuntimeEvent = <Kusama as Chain>::RuntimeEvent;

			assert_expected_events!(
				Kusama,
				vec![
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
		});

		// Receive XCM message in Assets Parachain
		AssetHubKusama::execute_with(|| {
			type RuntimeEvent = <AssetHubKusama as Chain>::RuntimeEvent;

			assert_expected_events!(
				AssetHubKusama,
				vec![
					RuntimeEvent::Balances(pallet_balances::Event::Deposit { who, .. }) => {
						who: *who == AssetHubKusamaReceiver::get().into(),
					},
				]
			);
		});
}

#[test]
fn limited_teleport_native_assets_from_relay_to_system_para() {
	// Get init values for Relay Chain
	let amount_to_send: Balance = KUSAMA_ED * 1000;

	let mut init = TestInit::<Kusama, AssetHubKusama>::new(
		KusamaSender::get(),
		AssetHubKusamaReceiver::get(),
		get_relay_dispatch_args(amount_to_send)
	);

	let sender_balance_before = init.sender.balance;
	let receiver_balance_before = init.receiver.balance;

	let init_clone = init.clone();

	let dispatchable = || {
		<Kusama as KusamaPallet>::XcmPallet::limited_teleport_assets(
			init_clone.signed_origin,
			bx!(init_clone.args.dest),
			bx!(init_clone.args.beneficiary),
			bx!(init_clone.args.assets),
			init_clone.args.fee_asset_item,
			init_clone.args.weight_limit,
		)
	};

	send_xcm_message(dispatchable);

	// Check if balances are updated accordingly in Relay Chain and Assets Parafter
	init.update_balances();

	let sender_balance_after = init.sender.balance;
	let receiver_balance_after = init.receiver.balance;

	println!("Limited 1 - {:?}", sender_balance_after);

	assert_eq!(sender_balance_before - amount_to_send, sender_balance_after);
	assert!(receiver_balance_after > receiver_balance_before);

	Kusama::move_ext_out(LIMITED_TELEPORT_ID);
	AssetHubKusama::move_ext_out(LIMITED_TELEPORT_ID);

}

#[test]
fn teleport_native_assets_from_relay_to_system_para() {
	// Get init values for Relay Chain
	let amount_to_send: Balance = KUSAMA_ED * 1000;

	let mut init = TestInit::<Kusama, AssetHubKusama>::new(
		KusamaSender::get(),
		AssetHubKusamaReceiver::get(),
		get_relay_dispatch_args(amount_to_send)
	);

	let sender_balance_before = init.sender.balance;
	let receiver_balance_before = init.receiver.balance;

	let init_clone = init.clone();

	let dispatchable = || {
		<Kusama as KusamaPallet>::XcmPallet::teleport_assets(
			init_clone.signed_origin,
			bx!(init_clone.args.dest),
			bx!(init_clone.args.beneficiary),
			bx!(init_clone.args.assets),
			init_clone.args.fee_asset_item
		)
	};

	send_xcm_message(dispatchable);

	// Check if balances are updated accordingly in Relay Chain and Assets Parafter
	init.update_balances();

	let sender_balance_after = init.sender.balance;
	let receiver_balance_after = init.receiver.balance;

	println!("No limited 1 - {:?}", sender_balance_after);

	assert_eq!(sender_balance_before - amount_to_send, sender_balance_after);
	assert!(receiver_balance_after > receiver_balance_before);

	println!("BLOCK NUMBER 1 - {:?}", <Kusama as NetworkComponent>::Network::relay_block_number());

	Kusama::move_ext_out(TELEPORT_ID);
	AssetHubKusama::move_ext_out(TELEPORT_ID);

}

#[test]
fn limited_teleport_native_assets_back_from_relay_to_system_para() {
	Kusama::move_ext_in(LIMITED_TELEPORT_ID);
	AssetHubKusama::move_ext_in(LIMITED_TELEPORT_ID);

	println!("BLOCK NUMBER 2 - {:?}", <Kusama as NetworkComponent>::Network::relay_block_number());


	// Get init values for Relay Chain
	let amount_to_send: Balance = ASSET_HUB_KUSAMA_ED * 1000;

	let mut init = TestInit::<AssetHubKusama, Kusama>::new(
		AssetHubKusamaSender::get(),
		KusamaReceiver::get(),
		get_relay_dispatch_args(amount_to_send)
	);

	let sender_balance_before = init.sender.balance;
	let receiver_balance_before = init.receiver.balance;

	let init_clone = init.clone();

	let dispatchable = || {
		<AssetHubKusama as AssetHubKusamaPallet>::PolkadotXcm::limited_teleport_assets(
			init_clone.signed_origin,
			bx!(init_clone.args.dest),
			bx!(init_clone.args.beneficiary),
			bx!(init_clone.args.assets),
			init_clone.args.fee_asset_item,
			init_clone.args.weight_limit,
		)
	};

	send_xcm_message(dispatchable);

	// Check if balances are updated accordingly in Relay Chain and Assets Parafter
	init.update_balances();

	let sender_balance_after = init.sender.balance;
	let receiver_balance_after = init.receiver.balance;

	println!("Limited 2 - {:?}", sender_balance_after);

	assert_eq!(sender_balance_before - amount_to_send, sender_balance_after);
	assert!(receiver_balance_after > receiver_balance_before);
}
