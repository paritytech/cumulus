use crate::*;

#[test]
fn reserve_transfer_native_asset_from_relay_to_assets() {
	// Init tests variables
	let amount = POLKADOT_ED * 1000;
	let relay_sender_balance_before = Westend::account_data_of(WestendSender::get()).free;
	let para_receiver_balance_before = Westmint::account_data_of(WestmintReceiver::get()).free;

	let origin = <Westend as Relay>::RuntimeOrigin::signed(WestendSender::get());
	let assets_para_destination: VersionedMultiLocation =
		Westend::child_location_of(Westmint::para_id()).into();
	let beneficiary: VersionedMultiLocation =
		AccountId32 { network: None, id: WestmintReceiver::get().into() }.into();
	let native_assets: VersionedMultiAssets = (Here, amount).into();
	let fee_asset_item = 0;
	let weight_limit = WeightLimit::Unlimited;

	// Send XCM message from Relay Chain
	Westend::execute_with(|| {
		assert_ok!(<Westend as WestendPallet>::XcmPallet::limited_reserve_transfer_assets(
			origin,
			bx!(assets_para_destination),
			bx!(beneficiary),
			bx!(native_assets),
			fee_asset_item,
			weight_limit,
		));

		type RuntimeEvent = <Westend as Relay>::RuntimeEvent;

		assert_expected_events!(
			Westend,
			vec![
				RuntimeEvent::XcmPallet(pallet_xcm::Event::Attempted { outcome: Outcome::Complete(weight) }) => {
					weight: weight_within_threshold((REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD), Weight::from_parts(2_000_000_000, 0), *weight),
				},
			]
		);
	});

	// Receive XCM message in Assets Parachain
	Westmint::execute_with(|| {
		type RuntimeEvent = <Westmint as Para>::RuntimeEvent;

		assert_expected_events!(
			Westmint,
			vec![
				RuntimeEvent::DmpQueue(cumulus_pallet_dmp_queue::Event::ExecutedDownward {
					outcome: Outcome::Incomplete(_, Error::UntrustedReserveLocation),
					..
				}) => {},
			]
		);
	});

	// Check if balances are updated accordingly in Relay Chain and Assets Parachain
	let relay_sender_balance_after = Westend::account_data_of(WestendSender::get()).free;
	let para_sender_balance_after = Westmint::account_data_of(WestmintReceiver::get()).free;

	assert_eq!(relay_sender_balance_before - amount, relay_sender_balance_after);
	assert_eq!(para_sender_balance_after, para_receiver_balance_before);
}
