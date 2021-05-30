use crate::{Error, PoolStorage, PoolUserSupplies};
use crate::tests::mock::{*};

use frame_support::{assert_noop, assert_ok};
use crate::pool::PoolRepository;
use sp_runtime::FixedU128;
use polkadot_parachain_primitives::BALANCE_ONE;

#[test]
fn floating_lend_supply_zero_balance() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            assert_noop!(
                FloatingRateLend::supply(Origin::signed(ROOT), 0, 0),
                Error::<Runtime>::BalanceTooLow
            );
        });
}

#[test]
fn floating_lend_supply_not_enabled() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let pool = default_pool_proxy();
            PoolRepository::save(pool);
            assert_noop!(
                FloatingRateLend::supply(Origin::signed(ROOT), 0, 100),
                Error::<Runtime>::PoolNotEnabled
            );
        });
}

#[test]
fn floating_lend_supply_below_minimal() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let pool = default_pool_proxy();
            PoolRepository::save(pool);
            FloatingRateLend::enable_pool(Origin::signed(ROOT), 0).ok();
            assert_noop!(
                FloatingRateLend::supply(Origin::signed(ROOT), 0, 1),
                Error::<Runtime>::BalanceTooLow
            );
        });
}

#[test]
fn floating_lend_supply_success() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let pool = default_pool_proxy();
            PoolRepository::save(pool.clone());
            FloatingRateLend::enable_pool(Origin::signed(ROOT), 0).ok();

            assert_ok!(FloatingRateLend::supply(Origin::signed(ROOT), 0, 100 * BALANCE_ONE));

            let pool = PoolStorage::<Runtime>::get(pool.id()).unwrap();
            assert_eq!(pool.supply(), FixedU128::from(100));

            let supply = PoolUserSupplies::<Runtime>::get(pool.id, ROOT).unwrap();
            assert_eq!(supply.amount(), FixedU128::from(100));
        });
}