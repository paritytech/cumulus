//! Unit tests for the currencies module.

#![cfg(test)]

use super::*;
use frame_support::{assert_ok};
use mock::{*};

#[test]
fn currencies_transfer_should_work() {
    ExtBuilder::default()
        .one_hundred_for_alice_n_bob()
        .build()
        .execute_with(|| {
            assert_ok!(Currencies::transfer(Origin::signed(ALICE), ALICE, NATIVE_CURRENCY_ID, 50));
        });
}
