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
struct InitValues {
	amount: u128,
	relay_sender_balance_before: u128,
	para_receiver_balance_before: u128,
	origin: <Kusama as Relay>::RuntimeOrigin,
	assets_para_destination: VersionedMultiLocation,
	beneficiary: VersionedMultiLocation,
	native_assets: VersionedMultiAssets,
	fee_asset_item: u32,
	weight_limit: WeightLimit,
}

fn get_init_values() -> InitValues {
	InitValues {
		amount: KUSAMA_ED * 1000,
		relay_sender_balance_before: Kusama::account_data_of(KusamaSender::get()).free,
		para_receiver_balance_before: AssetHubKusama::account_data_of(AssetHubKusamaReceiver::get()).free,
		origin: <Kusama as Relay>::RuntimeOrigin::signed(KusamaSender::get()),
		assets_para_destination: Kusama::child_location_of(AssetHubKusama::para_id()).into(),
		beneficiary: AccountId32 { network: None, id: AssetHubKusamaReceiver::get().into() }.into(),
		native_assets: (Here, amount).into(),
		fee_asset_item: 0,
		weight_limit: WeightLimit::Unlimited,
	}
}

#[test]
fn teleport_native_assets_from_relay_to_assets_para() {
	// Init tests variables
	// let amount = KUSAMA_ED * 1000;
	// let relay_sender_balance_before = Kusama::account_data_of(KusamaSender::get()).free;
	// let para_receiver_balance_before =
	// 	AssetHubKusama::account_data_of(AssetHubKusamaReceiver::get()).free;

	// let origin = <Kusama as Relay>::RuntimeOrigin::signed(KusamaSender::get());
	// let assets_para_destination: VersionedMultiLocation =
	// 	Kusama::child_location_of(AssetHubKusama::para_id()).into();
	// let beneficiary: VersionedMultiLocation =
	// 	AccountId32 { network: None, id: AssetHubKusamaReceiver::get().into() }.into();
	// let native_assets: VersionedMultiAssets = (Here, amount).into();
	// let fee_asset_item = 0;
	// let weight_limit = WeightLimit::Unlimited;
	let init = get_init_values();

	// -- LIMITED --
	// Send XCM message from Relay Chain
	Kusama::execute_with(|| {
		assert_ok!(<Kusama as KusamaPallet>::XcmPallet::limited_teleport_assets(
			init.origin,
			bx!(init.assets_para_destination),
			bx!(init.beneficiary),
			bx!(init.native_assets),
			init.fee_asset_item,
			init.weight_limit,
		));

		type RuntimeEvent = <Kusama as Relay>::RuntimeEvent;

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
		type RuntimeEvent = <AssetHubKusama as Para>::RuntimeEvent;

		assert_expected_events!(
			AssetHubKusama,
			vec![
				RuntimeEvent::Balances(pallet_balances::Event::Deposit { who, .. }) => {
					who: *who == AssetHubKusamaReceiver::get().into(),
				},
			]
		);
	});

	// Check if balances are updated accordingly in Relay Chain and Assets Parachain
	let relay_sender_balance_after = Kusama::account_data_of(KusamaSender::get()).free;
	let para_sender_balance_after =
		AssetHubKusama::account_data_of(AssetHubKusamaReceiver::get()).free;

	assert_eq!(init.relay_sender_balance_before - init.amount, relay_sender_balance_after);
	assert!(para_sender_balance_after > init.para_receiver_balance_before);
}
