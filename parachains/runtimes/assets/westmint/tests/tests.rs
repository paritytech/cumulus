use asset_test_utils::{ExtBuilder, RuntimeHelper, XcmReceivedFrom};
use codec::{DecodeLimit, Encode};
use cumulus_primitives_utility::ChargeWeightInFungibles;
use frame_support::{
	assert_noop, assert_ok, sp_io,
	traits::fungibles::InspectEnumerable,
	weights::{Weight, WeightToFee as WeightToFeeT},
};
use parachains_common::{AccountId, AssetIdForTrustBackedAssets, AuraId, Balance};
use std::convert::Into;
pub use westmint_runtime::{
	constants::fee::WeightToFee,
	xcm_config::{TrustBackedAssetsPalletLocation, XcmConfig},
	AssetDeposit, Assets, Balances, ExistentialDeposit, ForeignAssets, ForeignAssetsInstance,
	Runtime, SessionKeys, System,
};
use westmint_runtime::{
	xcm_config::{
		AssetFeeAsExistentialDepositMultiplierFeeCharger, ForeignCreatorsSovereignAccountOf,
		WestendLocation,
	},
	MetadataDepositBase, RuntimeCall, RuntimeEvent,
};
use xcm::{latest::prelude::*, VersionedXcm, MAX_XCM_DECODE_DEPTH};
use xcm_builder::AsPrefixedGeneralIndex;
use xcm_executor::{
	traits::{Convert, Identity, JustTry, WeightTrader},
	XcmExecutor,
};

pub const ALICE: [u8; 32] = [1u8; 32];
pub const BOB: [u8; 32] = [2u8; 32];
pub const CHARLIE: [u8; 32] = [3u8; 32];
pub const SOME_ASSET_OWNER: [u8; 32] = [4u8; 32];
pub const SOME_ASSET_ADMIN: [u8; 32] = [5u8; 32];

type AssetIdForTrustBackedAssetsConvert =
	assets_common::AssetIdForTrustBackedAssetsConvert<TrustBackedAssetsPalletLocation>;

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
			let asset_multilocation =
				AssetIdForTrustBackedAssetsConvert::reverse_ref(local_asset_id).unwrap();

			// Set Alice as block author, who will receive fees
			RuntimeHelper::<Runtime>::run_to_block(2, Some(AccountId::from(ALICE)));

			// We are going to buy 4e9 weight
			let bought = Weight::from_parts(4_000_000_000u64, 0);

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
				Assets::balance(local_asset_id, AccountId::from(ALICE)),
				minimum_asset_balance + asset_amount_needed
			);

			// We also need to ensure the total supply increased
			assert_eq!(
				Assets::total_supply(local_asset_id),
				minimum_asset_balance + asset_amount_needed
			);
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
			let bought = Weight::from_parts(4_000_000_000u64, 0);
			let asset_multilocation = AssetIdForTrustBackedAssetsConvert::reverse_ref(1).unwrap();

			// lets calculate amount needed
			let amount_bought = WeightToFee::weight_to_fee(&bought);

			let asset: MultiAsset = (asset_multilocation.clone(), amount_bought).into();

			// Make sure buy_weight does not return an error
			assert_ok!(trader.buy_weight(bought, asset.clone().into()));

			// Make sure again buy_weight does return an error
			// This assert relies on the fact, that we use `TakeFirstAssetTrader` in `WeightTrader` tuple chain, which cannot be called twice
			assert_noop!(trader.buy_weight(bought, asset.into()), XcmError::TooExpensive);

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
			let bought = Weight::from_parts(500_000_000u64, 0);

			let asset_multilocation = AssetIdForTrustBackedAssetsConvert::reverse_ref(1).unwrap();

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

			let bought = Weight::from_parts(500_000_000u64, 0);

			let asset_multilocation = AssetIdForTrustBackedAssetsConvert::reverse_ref(1).unwrap();

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
			let bought = Weight::from_parts(4_000_000_000u64, 0);

			// lets calculate amount needed
			let asset_amount_needed = WeightToFee::weight_to_fee(&bought);

			let asset_multilocation = AssetIdForTrustBackedAssetsConvert::reverse_ref(1).unwrap();

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
fn test_assets_balances_api_works() {
	use assets_common::runtime_api::runtime_decl_for_FungiblesApi::FungiblesApi;

	ExtBuilder::<Runtime>::default()
		.with_collators(vec![AccountId::from(ALICE)])
		.with_session_keys(vec![(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) },
		)])
		.build()
		.execute_with(|| {
			let local_asset_id = 1;
			let foreign_asset_id_multilocation =
				MultiLocation { parents: 2, interior: X1(GlobalConsensus(Kusama)) };

			// check before
			assert_eq!(Assets::balance(local_asset_id, AccountId::from(ALICE)), 0);
			assert_eq!(
				ForeignAssets::balance(foreign_asset_id_multilocation, AccountId::from(ALICE)),
				0
			);
			assert_eq!(Balances::free_balance(AccountId::from(ALICE)), 0);
			assert!(Runtime::query_account_balances(AccountId::from(ALICE)).unwrap().is_empty());

			// Drip some balance
			use frame_support::traits::fungible::Mutate;
			let some_currency = ExistentialDeposit::get();
			Balances::mint_into(&AccountId::from(ALICE), some_currency).unwrap();

			// We need root origin to create a sufficient asset
			let minimum_asset_balance = 3333333_u128;
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

			// create foreign asset
			let foreign_asset_minimum_asset_balance = 3333333_u128;
			assert_ok!(ForeignAssets::force_create(
				RuntimeHelper::<Runtime>::root_origin(),
				foreign_asset_id_multilocation.clone().into(),
				AccountId::from(SOME_ASSET_ADMIN).into(),
				false,
				foreign_asset_minimum_asset_balance
			));

			// We first mint enough asset for the account to exist for assets
			assert_ok!(ForeignAssets::mint(
				RuntimeHelper::<Runtime>::origin_of(AccountId::from(SOME_ASSET_ADMIN)),
				foreign_asset_id_multilocation.clone().into(),
				AccountId::from(ALICE).into(),
				6 * foreign_asset_minimum_asset_balance
			));

			// check after
			assert_eq!(
				Assets::balance(local_asset_id, AccountId::from(ALICE)),
				minimum_asset_balance
			);
			assert_eq!(
				ForeignAssets::balance(foreign_asset_id_multilocation, AccountId::from(ALICE)),
				6 * minimum_asset_balance
			);
			assert_eq!(Balances::free_balance(AccountId::from(ALICE)), some_currency);

			let result = Runtime::query_account_balances(AccountId::from(ALICE)).unwrap();
			assert_eq!(result.len(), 3);

			// check currency
			assert!(result.iter().any(|asset| asset.eq(
				&assets_common::fungible_conversion::convert_balance::<WestendLocation, Balance>(
					some_currency
				)
				.unwrap()
			)));
			// check trusted asset
			assert!(result.iter().any(|asset| asset.eq(&(
				AssetIdForTrustBackedAssetsConvert::reverse_ref(local_asset_id).unwrap(),
				minimum_asset_balance
			)
				.into())));
			// check foreign asset
			assert!(result.iter().any(|asset| asset.eq(&(
				Identity::reverse_ref(foreign_asset_id_multilocation).unwrap(),
				6 * foreign_asset_minimum_asset_balance
			)
				.into())));
		});
}

asset_test_utils::include_receive_teleported_asset_for_native_asset_works!(
	Runtime,
	XcmConfig,
	asset_test_utils::CollatorSessionKeys::new(
		AccountId::from(ALICE),
		AccountId::from(ALICE),
		SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) }
	)
);

asset_test_utils::include_receive_teleported_asset_from_foreign_creator_works!(
	Runtime,
	XcmConfig,
	WeightToFee,
	ForeignCreatorsSovereignAccountOf,
	ForeignAssetsInstance,
	asset_test_utils::CollatorSessionKeys::new(
		AccountId::from(ALICE),
		AccountId::from(ALICE),
		SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) }
	),
	ExistentialDeposit::get()
);

#[test]
fn plain_receive_teleported_asset_works() {
	ExtBuilder::<Runtime>::default()
		.with_collators(vec![AccountId::from(ALICE)])
		.with_session_keys(vec![(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) },
		)])
		.build()
		.execute_with(|| {
			let data = hex_literal::hex!("02100204000100000b00a0724e18090a13000100000b00a0724e180901e20f5e480d010004000101001299557001f55815d3fcb53c74463acb0cf6d14d4639b340982c60877f384609").to_vec();
			let message_id = sp_io::hashing::blake2_256(&data);

			let maybe_msg = VersionedXcm::<RuntimeCall>::decode_all_with_depth_limit(
				MAX_XCM_DECODE_DEPTH,
				&mut data.as_ref(),
			)
				.map(xcm::v3::Xcm::<RuntimeCall>::try_from).expect("failed").expect("failed");

			let outcome =
				XcmExecutor::<XcmConfig>::execute_xcm(Parent, maybe_msg, message_id, RuntimeHelper::<Runtime>::xcm_max_weight(XcmReceivedFrom::Parent));
			assert_eq!(outcome.ensure_complete(), Ok(()));
		})
}

asset_test_utils::include_asset_transactor_transfer_with_local_consensus_currency_works!(
	Runtime,
	XcmConfig,
	asset_test_utils::CollatorSessionKeys::new(
		AccountId::from(ALICE),
		AccountId::from(ALICE),
		SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) }
	),
	ExistentialDeposit::get(),
	Box::new(|| {
		assert!(Assets::asset_ids().collect::<Vec<_>>().is_empty());
		assert!(ForeignAssets::asset_ids().collect::<Vec<_>>().is_empty());
	}),
	Box::new(|| {
		assert!(Assets::asset_ids().collect::<Vec<_>>().is_empty());
		assert!(ForeignAssets::asset_ids().collect::<Vec<_>>().is_empty());
	})
);

#[test]
fn test_asset_transactor_transfer_with_trust_backed_assets_works() {
	ExtBuilder::<Runtime>::default()
		.with_collators(vec![AccountId::from(ALICE)])
		.with_session_keys(vec![(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) },
		)])
		.with_balances(vec![
			(AccountId::from(SOME_ASSET_OWNER), ExistentialDeposit::get() + AssetDeposit::get()),
			(AccountId::from(ALICE), ExistentialDeposit::get()),
			(AccountId::from(BOB), ExistentialDeposit::get()),
		])
		.with_tracing()
		.build()
		.execute_with(|| {
			// create  some asset class
			let asset_minimum_asset_balance = 3333333_u128;
			let local_asset_id = 1;
			let local_asset_id_as_multilocation = {
				type AssetIdConverter = AsPrefixedGeneralIndex<
					TrustBackedAssetsPalletLocation,
					AssetIdForTrustBackedAssets,
					JustTry,
				>;
				AssetIdConverter::reverse_ref(local_asset_id).unwrap()
			};
			assert_ok!(Assets::create(
				RuntimeHelper::<Runtime>::origin_of(AccountId::from(SOME_ASSET_OWNER)),
				local_asset_id.into(),
				AccountId::from(SOME_ASSET_OWNER).into(),
				asset_minimum_asset_balance
			));

			// We first mint enough asset for the account to exist for assets
			assert_ok!(Assets::mint(
				RuntimeHelper::<Runtime>::origin_of(AccountId::from(SOME_ASSET_OWNER)),
				local_asset_id.into(),
				AccountId::from(ALICE).into(),
				6 * asset_minimum_asset_balance
			));

			// check Assets before
			assert_eq!(
				Assets::balance(local_asset_id, AccountId::from(ALICE)),
				6 * asset_minimum_asset_balance
			);
			assert_eq!(Assets::balance(local_asset_id, AccountId::from(BOB)), 0);
			assert_eq!(Assets::balance(local_asset_id, AccountId::from(CHARLIE)), 0);
			assert_eq!(Assets::balance(local_asset_id, AccountId::from(SOME_ASSET_OWNER)), 0);
			assert!(ForeignAssets::asset_ids().collect::<Vec<_>>().is_empty());
			assert_eq!(Balances::free_balance(AccountId::from(ALICE)), ExistentialDeposit::get());
			assert_eq!(Balances::free_balance(AccountId::from(BOB)), ExistentialDeposit::get());
			assert_eq!(Balances::free_balance(AccountId::from(CHARLIE)), 0);
			assert_eq!(
				Balances::free_balance(AccountId::from(SOME_ASSET_OWNER)),
				ExistentialDeposit::get()
			);

			// transfer_asset (deposit/withdraw) ALICE -> CHARLIE (not ok - Charlie does not have ExistentialDeposit)
			assert_noop!(
				RuntimeHelper::<XcmConfig>::do_transfer(
					MultiLocation {
						parents: 0,
						interior: X1(AccountId32 {
							network: None,
							id: AccountId::from(ALICE).into()
						}),
					},
					MultiLocation {
						parents: 0,
						interior: X1(AccountId32 {
							network: None,
							id: AccountId::from(CHARLIE).into()
						}),
					},
					(local_asset_id_as_multilocation, 1 * asset_minimum_asset_balance),
				),
				XcmError::FailedToTransactAsset(Into::<&str>::into(
					sp_runtime::TokenError::CannotCreate
				))
			);

			// transfer_asset (deposit/withdraw) ALICE -> BOB (ok - has ExistentialDeposit)
			assert!(matches!(
				RuntimeHelper::<XcmConfig>::do_transfer(
					MultiLocation {
						parents: 0,
						interior: X1(AccountId32 {
							network: None,
							id: AccountId::from(ALICE).into()
						}),
					},
					MultiLocation {
						parents: 0,
						interior: X1(AccountId32 {
							network: None,
							id: AccountId::from(BOB).into()
						}),
					},
					(local_asset_id_as_multilocation, 1 * asset_minimum_asset_balance),
				),
				Ok(_)
			));

			// check Assets after
			assert_eq!(
				Assets::balance(local_asset_id, AccountId::from(ALICE)),
				5 * asset_minimum_asset_balance
			);
			assert_eq!(
				Assets::balance(local_asset_id, AccountId::from(BOB)),
				1 * asset_minimum_asset_balance
			);
			assert_eq!(Assets::balance(local_asset_id, AccountId::from(CHARLIE)), 0);
			assert_eq!(Assets::balance(local_asset_id, AccountId::from(SOME_ASSET_OWNER)), 0);
			assert!(ForeignAssets::asset_ids().collect::<Vec<_>>().is_empty());
			assert_eq!(Balances::free_balance(AccountId::from(ALICE)), ExistentialDeposit::get());
			assert_eq!(Balances::free_balance(AccountId::from(BOB)), ExistentialDeposit::get());
			assert_eq!(Balances::free_balance(AccountId::from(CHARLIE)), 0);
			assert_eq!(
				Balances::free_balance(AccountId::from(SOME_ASSET_OWNER)),
				ExistentialDeposit::get()
			);
		})
}

#[test]
fn test_asset_transactor_transfer_with_foreign_assets_works() {
	ExtBuilder::<Runtime>::default()
		.with_collators(vec![AccountId::from(ALICE)])
		.with_session_keys(vec![(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) },
		)])
		.with_balances(vec![
			(AccountId::from(SOME_ASSET_ADMIN), ExistentialDeposit::get()),
			(AccountId::from(ALICE), ExistentialDeposit::get()),
			(AccountId::from(BOB), ExistentialDeposit::get()),
		])
		.with_tracing()
		.build()
		.execute_with(|| {
			// create foreign asset
			// foreign relaychain currency as asset
			let foreign_asset_id_multilocation =
				MultiLocation { parents: 2, interior: X1(GlobalConsensus(Kusama)) };
			let asset_minimum_asset_balance = 3333333_u128;
			assert_ok!(ForeignAssets::force_create(
				RuntimeHelper::<Runtime>::root_origin(),
				foreign_asset_id_multilocation.clone().into(),
				AccountId::from(SOME_ASSET_ADMIN).into(),
				false,
				asset_minimum_asset_balance
			));

			// We first mint enough asset for the account to exist for assets
			assert_ok!(ForeignAssets::mint(
				RuntimeHelper::<Runtime>::origin_of(AccountId::from(SOME_ASSET_ADMIN)),
				foreign_asset_id_multilocation.clone().into(),
				AccountId::from(ALICE).into(),
				6 * asset_minimum_asset_balance
			));

			// check Assets before
			assert_eq!(
				ForeignAssets::balance(foreign_asset_id_multilocation, AccountId::from(ALICE)),
				6 * asset_minimum_asset_balance
			);
			assert_eq!(
				ForeignAssets::balance(foreign_asset_id_multilocation, AccountId::from(BOB)),
				0
			);
			assert_eq!(
				ForeignAssets::balance(foreign_asset_id_multilocation, AccountId::from(CHARLIE)),
				0
			);
			assert_eq!(
				ForeignAssets::balance(
					foreign_asset_id_multilocation,
					AccountId::from(SOME_ASSET_ADMIN)
				),
				0
			);
			assert!(Assets::asset_ids().collect::<Vec<_>>().is_empty());
			assert_eq!(Balances::free_balance(AccountId::from(ALICE)), ExistentialDeposit::get());
			assert_eq!(Balances::free_balance(AccountId::from(BOB)), ExistentialDeposit::get());
			assert_eq!(Balances::free_balance(AccountId::from(CHARLIE)), 0);
			assert_eq!(
				Balances::free_balance(AccountId::from(SOME_ASSET_ADMIN)),
				ExistentialDeposit::get()
			);

			// transfer_asset (deposit/withdraw) ALICE -> CHARLIE (not ok - Charlie does not have ExistentialDeposit)
			assert_noop!(
				RuntimeHelper::<XcmConfig>::do_transfer(
					MultiLocation {
						parents: 0,
						interior: X1(AccountId32 {
							network: None,
							id: AccountId::from(ALICE).into()
						}),
					},
					MultiLocation {
						parents: 0,
						interior: X1(AccountId32 {
							network: None,
							id: AccountId::from(CHARLIE).into()
						}),
					},
					(foreign_asset_id_multilocation, 1 * asset_minimum_asset_balance),
				),
				XcmError::FailedToTransactAsset(Into::<&str>::into(
					sp_runtime::TokenError::CannotCreate
				))
			);

			// transfer_asset (deposit/withdraw) ALICE -> BOB (ok - has ExistentialDeposit)
			assert!(matches!(
				RuntimeHelper::<XcmConfig>::do_transfer(
					MultiLocation {
						parents: 0,
						interior: X1(AccountId32 {
							network: None,
							id: AccountId::from(ALICE).into()
						}),
					},
					MultiLocation {
						parents: 0,
						interior: X1(AccountId32 {
							network: None,
							id: AccountId::from(BOB).into()
						}),
					},
					(foreign_asset_id_multilocation, 1 * asset_minimum_asset_balance),
				),
				Ok(_)
			));

			// check Assets after
			assert_eq!(
				ForeignAssets::balance(foreign_asset_id_multilocation, AccountId::from(ALICE)),
				5 * asset_minimum_asset_balance
			);
			assert_eq!(
				ForeignAssets::balance(foreign_asset_id_multilocation, AccountId::from(BOB)),
				1 * asset_minimum_asset_balance
			);
			assert_eq!(
				ForeignAssets::balance(foreign_asset_id_multilocation, AccountId::from(CHARLIE)),
				0
			);
			assert_eq!(
				ForeignAssets::balance(
					foreign_asset_id_multilocation,
					AccountId::from(SOME_ASSET_ADMIN)
				),
				0
			);
			assert!(Assets::asset_ids().collect::<Vec<_>>().is_empty());
			assert_eq!(Balances::free_balance(AccountId::from(ALICE)), ExistentialDeposit::get());
			assert_eq!(Balances::free_balance(AccountId::from(BOB)), ExistentialDeposit::get());
			assert_eq!(Balances::free_balance(AccountId::from(CHARLIE)), 0);
			assert_eq!(
				Balances::free_balance(AccountId::from(SOME_ASSET_ADMIN)),
				ExistentialDeposit::get()
			);
		})
}

#[test]
fn create_and_manage_foreign_assets_for_local_consensus_parachain_assets_works() {
	// foreign parachain with the same consenus currency as asset
	let foreign_asset_id_multilocation =
		MultiLocation { parents: 1, interior: X2(Parachain(2222), GeneralIndex(1234567)) };

	// foreign creator, which can be sibling parachain to match ForeignCreators
	let foreign_creator = MultiLocation { parents: 1, interior: X1(Parachain(2222)) };
	let foreign_creator_as_account_id =
		ForeignCreatorsSovereignAccountOf::convert(foreign_creator).expect("");

	// we want to buy execution with local relay chain currency
	let buy_execution_fee_amount =
		WeightToFee::weight_to_fee(&Weight::from_parts(90_000_000_000, 0));
	let buy_execution_fee = MultiAsset {
		id: Concrete(MultiLocation::parent()),
		fun: Fungible(buy_execution_fee_amount),
	};

	let bob = AccountId::from(BOB);

	ExtBuilder::<Runtime>::default()
		.with_collators(vec![AccountId::from(ALICE)])
		.with_session_keys(vec![(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) },
		)])
		.with_balances(vec![(
			foreign_creator_as_account_id.clone(),
			ExistentialDeposit::get() +
				AssetDeposit::get() +
				MetadataDepositBase::get() +
				buy_execution_fee_amount,
		)])
		.with_tracing()
		.build()
		.execute_with(|| {
			assert!(Assets::asset_ids().collect::<Vec<_>>().is_empty());
			assert!(ForeignAssets::asset_ids().collect::<Vec<_>>().is_empty());
			assert_eq!(
				Balances::free_balance(&foreign_creator_as_account_id),
				ExistentialDeposit::get() +
					AssetDeposit::get() + MetadataDepositBase::get() +
					buy_execution_fee_amount
			);

			// execute XCM with Transacts to create/manage foreign assets by foreign governance
			// prepapre data for xcm::Transact(create)
			let foreign_asset_create: RuntimeCall = RuntimeCall::ForeignAssets(
				pallet_assets::Call::<Runtime, ForeignAssetsInstance>::create {
					id: foreign_asset_id_multilocation,
					// admin as sovereign_account
					admin: foreign_creator_as_account_id.clone().into(),
					min_balance: 1,
				},
			);
			// prepapre data for xcm::Transact(set_metadata)
			let foreign_asset_set_metadata: RuntimeCall = RuntimeCall::ForeignAssets(
				pallet_assets::Call::<Runtime, ForeignAssetsInstance>::set_metadata {
					id: foreign_asset_id_multilocation,
					name: Vec::from("My super coin"),
					symbol: Vec::from("MY_S_COIN"),
					decimals: 12,
				},
			);
			// prepapre data for xcm::Transact(set_team - change just freezer to Bob)
			let foreign_asset_set_team: RuntimeCall = RuntimeCall::ForeignAssets(
				pallet_assets::Call::<Runtime, ForeignAssetsInstance>::set_team {
					id: foreign_asset_id_multilocation,
					issuer: foreign_creator_as_account_id.clone().into(),
					admin: foreign_creator_as_account_id.clone().into(),
					freezer: bob.clone().into(),
				},
			);

			// lets simulate this was triggered by relay chain from local consensus sibling parachain
			let xcm = Xcm(vec![
				WithdrawAsset(buy_execution_fee.clone().into()),
				BuyExecution { fees: buy_execution_fee.clone().into(), weight_limit: Unlimited },
				Transact {
					origin_kind: OriginKind::Xcm,
					require_weight_at_most: Weight::from_parts(40_000_000_000, 6000),
					call: foreign_asset_create.encode().into(),
				},
				Transact {
					origin_kind: OriginKind::SovereignAccount,
					require_weight_at_most: Weight::from_parts(20_000_000_000, 6000),
					call: foreign_asset_set_metadata.encode().into(),
				},
				Transact {
					origin_kind: OriginKind::SovereignAccount,
					require_weight_at_most: Weight::from_parts(20_000_000_000, 6000),
					call: foreign_asset_set_team.encode().into(),
				},
			]);

			// messages with different consensus should go through the local bridge-hub
			let hash = xcm.using_encoded(sp_io::hashing::blake2_256);

			// execute xcm as XcmpQueue would do
			let outcome = XcmExecutor::<XcmConfig>::execute_xcm(
				foreign_creator,
				xcm,
				hash,
				RuntimeHelper::<Runtime>::xcm_max_weight(XcmReceivedFrom::Sibling),
			);
			assert_eq!(outcome.ensure_complete(), Ok(()));

			// check events
			let mut events = System::events().into_iter().map(|e| e.event);
			assert!(events.any(|e| matches!(
				e,
				RuntimeEvent::ForeignAssets(pallet_assets::Event::Created { .. })
			)));
			assert!(events.any(|e| matches!(
				e,
				RuntimeEvent::ForeignAssets(pallet_assets::Event::MetadataSet { .. })
			)));
			assert!(events.any(|e| matches!(
				e,
				RuntimeEvent::ForeignAssets(pallet_assets::Event::TeamChanged { .. })
			)));

			// check assets after
			assert!(Assets::asset_ids().collect::<Vec<_>>().is_empty());
			assert!(!ForeignAssets::asset_ids().collect::<Vec<_>>().is_empty());

			// check update metadata
			use frame_support::traits::tokens::fungibles::roles::Inspect as InspectRoles;
			assert_eq!(
				ForeignAssets::owner(foreign_asset_id_multilocation),
				Some(foreign_creator_as_account_id.clone())
			);
			assert_eq!(
				ForeignAssets::admin(foreign_asset_id_multilocation),
				Some(foreign_creator_as_account_id.clone())
			);
			assert_eq!(
				ForeignAssets::issuer(foreign_asset_id_multilocation),
				Some(foreign_creator_as_account_id.clone())
			);
			assert_eq!(ForeignAssets::freezer(foreign_asset_id_multilocation), Some(bob.clone()));
			assert!(
				Balances::free_balance(&foreign_creator_as_account_id) <
					ExistentialDeposit::get() + buy_execution_fee_amount
			);
			RuntimeHelper::<ForeignAssets>::assert_metadata(
				&foreign_asset_id_multilocation,
				"My super coin",
				"MY_S_COIN",
				12,
			);

			// check if changed freezer, can freeze
			assert_noop!(
				ForeignAssets::freeze(
					RuntimeHelper::<Runtime>::origin_of(bob),
					foreign_asset_id_multilocation.clone(),
					AccountId::from(ALICE).into()
				),
				pallet_assets::Error::<Runtime, ForeignAssetsInstance>::NoAccount
			);
			assert_noop!(
				ForeignAssets::freeze(
					RuntimeHelper::<Runtime>::origin_of(foreign_creator_as_account_id),
					foreign_asset_id_multilocation.clone(),
					AccountId::from(ALICE).into()
				),
				pallet_assets::Error::<Runtime, ForeignAssetsInstance>::NoPermission
			);
		})
}
