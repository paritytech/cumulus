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

const LIMITED_TELEPORT_ID: &'static str = "limited_teleport";

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

#[test]
fn limited_teleport_native_assets_from_relay_to_assets_para() {
	// Get init values for Relay Chain
	let amount_to_send: Balance = KUSAMA_ED * 1000;

	let mut init = TestInit::<Kusama, AssetHubKusama>::new(
		KusamaSender::get(),
		AssetHubKusamaReceiver::get(),
		get_relay_dispatch_args(amount_to_send)
	);

	let sender_balance_before = init.sender.balance;
	let receiver_balance_before = init.receiver.balance;

	let TestInit {
		signed_origin,
		args: DispatchArgs {
			dest,
			beneficiary,
			assets,
			fee_asset_item,
			weight_limit,
		},
		..
	} = init.clone();

	// Send XCM message from Relay Chain
	Kusama::execute_with(|| {
		assert_ok!(<Kusama as KusamaPallet>::XcmPallet::limited_teleport_assets(
			signed_origin,
			bx!(dest),
			bx!(beneficiary),
			bx!(assets),
			fee_asset_item,
			weight_limit,
		));

		type RuntimeEvent = <Kusama as Chain>::RuntimeEvent;

		assert_expected_events!(
			Kusama,
			vec![
				RuntimeEvent::XcmPallet(
					pallet_xcm::Event::Attempted { outcome: Outcome::Complete(weight) }
				) => {
					weight: weight_within_threshold((REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD), Weight::from_parts(763_770_000, 0), *weight),
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

	// Check if balances are updated accordingly in Relay Chain and Assets Parafter
	init.update_balances();

	let sender_balance_after = init.sender.balance;
	let receiver_balance_after = init.receiver.balance;

	println!("1 - {:?}", sender_balance_after);

	assert_eq!(sender_balance_before - amount_to_send, sender_balance_after);
	assert!(receiver_balance_after > receiver_balance_before);

	Kusama::move_out_ext(LIMITED_TELEPORT_ID);
	AssetHubKusama::move_out_ext(LIMITED_TELEPORT_ID);

}

#[test]
fn teleport_2_native_assets_from_relay_to_assets_para() {
	Kusama::move_in_ext(LIMITED_TELEPORT_ID);
	AssetHubKusama::move_out_ext(LIMITED_TELEPORT_ID);

	// Get init values for Relay Chain
	let amount_to_send: Balance = KUSAMA_ED * 1000;

	let mut init = TestInit::<Kusama, AssetHubKusama>::new(
		KusamaSender::get(),
		AssetHubKusamaReceiver::get(),
		get_relay_dispatch_args(amount_to_send)
	);

	let sender_balance_before = init.sender.balance;

	println!("2 - {:?}", sender_balance_before);
}
