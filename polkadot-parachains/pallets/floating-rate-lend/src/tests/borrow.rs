use crate::{Error, PoolStorage, PoolUserDebts};
use crate::tests::mock::{*};

use frame_support::{assert_noop, assert_ok};
use crate::pool::PoolRepository;
use sp_runtime::FixedU128;
use polkadot_parachain_primitives::BALANCE_ONE;
use sp_runtime::traits::{One};
use crate::types::UserAccountUtil;

#[test]
fn floating_lend_borrow_zero_balance() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            assert_noop!(FloatingRateLend::borrow(Origin::signed(ROOT), 0, 0), Error::<Runtime>::BalanceTooLow);
        });
}

#[test]
fn floating_lend_borrow_not_enabled() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let pool = default_pool_proxy();
            PoolRepository::save(pool);
            assert_noop!(
                FloatingRateLend::borrow(Origin::signed(ROOT), 0, 100),
                Error::<Runtime>::PoolNotEnabled
            );
        });
}

#[test]
fn floating_lend_borrow_not_enough_liquidity() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let mut pool = default_pool_proxy();
            pool.increment_supply(&FixedU128::one());
            PoolRepository::save(pool.clone());

            FloatingRateLend::enable_pool(Origin::signed(ROOT), 0).ok();
            assert_noop!(
                FloatingRateLend::borrow(Origin::signed(ROOT), 0, 2 * BALANCE_ONE),
                Error::<Runtime>::NotEnoughLiquidity
            );
        });
}

#[test]
fn floating_lend_borrow_success() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let mut pool_0 = pool_proxy(0, true);
            let mut pool_1 = pool_proxy(1, false);
            let mut pool_2 = pool_proxy(2, false);

            pool_0.increment_supply(&FixedU128::from(3000));
            pool_0.increment_debt(&FixedU128::from(1000));

            pool_1.increment_supply(&FixedU128::from(3000));
            pool_1.increment_debt(&FixedU128::from(2000));

            pool_2.increment_supply(&FixedU128::from(4000));
            pool_2.increment_debt(&FixedU128::from(2000));

            PoolRepository::save(pool_0.clone());
            PoolRepository::save(pool_1.clone());
            PoolRepository::save(pool_2.clone());
            // this is to simulate the user supply. User supplied 1000 pool has 3000
            let amount = FixedU128::from(1000);
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_0, ROOT.clone(), &FixedU128::from(2000)).unwrap();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_1, ROOT.clone(), &amount).unwrap();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&pool_2, ROOT.clone(), &FixedU128::from(800)).unwrap();

            FloatingRateLend::enable_pool(Origin::signed(ROOT), 0).ok();
            FloatingRateLend::enable_pool(Origin::signed(ROOT), 2).ok();
            FloatingRateLend::update_liquidation_threshold(Origin::signed(ROOT), 100).ok();

            // this is just to test we can withdraw if within liquidation threshold
            assert_ok!(FloatingRateLend::borrow(Origin::signed(ROOT), 2, 1000 * BALANCE_ONE));

            // check supplies
            let user = PoolUserDebts::<Runtime>::get(pool_2.id(), ROOT.clone()).unwrap();
            assert_eq!(user.amount(), FixedU128::from(1800));

            let pool = PoolStorage::<Runtime>::get(pool_2.id()).unwrap();
            assert_eq!(pool.debt(), FixedU128::from(3000));
        });
}

#[test]
fn floating_lend_borrow_failed() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let mut pool_0 = pool_proxy(0, true);
            let mut pool_1 = pool_proxy(1, false);
            let mut pool_2 = pool_proxy(2, false);

            pool_0.increment_supply(&FixedU128::from(3000));
            pool_0.increment_debt(&FixedU128::from(1000));

            pool_1.increment_supply(&FixedU128::from(3000));
            pool_1.increment_debt(&FixedU128::from(2000));

            pool_2.increment_supply(&FixedU128::from(4000));
            pool_2.increment_debt(&FixedU128::from(2000));

            PoolRepository::save(pool_0.clone());
            PoolRepository::save(pool_1.clone());
            PoolRepository::save(pool_2.clone());
            // this is to simulate the user supply. User supplied 1000 pool has 3000
            let amount = FixedU128::from(1000);
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_0, ROOT.clone(), &FixedU128::from(2000)).unwrap();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_1, ROOT.clone(), &amount).unwrap();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&pool_2, ROOT.clone(), &FixedU128::from(800)).unwrap();

            FloatingRateLend::enable_pool(Origin::signed(ROOT), 0).ok();
            FloatingRateLend::enable_pool(Origin::signed(ROOT), 2).ok();
            FloatingRateLend::update_liquidation_threshold(Origin::signed(ROOT), 100).ok();

            // this is just to test we can withdraw if within liquidation threshold
            assert_noop!(
                FloatingRateLend::borrow(Origin::signed(ROOT), 2, 1001 * BALANCE_ONE),
                Error::<Runtime>::BelowLiquidationThreshold
            );

            // check supplies
            let user = PoolUserDebts::<Runtime>::get(pool_2.id(), ROOT.clone()).unwrap();
            assert_eq!(user.amount(), FixedU128::from(800));

            let pool = PoolStorage::<Runtime>::get(pool_2.id()).unwrap();
            assert_eq!(pool.debt(), FixedU128::from(2000));
        });
}