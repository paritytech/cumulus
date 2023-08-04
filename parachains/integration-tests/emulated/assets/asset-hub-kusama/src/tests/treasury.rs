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
use frame_support::traits::fungibles::{Create, Inspect, Mutate};
use polkadot_runtime_common::impls::LocatableAssetId;
use xcm_executor::traits::ConvertLocation;

#[test]
fn create_and_claim_treasury_spend() {
	const ASSET_ID: u32 = 1984;
	const SPEND_AMOUNT: u128 = 1_000_000;
	// treasury location from a sibling parachain.
	let treasury_location: MultiLocation = MultiLocation::new(1, PalletInstance(18));
	// treasury account on a sibling parachain.
	let treasury_account =
		asset_hub_kusama_runtime::xcm_config::LocationToAccountId::convert_location(
			&treasury_location,
		)
		.unwrap();
	let asset_hub_location = MultiLocation::new(0, Parachain(AssetHubKusama::para_id().into()));
	let root = <Kusama as Relay>::RuntimeOrigin::root();
	// asset kind to be spend from the treasury.
	let asset_kind = LocatableAssetId::new(
		asset_hub_location,
		AssetId::Concrete((PalletInstance(50), GeneralIndex(ASSET_ID.into())).into()),
	);
	// treasury spend beneficiary.
	let alice: AccountId = Kusama::account_id_of(ALICE);
	let bob: AccountId = Kusama::account_id_of(BOB);
	let bob_signed = <Kusama as Relay>::RuntimeOrigin::signed(bob.clone());

	AssetHubKusama::execute_with(|| {
		type Assets = <AssetHubKusama as AssetHubKusamaPallet>::Assets;

		// create an asset class and mint some assets to the treasury account.
		assert_ok!(<Assets as Create<_>>::create(
			ASSET_ID,
			treasury_account.clone(),
			true,
			SPEND_AMOUNT / 2
		));
		assert_ok!(<Assets as Mutate<_>>::mint_into(ASSET_ID, &treasury_account, SPEND_AMOUNT * 4));
		// beneficiary has zero balance.
		assert_eq!(<Assets as Inspect<_>>::balance(ASSET_ID, &alice,), 0u128,);
	});

	Kusama::execute_with(|| {
		type RuntimeEvent = <Kusama as Relay>::RuntimeEvent;
		type Treasury = <Kusama as KusamaPallet>::Treasury;
		type AssetRate = <Kusama as KusamaPallet>::AssetRate;

		// create a conversion rate from `asset_kind` to the native currency.
		assert_ok!(AssetRate::create(root.clone(), asset_kind.clone(), 2.into()));

		// create and approve a treasury spend.
		assert_ok!(Treasury::spend(
			root,
			asset_kind,
			SPEND_AMOUNT,
			MultiLocation::new(0, Into::<[u8; 32]>::into(alice.clone())),
			None,
		));
		// claim the spend.
		assert_ok!(Treasury::payout(bob_signed.clone(), 0));

		assert_expected_events!(
			Kusama,
			vec![
				RuntimeEvent::Treasury(pallet_treasury::Event::Paid { .. }) => {},
			]
		);
	});

	AssetHubKusama::execute_with(|| {
		type RuntimeEvent = <AssetHubKusama as Para>::RuntimeEvent;
		type Assets = <AssetHubKusama as AssetHubKusamaPallet>::Assets;

		// assets transferred, response sent back via UMP.
		assert_expected_events!(
			AssetHubKusama,
			vec![
				RuntimeEvent::Assets(pallet_assets::Event::Transferred { asset_id: id, from, to, amount }) => {
					id: id == &ASSET_ID,
					from: from == &treasury_account,
					to: to == &alice,
					amount: amount == &SPEND_AMOUNT,
				},
				RuntimeEvent::ParachainSystem(cumulus_pallet_parachain_system::Event::UpwardMessageSent { .. }) => {},
				RuntimeEvent::DmpQueue(cumulus_pallet_dmp_queue::Event::ExecutedDownward { outcome: Outcome::Complete(..) ,.. }) => {},
			]
		);
		// beneficiary received the assets from the treasury.
		assert_eq!(<Assets as Inspect<_>>::balance(ASSET_ID, &alice,), SPEND_AMOUNT,);
	});

	Kusama::execute_with(|| {
		type RuntimeEvent = <Kusama as Relay>::RuntimeEvent;
		type Treasury = <Kusama as KusamaPallet>::Treasury;

		// check the payment status to ensure the response from the AssetHub was received.
		assert_ok!(Treasury::check_status(bob_signed, 0));
		assert_expected_events!(
			Kusama,
			vec![
				RuntimeEvent::Treasury(pallet_treasury::Event::SpendProcessed { .. }) => {},
			]
		);
	});
}
