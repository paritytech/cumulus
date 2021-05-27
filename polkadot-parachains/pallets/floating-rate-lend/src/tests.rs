//! Unit tests for the floating rate lending module.

#![cfg(test)]

use super::*;
use frame_support::{assert_ok};
use mock::{*};
use sp_runtime::{FixedU128, FixedPointNumber};
use sp_runtime::traits::{Zero, One, Convert};
use sp_runtime::sp_std::ops::{Add};
use crate::pool::Pool;
use crate::types::Convertor;
use crate::Error::BalanceTooLow;
use polkadot_parachain_primitives::BALANCE_ONE;

const DEFAULT_CLOSE_FACTOR: f64 = 0.9;
const DEFAULT_SAFE_FACTOR: f64 = 0.9;

fn default_test_pool() -> Pool<Runtime> {
    let utilization_factor= FixedU128::saturating_from_rational(385, 1000000000u64);
    let initial_interest_rate = FixedU128::saturating_from_rational(385, 10000000000u64);
    let discount_factor = FixedU128::saturating_from_rational(90, 100);
    Pool::<Runtime>::new(
        0,
        vec![],
        0,
        false,
        FixedU128::from_float(DEFAULT_SAFE_FACTOR),
        FixedU128::from_float(DEFAULT_CLOSE_FACTOR),
        discount_factor,
        utilization_factor,
        initial_interest_rate,
        Zero::zero(),
        ROOT,
        1
    )
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

/// This tests the initialization and the decrement/increment operations on the pool
#[test]
fn floating_lend_pool_increment_decrement() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let mut p = default_test_pool();

            // supply
            assert_eq!(p.supply, FixedU128::zero());
            let amount = FixedU128::saturating_from_integer(100);
            p.increment_supply(&amount);
            assert_eq!(p.supply, amount);
            p.decrement_supply(&amount);
            assert_eq!(p.supply, FixedU128::zero());

            // debt
            assert_eq!(p.debt, FixedU128::zero());
            p.increment_debt(&amount);
            assert_eq!(p.debt, amount);
            p.decrement_debt(&amount);
            assert_eq!(p.debt, FixedU128::zero());
        });
}

#[test]
fn floating_lend_pool_closable_amount_below_min() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let p = default_test_pool();

            // below the min closable amount, will just take all
            let amount = FixedU128::saturating_from_integer(100);
            let price = FixedU128::one();
            let closable_amount = p.closable_amount(&amount, &price);
            assert_eq!(amount, closable_amount);

            let amount = FixedU128::saturating_from_integer(100);
            let price = FixedU128::saturating_from_rational(90, 100);
            let closable_amount = p.closable_amount(&amount, &price);
            assert_eq!(amount, closable_amount);
        });
}

#[test]
fn floating_lend_pool_closable_amount_above_min() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let p = default_test_pool();

            // below the min closable amount, will just take all
            let amount = FixedU128::saturating_from_integer(100);
            let price = FixedU128::from(2);

            let closable_amount = p.closable_amount(&amount, &price);
            assert_eq!(FixedU128::from(90), closable_amount);
        });
}

#[test]
fn floating_lend_pool_debt_interest_rate() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let utilization_factor= Convertor::convert_percentage_annum_to_per_block(20);
            let initial_interest_rate = Convertor::convert_percentage_annum_to_per_block(2);

            let mut p = Pool::<Runtime>::new(
                0,
                vec![],
                0,
                false,
                Convertor::convert_percentage(90),
                Convertor::convert_percentage(90),
                Convertor::convert_percentage(90),
                utilization_factor,
                FixedU128::saturating_from_rational(385, 10000000000u64),
                Zero::zero(),
                ROOT,
                1
            );

            // no supply, interest should be `initial_interest_rate`
            match p.debt_interest_rate() {
                Ok(rate) => {
                    assert_eq!(rate, initial_interest_rate);
                },
                Err(_) => {
                    assert!(false, "should not have thrown error here");
                }
            }

            // only supply, no debt, interest should be `initial_interest_rate`
            let supply_amount = FixedU128::from(100);
            p.increment_supply(&supply_amount);
            match p.debt_interest_rate() {
                Ok(rate) => {
                    assert_eq!(p.supply, supply_amount);
                    assert_eq!(rate, initial_interest_rate);
                },
                Err(_) => {
                    assert!(false, "should not have thrown error here");
                }
            }

            // with supply and debt, interest should be updated
            let debt_amount = FixedU128::from(50);
            p.increment_debt(&debt_amount);
            // Rate calculation:
            //  0.0000000385 + 0.000000385 * 50 / 100
            match p.debt_interest_rate() {
                Ok(rate) => {
                    assert_eq!(rate, initial_interest_rate);
                },
                Err(_) => {
                    assert!(false, "should not have thrown error here");
                }
            }

        });
}