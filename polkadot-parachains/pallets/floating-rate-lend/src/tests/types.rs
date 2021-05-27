//! Unit tests for the floating rate lending module.

#![cfg(test)]

use sp_runtime::{FixedU128, FixedPointNumber};
use sp_runtime::traits::{One, Convert};
use crate::types::{Convertor, UserData};
use polkadot_parachain_primitives::BALANCE_ONE;
use crate::mock::{*};
use sp_std::ops::Add;

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
