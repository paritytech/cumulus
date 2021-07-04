//! Unit tests for the tokens module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};

#[test]
fn minimum_balance_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Tokens::minimum_balance(BTC), 1);
		assert_eq!(Tokens::minimum_balance(DOT), 2);
		assert_eq!(Tokens::minimum_balance(ETH), 0);
	});
}

#[test]
fn is_module_account_id_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Tokens::is_module_account_id(&ALICE), false);
		assert_eq!(Tokens::is_module_account_id(&BOB), false);
		assert_eq!(Tokens::is_module_account_id(&TREASURY_ACCOUNT), false);
		assert_eq!(Tokens::is_module_account_id(&DustAccount::get()), true);
	});
}

#[test]
fn remove_dust_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(Tokens::deposit(DOT, &ALICE, 100));
		assert_eq!(Tokens::total_issuance(DOT), 100);
		assert_eq!(Accounts::<Runtime>::contains_key(ALICE, DOT), true);
		assert_eq!(Tokens::free_balance(DOT, &ALICE), 100);
		assert_eq!(System::providers(&ALICE), 1);
		assert_eq!(Accounts::<Runtime>::contains_key(DustAccount::get(), DOT), false);
		assert_eq!(Tokens::free_balance(DOT, &DustAccount::get()), 0);
		assert_eq!(System::providers(&DustAccount::get()), 0);

		// total is gte ED, will not handle dust
		assert_ok!(Tokens::withdraw(DOT, &ALICE, 98));
		assert_eq!(Tokens::total_issuance(DOT), 2);
		assert_eq!(Accounts::<Runtime>::contains_key(ALICE, DOT), true);
		assert_eq!(Tokens::free_balance(DOT, &ALICE), 2);
		assert_eq!(System::providers(&ALICE), 1);
		assert_eq!(Accounts::<Runtime>::contains_key(DustAccount::get(), DOT), false);
		assert_eq!(Tokens::free_balance(DOT, &DustAccount::get()), 0);
		assert_eq!(System::providers(&DustAccount::get()), 0);

		assert_ok!(Tokens::withdraw(DOT, &ALICE, 1));

		// total is lte ED, will handle dust
		assert_eq!(Tokens::total_issuance(DOT), 1);
		assert_eq!(Accounts::<Runtime>::contains_key(ALICE, DOT), false);
		assert_eq!(Tokens::free_balance(DOT, &ALICE), 0);
		assert_eq!(System::providers(&ALICE), 0);

		// will not handle dust for module account
		assert_eq!(Accounts::<Runtime>::contains_key(DustAccount::get(), DOT), true);
		assert_eq!(Tokens::free_balance(DOT, &DustAccount::get()), 1);
		assert_eq!(System::providers(&DustAccount::get()), 1);

		System::assert_last_event(Event::Tokens(crate::Event::DustLost(DOT, ALICE, 1)));
	});
}

#[test]
fn set_free_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		Tokens::set_free_balance(DOT, &ALICE, 100);
		System::assert_last_event(Event::Tokens(crate::Event::Endowed(DOT, ALICE, 100)));
	});
}

#[test]
fn set_lock_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Tokens::set_lock(ID_1, DOT, &ALICE, 10));
			assert_eq!(Tokens::accounts(&ALICE, DOT).frozen, 10);
			assert_eq!(Tokens::accounts(&ALICE, DOT).frozen(), 10);
			assert_eq!(Tokens::locks(ALICE, DOT).len(), 1);
			assert_ok!(Tokens::set_lock(ID_1, DOT, &ALICE, 50));
			assert_eq!(Tokens::accounts(&ALICE, DOT).frozen, 50);
			assert_eq!(Tokens::locks(ALICE, DOT).len(), 1);
			assert_ok!(Tokens::set_lock(ID_2, DOT, &ALICE, 60));
			assert_eq!(Tokens::accounts(&ALICE, DOT).frozen, 60);
			assert_eq!(Tokens::locks(ALICE, DOT).len(), 2);
		});
}

#[test]
fn extend_lock_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Tokens::set_lock(ID_1, DOT, &ALICE, 10));
			assert_eq!(Tokens::locks(ALICE, DOT).len(), 1);
			assert_eq!(Tokens::accounts(&ALICE, DOT).frozen, 10);
			assert_ok!(Tokens::extend_lock(ID_1, DOT, &ALICE, 20));
			assert_eq!(Tokens::locks(ALICE, DOT).len(), 1);
			assert_eq!(Tokens::accounts(&ALICE, DOT).frozen, 20);
			assert_ok!(Tokens::extend_lock(ID_2, DOT, &ALICE, 10));
			assert_ok!(Tokens::extend_lock(ID_1, DOT, &ALICE, 20));
			assert_eq!(Tokens::locks(ALICE, DOT).len(), 2);
		});
}

#[test]
fn remove_lock_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Tokens::set_lock(ID_1, DOT, &ALICE, 10));
			assert_ok!(Tokens::set_lock(ID_2, DOT, &ALICE, 20));
			assert_eq!(Tokens::locks(ALICE, DOT).len(), 2);
			assert_ok!(Tokens::remove_lock(ID_2, DOT, &ALICE));
			assert_eq!(Tokens::locks(ALICE, DOT).len(), 1);
		});
}

#[test]
fn frozen_can_limit_liquidity() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Tokens::set_lock(ID_1, DOT, &ALICE, 90));
			assert_noop!(
				<Tokens as MultiCurrency<_>>::transfer(DOT, &ALICE, &BOB, 11),
				Error::<Runtime>::LiquidityRestrictions,
			);
			assert_ok!(Tokens::set_lock(ID_1, DOT, &ALICE, 10));
			assert_ok!(<Tokens as MultiCurrency<_>>::transfer(DOT, &ALICE, &BOB, 11));
		});
}

#[test]
fn can_reserve_is_correct() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_eq!(Tokens::can_reserve(DOT, &ALICE, 0), true);
			assert_eq!(Tokens::can_reserve(DOT, &ALICE, 101), false);
			assert_eq!(Tokens::can_reserve(DOT, &ALICE, 100), true);
		});
}

#[test]
fn reserve_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_noop!(Tokens::reserve(DOT, &ALICE, 101), Error::<Runtime>::BalanceTooLow);
			assert_ok!(Tokens::reserve(DOT, &ALICE, 0));
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 100);
			assert_eq!(Tokens::reserved_balance(DOT, &ALICE), 0);
			assert_eq!(Tokens::total_balance(DOT, &ALICE), 100);
			assert_ok!(Tokens::reserve(DOT, &ALICE, 50));
			System::assert_last_event(Event::Tokens(crate::Event::Reserved(DOT, ALICE, 50)));
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 50);
			assert_eq!(Tokens::reserved_balance(DOT, &ALICE), 50);
			assert_eq!(Tokens::total_balance(DOT, &ALICE), 100);
		});
}

#[test]
fn unreserve_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 100);
			assert_eq!(Tokens::reserved_balance(DOT, &ALICE), 0);
			assert_eq!(Tokens::unreserve(DOT, &ALICE, 0), 0);
			assert_eq!(Tokens::unreserve(DOT, &ALICE, 50), 50);
			System::assert_last_event(Event::Tokens(crate::Event::Unreserved(DOT, ALICE, 0)));
			assert_ok!(Tokens::reserve(DOT, &ALICE, 30));
			System::assert_last_event(Event::Tokens(crate::Event::Reserved(DOT, ALICE, 30)));
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 70);
			assert_eq!(Tokens::reserved_balance(DOT, &ALICE), 30);
			assert_eq!(Tokens::unreserve(DOT, &ALICE, 15), 0);
			System::assert_last_event(Event::Tokens(crate::Event::Unreserved(DOT, ALICE, 15)));
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 85);
			assert_eq!(Tokens::reserved_balance(DOT, &ALICE), 15);
			assert_eq!(Tokens::unreserve(DOT, &ALICE, 30), 15);
			System::assert_last_event(Event::Tokens(crate::Event::Unreserved(DOT, ALICE, 15)));
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 100);
			assert_eq!(Tokens::reserved_balance(DOT, &ALICE), 0);
		});
}

#[test]
fn slash_reserved_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Tokens::reserve(DOT, &ALICE, 50));
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 50);
			assert_eq!(Tokens::reserved_balance(DOT, &ALICE), 50);
			assert_eq!(Tokens::total_issuance(DOT), 200);
			assert_eq!(Tokens::slash_reserved(DOT, &ALICE, 0), 0);
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 50);
			assert_eq!(Tokens::reserved_balance(DOT, &ALICE), 50);
			assert_eq!(Tokens::total_issuance(DOT), 200);
			assert_eq!(Tokens::slash_reserved(DOT, &ALICE, 100), 50);
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 50);
			assert_eq!(Tokens::reserved_balance(DOT, &ALICE), 0);
			assert_eq!(Tokens::total_issuance(DOT), 150);
		});
}

#[test]
fn repatriate_reserved_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 100);
			assert_eq!(Tokens::reserved_balance(DOT, &ALICE), 0);
			assert_eq!(
				Tokens::repatriate_reserved(DOT, &ALICE, &ALICE, 0, BalanceStatus::Free),
				Ok(0)
			);
			assert_eq!(
				Tokens::repatriate_reserved(DOT, &ALICE, &ALICE, 50, BalanceStatus::Free),
				Ok(50)
			);
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 100);
			assert_eq!(Tokens::reserved_balance(DOT, &ALICE), 0);

			assert_eq!(Tokens::free_balance(DOT, &BOB), 100);
			assert_eq!(Tokens::reserved_balance(DOT, &BOB), 0);
			assert_ok!(Tokens::reserve(DOT, &BOB, 50));
			assert_eq!(Tokens::free_balance(DOT, &BOB), 50);
			assert_eq!(Tokens::reserved_balance(DOT, &BOB), 50);
			assert_eq!(
				Tokens::repatriate_reserved(DOT, &BOB, &BOB, 60, BalanceStatus::Reserved),
				Ok(10)
			);
			assert_eq!(Tokens::free_balance(DOT, &BOB), 50);
			assert_eq!(Tokens::reserved_balance(DOT, &BOB), 50);

			assert_eq!(
				Tokens::repatriate_reserved(DOT, &BOB, &ALICE, 30, BalanceStatus::Reserved),
				Ok(0)
			);
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 100);
			assert_eq!(Tokens::reserved_balance(DOT, &ALICE), 30);
			assert_eq!(Tokens::free_balance(DOT, &BOB), 50);
			assert_eq!(Tokens::reserved_balance(DOT, &BOB), 20);

			assert_eq!(
				Tokens::repatriate_reserved(DOT, &BOB, &ALICE, 30, BalanceStatus::Free),
				Ok(10)
			);
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 120);
			assert_eq!(Tokens::reserved_balance(DOT, &ALICE), 30);
			assert_eq!(Tokens::free_balance(DOT, &BOB), 50);
			assert_eq!(Tokens::reserved_balance(DOT, &BOB), 0);
		});
}

#[test]
fn slash_draw_reserved_correct() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Tokens::reserve(DOT, &ALICE, 50));
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 50);
			assert_eq!(Tokens::reserved_balance(DOT, &ALICE), 50);
			assert_eq!(Tokens::total_issuance(DOT), 200);

			assert_eq!(Tokens::slash(DOT, &ALICE, 80), 0);
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 0);
			assert_eq!(Tokens::reserved_balance(DOT, &ALICE), 20);
			assert_eq!(Tokens::total_issuance(DOT), 120);

			assert_eq!(Tokens::slash(DOT, &ALICE, 50), 30);
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 0);
			assert_eq!(Tokens::reserved_balance(DOT, &ALICE), 0);
			assert_eq!(Tokens::total_issuance(DOT), 100);
		});
}

#[test]
fn genesis_issuance_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 100);
			assert_eq!(Tokens::free_balance(DOT, &BOB), 100);
			assert_eq!(Tokens::total_issuance(DOT), 200);
		});
}

#[test]
fn transfer_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			System::set_block_number(1);

			assert_ok!(Tokens::transfer(Some(ALICE).into(), BOB, DOT, 50));
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 50);
			assert_eq!(Tokens::free_balance(DOT, &BOB), 150);
			assert_eq!(Tokens::total_issuance(DOT), 200);

			System::assert_last_event(Event::Tokens(crate::Event::Transfer(DOT, ALICE, BOB, 50)));

			assert_noop!(
				Tokens::transfer(Some(ALICE).into(), BOB, DOT, 60),
				Error::<Runtime>::BalanceTooLow,
			);
			assert_noop!(
				Tokens::transfer(Some(ALICE).into(), CHARLIE, DOT, 1),
				Error::<Runtime>::ExistentialDeposit,
			);
			assert_ok!(Tokens::transfer(Some(ALICE).into(), CHARLIE, DOT, 2));
		});
}

#[test]
fn transfer_all_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			System::set_block_number(1);

			assert_ok!(Tokens::transfer_all(Some(ALICE).into(), BOB, DOT));
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 0);
			assert_eq!(Tokens::free_balance(DOT, &BOB), 200);

			System::assert_last_event(Event::Tokens(crate::Event::Transfer(DOT, ALICE, BOB, 100)));
		});
}

#[test]
fn deposit_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Tokens::deposit(DOT, &ALICE, 100));
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 200);
			assert_eq!(Tokens::total_issuance(DOT), 300);

			assert_noop!(
				Tokens::deposit(DOT, &ALICE, Balance::max_value()),
				ArithmeticError::Overflow,
			);
		});
}

#[test]
fn withdraw_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Tokens::withdraw(DOT, &ALICE, 50));
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 50);
			assert_eq!(Tokens::total_issuance(DOT), 150);

			assert_noop!(Tokens::withdraw(DOT, &ALICE, 60), Error::<Runtime>::BalanceTooLow);
		});
}

#[test]
fn slash_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// slashed_amount < amount
			assert_eq!(Tokens::slash(DOT, &ALICE, 50), 0);
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 50);
			assert_eq!(Tokens::total_issuance(DOT), 150);

			// slashed_amount == amount
			assert_eq!(Tokens::slash(DOT, &ALICE, 51), 1);
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 0);
			assert_eq!(Tokens::total_issuance(DOT), 100);
		});
}

#[test]
fn update_balance_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Tokens::update_balance(DOT, &ALICE, 50));
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 150);
			assert_eq!(Tokens::total_issuance(DOT), 250);

			assert_ok!(Tokens::update_balance(DOT, &BOB, -50));
			assert_eq!(Tokens::free_balance(DOT, &BOB), 50);
			assert_eq!(Tokens::total_issuance(DOT), 200);

			assert_noop!(Tokens::update_balance(DOT, &BOB, -60), Error::<Runtime>::BalanceTooLow);
		});
}

#[test]
fn ensure_can_withdraw_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_noop!(
				Tokens::ensure_can_withdraw(DOT, &ALICE, 101),
				Error::<Runtime>::BalanceTooLow
			);

			assert_ok!(Tokens::ensure_can_withdraw(DOT, &ALICE, 1));
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 100);
		});
}

#[test]
fn no_op_if_amount_is_zero() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Tokens::ensure_can_withdraw(DOT, &ALICE, 0));
		assert_ok!(Tokens::transfer(Some(ALICE).into(), BOB, DOT, 0));
		assert_ok!(Tokens::transfer(Some(ALICE).into(), ALICE, DOT, 0));
		assert_ok!(Tokens::deposit(DOT, &ALICE, 0));
		assert_ok!(Tokens::withdraw(DOT, &ALICE, 0));
		assert_eq!(Tokens::slash(DOT, &ALICE, 0), 0);
		assert_eq!(Tokens::slash(DOT, &ALICE, 1), 1);
		assert_ok!(Tokens::update_balance(DOT, &ALICE, 0));
	});
}

#[test]
fn transfer_all_trait_should_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, DOT, 100), (ALICE, BTC, 200)])
		.build()
		.execute_with(|| {
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 100);
			assert_eq!(Tokens::free_balance(BTC, &ALICE), 200);
			assert_eq!(Tokens::free_balance(DOT, &BOB), 0);

			assert_ok!(<Tokens as TransferAll<AccountId>>::transfer_all(&ALICE, &BOB));
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 0);
			assert_eq!(Tokens::free_balance(BTC, &ALICE), 0);
			assert_eq!(Tokens::free_balance(DOT, &BOB), 100);
			assert_eq!(Tokens::free_balance(BTC, &BOB), 200);

			assert_ok!(Tokens::reserve(DOT, &BOB, 1));
			assert_ok!(<Tokens as TransferAll<AccountId>>::transfer_all(&BOB, &ALICE));
			assert_eq!(Tokens::free_balance(DOT, &ALICE), 99);
			assert_eq!(Tokens::free_balance(BTC, &ALICE), 200);
			assert_eq!(Tokens::free_balance(DOT, &BOB), 0);
			assert_eq!(Tokens::free_balance(BTC, &BOB), 0);
		});
}

#[test]
fn currency_adapter_ensure_currency_adapter_should_work() {
	ExtBuilder::default()
		.one_hundred_for_treasury_account()
		.build()
		.execute_with(|| {
			assert_eq!(Tokens::total_issuance(DOT), 102);
			assert_eq!(Tokens::total_balance(DOT, &Treasury::account_id()), 2);
			assert_eq!(Tokens::total_balance(DOT, &TREASURY_ACCOUNT), 100);
			assert_eq!(Tokens::reserved_balance(DOT, &TREASURY_ACCOUNT), 0);
			assert_eq!(Tokens::free_balance(DOT, &TREASURY_ACCOUNT), 100);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::total_balance(&TREASURY_ACCOUNT),
				100
			);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::can_slash(&TREASURY_ACCOUNT, 10),
				true
			);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::total_issuance(),
				102
			);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::minimum_balance(),
				2
			);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::can_reserve(&TREASURY_ACCOUNT, 5),
				true
			);

			// burn
			let imbalance = <Runtime as pallet_elections_phragmen::Config>::Currency::burn(10);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::total_issuance(),
				92
			);
			drop(imbalance);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::total_issuance(),
				102
			);

			// issue
			let imbalance = <Runtime as pallet_elections_phragmen::Config>::Currency::issue(20);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::total_issuance(),
				122
			);
			drop(imbalance);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::total_issuance(),
				102
			);

			// transfer
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::free_balance(&TREASURY_ACCOUNT),
				100
			);
			assert_ok!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::ensure_can_withdraw(
					&TREASURY_ACCOUNT,
					10,
					WithdrawReasons::TRANSFER,
					0
				)
			);
			assert_ok!(<Runtime as pallet_elections_phragmen::Config>::Currency::transfer(
				&TREASURY_ACCOUNT,
				&ALICE,
				11,
				ExistenceRequirement::KeepAlive
			));
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::free_balance(&TREASURY_ACCOUNT),
				89
			);

			// deposit
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::total_issuance(),
				102
			);
			let imbalance = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 11);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::free_balance(&TREASURY_ACCOUNT),
				100
			);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::total_issuance(),
				102
			);
			drop(imbalance);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::free_balance(&TREASURY_ACCOUNT),
				100
			);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::total_issuance(),
				113
			);

			// withdraw
			let imbalance = <Runtime as pallet_elections_phragmen::Config>::Currency::withdraw(
				&TREASURY_ACCOUNT,
				10,
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::KeepAlive,
			);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::free_balance(&TREASURY_ACCOUNT),
				90
			);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::total_issuance(),
				113
			);
			drop(imbalance);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::free_balance(&TREASURY_ACCOUNT),
				90
			);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::total_issuance(),
				103
			);
		});
}

#[test]
fn currency_adapter_burn_must_work() {
	ExtBuilder::default()
		.one_hundred_for_treasury_account()
		.build()
		.execute_with(|| {
			let init_total_issuance = TreasuryCurrencyAdapter::total_issuance();
			let imbalance = TreasuryCurrencyAdapter::burn(10);
			assert_eq!(TreasuryCurrencyAdapter::total_issuance(), init_total_issuance - 10);
			drop(imbalance);
			assert_eq!(TreasuryCurrencyAdapter::total_issuance(), init_total_issuance);
		});
}

#[test]
fn currency_adapter_reserving_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 111);

		assert_eq!(TreasuryCurrencyAdapter::total_balance(&TREASURY_ACCOUNT), 111);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 111);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&TREASURY_ACCOUNT), 0);

		assert_ok!(TreasuryCurrencyAdapter::reserve(&TREASURY_ACCOUNT, 69));

		assert_eq!(TreasuryCurrencyAdapter::total_balance(&TREASURY_ACCOUNT), 111);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 42);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&TREASURY_ACCOUNT), 69);
	});
}

#[test]
fn currency_adapter_balance_transfer_when_reserved_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 111);
		assert_ok!(TreasuryCurrencyAdapter::reserve(&TREASURY_ACCOUNT, 69));
		assert_noop!(
			TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 69, ExistenceRequirement::AllowDeath),
			Error::<Runtime>::BalanceTooLow,
		);
	});
}

#[test]
fn currency_adapter_deducting_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 111);
		assert_ok!(TreasuryCurrencyAdapter::reserve(&TREASURY_ACCOUNT, 69));
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 42);
	});
}

#[test]
fn currency_adapter_refunding_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 42);
		Tokens::set_reserved_balance(DOT, &TREASURY_ACCOUNT, 69);
		TreasuryCurrencyAdapter::unreserve(&TREASURY_ACCOUNT, 69);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 111);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&TREASURY_ACCOUNT), 0);
	});
}

#[test]
fn currency_adapter_slashing_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 111);
		assert_ok!(TreasuryCurrencyAdapter::reserve(&TREASURY_ACCOUNT, 69));
		assert!(TreasuryCurrencyAdapter::slash(&TREASURY_ACCOUNT, 69).1.is_zero());
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 0);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&TREASURY_ACCOUNT), 42);
		assert_eq!(TreasuryCurrencyAdapter::total_issuance(), 42);
	});
}

#[test]
fn currency_adapter_slashing_incomplete_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 42);
		assert_ok!(TreasuryCurrencyAdapter::reserve(&TREASURY_ACCOUNT, 21));
		assert_eq!(TreasuryCurrencyAdapter::slash(&TREASURY_ACCOUNT, 69).1, 27);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 0);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&TREASURY_ACCOUNT), 0);
		assert_eq!(TreasuryCurrencyAdapter::total_issuance(), 0);
	});
}

#[test]
fn currency_adapter_basic_locking_should_work() {
	ExtBuilder::default()
		.one_hundred_for_treasury_account()
		.build()
		.execute_with(|| {
			assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 100);
			TreasuryCurrencyAdapter::set_lock(ID_1, &TREASURY_ACCOUNT, 91, WithdrawReasons::all());
			assert_noop!(
				TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 10, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::LiquidityRestrictions
			);
		});
}

#[test]
fn currency_adapter_partial_locking_should_work() {
	ExtBuilder::default()
		.one_hundred_for_treasury_account()
		.build()
		.execute_with(|| {
			TreasuryCurrencyAdapter::set_lock(ID_1, &TREASURY_ACCOUNT, 5, WithdrawReasons::all());
			assert_ok!(TreasuryCurrencyAdapter::transfer(
				&TREASURY_ACCOUNT,
				&ALICE,
				2,
				ExistenceRequirement::AllowDeath
			));
		});
}

#[test]
fn currency_adapter_lock_removal_should_work() {
	ExtBuilder::default()
		.one_hundred_for_treasury_account()
		.build()
		.execute_with(|| {
			TreasuryCurrencyAdapter::set_lock(ID_1, &TREASURY_ACCOUNT, u64::max_value(), WithdrawReasons::all());
			TreasuryCurrencyAdapter::remove_lock(ID_1, &TREASURY_ACCOUNT);
			assert_ok!(TreasuryCurrencyAdapter::transfer(
				&TREASURY_ACCOUNT,
				&ALICE,
				2,
				ExistenceRequirement::AllowDeath
			));
		});
}

#[test]
fn currency_adapter_lock_replacement_should_work() {
	ExtBuilder::default()
		.one_hundred_for_treasury_account()
		.build()
		.execute_with(|| {
			TreasuryCurrencyAdapter::set_lock(ID_1, &TREASURY_ACCOUNT, u64::max_value(), WithdrawReasons::all());
			TreasuryCurrencyAdapter::set_lock(ID_1, &TREASURY_ACCOUNT, 5, WithdrawReasons::all());
			assert_ok!(TreasuryCurrencyAdapter::transfer(
				&TREASURY_ACCOUNT,
				&ALICE,
				2,
				ExistenceRequirement::AllowDeath
			));
		});
}

#[test]
fn currency_adapter_double_locking_should_work() {
	ExtBuilder::default()
		.one_hundred_for_treasury_account()
		.build()
		.execute_with(|| {
			TreasuryCurrencyAdapter::set_lock(ID_1, &TREASURY_ACCOUNT, 5, WithdrawReasons::empty());
			TreasuryCurrencyAdapter::set_lock(ID_2, &TREASURY_ACCOUNT, 5, WithdrawReasons::all());
			assert_ok!(TreasuryCurrencyAdapter::transfer(
				&TREASURY_ACCOUNT,
				&ALICE,
				2,
				ExistenceRequirement::AllowDeath
			));
		});
}

#[test]
fn currency_adapter_combination_locking_should_work() {
	ExtBuilder::default()
		.one_hundred_for_treasury_account()
		.build()
		.execute_with(|| {
			// withdrawReasons not work
			TreasuryCurrencyAdapter::set_lock(ID_1, &TREASURY_ACCOUNT, u64::max_value(), WithdrawReasons::empty());
			TreasuryCurrencyAdapter::set_lock(ID_2, &TREASURY_ACCOUNT, 0, WithdrawReasons::all());
			assert_noop!(
				TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 2, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::LiquidityRestrictions
			);
		});
}

#[test]
fn currency_adapter_lock_value_extension_should_work() {
	ExtBuilder::default()
		.one_hundred_for_treasury_account()
		.build()
		.execute_with(|| {
			TreasuryCurrencyAdapter::set_lock(ID_1, &TREASURY_ACCOUNT, 100, WithdrawReasons::all());
			assert_noop!(
				TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 6, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::LiquidityRestrictions
			);
			TreasuryCurrencyAdapter::extend_lock(ID_1, &TREASURY_ACCOUNT, 2, WithdrawReasons::all());
			assert_noop!(
				TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 6, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::LiquidityRestrictions
			);
			TreasuryCurrencyAdapter::extend_lock(ID_1, &TREASURY_ACCOUNT, 8, WithdrawReasons::all());
			assert_noop!(
				TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 3, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::LiquidityRestrictions
			);
		});
}

#[test]
fn currency_adapter_lock_block_number_extension_should_work() {
	ExtBuilder::default()
		.one_hundred_for_treasury_account()
		.build()
		.execute_with(|| {
			TreasuryCurrencyAdapter::set_lock(ID_1, &TREASURY_ACCOUNT, 200, WithdrawReasons::all());
			assert_noop!(
				TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 6, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::LiquidityRestrictions
			);
			TreasuryCurrencyAdapter::extend_lock(ID_1, &TREASURY_ACCOUNT, 90, WithdrawReasons::all());
			assert_noop!(
				TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 6, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::LiquidityRestrictions
			);
			System::set_block_number(2);
			TreasuryCurrencyAdapter::extend_lock(ID_1, &TREASURY_ACCOUNT, 90, WithdrawReasons::all());
			assert_noop!(
				TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 3, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::LiquidityRestrictions
			);
		});
}

#[test]
fn currency_adapter_lock_reasons_extension_should_work() {
	ExtBuilder::default()
		.one_hundred_for_treasury_account()
		.build()
		.execute_with(|| {
			TreasuryCurrencyAdapter::set_lock(ID_1, &TREASURY_ACCOUNT, 90, WithdrawReasons::TRANSFER);
			assert_noop!(
				TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 11, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::LiquidityRestrictions
			);
			TreasuryCurrencyAdapter::extend_lock(ID_1, &TREASURY_ACCOUNT, 90, WithdrawReasons::empty());
			assert_noop!(
				TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 11, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::LiquidityRestrictions
			);
			TreasuryCurrencyAdapter::extend_lock(ID_1, &TREASURY_ACCOUNT, 90, WithdrawReasons::RESERVE);
			assert_noop!(
				TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 11, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::LiquidityRestrictions
			);
		});
}

#[test]
fn currency_adapter_reward_should_work() {
	ExtBuilder::default()
		.one_hundred_for_treasury_account()
		.build()
		.execute_with(|| {
			assert_eq!(TreasuryCurrencyAdapter::total_issuance(), 102);
			assert_eq!(TreasuryCurrencyAdapter::total_balance(&TREASURY_ACCOUNT), 100);
			assert_eq!(TreasuryCurrencyAdapter::total_balance(&Treasury::account_id()), 2);
			assert_ok!(TreasuryCurrencyAdapter::deposit_into_existing(&TREASURY_ACCOUNT, 10).map(drop));
			assert_eq!(TreasuryCurrencyAdapter::total_balance(&TREASURY_ACCOUNT), 110);
			assert_eq!(TreasuryCurrencyAdapter::total_issuance(), 112);
		});
}

#[test]
fn currency_adapter_slashing_reserved_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 111);
		assert_ok!(TreasuryCurrencyAdapter::reserve(&TREASURY_ACCOUNT, 111));
		assert_eq!(TreasuryCurrencyAdapter::slash_reserved(&TREASURY_ACCOUNT, 42).1, 0);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&TREASURY_ACCOUNT), 69);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 0);
		assert_eq!(TreasuryCurrencyAdapter::total_issuance(), 69);
	});
}

#[test]
fn currency_adapter_slashing_incomplete_reserved_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 111);
		assert_ok!(TreasuryCurrencyAdapter::reserve(&TREASURY_ACCOUNT, 42));
		assert_eq!(TreasuryCurrencyAdapter::slash_reserved(&TREASURY_ACCOUNT, 69).1, 27);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 69);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&TREASURY_ACCOUNT), 0);
		assert_eq!(TreasuryCurrencyAdapter::total_issuance(), 69);
	});
}

#[test]
fn currency_adapter_repatriating_reserved_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 110);
		let _ = TreasuryCurrencyAdapter::deposit_creating(&ALICE, 2);
		assert_ok!(TreasuryCurrencyAdapter::reserve(&TREASURY_ACCOUNT, 110));
		assert_ok!(
			TreasuryCurrencyAdapter::repatriate_reserved(&TREASURY_ACCOUNT, &ALICE, 41, Status::Free),
			0
		);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&TREASURY_ACCOUNT), 69);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 0);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&ALICE), 0);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&ALICE), 43);
	});
}

#[test]
fn currency_adapter_transferring_reserved_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 110);
		let _ = TreasuryCurrencyAdapter::deposit_creating(&ALICE, 2);
		assert_ok!(TreasuryCurrencyAdapter::reserve(&TREASURY_ACCOUNT, 110));
		assert_ok!(
			TreasuryCurrencyAdapter::repatriate_reserved(&TREASURY_ACCOUNT, &ALICE, 41, Status::Reserved),
			0
		);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&TREASURY_ACCOUNT), 69);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 0);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&ALICE), 41);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&ALICE), 2);
	});
}

#[test]
fn currency_adapter_transferring_reserved_balance_to_nonexistent_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 111);
		assert_ok!(TreasuryCurrencyAdapter::reserve(&TREASURY_ACCOUNT, 111));
		assert_ok!(TreasuryCurrencyAdapter::repatriate_reserved(
			&TREASURY_ACCOUNT,
			&ALICE,
			42,
			Status::Free
		));
	});
}

#[test]
fn currency_adapter_transferring_incomplete_reserved_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 110);
		let _ = TreasuryCurrencyAdapter::deposit_creating(&ALICE, 2);
		assert_ok!(TreasuryCurrencyAdapter::reserve(&TREASURY_ACCOUNT, 41));
		assert_ok!(
			TreasuryCurrencyAdapter::repatriate_reserved(&TREASURY_ACCOUNT, &ALICE, 69, Status::Free),
			28
		);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&TREASURY_ACCOUNT), 0);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 69);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&ALICE), 0);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&ALICE), 43);
	});
}

#[test]
fn currency_adapter_transferring_too_high_value_should_not_panic() {
	ExtBuilder::default().build().execute_with(|| {
		TreasuryCurrencyAdapter::make_free_balance_be(&TREASURY_ACCOUNT, u64::max_value());
		TreasuryCurrencyAdapter::make_free_balance_be(&ALICE, 2);

		assert_noop!(
			TreasuryCurrencyAdapter::transfer(
				&TREASURY_ACCOUNT,
				&ALICE,
				u64::max_value(),
				ExistenceRequirement::AllowDeath
			),
			ArithmeticError::Overflow,
		);

		assert_eq!(
			TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT),
			u64::max_value()
		);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&ALICE), 2);
	});
}

#[test]
fn exceeding_max_locks_should_fail() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Tokens::set_lock(ID_1, DOT, &ALICE, 10));
			assert_eq!(Tokens::locks(ALICE, DOT).len(), 1);
			assert_ok!(Tokens::set_lock(ID_2, DOT, &ALICE, 10));
			assert_eq!(Tokens::locks(ALICE, DOT).len(), 2);
			assert_noop!(
				Tokens::set_lock(ID_3, DOT, &ALICE, 10),
				Error::<Runtime>::MaxLocksExceeded
			);
			assert_eq!(Tokens::locks(ALICE, DOT).len(), 2);
		});
}

#[test]
fn fungibles_inspect_trait_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_eq!(<Tokens as fungibles::Inspect<_>>::total_issuance(DOT), 200);
			assert_eq!(<Tokens as fungibles::Inspect<_>>::minimum_balance(DOT), 2);
			assert_eq!(<Tokens as fungibles::Inspect<_>>::balance(DOT, &ALICE), 100);
			assert_eq!(
				<Tokens as fungibles::Inspect<_>>::reducible_balance(DOT, &ALICE, true),
				98
			);
			assert_ok!(<Tokens as fungibles::Inspect<_>>::can_deposit(DOT, &ALICE, 1).into_result());
			assert_ok!(<Tokens as fungibles::Inspect<_>>::can_withdraw(DOT, &ALICE, 1).into_result());
		});
}

#[test]
fn fungibles_mutate_trait_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(<Tokens as fungibles::Mutate<_>>::mint_into(DOT, &ALICE, 10));
			assert_eq!(<Tokens as fungibles::Mutate<_>>::burn_from(DOT, &ALICE, 8), Ok(8));
		});
}

#[test]
fn fungibles_transfer_trait_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_eq!(<Tokens as fungibles::Inspect<_>>::balance(DOT, &ALICE), 100);
			assert_eq!(<Tokens as fungibles::Inspect<_>>::balance(DOT, &BOB), 100);
			assert_ok!(<Tokens as fungibles::Transfer<_>>::transfer(
				DOT, &ALICE, &BOB, 10, true
			));
			assert_eq!(<Tokens as fungibles::Inspect<_>>::balance(DOT, &ALICE), 90);
			assert_eq!(<Tokens as fungibles::Inspect<_>>::balance(DOT, &BOB), 110);
		});
}

#[test]
fn fungibles_unbalanced_trait_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_eq!(<Tokens as fungibles::Inspect<_>>::balance(DOT, &ALICE), 100);
			assert_ok!(<Tokens as fungibles::Unbalanced<_>>::set_balance(DOT, &ALICE, 10));
			assert_eq!(<Tokens as fungibles::Inspect<_>>::balance(DOT, &ALICE), 10);

			assert_eq!(<Tokens as fungibles::Inspect<_>>::total_issuance(DOT), 200);
			<Tokens as fungibles::Unbalanced<_>>::set_total_issuance(DOT, 10);
			assert_eq!(<Tokens as fungibles::Inspect<_>>::total_issuance(DOT), 10);
		});
}

#[test]
fn fungibles_inspect_hold_trait_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_eq!(<Tokens as fungibles::InspectHold<_>>::balance_on_hold(DOT, &ALICE), 0);
			assert_eq!(<Tokens as fungibles::InspectHold<_>>::can_hold(DOT, &ALICE, 50), true);
			assert_eq!(<Tokens as fungibles::InspectHold<_>>::can_hold(DOT, &ALICE, 100), false);
		});
}

#[test]
fn fungibles_mutate_hold_trait_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_noop!(
				<Tokens as fungibles::MutateHold<_>>::hold(DOT, &ALICE, 200),
				Error::<Runtime>::BalanceTooLow
			);
			assert_ok!(<Tokens as fungibles::MutateHold<_>>::hold(DOT, &ALICE, 100));
			assert_eq!(
				<Tokens as fungibles::MutateHold<_>>::release(DOT, &ALICE, 50, true),
				Ok(50)
			);
			assert_eq!(
				<Tokens as fungibles::MutateHold<_>>::transfer_held(DOT, &ALICE, &BOB, 100, true, true),
				Ok(50)
			);
		});
}
