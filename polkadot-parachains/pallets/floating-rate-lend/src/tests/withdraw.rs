use crate::{Error, PoolStorage, PoolUserSupplies};
use crate::tests::mock::{*};

use frame_support::{assert_noop, assert_ok};
use crate::pool::PoolRepository;
use sp_runtime::FixedU128;
use polkadot_parachain_primitives::BALANCE_ONE;
use sp_runtime::traits::{One};
use crate::types::UserAccountUtil;

#[test]
fn floating_lend_withdraw_zero_balance() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            assert_noop!(
                FloatingRateLend::withdraw(Origin::signed(ROOT), 0, 0),
                Error::<Runtime>::BalanceTooLow
            );
        });
}

#[test]
fn floating_lend_withdraw_not_enabled() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let pool = default_pool_proxy();
            PoolRepository::save(pool);
            assert_noop!(
                FloatingRateLend::withdraw(Origin::signed(ROOT), 0, 100),
                Error::<Runtime>::PoolNotEnabled
            );
        });
}

#[test]
fn floating_lend_withdraw_not_enough_liquidity() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let mut pool = default_pool_proxy();
            pool.increment_supply(&FixedU128::one());
            PoolRepository::save(pool.clone());

            FloatingRateLend::enable_pool(Origin::signed(ROOT), 0).ok();
            assert_noop!(
                FloatingRateLend::withdraw(Origin::signed(ROOT), 0, 2 * BALANCE_ONE),
                Error::<Runtime>::NotEnoughLiquidity
            );
        });
}


#[test]
fn floating_lend_repay_no_supply() {
        ExtBuilder::default()
            .build()
            .execute_with(|| {
                    let mut pool = default_pool_proxy();
                    pool.increment_supply(&FixedU128::one());
                    PoolRepository::save(pool.clone());

                    FloatingRateLend::enable_pool(Origin::signed(ROOT), 0).ok();
                    assert_noop!(
                FloatingRateLend::withdraw(Origin::signed(ROOT), 0, 1 * BALANCE_ONE),
                Error::<Runtime>::UserNoSupplyInPool
            );
            });
}

#[test]
fn floating_lend_withdraw_full_not_collateral() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let mut pool = default_pool_proxy();
            pool.increment_supply(&FixedU128::from(3000));
            PoolRepository::save(pool.clone());

            // this is to simulate the user supply. User supplied 1000 pool has 3000
            let amount = FixedU128::from(1000);
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool, ROOT.clone(), &amount).unwrap();

            FloatingRateLend::enable_pool(Origin::signed(ROOT), 0).ok();
            assert_ok!(
                // Only 1000 can be withdrawn
                FloatingRateLend::withdraw(Origin::signed(ROOT), 0, 2000 * BALANCE_ONE)
            );

            let pool = PoolStorage::<Runtime>::get(pool.id()).unwrap();
            assert_eq!(pool.supply(), FixedU128::from(2000));
            let user_supply = PoolUserSupplies::<Runtime>::get(pool.id, ROOT.clone());
            assert_eq!(user_supply.is_none(), true);
        });
}

#[test]
fn floating_lend_withdraw_full_not_collateral_pool_empty() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let mut pool = default_pool_proxy();
            pool.increment_supply(&FixedU128::from(3000));
            PoolRepository::save(pool.clone());

            // this is to simulate the user supply. User supplied 1000 pool has 3000
            let amount = FixedU128::from(3000);
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool, ROOT.clone(), &amount).unwrap();

            FloatingRateLend::enable_pool(Origin::signed(ROOT), 0).ok();
            assert_ok!(
                // Only 1000 can be withdrawn
                FloatingRateLend::withdraw(Origin::signed(ROOT), 0, 3000 * BALANCE_ONE)
            );

            let pool = PoolStorage::<Runtime>::get(pool.id()).unwrap();
            assert_eq!(pool.supply(), FixedU128::from(0));
            let user_supply = PoolUserSupplies::<Runtime>::get(pool.id, ROOT.clone());
            assert_eq!(user_supply.is_none(), true);
        });
}

#[test]
fn floating_lend_withdraw_partial_not_collateral() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let mut pool = default_pool_proxy();
            pool.increment_supply(&FixedU128::from(3000));
            PoolRepository::save(pool.clone());

            // this is to simulate the user supply. User supplied 1000 pool has 3000
            let amount = FixedU128::from(1000);
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool, ROOT.clone(), &amount).unwrap();

            FloatingRateLend::enable_pool(Origin::signed(ROOT), 0).ok();
            assert_ok!(
                // Only 1000 can be withdrawn
                FloatingRateLend::withdraw(Origin::signed(ROOT), 0, 500 * BALANCE_ONE)
            );

            let pool = PoolStorage::<Runtime>::get(pool.id()).unwrap();
            assert_eq!(pool.supply(), FixedU128::from(2500));
            let user_supply = PoolUserSupplies::<Runtime>::get(pool.id, ROOT.clone()).unwrap();
            assert_eq!(user_supply.amount(), FixedU128::from(500));
        });
}

#[test]
fn floating_lend_withdraw_partial_collateral_success() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let mut pool_0 = pool_proxy(0, true);
            let mut pool_1 = pool_proxy(1, true);
            let mut pool_2 = pool_proxy(2, false);
            let mut pool_3 = pool_proxy(3, false);
            let mut pool_4 = pool_proxy(4, true);

            pool_0.increment_supply(&FixedU128::from(3000));
            pool_0.increment_debt(&FixedU128::from(1000));

            pool_1.increment_supply(&FixedU128::from(3000));
            pool_1.increment_debt(&FixedU128::from(2000));

            pool_2.increment_supply(&FixedU128::from(3000));
            pool_2.increment_debt(&FixedU128::from(2000));

            pool_3.increment_supply(&FixedU128::from(3000));
            pool_3.increment_debt(&FixedU128::from(2000));

            pool_4.increment_supply(&FixedU128::from(3000));
            pool_4.increment_debt(&FixedU128::from(1000));

            PoolRepository::save(pool_0.clone());
            PoolRepository::save(pool_1.clone());
            PoolRepository::save(pool_2.clone());
            PoolRepository::save(pool_3.clone());
            PoolRepository::save(pool_4.clone());

            // this is to simulate the user supply. User supplied 1000 pool has 3000
            let amount = FixedU128::from(1000);
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_0, ROOT.clone(), &FixedU128::from(2000)).unwrap();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_2, ROOT.clone(), &amount).unwrap();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_3, ROOT.clone(), &amount).unwrap();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_4, ROOT.clone(), &amount).unwrap();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&pool_1, ROOT.clone(), &&FixedU128::from(1800)).unwrap();

            FloatingRateLend::enable_pool(Origin::signed(ROOT), 0).ok();
            FloatingRateLend::enable_pool(Origin::signed(ROOT), 4).ok();
            FloatingRateLend::update_liquidation_threshold(Origin::signed(ROOT), 100).ok();
            FloatingRateLend::withdraw(Origin::signed(ROOT), 4, 1 * BALANCE_ONE).ok();
            let user_supply = PoolUserSupplies::<Runtime>::get(pool_4.id(), ROOT.clone()).unwrap();
            assert_eq!(user_supply.amount(), FixedU128::from(999));

            // this is just to test we can withdraw if within liquidation threshold
            assert_ok!(FloatingRateLend::withdraw(Origin::signed(ROOT), 4, 1000 * BALANCE_ONE));

            // check supplies
            let user_supply = PoolUserSupplies::<Runtime>::get(pool_4.id(), ROOT.clone());
            assert_eq!(user_supply.is_none(), true);

            let pool = PoolStorage::<Runtime>::get(pool_4.id()).unwrap();
            assert_eq!(pool.supply(), FixedU128::from(2000));
        });
}

#[test]
fn floating_lend_withdraw_partial_collateral() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let mut pool_0 = pool_proxy(0, true);
            let mut pool_1 = pool_proxy(1, true);
            let mut pool_2 = pool_proxy(2, false);
            let mut pool_3 = pool_proxy(3, false);

            pool_0.increment_supply(&FixedU128::from(3000));
            pool_0.increment_debt(&FixedU128::from(2000));

            pool_1.increment_supply(&FixedU128::from(3000));
            pool_1.increment_debt(&FixedU128::from(2000));

            pool_2.increment_supply(&FixedU128::from(3000));
            pool_2.increment_debt(&FixedU128::from(2000));

            pool_3.increment_supply(&FixedU128::from(3000));
            pool_3.increment_debt(&FixedU128::from(2000));

            PoolRepository::save(pool_0.clone());
            PoolRepository::save(pool_1.clone());
            PoolRepository::save(pool_2.clone());
            PoolRepository::save(pool_3.clone());

            // this is to simulate the user supply. User supplied 1000 pool has 3000
            let amount = FixedU128::from(1000);
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_0, ROOT.clone(), &FixedU128::from(2000)).unwrap();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_2, ROOT.clone(), &amount).unwrap();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_3, ROOT.clone(), &amount).unwrap();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&pool_1, ROOT.clone(), &&FixedU128::from(1800)).unwrap();

            FloatingRateLend::enable_pool(Origin::signed(ROOT), 0).ok();
            FloatingRateLend::update_liquidation_threshold(Origin::signed(ROOT), 100).ok();
            assert_noop!(
                // Only 1000 can be withdrawn
                FloatingRateLend::withdraw(Origin::signed(ROOT), 0, 1 * BALANCE_ONE),
                Error::<Runtime>::BelowLiquidationThreshold
            );
        });
}