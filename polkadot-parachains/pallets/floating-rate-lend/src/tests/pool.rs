//! Unit tests for the floating rate lending module.

#![cfg(test)]

use sp_runtime::{FixedU128, FixedPointNumber};
use sp_runtime::traits::{Zero, One};
use crate::pool::Pool;
use crate::types::Convertor;
use crate::tests::mock::{*};
use sp_std::ops::{Mul};


/// This tests the initialization and the decrement/increment operations on the pool
#[test]
fn floating_lend_pool_increment_decrement() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let mut p = default_test_pool();

            // supply
            assert_eq!(p.supply(), FixedU128::zero());
            let amount = FixedU128::saturating_from_integer(100);
            p.increment_supply(&amount);
            assert_eq!(p.supply(), amount);
            p.decrement_supply(&amount);
            assert_eq!(p.supply(), FixedU128::zero());

            // debt
            assert_eq!(p.debt(), FixedU128::zero());
            p.increment_debt(&amount);
            assert_eq!(p.debt(), amount);
            p.decrement_debt(&amount);
            assert_eq!(p.debt(), FixedU128::zero());
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
fn floating_lend_pool_check_interest_rates() {
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
                initial_interest_rate,
                Zero::zero(),
                ROOT,
                1
            );

            // no supply, interest should be `initial_interest_rate`
            assert_eq!(p.debt_interest_rate().unwrap(), initial_interest_rate);
            assert_eq!(p.supply_interest_rate().unwrap(), FixedU128::zero());

            let supply = FixedU128::from(1000);
            let debt = FixedU128::from(500);
            // only supply, no debt, interest should be `initial_interest_rate`
            let supply_amount = FixedU128::from(supply);
            p.increment_supply(&supply_amount);
            assert_eq!(p.debt_interest_rate().unwrap(), initial_interest_rate);

            // with supply and debt, interest should be updated
            let expected_debt_rate = FixedU128::saturating_from_rational(22831050228 as u128, FixedU128::accuracy());
            let expected_supply_rate = FixedU128::saturating_from_rational(11415525114 as u128, FixedU128::accuracy());
            let debt_amount = FixedU128::from(debt);
            p.increment_debt(&debt_amount);
            // Rate calculation:
            //  initial_interest_rate + utilization_factor / FixedU128::from(2)
            assert_eq!(p.debt_interest_rate().unwrap(), expected_debt_rate);
            assert_eq!(p.supply_interest_rate().unwrap(), expected_supply_rate);
        });
}

#[test]
fn floating_lend_pool_discounted_price() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let p = default_test_pool();
            assert_eq!(p.discounted_price(&FixedU128::one()), FixedU128::from_float(0.9));
        });
}

#[test]
fn floating_lend_pool_accrue_interest_past_blocks() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let mut p = default_test_pool();
            let s = p.supply();
            let d = p.debt();
            let s_i = p.total_supply_index();
            let d_i = p.total_debt_index();
            p.accrue_interest(1).unwrap();

            assert_eq!(p.supply(), s);
            assert_eq!(p.debt(), d);
            assert_eq!(p.total_supply_index(), s_i);
            assert_eq!(p.total_debt_index(), d_i);
        });
}

#[test]
fn floating_lend_pool_accrue_interest_multiple_times() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let mut p = default_test_pool();
            let supply_amount = FixedU128::from(2);
            let borrow_amount = FixedU128::from(1);
            p.increment_supply(&supply_amount);
            p.increment_debt(&borrow_amount);

            System::set_block_number(11);

            // At this point:
            // utilization_ratio: 0.5
            // debt_interest_rate: 2.31e-8
            // supply_interest_rate: 1.155e-8
            // These two values are derived from the protocol before hand.
            let debt_multiplier = FixedU128::saturating_from_rational(1000000231 as u128, 1000000000 as u128);
            let supply_multiplier = FixedU128::saturating_from_rational(10000001155 as u128, 10000000000 as u128);

            // an extra `supply_amount` is added due to previous supply at block 11
            let expected_supply = supply_multiplier.mul(supply_amount);
            let expected_borrow = debt_multiplier.mul(borrow_amount);

            p.accrue_interest(11).unwrap();
            assert_eq!(expected_supply, p.supply());
            assert_eq!(expected_borrow, p.debt());

            p.increment_supply(&supply_amount);
            p.accrue_interest(21).unwrap();
            assert_eq!(FixedU128::from_inner(4000000365750047781), p.supply());
            assert_eq!(FixedU128::from_inner(1000000365750047797), p.debt());
        });
}
