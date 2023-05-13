// use statemint_it::*;

// #[test]
// fn transact_sudo_from_relay_to_assets_para() {
// 	Polkadot::execute_with(|| {
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
