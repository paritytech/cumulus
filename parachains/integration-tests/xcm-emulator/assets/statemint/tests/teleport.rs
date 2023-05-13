use statemint_it::*;

#[test]
// NOTE: This needs to be run before every other test to ensure that chains can communicate with one
// another.
pub fn force_xcm_version() {
	let xcm_version = XCM_VERSION_3;

	Polkadot::execute_with(|| {
		let statemint_location: MultiLocation = Polkadot::child_location_of(Statemint::para_id());
		let penpal_location: MultiLocation = Polkadot::child_location_of(PenpalPolkadot::para_id());

		// Check that we can force xcm version for Statemint and PenpalPolkadot from Polkadot.
		for location in [statemint_location, penpal_location] {
			assert_ok!(<Polkadot as Relay>::XcmPallet::force_xcm_version(
				<Polkadot as Relay>::RuntimeOrigin::root(),
				bx!(location),
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
		let origin = <PenpalPolkadot as Para>::RuntimeOrigin::root();
		let location: MultiLocation = PenpalPolkadot::parent_location();

		assert_ok!(<PenpalPolkadot as Para>::XcmPallet::force_xcm_version(
			origin,
			bx!(location),
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
	// Init tests variables
	let amount = POLKADOT_ED * 1000;
	let relay_sender_balance_before = Polkadot::account_data_of(PolkadotSender::get()).free;
	let para_receiver_balance_before = Statemint::account_data_of(StatemintReceiver::get()).free;

	let origin = <Polkadot as Relay>::RuntimeOrigin::signed(PolkadotSender::get());
	let assets_para_destination: VersionedMultiLocation = Polkadot::child_location_of(Statemint::para_id()).into();
	let beneficiary: VersionedMultiLocation = AccountId32 { network: None, id: StatemintReceiver::get().into() }.into();
	let native_assets: VersionedMultiAssets = (Here, amount).into();
	let fee_asset_item = 0;
	let weight_limit = WeightLimit::Unlimited;

	// Send XCM message from Relay Chain
	Polkadot::execute_with(|| {
		assert_ok!(<Polkadot as Relay>::XcmPallet::limited_teleport_assets(
			origin,
			bx!(assets_para_destination),
			bx!(beneficiary),
			bx!(native_assets),
			fee_asset_item,
			weight_limit,
		));

		type RuntimeEvent = <Polkadot as Relay>::RuntimeEvent;

		assert_expected_events!(
			Polkadot,
			vec![
				RuntimeEvent::XcmPallet(pallet_xcm::Event::Attempted(Outcome::Complete { .. })) => {},
			]
		);
	});

	// Receive XCM message in Assets Parachain
	Statemint::execute_with(|| {
		type RuntimeEvent = <Statemint as Para>::RuntimeEvent;

		assert_expected_events!(
			Statemint,
			vec![
				RuntimeEvent::Balances(pallet_balances::Event::Deposit { who, .. }) => {
					who: *who == StatemineReceiver::get().into(),
				},
			]
		);
	});

	// Check if balances are updated accordingly in Relay Chain and Assets Parachain
	let relay_sender_balance_after = Polkadot::account_data_of(PolkadotSender::get()).free;
	let para_sender_balance_after = Statemint::account_data_of(StatemintReceiver::get()).free;

	assert_eq!(relay_sender_balance_before - amount, relay_sender_balance_after);
	assert!(para_sender_balance_after > para_receiver_balance_before);
}
