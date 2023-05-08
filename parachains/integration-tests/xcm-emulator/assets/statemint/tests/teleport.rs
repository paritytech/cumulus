use statemint_it::*;

#[test]
// NOTE: This needs to be run before every other test to ensure that chains can communicate with one
// another.
pub fn force_xcm_version() {
	let xcm_version = 3;
	Polkadot::execute_with(|| {
		let statemint_location: MultiLocation = (Ancestor(0), Parachain(1000)).into();
		let penpal_location: MultiLocation = (Ancestor(0), Parachain(2000)).into();

		// Check that we can force xcm version for Statemint and PenpalPolkadot from Polkadot.
		for location in [statemint_location, penpal_location] {
			assert_ok!(<Polkadot as Relay>::XcmPallet::force_xcm_version(
				<Polkadot as Relay>::RuntimeOrigin::root(),
				Box::new(location),
				xcm_version,
			));
			assert!(<Polkadot as Relay>::System::events().iter().any(|r| matches!(
				r.event,
				polkadot_runtime::RuntimeEvent::XcmPallet(pallet_xcm::Event::SupportedVersionChanged {
					0: loc,
					1: ver,
				}) if loc == location && ver == xcm_version
			)));
		}
	});

	// PenpalPolkadot forces Polkadot xcm version.
	PenpalPolkadot::execute_with(|| {
		let location: MultiLocation = (Parent).into();

		assert_ok!(<PenpalPolkadot as Para>::XcmPallet::force_xcm_version(
			penpal_runtime::RuntimeOrigin::root(),
			Box::new(location),
			xcm_version,
		));

		type RuntimeEvent = <PenpalPolkadot as Para>::RuntimeEvent;

		assert!(<PenpalPolkadot as Para>::System::events().iter().any(|r| matches!(
			r.event,
			RuntimeEvent::PolkadotXcm(pallet_xcm::Event::SupportedVersionChanged {
				0: loc,
				1: ver,
			}) if loc == location && ver == xcm_version
		)));
	});
}

#[test]
fn teleport_native_assets_from_relay_to_assets_para() {
	let amount = POLKADOT_ED * 1000;
	let native_assets: VersionedMultiAssets = (Here, amount).into();
	let relay_sender_balance_before = Polkadot::account_data_of(PolkadotSender::get()).free;
	let para_receiver_balance_before = Statemint::account_data_of(StatemintReceiver::get()).free;

	Polkadot::execute_with(|| {
		use polkadot_runtime::{RuntimeEvent, RuntimeOrigin, System};

		assert_ok!(<Polkadot as Relay>::XcmPallet::limited_teleport_assets(
			RuntimeOrigin::signed(PolkadotSender::get()),
			bx!(
				(Ancestor(0), Parachain(Statemint::para_id().into())).into()
			),
			bx!(
				AccountId32 { network: None, id: Statemint::account_id_of(BOB).into()}.into())
				,
			bx!(native_assets),
			0,
			WeightLimit::Unlimited,
		));
		assert!(<Polkadot as Relay>::System::events().iter().any(|r| matches!(
			r.event,
			RuntimeEvent::XcmPallet(pallet_xcm::Event::Attempted(Outcome::Complete { .. }))
		)));
	});

	Statemint::execute_with(|| {
		use statemint_runtime::{Runtime, RuntimeEvent, System};
		// assert!(System::events().iter().any(|r| matches!(
		// 	&r.event,
		// 	RuntimeEvent::Balances(pallet_balances::Event::Deposit { who, .. })
		// 	if *who == Statemint::account_id_of(BOB).into()
		// )));
	});

	// let (relay_balance_after, ap_balance_after) = get_balances();
	let relay_sender_balance_after = Polkadot::account_data_of(PolkadotSender::get()).free;
	let para_sender_balance_after = Statemint::account_data_of(StatemintReceiver::get()).free;

	// assert_eq!(relay_sender_balance_before - amount, relay_sender_balance_after);
	// assert!(para_sender_balance_after > para_receiver_balance_before);
}
