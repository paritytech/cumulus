use asset_test_utils::{ExtBuilder, RuntimeHelper};
use codec::Encode;
use cumulus_primitives_utility::ChargeWeightInFungibles;
use frame_support::{
	assert_noop, assert_ok, sp_io,
	traits::PalletInfo,
	weights::{Weight, WeightToFee as WeightToFeeT},
};
use parachains_common::{AccountId, AuraId};
use westmint_runtime::xcm_config::AssetFeeAsExistentialDepositMultiplierFeeCharger;
pub use westmint_runtime::{
	constants::fee::WeightToFee,
	xcm_config::{LocationToAccountId, XcmConfig},
	Assets, Balances, ExistentialDeposit, Runtime, RuntimeCall, RuntimeEvent, SessionKeys, System,
};
use xcm::latest::prelude::*;
use xcm_executor::{traits::WeightTrader, XcmExecutor};

pub const ALICE: [u8; 32] = [1u8; 32];

#[test]
fn test_asset_xcm_trader() {
	ExtBuilder::<Runtime>::default()
		.with_collators(vec![AccountId::from(ALICE)])
		.with_session_keys(vec![(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) },
		)])
		.build()
		.execute_with(|| {
			// We need root origin to create a sufficient asset
			let minimum_asset_balance = 3333333_u128;
			let local_asset_id = 1;
			assert_ok!(Assets::force_create(
				RuntimeHelper::<Runtime>::root_origin(),
				local_asset_id.into(),
				AccountId::from(ALICE).into(),
				true,
				minimum_asset_balance
			));

			// We first mint enough asset for the account to exist for assets
			assert_ok!(Assets::mint(
				RuntimeHelper::<Runtime>::origin_of(AccountId::from(ALICE)),
				local_asset_id.into(),
				AccountId::from(ALICE).into(),
				minimum_asset_balance
			));

			// get asset id as multilocation
			let asset_multilocation = MultiLocation::new(
				0,
				X2(
					PalletInstance(
						<Runtime as frame_system::Config>::PalletInfo::index::<Assets>().unwrap()
							as u8,
					),
					GeneralIndex(local_asset_id.into()),
				),
			);

			// Set Alice as block author, who will receive fees
			RuntimeHelper::<Runtime>::run_to_block(2, Some(AccountId::from(ALICE)));

			// We are going to buy 4e9 weight
			let bought = Weight::from_ref_time(4_000_000_000u64);

			// Lets calculate amount needed
			let asset_amount_needed =
				AssetFeeAsExistentialDepositMultiplierFeeCharger::charge_weight_in_fungibles(
					local_asset_id,
					bought,
				)
				.expect("failed to compute");

			// Lets pay with: asset_amount_needed + asset_amount_extra
			let asset_amount_extra = 100_u128;
			let asset: MultiAsset =
				(asset_multilocation.clone(), asset_amount_needed + asset_amount_extra).into();

			let mut trader = <XcmConfig as xcm_executor::Config>::Trader::new();

			// Lets buy_weight and make sure buy_weight does not return an error
			match trader.buy_weight(bought, asset.into()) {
				Ok(unused_assets) => {
					// Check whether a correct amount of unused assets is returned
					assert_ok!(unused_assets
						.ensure_contains(&(asset_multilocation, asset_amount_extra).into()));
				},
				Err(e) => assert!(false, "Expected Ok(_). Got {:#?}", e),
			}

			// Drop trader
			drop(trader);

			// Make sure author(Alice) has received the amount
			assert_eq!(
				Assets::balance(1, AccountId::from(ALICE)),
				minimum_asset_balance + asset_amount_needed
			);

			// We also need to ensure the total supply increased
			assert_eq!(Assets::total_supply(1), minimum_asset_balance + asset_amount_needed);
		});
}

#[test]
fn test_asset_xcm_trader_with_refund() {
	ExtBuilder::<Runtime>::default()
		.with_collators(vec![AccountId::from(ALICE)])
		.with_session_keys(vec![(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) },
		)])
		.build()
		.execute_with(|| {
			// We need root origin to create a sufficient asset
			// We set existential deposit to be identical to the one for Balances first
			assert_ok!(Assets::force_create(
				RuntimeHelper::<Runtime>::root_origin(),
				1.into(),
				AccountId::from(ALICE).into(),
				true,
				ExistentialDeposit::get()
			));

			// We first mint enough asset for the account to exist for assets
			assert_ok!(Assets::mint(
				RuntimeHelper::<Runtime>::origin_of(AccountId::from(ALICE)),
				1.into(),
				AccountId::from(ALICE).into(),
				ExistentialDeposit::get()
			));

			let mut trader = <XcmConfig as xcm_executor::Config>::Trader::new();

			// Set Alice as block author, who will receive fees
			RuntimeHelper::<Runtime>::run_to_block(2, Some(AccountId::from(ALICE)));

			// We are going to buy 4e9 weight
			let bought = Weight::from_ref_time(4_000_000_000u64);
			let asset_multilocation = MultiLocation::new(
				0,
				X2(
					PalletInstance(
						<Runtime as frame_system::Config>::PalletInfo::index::<Assets>().unwrap()
							as u8,
					),
					GeneralIndex(1),
				),
			);

			// lets calculate amount needed
			let amount_bought = WeightToFee::weight_to_fee(&bought);

			let asset: MultiAsset = (asset_multilocation.clone(), amount_bought).into();

			// Make sure buy_weight does not return an error
			assert_ok!(trader.buy_weight(bought, asset.clone().into()));

			// Make sure again buy_weight does return an error
			assert_noop!(trader.buy_weight(bought, asset.into()), XcmError::NotWithdrawable);

			// We actually use half of the weight
			let weight_used = bought / 2;

			// Make sure refurnd works.
			let amount_refunded = WeightToFee::weight_to_fee(&(bought - weight_used));

			assert_eq!(
				trader.refund_weight(bought - weight_used),
				Some((asset_multilocation, amount_refunded).into())
			);

			// Drop trader
			drop(trader);

			// We only should have paid for half of the bought weight
			let fees_paid = WeightToFee::weight_to_fee(&weight_used);

			assert_eq!(
				Assets::balance(1, AccountId::from(ALICE)),
				ExistentialDeposit::get() + fees_paid
			);

			// We also need to ensure the total supply increased
			assert_eq!(Assets::total_supply(1), ExistentialDeposit::get() + fees_paid);
		});
}

#[test]
fn test_asset_xcm_trader_refund_not_possible_since_amount_less_than_ed() {
	ExtBuilder::<Runtime>::default()
		.with_collators(vec![AccountId::from(ALICE)])
		.with_session_keys(vec![(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) },
		)])
		.build()
		.execute_with(|| {
			// We need root origin to create a sufficient asset
			// We set existential deposit to be identical to the one for Balances first
			assert_ok!(Assets::force_create(
				RuntimeHelper::<Runtime>::root_origin(),
				1.into(),
				AccountId::from(ALICE).into(),
				true,
				ExistentialDeposit::get()
			));

			let mut trader = <XcmConfig as xcm_executor::Config>::Trader::new();

			// Set Alice as block author, who will receive fees
			RuntimeHelper::<Runtime>::run_to_block(2, Some(AccountId::from(ALICE)));

			// We are going to buy 5e9 weight
			let bought = Weight::from_ref_time(500_000_000u64);

			let asset_multilocation = MultiLocation::new(
				0,
				X2(
					PalletInstance(
						<Runtime as frame_system::Config>::PalletInfo::index::<Assets>().unwrap()
							as u8,
					),
					GeneralIndex(1),
				),
			);

			let amount_bought = WeightToFee::weight_to_fee(&bought);

			assert!(
				amount_bought < ExistentialDeposit::get(),
				"we are testing what happens when the amount does not exceed ED"
			);

			let asset: MultiAsset = (asset_multilocation.clone(), amount_bought).into();

			// Buy weight should return an error
			assert_noop!(trader.buy_weight(bought, asset.into()), XcmError::TooExpensive);

			// not credited since the ED is higher than this value
			assert_eq!(Assets::balance(1, AccountId::from(ALICE)), 0);

			// We also need to ensure the total supply did not increase
			assert_eq!(Assets::total_supply(1), 0);
		});
}

#[test]
fn test_that_buying_ed_refund_does_not_refund() {
	ExtBuilder::<Runtime>::default()
		.with_collators(vec![AccountId::from(ALICE)])
		.with_session_keys(vec![(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) },
		)])
		.build()
		.execute_with(|| {
			// We need root origin to create a sufficient asset
			// We set existential deposit to be identical to the one for Balances first
			assert_ok!(Assets::force_create(
				RuntimeHelper::<Runtime>::root_origin(),
				1.into(),
				AccountId::from(ALICE).into(),
				true,
				ExistentialDeposit::get()
			));

			let mut trader = <XcmConfig as xcm_executor::Config>::Trader::new();

			// Set Alice as block author, who will receive fees
			RuntimeHelper::<Runtime>::run_to_block(2, Some(AccountId::from(ALICE)));

			let bought = Weight::from_ref_time(500_000_000u64);

			let asset_multilocation = MultiLocation::new(
				0,
				X2(
					PalletInstance(
						<Runtime as frame_system::Config>::PalletInfo::index::<Assets>().unwrap()
							as u8,
					),
					GeneralIndex(1),
				),
			);

			let amount_bought = WeightToFee::weight_to_fee(&bought);

			assert!(
				amount_bought < ExistentialDeposit::get(),
				"we are testing what happens when the amount does not exceed ED"
			);

			// We know we will have to buy at least ED, so lets make sure first it will
			// fail with a payment of less than ED
			let asset: MultiAsset = (asset_multilocation.clone(), amount_bought).into();
			assert_noop!(trader.buy_weight(bought, asset.into()), XcmError::TooExpensive);

			// Now lets buy ED at least
			let asset: MultiAsset = (asset_multilocation.clone(), ExistentialDeposit::get()).into();

			// Buy weight should work
			assert_ok!(trader.buy_weight(bought, asset.into()));

			// Should return None. We have a specific check making sure we dont go below ED for
			// drop payment
			assert_eq!(trader.refund_weight(bought), None);

			// Drop trader
			drop(trader);

			// Make sure author(Alice) has received the amount
			assert_eq!(Assets::balance(1, AccountId::from(ALICE)), ExistentialDeposit::get());

			// We also need to ensure the total supply increased
			assert_eq!(Assets::total_supply(1), ExistentialDeposit::get());
		});
}

#[test]
fn test_asset_xcm_trader_not_possible_for_non_sufficient_assets() {
	ExtBuilder::<Runtime>::default()
		.with_collators(vec![AccountId::from(ALICE)])
		.with_session_keys(vec![(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) },
		)])
		.build()
		.execute_with(|| {
			// Create a non-sufficient asset with specific existential deposit
			let minimum_asset_balance = 1_000_000_u128;
			assert_ok!(Assets::force_create(
				RuntimeHelper::<Runtime>::root_origin(),
				1.into(),
				AccountId::from(ALICE).into(),
				false,
				minimum_asset_balance
			));

			// We first mint enough asset for the account to exist for assets
			assert_ok!(Assets::mint(
				RuntimeHelper::<Runtime>::origin_of(AccountId::from(ALICE)),
				1.into(),
				AccountId::from(ALICE).into(),
				minimum_asset_balance
			));

			let mut trader = <XcmConfig as xcm_executor::Config>::Trader::new();

			// Set Alice as block author, who will receive fees
			RuntimeHelper::<Runtime>::run_to_block(2, Some(AccountId::from(ALICE)));

			// We are going to buy 4e9 weight
			let bought = Weight::from_ref_time(4_000_000_000u64);

			// lets calculate amount needed
			let asset_amount_needed = WeightToFee::weight_to_fee(&bought);

			let asset_multilocation = MultiLocation::new(
				0,
				X2(
					PalletInstance(
						<Runtime as frame_system::Config>::PalletInfo::index::<Assets>().unwrap()
							as u8,
					),
					GeneralIndex(1),
				),
			);

			let asset: MultiAsset = (asset_multilocation, asset_amount_needed).into();

			// Make sure again buy_weight does return an error
			assert_noop!(trader.buy_weight(bought, asset.into()), XcmError::TooExpensive);

			// Drop trader
			drop(trader);

			// Make sure author(Alice) has NOT received the amount
			assert_eq!(Assets::balance(1, AccountId::from(ALICE)), minimum_asset_balance);

			// We also need to ensure the total supply NOT increased
			assert_eq!(Assets::total_supply(1), minimum_asset_balance);
		});
}

#[test]
fn test_receive_bridged_xcm_transact_with_remark_with_event_works() {
	ExtBuilder::<Runtime>::default()
		.with_collators(vec![AccountId::from(ALICE)])
		.with_session_keys(vec![(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) },
		)])
		.with_tracing()
		.build()
		.execute_with(|| {
			let remark_with_event: RuntimeCall =
				RuntimeCall::System(frame_system::Call::<Runtime>::remark_with_event {
					remark: b"Hello".to_vec(),
				});

			// simulate received message:
			// 2022-12-21 14:38:54.047 DEBUG tokio-runtime-worker xcm::execute_xcm: [Parachain] origin: MultiLocation { parents: 1, interior: X1(Parachain(1014)) }, message: Xcm([UniversalOrigin(GlobalConsensus(Rococo)), DescendOrigin(X1(AccountId32 { network: Some(Rococo), id: [28, 189, 45, 67, 83, 10, 68, 112, 90, 208, 136, 175, 49, 62, 24, 248, 11, 83, 239, 22, 179, 97, 119, 205, 75, 119, 184, 70, 242, 165, 240, 124] })), Transact { origin_kind: SovereignAccount, require_weight_at_most: 1000000000, call: [0, 8, 20, 104, 101, 108, 108, 111] }]), weight_limit: 41666666666
			// origin as local BridgeHub (Wococo)
			let origin = MultiLocation { parents: 1, interior: X1(Parachain(1014)) };
			let xcm = Xcm(vec![
				UniversalOrigin(GlobalConsensus(Rococo)),
				DescendOrigin(X2(
					Parachain(1000),
					AccountId32 {
						network: Some(Rococo),
						id: [
							28, 189, 45, 67, 83, 10, 68, 112, 90, 208, 136, 175, 49, 62, 24, 248,
							11, 83, 239, 22, 179, 97, 119, 205, 75, 119, 184, 70, 242, 165, 240,
							124,
						],
					},
				)),
				Transact {
					origin_kind: OriginKind::SovereignAccount,
					require_weight_at_most: Weight::from_ref_time(1000000000),
					call: remark_with_event.encode().into(),
				},
			]);
			let hash = xcm.using_encoded(sp_io::hashing::blake2_256);
			let weight_limit = Weight::from_ref_time(41666666666);

			let outcome = XcmExecutor::<XcmConfig>::execute_xcm(origin, xcm, hash, weight_limit);
			assert_eq!(outcome.ensure_complete(), Ok(()));

			// check Event::Remarked occured
			let events = System::events();
			assert!(!events.is_empty());

			let expected_event = {
				use sp_runtime::traits::Hash;
				use xcm_executor::traits::Convert;
				RuntimeEvent::System(frame_system::Event::Remarked {
					hash: <Runtime as frame_system::Config>::Hashing::hash(b"Hello"),
					// origin should match here according to [`BridgedSignedAccountId32AsNative`]
					sender: LocationToAccountId::convert(origin).unwrap(),
				})
			};
			assert!(System::events().iter().any(|r| r.event == expected_event));
		});
}

#[test]
fn test_receive_bridged_xcm_trap_works() {
	ExtBuilder::<Runtime>::default()
		.with_collators(vec![AccountId::from(ALICE)])
		.with_session_keys(vec![(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) },
		)])
		.with_tracing()
		.build()
		.execute_with(|| {
			// simulate received message:
			// 2022-12-21 14:38:54.047 DEBUG tokio-runtime-worker xcm::execute_xcm: [Parachain] origin: MultiLocation { parents: 1, interior: X1(Parachain(1014)) }, message: Xcm([UniversalOrigin(GlobalConsensus(Rococo)), DescendOrigin(X1(AccountId32 { network: Some(Rococo), id: [28, 189, 45, 67, 83, 10, 68, 112, 90, 208, 136, 175, 49, 62, 24, 248, 11, 83, 239, 22, 179, 97, 119, 205, 75, 119, 184, 70, 242, 165, 240, 124] })), Transact { origin_kind: SovereignAccount, require_weight_at_most: 1000000000, call: [0, 8, 20, 104, 101, 108, 108, 111] }]), weight_limit: 41666666666
			// origin as BridgeHub
			let origin = MultiLocation { parents: 1, interior: X1(Parachain(1014)) };
			let xcm = Xcm(vec![
				UniversalOrigin(GlobalConsensus(Rococo)),
				DescendOrigin(X1(AccountId32 {
					network: Some(Rococo),
					id: [
						28, 189, 45, 67, 83, 10, 68, 112, 90, 208, 136, 175, 49, 62, 24, 248, 11,
						83, 239, 22, 179, 97, 119, 205, 75, 119, 184, 70, 242, 165, 240, 124,
					],
				})),
				Trap(1234),
			]);
			let hash = xcm.using_encoded(sp_io::hashing::blake2_256);
			let weight_limit = Weight::from_ref_time(41666666666);

			let outcome = XcmExecutor::<XcmConfig>::execute_xcm(origin, xcm, hash, weight_limit);
			assert_eq!(outcome.ensure_complete(), Err(xcm::latest::Error::Trap(1234)));
		});
}
