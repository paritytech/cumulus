use super::*;
use crate::mock::*;
use frame_support::sp_runtime::traits::Hash;
use frame_support::traits::OnFinalize;
use frame_support::{assert_noop, assert_ok};
use frame_system::InitKind;
use sp_runtime::{FixedU128, DispatchError, FixedPointNumber};

const user1: u64 = 1;
const user2: u64 = 2;

const asset1: u64 = 0;
const asset2: u64 = 1;

fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = ExtBuilder::default().build();
	ext.execute_with(|| System::set_block_number(1));
	ext
}

///  Under current configuration, user 1 has 1000000000000000000 - 500000 of every assets and user 2 has 500000 of every assets
///  Inital price of asset 0 is 100, 1 is 60
///  TODO: need to make this easier
#[test]
fn can_supply() {
	new_test_ext().execute_with(|| {

		let user_balance_before = Assets::get_asset_balance((asset1, user1));

		assert_ok!(Lending::supply(
			Origin::signed(user1),
			asset1,
			100000,
		));

		let user_supply = Lending::user_supply(asset1, user1).unwrap();

		assert_eq!(user_supply.amount, 100000);
		assert_eq!(user_supply.index, FixedU128::one());

		let pool_supply = Lending::pool(asset1).unwrap();

		assert_eq!(pool_supply.supply, 100000);
		assert_eq!(pool_supply.total_supply_index, FixedU128::one());
		assert_eq!(pool_supply.last_updated, System::block_number());

		let user_supply_set = Lending::user_supply_set(user1);
		assert_eq!(user_supply_set, vec![0]);

		let user_balance_after = Assets::get_asset_balance((asset1, user1));
		assert_eq!(user_balance_before - user_balance_after, 100000);

    });
}

#[test]
fn can_borrow() {
	new_test_ext().execute_with(|| {

		System::set_block_number(1);

		assert_ok!(Lending::supply(
			Origin::signed(user1),
			asset1,
			100000,
		));

        assert_ok!(Lending::supply(
			Origin::signed(user2),
			asset2,
			100000,
		));

        assert_ok!(Lending::borrow(
			Origin::signed(user2),
			asset1,
			10000,
		));


		System::set_block_number(100000);

		// update the index
		assert_ok!(Lending::supply(
			Origin::signed(user1),
			asset1,
			1,
		));

		let user1_supply = Lending::user_supply(asset1, user1).unwrap();

		assert!(user1_supply.amount > 100001);
		assert!(user1_supply.index > FixedU128::one());

    });
}

#[test]
fn can_repay() {
	new_test_ext().execute_with(|| {

		assert_ok!(Lending::supply(
			Origin::signed(1),
			0,
			100000,
		));

        assert_ok!(Lending::supply(
			Origin::signed(2),
			1,
			100000,
		));

        assert_ok!(Lending::borrow(
			Origin::signed(2),
			0,
			10000,
		));

		assert_ok!(Lending::repay(
			Origin::signed(2),
			0,
			10000,
		));

    });
}

#[test]
fn can_withdraw() {
	new_test_ext().execute_with(|| {

		assert_ok!(Lending::supply(
			Origin::signed(1),
			0,
			100000,
		));

        assert_ok!(Lending::supply(
			Origin::signed(2),
			1,
			100000,
		));

        assert_ok!(Lending::borrow(
			Origin::signed(2),
			0,
			10000,
		));

		assert_ok!(Lending::withdraw(
			Origin::signed(1),
			0,
			50000,
		));

    });
}

#[test]
fn can_liquidate() {
	new_test_ext().execute_with(|| {

		assert_ok!(Lending::supply(
			Origin::signed(1),
			0,
			100000,
		));

        assert_ok!(Lending::supply(
			Origin::signed(2),
			1,
			100000,
		));

        assert_ok!(Lending::borrow(
			Origin::signed(2),
			0,
			10000,
		));

		assert_ok!(Assets::set_price(
			Origin::root(),
			1,
			FixedU128::saturating_from_integer(5),
		));

        assert_ok!(Lending::liquidate(
			Origin::signed(1),
			2,
			0,
			1,
			10000,
		));


    });
}
