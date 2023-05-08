// use statemint_it::*;

// #[test]
// fn transact_sudo_relay_to_assets_para_works() {
// 	force_xcm_version();

// 	Polkadot::execute_with(|| {
// 		use polkadot_runtime::{RuntimeEvent, RuntimeOrigin, System};

// 		let call = statemint_runtime::RuntimeCall::Assets(pallet_assets::Call::<
// 			statemint_runtime::Runtime,
// 			Instance1,
// 		>::force_create {
// 			id: 1.into(),
// 			is_sufficient: true,
// 			min_balance: 1000,
// 			owner: ALICE.into(),
// 		});
// 		let xcm = Xcm(vec![
// 			UnpaidExecution { weight_limit: WeightLimit::Unlimited, check_origin: None },
// 			Transact {
// 				require_weight_at_most: Weight::from_parts(1000000000, 200000),
// 				origin_kind: OriginKind::Superuser,
// 				call: call.encode().into(),
// 			},
// 		]);
// 		assert_ok!(RelayChainPalletXcm::send(
// 			RuntimeOrigin::root(),
// 			Box::new(AP_DEST.into()),
// 			Box::new(VersionedXcm::from(xcm)),
// 		));
// 		assert!(System::events().iter().any(|r| matches!(
// 			r.event,
// 			RuntimeEvent::XcmPallet(pallet_xcm::Event::Sent { .. })
// 		)));
// 	});

// 	Statemint::execute_with(|| {
// 		assert!(statemint_runtime::Assets::asset_exists(1));
// 	});
// }

// #[test]
// fn reserved_transfer_native_relay_to_assets_para_fails() {
// 	force_xcm_version();
// 	let (relay_balance, ap_balance) = get_balances();
// 	let amount = 1000_000_000;
// 	let assets: VersionedMultiAssets = (Here, amount).into();

// 	Polkadot::execute_with(|| {
// 		use polkadot_runtime::{RuntimeEvent, RuntimeOrigin, System};

// 		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
// 			RuntimeOrigin::signed(ALICE.into()),
// 			Box::new(AP_DEST.into()),
// 			Box::new(get_benf().into()),
// 			Box::new(assets),
// 			0,
// 			WeightLimit::Unlimited,
// 		));

// 		assert!(System::events().iter().any(|r| matches!(
// 			r.event,
// 			RuntimeEvent::XcmPallet(pallet_xcm::Event::Attempted(Outcome::Complete { .. }))
// 		)));
// 	});

// 	Statemint::execute_with(|| {
// 		use statemint_runtime::{RuntimeEvent, System};

// 		assert!(System::events().iter().any(|r| matches!(
// 			r.event,
// 			RuntimeEvent::DmpQueue(cumulus_pallet_dmp_queue::Event::ExecutedDownward {
// 				outcome: Outcome::Incomplete(_, Error::UntrustedReserveLocation),
// 				..
// 			})
// 		)));
// 	});

// 	let (relay_balance_after, ap_balance_after) = get_balances();
// 	assert_eq!(relay_balance - amount, relay_balance_after);
// 	assert_eq!(ap_balance_after, ap_balance);
// }
