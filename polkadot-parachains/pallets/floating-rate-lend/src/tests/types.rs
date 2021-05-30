//! Unit tests for the floating rate lending module.

#![cfg(test)]

use sp_runtime::{FixedU128, FixedPointNumber};
use sp_runtime::traits::{One, Convert};
use crate::types::{Convertor, UserData, UserAccountUtil};
use polkadot_parachain_primitives::{BALANCE_ONE, PoolId};
use crate::tests::mock::{*};
use sp_std::ops::{Add, Mul};
use crate::{PoolUserSupplies, Error, UserSupplySet, PoolUserDebts};
use sp_std::collections::btree_map::BTreeMap;

fn default_user_data() -> UserData {
    UserData::new(FixedU128::one())
}

#[test]
fn convert_percentage_int_to_fixed_u128() {
    assert_eq!(Convertor::convert_percentage(2), FixedU128::saturating_from_rational(2, 100))
}

#[test]
fn convert_percentage_annum_int_to_fixed_u128() {
    assert_eq!(
        Convertor::convert_percentage_annum_to_per_block(2),
        FixedU128::saturating_from_rational(3805175038 as u128, u128::pow(10, 18)))
}

#[test]
fn test_conversion() {
    assert_eq!(
        Conversion::convert(1000 * BALANCE_ONE),
        FixedU128::from(1000)
    );

    assert_eq!(
        1000 * BALANCE_ONE,
        Conversion::convert(FixedU128::from(1000))
    );
}

#[test]
fn floating_lend_types_user_data() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let mut p = default_user_data();

            let increment = FixedU128::from(100);
            p.increment(&increment);
            // we have 1 in the default pool
            assert_eq!(p.amount(), increment.add(FixedU128::one()));

            let decrement = FixedU128::from(10);
            p.decrement(&decrement);
            // we have 1 in the default pool
            assert_eq!(p.amount(), FixedU128::from(91));
        });
}

#[test]
fn floating_lend_types_user_data_accrue_interest() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let mut p = default_user_data();

            let increment = FixedU128::from(1256);
            p.increment(&increment);
            let pool_index = FixedU128::saturating_from_rational(15, 10);
            let target_amount = FixedU128::saturating_from_rational(18855, 10);
            p.accrue_interest(&pool_index).unwrap();
            assert_eq!(p.amount(), target_amount);
            assert_eq!(p.index(), pool_index);

            let next_pool_index = FixedU128::saturating_from_rational(12, 10);
            p.accrue_interest(&next_pool_index).unwrap();
            assert_eq!(p.amount(), target_amount);
            assert_eq!(p.index(), pool_index);
        });
}

/// Increment user supply when the user has no supply
#[test]
fn floating_lend_increment_user_supply() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let pool = default_pool_proxy();
            let user_data_1 = PoolUserSupplies::<Runtime>::get(pool.id(), ACCOUNT_1.clone());
            let user_data_2 = PoolUserSupplies::<Runtime>::get(pool.id(), ACCOUNT_2.clone());

            // ensure user has no data first
            assert_eq!(user_data_1.is_none(), true);
            assert_eq!(user_data_2.is_none(), true);

            let amount_1 = FixedU128::from(100);
            let amount_2 = FixedU128::from(200);
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool, ACCOUNT_1.clone(), &amount_1).ok();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool, ACCOUNT_2.clone(), &amount_2).ok();

            // now, we should have supply already
            let user_data_1 = PoolUserSupplies::<Runtime>::get(pool.id(), ACCOUNT_1.clone());
            assert_eq!(user_data_1.is_some(), true);
            let user_data_1 = user_data_1.unwrap();
            assert_eq!(user_data_1.amount(), amount_1);
            assert_eq!(user_data_1.index(), FixedU128::one());

            // now, we supply again
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool, ACCOUNT_1.clone(), &amount_1).ok();

            let user_data_1 = PoolUserSupplies::<Runtime>::get(pool.id(), ACCOUNT_1.clone());
            assert_eq!(user_data_1.is_some(), true);
            let user_data_1 = user_data_1.unwrap();
            assert_eq!(user_data_1.amount(), amount_1 * FixedU128::from(2));
            assert_eq!(user_data_1.index(), FixedU128::one());

            // check user 2
            let user_data_2 = PoolUserSupplies::<Runtime>::get(pool.id(), ACCOUNT_2.clone());
            assert_eq!(user_data_2.is_some(), true);
            let user_data_2 = user_data_2.unwrap();
            assert_eq!(user_data_2.amount(), amount_2);
            assert_eq!(user_data_2.index(), FixedU128::one());

            // update block time
            // TODO
        })
}

/// Decrement user supply when the user has no supply
#[test]
fn floating_lend_decrement_user_supply() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let pool = default_pool_proxy();
            let user_data = PoolUserSupplies::<Runtime>::get(pool.id(), ROOT.clone());
            // ensure user has no data first
            assert_eq!(user_data.is_none(), true);

            let amount = FixedU128::from(100);
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool, ROOT.clone(), &amount).ok();
            let user_data = PoolUserSupplies::<Runtime>::get(pool.id(), ROOT.clone()).unwrap();

            let amount_deduct = FixedU128::from(50);
            UserAccountUtil::<Runtime>::decrement_supply(&pool, ROOT.clone(), &amount_deduct, user_data).ok();

            let user_data = PoolUserSupplies::<Runtime>::get(pool.id(), ROOT.clone());
            assert_eq!(user_data.is_some(), true);
            let user_data = user_data.unwrap();
            assert_eq!(user_data.amount(), amount - amount_deduct);
            assert_eq!(user_data.index(), FixedU128::one());
            let supplies = UserSupplySet::<Runtime>::get(ROOT.clone());
            assert_eq!(supplies.contains(&(pool.id(), pool.currency_id())), true);

            // now, we decrement supply again
            UserAccountUtil::<Runtime>::decrement_supply(&pool, ROOT.clone(), &amount_deduct,  user_data).ok();

            let user_data = PoolUserSupplies::<Runtime>::get(pool.id(), ROOT.clone());
            assert_eq!(user_data.is_none(), true);
            let supplies = UserSupplySet::<Runtime>::get(ROOT.clone());
            assert_eq!(supplies.contains(&(pool.id(), pool.currency_id())), false);
        })
}

/// Decrement user supply when the user has no supply
#[test]
fn floating_lend_decrement_user_supply_over_deduction() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let pool = default_pool_proxy();
            let user_data = PoolUserSupplies::<Runtime>::get(pool.id(), ROOT.clone());
            // ensure user has no data first
            assert_eq!(user_data.is_none(), true);

            let amount = FixedU128::from(100);
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool, ROOT.clone(), &amount).ok();
            let user_data = PoolUserSupplies::<Runtime>::get(pool.id(), ROOT.clone()).unwrap();

            // now, we attempt to over deduct
            let result = UserAccountUtil::<Runtime>::decrement_supply(
                &pool,
                ROOT.clone(),
                &(user_data.clone().amount().add(FixedU128::one())),
                user_data.clone())
                .map_err(|e| e.error);
            let expected = Err(Error::<Runtime>::UserSupplyTooLow.into());
            assert_eq!(expected, result);
            assert_eq!(user_data.amount(), amount);
        })
}

#[test]
fn floating_lend_increment_debt_no_debt() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let pool = default_pool_proxy();

            let user_data = PoolUserDebts::<Runtime>::get(pool.id(), ROOT.clone());
            // ensure user has no data first
            assert_eq!(user_data.is_none(), true);

            let amount = FixedU128::from(100);
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&pool, ROOT.clone(), &amount).ok();
            let user_data = PoolUserDebts::<Runtime>::get(pool.id(), ROOT.clone());
            assert_eq!(user_data.is_some(), true);
            let user_data = user_data.unwrap();
            assert_eq!(user_data.amount(), amount);
        })
}

#[test]
fn floating_lend_increment_debt_got_debt() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let pool = default_pool_proxy();

            let user_data = PoolUserDebts::<Runtime>::get(pool.id(), ROOT.clone());
            // ensure user has no data first
            assert_eq!(user_data.is_none(), true);

            let amount = FixedU128::from(100);
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&pool, ROOT.clone(), &amount).ok();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&pool, ROOT.clone(), &amount).ok();
            let user_data = PoolUserDebts::<Runtime>::get(pool.id(), ROOT.clone());
            assert_eq!(user_data.is_some(), true);
            let user_data = user_data.unwrap();
            assert_eq!(user_data.amount(), FixedU128::from(200));

            // TODO: update block time
        })
}

#[test]
fn floating_lend_get_supply_pools() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let amount = FixedU128::from(100);
            let pool_0 = pool_proxy(0, true);
            let pool_1 = pool_proxy(1, true);
            let pool_2 = pool_proxy(2, true);
            let pool_3 = pool_proxy(3, true);

            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_0, ROOT.clone(), &amount).ok();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_1, ROOT.clone(), &amount).ok();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_2, ROOT.clone(), &amount).ok();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_3, ROOT.clone(), &amount).ok();

            let user_data = UserAccountUtil::<Runtime>::get_supply_pools(ROOT.clone());
            assert_eq!(user_data.contains(&0), true);
            assert_eq!(user_data.contains(&1), true);
            assert_eq!(user_data.contains(&2), true);
            assert_eq!(user_data.contains(&3), true);
            assert_eq!(user_data.len(), 4);
        })
}

#[test]
fn floating_lend_get_debt_pools() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let amount = FixedU128::from(100);
            let pool_0 = pool_proxy(0, true);
            let pool_1 = pool_proxy(1, true);
            let pool_2 = pool_proxy(2, true);
            let pool_3 = pool_proxy(3, true);

            UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&pool_0, ROOT.clone(), &amount).ok();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&pool_1, ROOT.clone(), &amount).ok();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&pool_2, ROOT.clone(), &amount).ok();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&pool_3, ROOT.clone(), &amount).ok();

            let user_data = UserAccountUtil::<Runtime>::get_debt_pools(ROOT.clone());
            assert_eq!(user_data.contains(&0), true);
            assert_eq!(user_data.contains(&1), true);
            assert_eq!(user_data.contains(&2), true);
            assert_eq!(user_data.contains(&3), true);
            assert_eq!(user_data.len(), 4);
        })
}

#[test]
fn floating_lend_accrue_interest_for_user() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            System::set_block_number(11);
            let mut pool_0 = pool_proxy(0, true);
            let mut pool_1 = pool_proxy(1, true);
            let mut pool_2 = pool_proxy(2, true);
            let mut pool_3 = pool_proxy(3, true);

            let supply = FixedU128::from(100);
            let borrow = FixedU128::from(50);

            pool_0.increment_supply(&supply);
            pool_0.increment_debt(&borrow);
            pool_0.accrue_interest().ok();

            pool_1.increment_supply(&supply);
            pool_1.increment_debt(&borrow);
            pool_1.accrue_interest().ok();

            pool_2.increment_supply(&supply);
            pool_2.increment_debt(&borrow);
            pool_2.accrue_interest().ok();

            pool_3.increment_supply(&supply);
            pool_3.increment_debt(&borrow);
            pool_3.accrue_interest().ok();

            let amount = FixedU128::from(100);
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&pool_0, ROOT.clone(), &amount).ok();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&pool_1, ROOT.clone(), &amount).ok();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&pool_2, ROOT.clone(), &amount).ok();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_2, ROOT.clone(), &amount).ok();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_3, ROOT.clone(), &amount).ok();

            // println!("{}", pool_3.total_debt_index());
            // println!("{}", pool_3.total_supply_index());

            let mut pool_map = BTreeMap::new();
            pool_map.insert(0 as PoolId, pool_0.clone());
            pool_map.insert(1 as PoolId, pool_1.clone());
            pool_map.insert(2 as PoolId, pool_2.clone());
            pool_map.insert(3 as PoolId, pool_3.clone());

            let result = UserAccountUtil::<Runtime>::accrue_interest_for_user(ROOT.clone(), &pool_map).unwrap();

            let s = result.supply.get(&3).unwrap();
            assert_eq!(s.amount(), amount.mul(pool_3.total_supply_index()));
            assert_eq!(s.index(), pool_3.total_supply_index());

            let s = result.supply.get(&2).unwrap();
            assert_eq!(s.amount(), amount.mul(pool_2.total_supply_index()));
            assert_eq!(s.index(), pool_2.total_supply_index());

            let s = result.debt.get(&0).unwrap();
            assert_eq!(s.amount(), amount.mul(pool_0.total_debt_index()));
            assert_eq!(s.index(), pool_0.total_debt_index());

            let s = result.debt.get(&1).unwrap();
            assert_eq!(s.amount(), amount.mul(pool_1.total_debt_index()));
            assert_eq!(s.index(), pool_1.total_debt_index());

            let s = result.debt.get(&2).unwrap();
            assert_eq!(s.amount(), amount.mul(pool_2.total_debt_index()));
            assert_eq!(s.index(), pool_2.total_debt_index());

            assert_eq!(result.supply.len(), 2);
            assert_eq!(result.debt.len(), 3);
        })
}

/// TODO
// #[test]
// fn floating_lend_accrue_interest_for_user_inconsistency() {
//     ExtBuilder::default()
//         .build()
//         .execute_with(|| {
//                 System::set_block_number(11);
//                 let mut pool_0 = pool_proxy(0, true);
//                 let mut pool_1 = pool_proxy(1, true);
//                 let mut pool_2 = pool_proxy(2, true);
//                 let mut pool_3 = pool_proxy(3, true);
//
//                 let supply = FixedU128::from(100);
//                 let borrow = FixedU128::from(50);
//         })
// }

#[test]
fn floating_lend_get_user_balances() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
                let pool_0 = pool_proxy(0, false);
                let pool_1 = pool_proxy(1, true);
                let pool_2 = pool_proxy(2, true);
                let pool_3 = pool_proxy(3, false);
                let pool_4 = pool_proxy(4, true);
                let mut pools = BTreeMap::new();

                pools.insert(0, pool_0.clone());
                pools.insert(1, pool_1.clone());
                pools.insert(2, pool_2.clone());
                pools.insert(3, pool_3.clone());
                pools.insert(4, pool_4.clone());
                let amount = FixedU128::from(100);
                UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&pool_0, ROOT.clone(), &amount).ok();
                UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&pool_1, ROOT.clone(), &amount).ok();
                UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&pool_2, ROOT.clone(), &amount).ok();
                UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_2, ROOT.clone(), &amount).ok();
                UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_3, ROOT.clone(), &amount).ok();
                UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_4, ROOT.clone(), &amount).ok();

                let user_supply_debt = UserAccountUtil::<Runtime>::accrue_interest_for_user(ROOT.clone(), &pools).unwrap();

                let balances = UserAccountUtil::<Runtime>::user_balances(&user_supply_debt, &pools).unwrap();
                assert_eq!(balances.debt_balance, FixedU128::from(300));
                assert_eq!(balances.supply_balance, FixedU128::from(300));
                assert_eq!(balances.collateral_balance, FixedU128::from(180));
        })
}

#[test]
fn floating_lend_get_check_withdraw_against_liquidation_trigger() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
                let pool_0 = pool_proxy(0, false);
                let pool_2 = pool_proxy(2, true);
                let pool_3 = pool_proxy(3, false);
                let pool_4 = pool_proxy(4, true);
                let mut pools = BTreeMap::new();

                pools.insert(0, pool_0.clone());
                pools.insert(2, pool_2.clone());
                pools.insert(3, pool_3.clone());
                pools.insert(4, pool_4.clone());
                let amount = FixedU128::from(100);
                UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&pool_0, ROOT.clone(), &amount).ok();
                UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_2, ROOT.clone(), &amount).ok();
                UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_3, ROOT.clone(), &amount).ok();
                UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_4, ROOT.clone(), &amount).ok();

                let user_supply_debt = UserAccountUtil::<Runtime>::accrue_interest_for_user(ROOT.clone(), &pools).unwrap();

                let is_trigger = UserAccountUtil::<Runtime>::is_withdraw_trigger_liquidation(
                        (4, FixedU128::from(89)),
                        user_supply_debt,
                        pools,
                        FixedU128::one(),
                ).unwrap();
                assert_eq!(is_trigger, true);
        })
}

#[test]
fn floating_lend_get_check_withdraw_against_liquidation_not_trigger() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
                let pool_0 = pool_proxy(0, false);
                let pool_2 = pool_proxy(2, true);
                let pool_3 = pool_proxy(3, false);
                let pool_4 = pool_proxy(4, true);
                let mut pools = BTreeMap::new();

                pools.insert(0, pool_0.clone());
                pools.insert(2, pool_2.clone());
                pools.insert(3, pool_3.clone());
                pools.insert(4, pool_4.clone());
                let amount = FixedU128::from(100);
                UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&pool_0, ROOT.clone(), &amount).ok();
                UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_2, ROOT.clone(), &amount).ok();
                UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_3, ROOT.clone(), &amount).ok();
                UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_4, ROOT.clone(), &amount).ok();

                let user_supply_debt = UserAccountUtil::<Runtime>::accrue_interest_for_user(ROOT.clone(), &pools).unwrap();

                let is_trigger = UserAccountUtil::<Runtime>::is_withdraw_trigger_liquidation(
                        (4, FixedU128::from(88)),
                        user_supply_debt,
                        pools,
                        FixedU128::one(),
                ).unwrap();
                assert_eq!(is_trigger, false);
        })
}

#[test]
fn floating_lend_get_check_borrow_against_liquidation_not_trigger() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
                let pool_0 = pool_proxy(0, false);
                let pool_2 = pool_proxy(2, true);
                let pool_3 = pool_proxy(3, false);
                let pool_4 = pool_proxy(4, true);
                let mut pools = BTreeMap::new();

                pools.insert(0, pool_0.clone());
                pools.insert(2, pool_2.clone());
                pools.insert(3, pool_3.clone());
                pools.insert(4, pool_4.clone());
                let amount = FixedU128::from(100);
                UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&pool_0, ROOT.clone(), &amount).ok();
                UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_2, ROOT.clone(), &amount).ok();
                UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_3, ROOT.clone(), &amount).ok();
                UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_4, ROOT.clone(), &amount).ok();

                let user_supply_debt = UserAccountUtil::<Runtime>::accrue_interest_for_user(ROOT.clone(), &pools).unwrap();

                let is_trigger = UserAccountUtil::<Runtime>::is_borrow_trigger_liquidation(
                        (0, FixedU128::from(80)),
                        user_supply_debt,
                        pools,
                        FixedU128::one(),
                ).unwrap();
                assert_eq!(is_trigger, false);
        })
}

#[test]
fn floating_lend_get_check_borrow_against_liquidation_trigger() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
                let pool_0 = pool_proxy(0, false);
                let pool_2 = pool_proxy(2, true);
                let pool_3 = pool_proxy(3, false);
                let pool_4 = pool_proxy(4, true);
                let mut pools = BTreeMap::new();

                pools.insert(0, pool_0.clone());
                pools.insert(2, pool_2.clone());
                pools.insert(3, pool_3.clone());
                pools.insert(4, pool_4.clone());
                let amount = FixedU128::from(100);
                UserAccountUtil::<Runtime>::accrue_interest_and_increment_debt(&pool_0, ROOT.clone(), &amount).ok();
                UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_2, ROOT.clone(), &amount).ok();
                UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_3, ROOT.clone(), &amount).ok();
                UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool_4, ROOT.clone(), &amount).ok();

                let user_supply_debt = UserAccountUtil::<Runtime>::accrue_interest_for_user(ROOT.clone(), &pools).unwrap();

                let is_trigger = UserAccountUtil::<Runtime>::is_borrow_trigger_liquidation(
                        (0, FixedU128::from_float(80.1)),
                        user_supply_debt,
                        pools,
                        FixedU128::one(),
                ).unwrap();
                assert_eq!(is_trigger, true);
        })
}