// This file includes the tests for the admin related functions
use crate::tests::mock::{*};
use crate::pool::PoolRepository;
use frame_support::{assert_noop, assert_ok};
use frame_support::error::BadOrigin;

#[test]
fn floating_lend_supply_enable_pool() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let pool = default_pool_proxy();
            PoolRepository::save(pool);
            assert_ok!(
                FloatingRateLend::enable_pool(Origin::signed(ROOT), 0)
            );
        });
}

#[test]
fn floating_lend_supply_enable_pool_not_authorized() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let pool = default_pool_proxy();
            PoolRepository::save(pool);
            assert_noop!(
                FloatingRateLend::enable_pool(Origin::signed(ACCOUNT_1), 0),
                BadOrigin
            );
        });
}