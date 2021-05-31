use crate::{Error, PoolStorage, PoolUserSupplies, PoolUserDebts};
use crate::tests::mock::{*};

use frame_support::{assert_noop};
use crate::pool::PoolRepository;
use sp_runtime::{FixedU128, FixedPointNumber};
use polkadot_parachain_primitives::BALANCE_ONE;
use crate::types::UserAccountUtil;

#[test]
fn floating_lend_liquidate_zero_balance() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            assert_noop!(
                FloatingRateLend::liquidate(Origin::signed(ROOT), ACCOUNT_1, 0, 0, 1),
                Error::<Runtime>::BalanceTooLow
            );
        });
}

#[test]
fn floating_lend_liquidate_collateral_pool_not_enabled() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let pool = default_pool_proxy();
            PoolRepository::save(pool);

            let pool = pool_proxy(1, false);
            PoolRepository::save(pool.clone());
            assert_noop!(
                FloatingRateLend::liquidate(Origin::signed(ROOT), ACCOUNT_1, 0, 100, 1),
                Error::<Runtime>::PoolNotEnabled
            );
        });
}

#[test]
fn floating_lend_liquidate_not_collateral() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let pool = default_pool_proxy();
            PoolRepository::save(pool.clone());

            let pool = pool_proxy(1, false);
            PoolRepository::save(pool.clone());

            FloatingRateLend::enable_pool(Origin::signed(ROOT), 0).ok();
            FloatingRateLend::enable_pool(Origin::signed(ROOT), 1).ok();

            let debt_pool_id = 0;
            let collateral_pool_id = 1;
            assert_noop!(
                FloatingRateLend::liquidate(Origin::signed(ROOT), ACCOUNT_1, debt_pool_id, 100, collateral_pool_id),
                Error::<Runtime>::AssetNotCollateral
            );
        });
}

#[test]
fn floating_lend_liquidate_debt_pool_not_enabled() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let pool = default_pool_proxy();
            PoolRepository::save(pool.clone());

            let pool = pool_proxy(1, true);
            PoolRepository::save(pool.clone());

            FloatingRateLend::enable_pool(Origin::signed(ROOT), 1).ok();

            let debt_pool_id = 0;
            let collateral_pool_id = 1;
            assert_noop!(
                FloatingRateLend::liquidate(Origin::signed(ROOT), ACCOUNT_1, debt_pool_id, 100, collateral_pool_id),
                Error::<Runtime>::PoolNotEnabled
            );
        });
}

#[test]
fn floating_lend_liquidate_no_debt() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let pool = default_pool_proxy();
            PoolRepository::save(pool.clone());

            let pool = pool_proxy(1, true);
            PoolRepository::save(pool.clone());

            FloatingRateLend::enable_pool(Origin::signed(ROOT), 1).ok();
            FloatingRateLend::enable_pool(Origin::signed(ROOT), 0).ok();

            let debt_pool_id = 0;
            let collateral_pool_id = 1;
            assert_noop!(
                FloatingRateLend::liquidate(Origin::signed(ROOT), ACCOUNT_1, debt_pool_id, 100, collateral_pool_id),
                Error::<Runtime>::UserNoDebtInPool
            );
        });
}

#[test]
fn floating_lend_liquidate_no_supply() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let debt_pool_0 = pool_proxy(0, false);
            PoolRepository::save(debt_pool_0.clone());

            let collateral_pool_0 = pool_proxy(1, true);
            PoolRepository::save(collateral_pool_0.clone());

            FloatingRateLend::enable_pool(Origin::signed(ROOT), 1).ok();
            FloatingRateLend::enable_pool(Origin::signed(ROOT), 0).ok();

            let amount = FixedU128::from(1000);
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&debt_pool_0, ACCOUNT_1.clone(), &amount).unwrap();

            let debt_pool_id = 0;
            let collateral_pool_id = 1;
            assert_noop!(
                FloatingRateLend::liquidate(Origin::signed(ROOT), ACCOUNT_1, debt_pool_id, 100, collateral_pool_id),
                Error::<Runtime>::UserNoSupplyInPool
            );
        });
}

#[test]
fn floating_lend_liquidate_user_not_liquidated() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let mut debt_pool = pool_proxy(0, false);
            debt_pool.increment_debt(&FixedU128::from(1000));
            debt_pool.increment_supply(&FixedU128::from(2000));
            PoolRepository::save(debt_pool.clone());

            let mut collateral_pool = pool_proxy(1, true);
            collateral_pool.increment_debt(&FixedU128::from(1000));
            collateral_pool.increment_supply(&FixedU128::from(2000));
            PoolRepository::save(collateral_pool.clone());

            let mut supply_pool = pool_proxy(2, false);
            supply_pool.increment_debt(&FixedU128::from(1000));
            supply_pool.increment_supply(&FixedU128::from(2000));
            PoolRepository::save(supply_pool.clone());

            FloatingRateLend::enable_pool(Origin::signed(ROOT), 0).ok();
            FloatingRateLend::enable_pool(Origin::signed(ROOT), 1).ok();
            FloatingRateLend::enable_pool(Origin::signed(ROOT), 2).ok();

            UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&debt_pool, ACCOUNT_1.clone(), &FixedU128::from(1800)).unwrap();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&supply_pool, ACCOUNT_1.clone(), &FixedU128::from(1800)).unwrap();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&collateral_pool, ACCOUNT_1.clone(), &FixedU128::from(2000)).unwrap();

            assert_noop!(
                FloatingRateLend::liquidate(Origin::signed(ROOT), ACCOUNT_1, debt_pool.id(), 100, collateral_pool.id()),
                Error::<Runtime>::UserNotUnderLiquidation
            );
        });
}

#[test]
fn floating_lend_liquidate_user_liquidated_not_fully_closable() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let mut debt_pool = pool_proxy_with_price(10, false, FixedU128::from(2));
            debt_pool.increment_debt(&FixedU128::from(1000));
            debt_pool.increment_supply(&FixedU128::from(2000));
            PoolRepository::save(debt_pool.clone());

            let mut collateral_pool = pool_proxy_with_price(11, true, FixedU128::saturating_from_rational(5, 10));
            collateral_pool.increment_debt(&FixedU128::from(1000));
            collateral_pool.increment_supply(&FixedU128::from(3000));
            PoolRepository::save(collateral_pool.clone());

            let mut supply_pool = pool_proxy(12, false);
            supply_pool.increment_debt(&FixedU128::from(1000));
            supply_pool.increment_supply(&FixedU128::from(2000));
            PoolRepository::save(supply_pool.clone());

            FloatingRateLend::enable_pool(Origin::signed(ROOT), 10).ok();
            FloatingRateLend::enable_pool(Origin::signed(ROOT), 11).ok();
            FloatingRateLend::enable_pool(Origin::signed(ROOT), 12).ok();
            FloatingRateLend::update_liquidation_threshold(Origin::signed(ROOT), 100).ok();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&debt_pool, ACCOUNT_1.clone(), &FixedU128::from(1000)).unwrap();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&supply_pool, ACCOUNT_1.clone(), &FixedU128::from(1800)).unwrap();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&collateral_pool, ACCOUNT_1.clone(), &FixedU128::from(2000)).unwrap();

            // Debt pool price: $2, collateral pool price: $0.5, discount factor: 0.9
            // Account_1 debt worth: 1000 * $2 = 2000
            // Account_1 collateral worth: 2000 * $0.5 * 0.9 = 900 > 100, close factor kick in
            // Closable balance: 2000 * 0.5 = 1000 Balance
            // Closable collateral worth: 1000 * ($0.5 * 0.9) = 450
            // Debt pool to deduct: 450 / $2 = 225, left 775
            // User debt to deduct 225, left 775
            // Collateral pool to deduct: 1000, left 2000
            // User collateral to deduct: 1000, left 1000
            // Transfer 225 of debt pool currency from the arbitrager to the pool
            // Transfer 1000 of collateral pool currency from the pool to the arbitrager
            FloatingRateLend::liquidate(Origin::signed(ROOT), ACCOUNT_1, debt_pool.id(), 1000 * BALANCE_ONE, collateral_pool.id()).ok();

            let pool = PoolStorage::<Runtime>::get(debt_pool.id()).unwrap();
            assert_eq!(pool.debt(), FixedU128::from(775));
            let pool = PoolStorage::<Runtime>::get(collateral_pool.id()).unwrap();
            assert_eq!(pool.supply(), FixedU128::from_inner(2000000000000000000100));

            let account_1_collateral = PoolUserSupplies::<Runtime>::get(collateral_pool.id(), ACCOUNT_1.clone()).unwrap();
            assert_eq!(account_1_collateral.amount(), FixedU128::from_inner(1000000000000000000100));
            let account_1_debt = PoolUserDebts::<Runtime>::get(debt_pool.id(), ACCOUNT_1.clone()).unwrap();
            assert_eq!(account_1_debt.amount(), FixedU128::from(775));
        });
}


#[test]
fn floating_lend_liquidate_user_liquidated_fully_closable() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let mut debt_pool = pool_proxy_with_price(10, false, FixedU128::from(2));
            debt_pool.increment_debt(&FixedU128::from(1000));
            debt_pool.increment_supply(&FixedU128::from(2000));
            PoolRepository::save(debt_pool.clone());

            let mut debt_pool_2 = pool_proxy_with_price(9, false, FixedU128::from(2));
            debt_pool_2.increment_debt(&FixedU128::from(1000));
            debt_pool_2.increment_supply(&FixedU128::from(2000));
            PoolRepository::save(debt_pool_2.clone());

            let mut collateral_pool = pool_proxy_with_price(11, true, FixedU128::saturating_from_rational(5, 10));
            collateral_pool.increment_debt(&FixedU128::from(1000));
            collateral_pool.increment_supply(&FixedU128::from(3000));
            PoolRepository::save(collateral_pool.clone());

            let mut supply_pool = pool_proxy(12, false);
            supply_pool.increment_debt(&FixedU128::from(1000));
            supply_pool.increment_supply(&FixedU128::from(2000));
            PoolRepository::save(supply_pool.clone());

            FloatingRateLend::enable_pool(Origin::signed(ROOT), 10).ok();
            FloatingRateLend::enable_pool(Origin::signed(ROOT), 11).ok();
            FloatingRateLend::enable_pool(Origin::signed(ROOT), 12).ok();
            FloatingRateLend::update_liquidation_threshold(Origin::signed(ROOT), 100).ok();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&debt_pool, ACCOUNT_1.clone(), &FixedU128::from(1000)).unwrap();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&debt_pool_2, ACCOUNT_1.clone(), &FixedU128::from(3000)).unwrap();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&supply_pool, ACCOUNT_1.clone(), &FixedU128::from(1800)).unwrap();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&collateral_pool, ACCOUNT_1.clone(), &FixedU128::from(180)).unwrap();

            // Debt pool price: $2, collateral pool price: $0.5, discount factor: 0.9
            // Account_1 debt worth: 1000 * $2 = 2000
            // Account_1 collateral worth: 180 * $0.5 = 90 < 100
            // Closable balance: 180 Balance
            // Closable collateral worth: 180 * ($0.5 * 0.9) = 81
            // Debt pool to deduct: 81 / $2 = 40.5, left 959.5
            // User debt to deduct 225, left 959.5
            // Collateral pool to deduct: 180, left 2820
            // User collateral to deduct: 180, left 0
            // Transfer 40.5 of debt pool currency from the arbitrager to the pool
            // Transfer 180 of collateral pool currency from the pool to the arbitrager
            FloatingRateLend::liquidate(Origin::signed(ROOT), ACCOUNT_1, debt_pool.id(), 1000 * BALANCE_ONE, collateral_pool.id()).ok();

            let pool = PoolStorage::<Runtime>::get(debt_pool.id()).unwrap();
            assert_eq!(pool.debt(), FixedU128::from_inner(959500000000000000000));
            let pool = PoolStorage::<Runtime>::get(collateral_pool.id()).unwrap();
            assert_eq!(pool.supply(), FixedU128::from_inner(2820000000000000000018));

            let account_1_collateral = PoolUserSupplies::<Runtime>::get(collateral_pool.id(), ACCOUNT_1.clone());
            assert_eq!(account_1_collateral.is_none(), true);
            let account_1_debt = PoolUserDebts::<Runtime>::get(debt_pool.id(), ACCOUNT_1.clone()).unwrap();
            assert_eq!(account_1_debt.amount(), FixedU128::from_inner(959500000000000000000));
        });
}

#[test]
fn floating_lend_liquidate_user_liquidated_fully_closable_not_enough_debt() {
        ExtBuilder::default()
            .build()
            .execute_with(|| {
                    let mut debt_pool = pool_proxy_with_price(10, false, FixedU128::from(2));
                    debt_pool.increment_debt(&FixedU128::from(1000));
                    debt_pool.increment_supply(&FixedU128::from(2000));
                    PoolRepository::save(debt_pool.clone());

                    let mut debt_pool_2 = pool_proxy_with_price(9, false, FixedU128::from(2));
                    debt_pool_2.increment_debt(&FixedU128::from(1000));
                    debt_pool_2.increment_supply(&FixedU128::from(2000));
                    PoolRepository::save(debt_pool_2.clone());

                    let mut collateral_pool = pool_proxy_with_price(11, true, FixedU128::saturating_from_rational(5, 10));
                    collateral_pool.increment_debt(&FixedU128::from(1000));
                    collateral_pool.increment_supply(&FixedU128::from(3000));
                    PoolRepository::save(collateral_pool.clone());

                    let mut supply_pool = pool_proxy(12, false);
                    supply_pool.increment_debt(&FixedU128::from(1000));
                    supply_pool.increment_supply(&FixedU128::from(2000));
                    PoolRepository::save(supply_pool.clone());

                    FloatingRateLend::enable_pool(Origin::signed(ROOT), 10).ok();
                    FloatingRateLend::enable_pool(Origin::signed(ROOT), 11).ok();
                    FloatingRateLend::enable_pool(Origin::signed(ROOT), 12).ok();
                    FloatingRateLend::update_liquidation_threshold(Origin::signed(ROOT), 100).ok();
                    UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&debt_pool, ACCOUNT_1.clone(), &FixedU128::from(30)).unwrap();
                    UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&debt_pool_2, ACCOUNT_1.clone(), &FixedU128::from(3000)).unwrap();
                    UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&supply_pool, ACCOUNT_1.clone(), &FixedU128::from(1800)).unwrap();
                    UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&collateral_pool, ACCOUNT_1.clone(), &FixedU128::from(180)).unwrap();

                    // Debt pool price: $2, collateral pool price: $0.5, discount factor: 0.9
                    // Account_1 debt worth: 30 * $2 = 60
                    // Account_1 collateral worth: 180 * $0.5 = 90 < 100
                    // Closable balance: 180 Balance
                    // Closable collateral worth: 180 * ($0.5 * 0.9) = 81
                    // Debt pool to deduct: 81 / $2 = 40.5 > 30 = 30, left 970
                    // User debt to deduct 30, left 970
                    // Collateral pool to deduct: 30 * 2 / 0.45 = 133.33, left 3000 - 133.33 = 2866.67
                    // User collateral to deduct: 180 - 133.33, left 46.67
                    // Transfer 30 of debt pool currency from the arbitrager to the pool
                    // Transfer 66.67 of collateral pool currency from the pool to the arbitrager
                    FloatingRateLend::liquidate(Origin::signed(ROOT), ACCOUNT_1, debt_pool.id(), 1000 * BALANCE_ONE, collateral_pool.id()).ok();

                    let pool = PoolStorage::<Runtime>::get(debt_pool.id()).unwrap();
                    assert_eq!(pool.debt(), FixedU128::from_inner(970000000000000000000));
                    let pool = PoolStorage::<Runtime>::get(collateral_pool.id()).unwrap();
                    assert_eq!(pool.supply(), FixedU128::from_inner(2866666666666666666680));

                    let account_1_collateral = PoolUserSupplies::<Runtime>::get(collateral_pool.id(), ACCOUNT_1.clone()).unwrap();
                    assert_eq!(account_1_collateral.amount(), FixedU128::from_inner(46666666666666666680));
                    let account_1_debt = PoolUserDebts::<Runtime>::get(debt_pool.id(), ACCOUNT_1.clone());
                    assert_eq!(account_1_debt.is_none(), true);
            });
}