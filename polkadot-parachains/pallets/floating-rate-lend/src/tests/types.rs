//! Unit tests for the floating rate lending module.

#![cfg(test)]

use sp_runtime::{FixedU128, FixedPointNumber};
use sp_runtime::traits::{One, Convert};
use crate::types::{Convertor, UserData, UserAccountUtil};
use polkadot_parachain_primitives::BALANCE_ONE;
use crate::mock::{*};
use sp_std::ops::Add;
use crate::{PoolUserSupplies, Error, UserSupplySet, PoolUserDebts};

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
            let pool = default_test_pool();
            let user_data_1 = PoolUserSupplies::<Runtime>::get(pool.id, ACCOUNT_1.clone());
            let user_data_2 = PoolUserSupplies::<Runtime>::get(pool.id, ACCOUNT_2.clone());

            // ensure user has no data first
            assert_eq!(user_data_1.is_none(), true);
            assert_eq!(user_data_2.is_none(), true);

            let amount_1 = FixedU128::from(100);
            let amount_2 = FixedU128::from(200);
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool, ACCOUNT_1.clone(), &amount_1).ok();
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool, ACCOUNT_2.clone(), &amount_2).ok();

            // now, we should have supply already
            let user_data_1 = PoolUserSupplies::<Runtime>::get(pool.id, ACCOUNT_1.clone());
            assert_eq!(user_data_1.is_some(), true);
            let user_data_1 = user_data_1.unwrap();
            assert_eq!(user_data_1.amount(), amount_1);
            assert_eq!(user_data_1.index(), FixedU128::one());

            // now, we supply again
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool, ACCOUNT_1.clone(), &amount_1).ok();

            let user_data_1 = PoolUserSupplies::<Runtime>::get(pool.id, ACCOUNT_1.clone());
            assert_eq!(user_data_1.is_some(), true);
            let user_data_1 = user_data_1.unwrap();
            assert_eq!(user_data_1.amount(), amount_1 * FixedU128::from(2));
            assert_eq!(user_data_1.index(), FixedU128::one());

            // check user 2
            let user_data_2 = PoolUserSupplies::<Runtime>::get(pool.id, ACCOUNT_2.clone());
            assert_eq!(user_data_2.is_some(), true);
            let user_data_2 = user_data_2.unwrap();
            assert_eq!(user_data_2.amount(), amount_2);
            assert_eq!(user_data_2.index(), FixedU128::one());
        })
}

/// Decrement user supply when the user has no supply
#[test]
fn floating_lend_decrement_user_supply() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let pool = default_test_pool();
            let user_data = PoolUserSupplies::<Runtime>::get(pool.id, ROOT.clone());
            // ensure user has no data first
            assert_eq!(user_data.is_none(), true);

            let amount = FixedU128::from(100);
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool, ROOT.clone(), &amount).ok();
            let user_data = PoolUserSupplies::<Runtime>::get(pool.id, ROOT.clone()).unwrap();

            let amount_deduct = FixedU128::from(50);
            UserAccountUtil::<Runtime>::decrement_supply(&pool, ROOT.clone(), &amount_deduct, user_data).ok();

            let user_data = PoolUserSupplies::<Runtime>::get(pool.id, ROOT.clone());
            assert_eq!(user_data.is_some(), true);
            let user_data = user_data.unwrap();
            assert_eq!(user_data.amount(), amount - amount_deduct);
            assert_eq!(user_data.index(), FixedU128::one());
            let supplies = UserSupplySet::<Runtime>::get(ROOT.clone());
            assert_eq!(supplies.contains(&(pool.id, pool.currency_id)), true);

            // now, we decrement supply again
            UserAccountUtil::<Runtime>::decrement_supply(&pool, ROOT.clone(), &amount_deduct,  user_data).ok();

            let user_data = PoolUserSupplies::<Runtime>::get(pool.id, ROOT.clone());
            assert_eq!(user_data.is_none(), true);
            let supplies = UserSupplySet::<Runtime>::get(ROOT.clone());
            assert_eq!(supplies.contains(&(pool.id, pool.currency_id)), false);
        })
}

/// Decrement user supply when the user has no supply
#[test]
fn floating_lend_decrement_user_supply_over_deduction() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let pool = default_test_pool();
            let user_data = PoolUserSupplies::<Runtime>::get(pool.id, ROOT.clone());
            // ensure user has no data first
            assert_eq!(user_data.is_none(), true);

            let amount = FixedU128::from(100);
            UserAccountUtil::<Runtime>::accrue_interest_and_increment_supply(&pool, ROOT.clone(), &amount).ok();
            let user_data = PoolUserSupplies::<Runtime>::get(pool.id, ROOT.clone()).unwrap();

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
            UserAccountUtil::<Runtime>::increment_debt(&pool, ROOT.clone(), amount).ok();
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
            UserAccountUtil::<Runtime>::increment_debt(&pool, ROOT.clone(), amount).ok();
            UserAccountUtil::<Runtime>::increment_debt(&pool, ROOT.clone(), amount).ok();
            let user_data = PoolUserDebts::<Runtime>::get(pool.id(), ROOT.clone());
            assert_eq!(user_data.is_some(), true);
            let user_data = user_data.unwrap();
            assert_eq!(user_data.amount(), FixedU128::from(200));
        })
}