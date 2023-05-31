use crate::*;
use frame_support::{instances::Instance2, BoundedVec};
// use pallet_asset_conversion::Call;
use xcm_emulator::Parachain;

#[test]
fn swap_locally_on_chain_using_local_assets() {
	const ASSET_ID: u32 = 1;

	let asset_native: MultiLocation = MultiLocation { parents: 0, interior: Here };
	let asset_one: MultiLocation =
		MultiLocation { parents: 0, interior: X2(PalletInstance(50), GeneralIndex(1)) };

	Statemine::execute_with(|| {
		use statemine_runtime::RuntimeEvent;

		assert_ok!(<Statemine as StateminePallet>::Assets::create(
			<Statemine as Parachain>::RuntimeOrigin::signed(StatemineSender::get()),
			ASSET_ID.into(),
			StatemineSender::get().into(),
			1000,
		));
		assert!(<Statemine as StateminePallet>::Assets::asset_exists(ASSET_ID));

		assert_ok!(<Statemine as StateminePallet>::Assets::mint(
			<Statemine as Parachain>::RuntimeOrigin::signed(StatemineSender::get()),
			ASSET_ID.into(),
			StatemineSender::get().into(),
			100_000_000_000,
		));

		assert_ok!(<Statemine as StateminePallet>::AssetConversion::create_pool(
			<Statemine as Parachain>::RuntimeOrigin::signed(StatemineSender::get()),
			asset_native,
			asset_one,
		));

		assert_expected_events!(
			Statemine,
			vec![
				RuntimeEvent::AssetConversion(pallet_asset_conversion::Event::PoolCreated { .. }) => {},
			]
		);

		assert_ok!(<Statemine as StateminePallet>::AssetConversion::add_liquidity(
			<Statemine as Parachain>::RuntimeOrigin::signed(StatemineSender::get()),
			asset_native,
			asset_one,
			1_000_000_000, // 33_333_333 min ksm
			2_000_000_000, // 1_000_000_000 min
			33_333_333,
			1_000,
			StatemineSender::get().into()
		));

		assert_expected_events!(
			Statemine,
			vec![
				RuntimeEvent::AssetConversion(pallet_asset_conversion::Event::LiquidityAdded {lp_token_minted: 1414213462, .. }) => {},
			]
		);

		let path = BoundedVec::<_, _>::truncate_from(vec![asset_native, asset_one]);

		assert_ok!(<Statemine as StateminePallet>::AssetConversion::swap_exact_tokens_for_tokens(
			<Statemine as Parachain>::RuntimeOrigin::signed(StatemineSender::get()),
			path,
			100,
			1,
			StatemineSender::get().into(),
			true
		));

		assert_expected_events!(
			Statemine,
			vec![
				RuntimeEvent::AssetConversion(pallet_asset_conversion::Event::SwapExecuted { amount_in: 100, amount_out: 199, .. }) => {},
			]
		);

		assert_ok!(<Statemine as StateminePallet>::AssetConversion::remove_liquidity(
			<Statemine as Parachain>::RuntimeOrigin::signed(StatemineSender::get()),
			asset_native,
			asset_one,
			(1_414_213_462 as f32 * 0.966/* all but exit fee */) as u128,
			33_333_333,
			1_000,
			StatemineSender::get().into(),
		));
	});
}

#[test]
fn swap_locally_on_chain_using_foreign_assets() {
	use frame_support::weights::WeightToFee;
	// Init tests variables
	// Call to be executed in Assets Parachain
	const ASSET_ID: u32 = 1;

	// let asset_native_at_statemine: MultiLocation = MultiLocation { parents: 0, interior: Here };

	let foreign_asset1_at_statemine: MultiLocation = MultiLocation {
		parents: 1,
		interior: X3(
			Parachain(PenpalKusama::para_id().into()),
			PalletInstance(50),
			GeneralIndex(1),
		),
	};

	let assets_para_destination: VersionedMultiLocation =
		MultiLocation { parents: 1, interior: X1(Parachain(Statemine::para_id().into())) }.into();

	let penpal_location =
		MultiLocation { parents: 1, interior: X1(Parachain(PenpalKusama::para_id().into())) };

	PenpalKusama::execute_with(|| {
		assert_ok!(<PenpalKusama as PenpalKusamaPallet>::Assets::create(
			<PenpalKusama as Parachain>::RuntimeOrigin::signed(PenpalKusamaSender::get()),
			ASSET_ID.into(),
			PenpalKusamaSender::get().into(),
			1000,
		));
		// assert!(<Statemine as StateminePallet>::Assets::asset_exists(ASSET_ID));

		assert!(<PenpalKusama as PenpalKusamaPallet>::Assets::asset_exists(ASSET_ID));
	});

	// XcmPallet send arguments
	let sudo_origin = <PenpalKusama as Parachain>::RuntimeOrigin::root();
	// let soverign_origin = <PenpalKusama as Parachain>::RuntimeOrigin::root();

	// let weight_limit = WeightLimit::Unlimited;
	let require_weight_at_most = Weight::from_parts(1_100_000_000_000, 30_000);
	let origin_kind = OriginKind::Xcm; //OriginKind::SovereignAccount;//Superuser;
								   // let check_origin = None;

	let sov_penpal_on_statemine = Statemine::sovereign_account_id_of(penpal_location);
	let sov_penpal_on_penpal = PenpalKusama::sovereign_account_id_of(penpal_location);
	Statemine::fund_accounts(vec![(sov_penpal_on_statemine.clone(), 1_000_000_000_000_000)]);
	PenpalKusama::fund_accounts(vec![(sov_penpal_on_penpal, 1_000_000_000_000_000)]);
	let sov_penpal_on_statemine_as_location: MultiLocation = MultiLocation {
		parents: 0,
		interior: X1(AccountId32 { network: None, id: sov_penpal_on_statemine.clone().into() }),
	};

	let call = <Statemine as Para>::RuntimeCall::ForeignAssets(pallet_assets::Call::<
		<Statemine as Para>::Runtime,
		Instance2,
	>::create {
		id: foreign_asset1_at_statemine,
		min_balance: 1000,
		admin: sov_penpal_on_statemine.into(),
	})
	.encode()
	.into();

	// let call = <Statemine as Para>::RuntimeCall::AssetConversion(pallet_asset_conversion::Call::<
	// 	<Statemine as Para>::Runtime
	// >::create_pool {
	// 	asset1: asset_native_at_statemine,
	// 	asset2: foreign_asset1_at_statemine,
	// })
	// 	.encode()
	// 	.into();

	let buy_execution_fee_amount = penpal_runtime::WeightToFee::weight_to_fee(&Weight::from_parts(
		10_100_000_000_000,
		300_000,
	));
	let buy_execution_fee = MultiAsset {
		id: Concrete(MultiLocation { parents: 1, interior: Here }),
		fun: Fungible(buy_execution_fee_amount),
	};

	let xcm = VersionedXcm::from(Xcm(vec![
		WithdrawAsset { 0: vec![buy_execution_fee.clone()].into() },
		BuyExecution { fees: buy_execution_fee, weight_limit: Unlimited },
		Transact { require_weight_at_most, origin_kind, call },
		RefundSurplus,
		DepositAsset { assets: All.into(), beneficiary: sov_penpal_on_statemine_as_location },
	]));

	// Send XCM message from penpal => statemine
	PenpalKusama::execute_with(|| {
		assert_ok!(<PenpalKusama as PenpalKusamaPallet>::PolkadotXcm::send(
			sudo_origin,
			bx!(assets_para_destination),
			bx!(xcm),
		));

		type RuntimeEvent = <PenpalKusama as Parachain>::RuntimeEvent;

		PenpalKusama::events().iter().for_each(|event| {
			println!("penpal {:?}", event);
		});
		assert_expected_events!(
			PenpalKusama,
			vec![
				RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
			]
		);
	});

	// Receive XCM message in Assets Parachain
	Statemine::execute_with(|| {
		Statemine::events().iter().for_each(|event| {
			println!("statemine {:?}", event);
		});
		assert!(<Statemine as StateminePallet>::ForeignAssets::asset_exists(
			foreign_asset1_at_statemine
		));
	});
}
