use asset_test_utils::{mock_open_hrmp_channel, ExtBuilder, RuntimeHelper};
use codec::{Decode, Encode};
use cumulus_primitives_utility::ChargeWeightInFungibles;
use frame_support::{
	assert_noop, assert_ok,
	traits::fungibles::InspectEnumerable,
	weights::{Weight, WeightToFee as WeightToFeeT},
};
use pallet_bridge_assets_transfer::BridgeConfig;
use parachains_common::{AccountId, AssetIdForTrustBackedAssets, AuraId, Balance};
use statemine_runtime::xcm_config::{
	AssetFeeAsExistentialDepositMultiplierFeeCharger, KsmLocation, TrustBackedAssetsPalletLocation,
};
pub use statemine_runtime::{
	constants::fee::WeightToFee,
	xcm_config::{ForeignCreatorsSovereignAccountOf, XcmConfig},
	AssetDeposit, Assets, Balances, ExistentialDeposit, ForeignAssets, ForeignAssetsInstance,
	MetadataDepositBase, ParachainSystem, PolkadotXcm, Runtime, RuntimeCall, RuntimeEvent,
	RuntimeOrigin, SessionKeys, System, TrustBackedAssetsInstance,
};
use xcm::{latest::prelude::*, VersionedMultiLocation, VersionedXcm};
use xcm_executor::{
	traits::{Convert, Identity, JustTry, WeightTrader},
	XcmExecutor,
};

const ALICE: [u8; 32] = [1u8; 32];
const SOME_ASSET_ADMIN: [u8; 32] = [5u8; 32];

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

			// We are going to buy small amount
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

			// We are gonna buy ED
			let bought = Weight::from_parts(ExistentialDeposit::get().try_into().unwrap(), 0);

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
fn test_send_xcm_transact_with_remark_with_event_works() {
	let runtime_para_id = 1015;
	let bridge_hub_para_id = 1013;
	ExtBuilder::<Runtime>::default()
		.with_collators(vec![AccountId::from(ALICE)])
		.with_session_keys(vec![(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) },
		)])
		.with_tracing()
		.with_safe_xcm_version(3)
		.with_para_id(runtime_para_id.into())
		.build()
		.execute_with(|| {
			// open hrmp channel
			mock_open_hrmp_channel::<Runtime, ParachainSystem>(
				runtime_para_id.into(),
				bridge_hub_para_id.into(),
			);

			// prepare xcm message with Transact
			let message = Xcm(vec![ExportMessage {
				network: Wococo,
				destination: X1(Parachain(1000)),
				xcm: Xcm(vec![Transact {
					origin_kind: OriginKind::SovereignAccount,
					require_weight_at_most: Weight::from_parts(1000000000, 0),
					call: vec![0, 8, 20, 104, 101, 108, 108, 111].into(),
				}]),
			}]);

			// simulate send export_message to bridge-hub
			assert_ok!(PolkadotXcm::send(
				RuntimeOrigin::signed(AccountId::from(ALICE)),
				Box::new(VersionedMultiLocation::V3(MultiLocation {
					parents: 1,
					interior: X1(Parachain(bridge_hub_para_id))
				})),
				Box::new(VersionedXcm::from(message.clone()))
			));

			// check xcm sent-like events occured
			let events = System::events();
			assert!(!events.is_empty());

			assert!(System::events().iter().any(|r| matches!(
				r.event,
				RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent(..))
			)));
			assert!(System::events().iter().any(|r| matches!(
				r.event,
				RuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::XcmpMessageSent { .. })
			)));
		});
}

#[test]
fn can_governance_call_xcm_transact_with_bridge_assets_transfer_configuration() {
	ExtBuilder::<Runtime>::default()
		.with_collators(vec![AccountId::from(ALICE)])
		.with_session_keys(vec![(
			AccountId::from(ALICE),
			AccountId::from(ALICE),
			SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) },
		)])
		.with_tracing()
		.with_safe_xcm_version(3)
		.build()
		.execute_with(|| {
			// bridge cfg data
			let bridged_network = NetworkId::Polkadot;
			let bridge_config = BridgeConfig {
				bridge_location: (Parent, Parachain(1013)).into(),
				allowed_target_location: MultiLocation::new(
					2,
					X2(GlobalConsensus(bridged_network), Parachain(1000)),
				),
				fee: None,
			};

			// check cfg before
			let cfg = pallet_bridge_assets_transfer::Pallet::<Runtime>::bridges(&bridged_network);
			assert!(cfg.is_none());

			// prepare xcm as governance will do
			let add_bridge_config: RuntimeCall = RuntimeCall::BridgeAssetsTransfer(
				pallet_bridge_assets_transfer::Call::<Runtime>::add_bridge_config {
					bridged_network,
					bridge_config: Box::new(bridge_config.clone()),
				},
			);

			// add bridge config call
			let xcm = Xcm(vec![
				UnpaidExecution { weight_limit: Unlimited, check_origin: None },
				Transact {
					origin_kind: OriginKind::Superuser,
					require_weight_at_most: Weight::from_parts(2000000000, 2000000000),
					call: add_bridge_config.encode().into(),
				},
			]);

			// origin as relay chain
			let origin = MultiLocation { parents: 1, interior: Here };

			// initialize bridge through governance-like
			let hash = xcm.using_encoded(sp_io::hashing::blake2_256);
			let weight_limit = Weight::from_parts(5000000000, 5000000000);
			let outcome = XcmExecutor::<XcmConfig>::execute_xcm(origin, xcm, hash, weight_limit);
			assert_eq!(outcome.ensure_complete(), Ok(()));

			// check cfg after
			let cfg = pallet_bridge_assets_transfer::Pallet::<Runtime>::bridges(&bridged_network);
			assert_eq!(cfg, Some(bridge_config));
		})
}

#[test]
fn test_assets_balances_api_works() {
	use assets_common::runtime_api::runtime_decl_for_fungibles_api::FungiblesApi;

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
				MultiLocation { parents: 1, interior: X2(Parachain(1234), GeneralIndex(12345)) };

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
				&assets_common::fungible_conversion::convert_balance::<KsmLocation, Balance>(
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

asset_test_utils::include_asset_transactor_transfer_with_pallet_assets_instance_works!(
	asset_transactor_transfer_with_trust_backed_assets_works,
	Runtime,
	XcmConfig,
	TrustBackedAssetsInstance,
	AssetIdForTrustBackedAssets,
	AssetIdForTrustBackedAssetsConvert,
	asset_test_utils::CollatorSessionKeys::new(
		AccountId::from(ALICE),
		AccountId::from(ALICE),
		SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) }
	),
	ExistentialDeposit::get(),
	12345,
	Box::new(|| {
		assert!(ForeignAssets::asset_ids().collect::<Vec<_>>().is_empty());
	}),
	Box::new(|| {
		assert!(ForeignAssets::asset_ids().collect::<Vec<_>>().is_empty());
	})
);

asset_test_utils::include_asset_transactor_transfer_with_pallet_assets_instance_works!(
	asset_transactor_transfer_with_foreign_assets_works,
	Runtime,
	XcmConfig,
	ForeignAssetsInstance,
	MultiLocation,
	JustTry,
	asset_test_utils::CollatorSessionKeys::new(
		AccountId::from(ALICE),
		AccountId::from(ALICE),
		SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) }
	),
	ExistentialDeposit::get(),
	MultiLocation { parents: 1, interior: X2(Parachain(1313), GeneralIndex(12345)) },
	Box::new(|| {
		assert!(Assets::asset_ids().collect::<Vec<_>>().is_empty());
	}),
	Box::new(|| {
		assert!(Assets::asset_ids().collect::<Vec<_>>().is_empty());
	})
);

asset_test_utils::include_create_and_manage_foreign_assets_for_local_consensus_parachain_assets_works!(
	Runtime,
	XcmConfig,
	WeightToFee,
	ForeignCreatorsSovereignAccountOf,
	ForeignAssetsInstance,
	MultiLocation,
	JustTry,
	asset_test_utils::CollatorSessionKeys::new(
		AccountId::from(ALICE),
		AccountId::from(ALICE),
		SessionKeys { aura: AuraId::from(sp_core::sr25519::Public::from_raw(ALICE)) }
	),
	ExistentialDeposit::get(),
	AssetDeposit::get(),
	MetadataDepositBase::get(),
	Box::new(|pallet_asset_call| RuntimeCall::ForeignAssets(pallet_asset_call).encode()),
	Box::new(|runtime_event_encoded: Vec<u8>| {
		match RuntimeEvent::decode(&mut &runtime_event_encoded[..]) {
			Ok(RuntimeEvent::ForeignAssets(pallet_asset_event)) => Some(pallet_asset_event),
			_ => None,
		}
	}),
	Box::new(|| {
		assert!(Assets::asset_ids().collect::<Vec<_>>().is_empty());
		assert!(ForeignAssets::asset_ids().collect::<Vec<_>>().is_empty());
	}),
	Box::new(|| {
		assert!(Assets::asset_ids().collect::<Vec<_>>().is_empty());
		assert_eq!(ForeignAssets::asset_ids().collect::<Vec<_>>().len(), 1);
	})
);
