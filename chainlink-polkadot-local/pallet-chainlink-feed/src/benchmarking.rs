use super::*;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use sp_runtime::traits::{AccountIdConversion, One, Zero};
use sp_std::{fmt::Debug, vec, vec::Vec};

use crate::Pallet as ChainlinkFeed;

const SEED: u32 = 0;

/// Either use `assert_ok!` or regular `assert!` depending on std/no_std
/// environment.
fn assert_is_ok<T: Debug, E: Debug>(r: Result<T, E>) {
	#[cfg(feature = "std")]
	frame_support::assert_ok!(r);
	#[cfg(not(feature = "std"))]
	assert!(r.is_ok());
}

fn whitelisted_account<T: Config>(name: &'static str, counter: u32) -> T::AccountId {
	let acc = account(name, counter, SEED);
	whitelist_acc::<T>(&acc);
	acc
}

fn whitelist_acc<T: Config>(acc: &T::AccountId) {
	frame_benchmarking::benchmarking::add_to_whitelist(
		frame_system::Account::<T>::hashed_key_for(acc).into(),
	);
}

benchmarks! {
	// _ {}

	create_feed {
		let o in 1 .. T::OracleCountLimit::get();

		let caller: T::AccountId = whitelisted_caller();
		let pallet_admin: T::AccountId = ChainlinkFeed::<T>::pallet_admin();
		assert_is_ok(ChainlinkFeed::<T>::set_feed_creator(RawOrigin::Signed(pallet_admin.clone()).into(), caller.clone()));
		let admin: T::AccountId = account("oracle_admin", 0, SEED);
		let oracles: Vec<(T::AccountId, T::AccountId)> = (0..o).map(|n| (account("oracle", n, SEED), admin.clone())).collect();
		let description = vec![1; T::StringLimit::get() as usize];
	}: _(
			RawOrigin::Signed(caller.clone()),
			600u32.into(),
			Zero::zero(),
			(1u8.into(), 100u8.into()),
			1u8.into(),
			5u8.into(),
			description,
			Zero::zero(),
			oracles
		)
	verify {
		let feed: T::FeedId = Zero::zero();
		assert_eq!(ChainlinkFeed::<T>::feed_config(feed).expect("feed should be there").oracle_count, o);
	}

	transfer_ownership {
		let caller: T::AccountId = whitelisted_caller();
		let pallet_admin: T::AccountId = ChainlinkFeed::<T>::pallet_admin();
		assert_is_ok(ChainlinkFeed::<T>::set_feed_creator(RawOrigin::Signed(pallet_admin.clone()).into(), caller.clone()));
		let oracle: T::AccountId = account("oracle", 0, SEED);
		let admin: T::AccountId = account("oracle_admin", 0, SEED);
		let description = vec![1; T::StringLimit::get() as usize];
		assert_is_ok(ChainlinkFeed::<T>::create_feed(
			RawOrigin::Signed(caller.clone()).into(),
			600u32.into(),
			Zero::zero(),
			(1u8.into(), 100u8.into()),
			1u8.into(),
			5u8.into(),
			description,
			Zero::zero(),
			vec![(oracle, admin)],
		));
		let feed = Zero::zero();
		let new_owner: T::AccountId = account("new_owner", 0, SEED);
	}: _(RawOrigin::Signed(caller.clone()), feed, new_owner.clone())
	verify {
		assert_eq!(ChainlinkFeed::<T>::feed_config(feed).expect("feed should be there").pending_owner, Some(new_owner));
	}

	accept_ownership {
		let caller: T::AccountId = whitelisted_caller();
		let pallet_admin: T::AccountId = ChainlinkFeed::<T>::pallet_admin();
		assert_is_ok(ChainlinkFeed::<T>::set_feed_creator(RawOrigin::Signed(pallet_admin.clone()).into(), caller.clone()));
		let oracle: T::AccountId = account("oracle", 0, SEED);
		let admin: T::AccountId = account("oracle_admin", 0, SEED);
		let description = vec![1; T::StringLimit::get() as usize];
		assert_is_ok(ChainlinkFeed::<T>::create_feed(
			RawOrigin::Signed(caller.clone()).into(),
			600u32.into(),
			Zero::zero(),
			(1u8.into(), 100u8.into()),
			1u8.into(),
			5u8.into(),
			description,
			Zero::zero(),
			vec![(oracle, admin)],
		));
		let feed = Zero::zero();
		let new_owner: T::AccountId = account("new_owner", 0, SEED);
		assert_is_ok(ChainlinkFeed::<T>::transfer_ownership(RawOrigin::Signed(caller.clone()).into(), feed, new_owner.clone()));
	}: _(RawOrigin::Signed(new_owner.clone()), feed)
	verify {
		assert_eq!(ChainlinkFeed::<T>::feed_config(feed).expect("feed should be there").owner, new_owner);
	}

	// The submit call opening a round is more expensive than a regular submission because of
	// the round init code as well as the closing of previous rounds.
	// It is most expensive in case it also directly closes the round.
	submit_opening_round_answers {
		let o = 3;
		let caller: T::AccountId = whitelisted_caller();
		let pallet_admin: T::AccountId = ChainlinkFeed::<T>::pallet_admin();
		assert_is_ok(ChainlinkFeed::<T>::set_feed_creator(RawOrigin::Signed(pallet_admin.clone()).into(), caller.clone()));
		let admin: T::AccountId = account("oracle_admin", 0, SEED);
		let oracle = whitelisted_account::<T>("oracle", 0);
		let other_oracle: T::AccountId = account("oracle", 1, SEED);
		let oracles: Vec<(T::AccountId, T::AccountId)> = vec![(oracle.clone(), admin.clone()), (other_oracle.clone(), admin.clone())];
		let description = vec![1; T::StringLimit::get() as usize];
		assert_is_ok(ChainlinkFeed::<T>::create_feed(
			RawOrigin::Signed(caller.clone()).into(),
			600u32.into(),
			Zero::zero(),
			(1u8.into(), 100u8.into()),
			1,
			5u8.into(),
			description,
			Zero::zero(),
			oracles.clone(),
		));
		let feed: T::FeedId = Zero::zero();
		let prev_round: RoundId = 1;
		let answer: T::Value = 5u8.into();
		// create the previous round that will be closed
		assert_is_ok(ChainlinkFeed::<T>::submit(
			RawOrigin::Signed(other_oracle.clone()).into(),
			feed,
			prev_round,
			answer
		));
		let round: RoundId = 2;
		assert_eq!(ChainlinkFeed::<T>::round(feed, round), None);
		// make sure we hit the `Debt` storage item
		let fund_account = T::PalletId::get().into_account();
		T::Currency::make_free_balance_be(&fund_account, Zero::zero());
	}: submit(
			RawOrigin::Signed(oracle.clone()),
			feed,
			round,
			answer
		)
	verify {
		let f = Feed::<T>::read_only_from(feed).unwrap();
		// previous round should be cleared
		assert_eq!(f.details(prev_round), None);
		let expected_round = Round {
			started_at: One::one(),
			answer: Some(answer),
			updated_at: Some(One::one()),
			answered_in_round: Some(2)
		};
		assert_eq!(ChainlinkFeed::<T>::round(feed, round), Some(expected_round));
	}

	// The closing answer is expensive because it induces the largest median calculation and
	// includes the bookkeeping for closing the round.
	// It is most expensive when there are `OracleCountLimit` answers.
	submit_closing_answer {
		let o in 2 .. T::OracleCountLimit::get();

		let caller: T::AccountId = whitelisted_caller();
		let pallet_admin: T::AccountId = ChainlinkFeed::<T>::pallet_admin();
		assert_is_ok(ChainlinkFeed::<T>::set_feed_creator(RawOrigin::Signed(pallet_admin.clone()).into(), caller.clone()));
		let admin: T::AccountId = account("oracle_admin", 0, SEED);
		let oracles: Vec<(T::AccountId, T::AccountId)> = (0..o).map(|n| (account("oracle", n, SEED), admin.clone())).collect();
		let description = vec![1; T::StringLimit::get() as usize];
		assert_is_ok(ChainlinkFeed::<T>::create_feed(
			RawOrigin::Signed(caller.clone()).into(),
			600u32.into(),
			Zero::zero(),
			(1u8.into(), 100u8.into()),
			oracles.len() as u32,
			5u8.into(),
			description,
			Zero::zero(),
			oracles.clone(),
		));
		let feed: T::FeedId = Zero::zero();
		let prev_round: RoundId = 1;
		let answer: T::Value = 42u8.into();
		let oracle = oracles.first().map(|(o, _a)| o.clone()).expect("first oracle should be there");
		let other_oracle = oracles.iter().nth(1).map(|(o, _a)| o.clone()).expect("there should be a second oracle");
		// create the previous round that will be closed
		for (o, _a) in oracles.iter() {
			assert_is_ok(ChainlinkFeed::<T>::submit(RawOrigin::Signed(o.clone()).into(), feed, prev_round, answer));
		}
		// advance the block number so we can supersede the prev round
		frame_system::Pallet::<T>::set_block_number(1u8.into());
		let round: RoundId = 2;
		for (o, _a) in oracles.iter().skip(1) {
			assert_is_ok(ChainlinkFeed::<T>::submit(RawOrigin::Signed(o.clone()).into(), feed, round, answer));
		}
		assert_eq!(ChainlinkFeed::<T>::round(feed, round), Some(Round::new(One::one())));
		// make sure we hit the `Debt` storage item
		let fund_account = T::PalletId::get().into_account();
		T::Currency::make_free_balance_be(&fund_account, Zero::zero());
	}: submit(
			RawOrigin::Signed(oracle.clone()),
			feed,
			round,
			answer
		)
	verify {
		let expected_round = Round {
			started_at: One::one(),
			answer: Some(answer),
			updated_at: Some(One::one()),
			answered_in_round: Some(2)
		};
		assert_eq!(ChainlinkFeed::<T>::round(feed, round), Some(expected_round));
	}

	change_oracles {
		let d in 1 .. T::OracleCountLimit::get();
		let n in 1 .. T::OracleCountLimit::get();

		let caller: T::AccountId = whitelisted_caller();
		let pallet_admin: T::AccountId = ChainlinkFeed::<T>::pallet_admin();
		assert_is_ok(ChainlinkFeed::<T>::set_feed_creator(RawOrigin::Signed(pallet_admin.clone()).into(), caller.clone()));
		let admin: T::AccountId = account("oracle_admin", 0, SEED);
		let oracles: Vec<(T::AccountId, T::AccountId)> = (0..d).map(|n| (account("oracle", n, SEED), admin.clone())).collect();
		let oracles_after: Vec<(T::AccountId, T::AccountId)> = (0..n).map(|n| (account("new_oracle", n, SEED), admin.clone())).collect();
		let description = vec![1; T::StringLimit::get() as usize];
		assert_is_ok(ChainlinkFeed::<T>::create_feed(
			RawOrigin::Signed(caller.clone()).into(),
			600u32.into(),
			Zero::zero(),
			(1u8.into(), 100u8.into()),
			1u8.into(),
			5u8.into(),
			description,
			Zero::zero(),
			oracles.clone(),
		));
		let oracles_before = oracles.into_iter().map(|(o, _a)| o).collect();
		let feed: T::FeedId = Zero::zero();
	}: _(
			RawOrigin::Signed(caller.clone()),
			feed,
			oracles_before,
			oracles_after
		)
	verify {
		assert_eq!(ChainlinkFeed::<T>::feed_config(feed).expect("feed should be there").oracle_count, n);
	}

	update_future_rounds {
		let o = 2;
		let caller: T::AccountId = whitelisted_caller();
		let pallet_admin: T::AccountId = ChainlinkFeed::<T>::pallet_admin();
		assert_is_ok(ChainlinkFeed::<T>::set_feed_creator(RawOrigin::Signed(pallet_admin.clone()).into(), caller.clone()));
		let admin: T::AccountId = account("oracle_admin", 0, SEED);
		let oracles: Vec<(T::AccountId, T::AccountId)> = (0..o).map(|n| (account("oracle", n, SEED), admin.clone())).collect();
		let description = vec![1; T::StringLimit::get() as usize];
		assert_is_ok(ChainlinkFeed::<T>::create_feed(
			RawOrigin::Signed(caller.clone()).into(),
			600u32.into(),
			Zero::zero(),
			(1u8.into(), 100u8.into()),
			1u8.into(),
			5u8.into(),
			description,
			Zero::zero(),
			oracles.clone(),
		));
		let payment: BalanceOf<T> = 42u32.into();
		let timeout: T::BlockNumber = 3u8.into();
		let feed: T::FeedId = Zero::zero();
	}: _(
			RawOrigin::Signed(caller.clone()),
			feed,
			payment,
			(1, oracles.len() as u32),
			1u8.into(),
			timeout
		)
	verify {
		let config = ChainlinkFeed::<T>::feed_config(feed).expect("feed should be there");
		assert_eq!(config.payment, payment);
		assert_eq!(config.timeout, timeout);
	}
	set_requester {
		let caller: T::AccountId = whitelisted_caller();
		let pallet_admin: T::AccountId = ChainlinkFeed::<T>::pallet_admin();
		assert_is_ok(ChainlinkFeed::<T>::set_feed_creator(RawOrigin::Signed(pallet_admin.clone()).into(), caller.clone()));
		let oracle: T::AccountId = account("oracle", 0, SEED);
		let admin: T::AccountId = account("oracle_admin", 0, SEED);
		let description = vec![1; T::StringLimit::get() as usize];
		assert_is_ok(ChainlinkFeed::<T>::create_feed(
			RawOrigin::Signed(caller.clone()).into(),
			600u32.into(),
			Zero::zero(),
			(1u8.into(), 100u8.into()),
			1u8.into(),
			5u8.into(),
			description,
			Zero::zero(),
			vec![(oracle, admin)],
		));
		let feed = Zero::zero();
		let requester: T::AccountId = account("requester", 0, SEED);
		let delay: RoundId = 3;
	}: _(RawOrigin::Signed(caller.clone()), feed, requester.clone(), delay)
	verify {
		assert_eq!(ChainlinkFeed::<T>::requester(feed, requester).expect("feed should be there").delay, delay);
	}

	remove_requester {
		let caller: T::AccountId = whitelisted_caller();
		let pallet_admin: T::AccountId = ChainlinkFeed::<T>::pallet_admin();
		assert_is_ok(ChainlinkFeed::<T>::set_feed_creator(RawOrigin::Signed(pallet_admin.clone()).into(), caller.clone()));
		let oracle: T::AccountId = account("oracle", 0, SEED);
		let admin: T::AccountId = account("oracle_admin", 0, SEED);
		let description = vec![1; T::StringLimit::get() as usize];
		assert_is_ok(ChainlinkFeed::<T>::create_feed(
			RawOrigin::Signed(caller.clone()).into(),
			600u32.into(),
			Zero::zero(),
			(1u8.into(), 100u8.into()),
			1u8.into(),
			5u8.into(),
			description,
			Zero::zero(),
			vec![(oracle, admin)],
		));
		let feed = Zero::zero();
		let requester: T::AccountId = account("requester", 0, SEED);
		let delay: RoundId = 3;
		assert_is_ok(ChainlinkFeed::<T>::set_requester(RawOrigin::Signed(caller.clone()).into(), feed, requester.clone(), delay));
	}: _(RawOrigin::Signed(caller.clone()), feed, requester.clone())
	verify {
		assert_eq!(ChainlinkFeed::<T>::requester(feed, requester), None);
	}

	request_new_round {
		let o = 3;
		let caller: T::AccountId = whitelisted_caller();
		let pallet_admin: T::AccountId = ChainlinkFeed::<T>::pallet_admin();
		assert_is_ok(ChainlinkFeed::<T>::set_feed_creator(RawOrigin::Signed(pallet_admin.clone()).into(), caller.clone()));
		let admin: T::AccountId = account("oracle_admin", 0, SEED);
		let oracles: Vec<(T::AccountId, T::AccountId)> = (0..o).map(|n| (account("oracle", n, SEED), admin.clone())).collect();
		let description = vec![1; T::StringLimit::get() as usize];
		assert_is_ok(ChainlinkFeed::<T>::create_feed(
			RawOrigin::Signed(caller.clone()).into(),
			600u32.into(),
			Zero::zero(),
			(1u8.into(), 100u8.into()),
			1,
			5u8.into(),
			description,
			Zero::zero(),
			oracles.clone(),
		));
		let feed: T::FeedId = Zero::zero();
		let round: RoundId = One::one();
		let answer: T::Value = 5u8.into();
		let oracle = oracles.first().map(|(o, _a)| o.clone()).expect("first oracle should be there");
		assert_is_ok(ChainlinkFeed::<T>::submit(
			RawOrigin::Signed(oracle.clone()).into(),
			feed,
			round,
			answer
		));
		let config = ChainlinkFeed::<T>::feed_config(feed).expect("config should be there");
		assert_eq!(config.reporting_round, round);
		let requester: T::AccountId = account("requester", 0, SEED);
		let delay: RoundId = 3;
		assert_is_ok(ChainlinkFeed::<T>::set_requester(RawOrigin::Signed(caller.clone()).into(), feed, requester.clone(), delay));
	}: _(
			RawOrigin::Signed(requester.clone()),
			feed
		)
	verify {
		let config = ChainlinkFeed::<T>::feed_config(feed).expect("config should be there");
		assert_eq!(config.reporting_round, 2);
	}

	withdraw_payment {
		let o = 3;
		let caller: T::AccountId = whitelisted_caller();
		let pallet_admin: T::AccountId = ChainlinkFeed::<T>::pallet_admin();
		assert_is_ok(ChainlinkFeed::<T>::set_feed_creator(RawOrigin::Signed(pallet_admin.clone()).into(), caller.clone()));
		let admin: T::AccountId = account("oracle_admin", 0, SEED);
		let oracles: Vec<(T::AccountId, T::AccountId)> = (0..o).map(|n| (account("oracle", n, SEED), admin.clone())).collect();
		let payment: BalanceOf<T> = 600u32.into(); // ExistentialDeposit is 500
		let description = vec![1; T::StringLimit::get() as usize];
		assert_is_ok(ChainlinkFeed::<T>::create_feed(
			RawOrigin::Signed(caller.clone()).into(),
			payment,
			Zero::zero(),
			(1u8.into(), 100u8.into()),
			1,
			5u8.into(),
			description,
			Zero::zero(),
			oracles.clone(),
		));
		let feed: T::FeedId = Zero::zero();
		let round: RoundId = One::one();
		let answer: T::Value = 5u8.into();
		let oracle = oracles.first().map(|(o, _a)| o.clone()).expect("first oracle should be there");
		assert_is_ok(ChainlinkFeed::<T>::submit(
			RawOrigin::Signed(oracle.clone()).into(),
			feed,
			round,
			answer
		));
		let recipient: T::AccountId = account("recipient", 0, SEED);
		let fund_account = T::PalletId::get().into_account();
		T::Currency::make_free_balance_be(&fund_account, payment + payment);
	}: _(
		RawOrigin::Signed(admin.clone()),
		oracle.clone(),
		recipient.clone(),
		payment
	)
	verify {
		assert_eq!(T::Currency::free_balance(&recipient), payment);
	}

	transfer_admin {
		let oracle: T::AccountId = account("oracle", 0, SEED);
		let admin: T::AccountId = account("oracle_admin", 0, SEED);
		Oracles::<T>::insert(&oracle, OracleMeta {
			withdrawable: Zero::zero(),
			admin: admin.clone(),
			pending_admin: None,
		});
		let new_admin: T::AccountId = account("new_admin", 0, SEED);
	}: _(
		RawOrigin::Signed(admin.clone()),
		oracle.clone(),
		new_admin.clone()
	)
	verify {
		let expected_meta = OracleMeta {
			withdrawable: Zero::zero(),
			admin: admin.clone(),
			pending_admin: Some(new_admin.clone()),
		};
		let meta = ChainlinkFeed::<T>::oracle(&oracle);
		assert_eq!(meta, Some(expected_meta));
	}

	accept_admin {
		let oracle: T::AccountId = account("oracle", 0, SEED);
		let admin: T::AccountId = account("oracle_admin", 0, SEED);
		Oracles::<T>::insert(&oracle, OracleMeta {
			withdrawable: Zero::zero(),
			admin: admin.clone(),
			pending_admin: None,
		});
		let new_admin: T::AccountId = whitelisted_account::<T>("new_admin", 0);
		assert_is_ok(ChainlinkFeed::<T>::transfer_admin(
			RawOrigin::Signed(admin.clone()).into(),
			oracle.clone(),
			new_admin.clone()
		));
	}: _(
		RawOrigin::Signed(new_admin.clone()),
		oracle.clone()
	)
	verify {
		let expected_meta = OracleMeta {
			withdrawable: Zero::zero(),
			admin: new_admin.clone(),
			pending_admin: None,
		};
		let meta = ChainlinkFeed::<T>::oracle(&oracle);
		assert_eq!(meta, Some(expected_meta));
	}

	withdraw_funds {
		let pallet_admin: T::AccountId = ChainlinkFeed::<T>::pallet_admin();
		whitelist_acc::<T>(&pallet_admin);
		let payment: BalanceOf<T> = 600u32.into(); // ExistentialDeposit is 500
		let recipient: T::AccountId = account("recipient", 0, SEED);
		let fund_account = T::PalletId::get().into_account();
		let multiplier = 1001u32.into();
		T::Currency::make_free_balance_be(&fund_account, payment * multiplier);
	}: _(
		RawOrigin::Signed(pallet_admin.clone()),
		recipient.clone(),
		payment
	)
	verify {
		assert_eq!(T::Currency::free_balance(&recipient), payment);
	}

	reduce_debt {
		let caller: T::AccountId = whitelisted_caller();
		let pallet_admin: T::AccountId = ChainlinkFeed::<T>::pallet_admin();
		assert_is_ok(ChainlinkFeed::<T>::set_feed_creator(RawOrigin::Signed(pallet_admin.clone()).into(), caller.clone()));
		let oracle: T::AccountId = account("oracle", 0, SEED);
		let admin: T::AccountId = account("oracle_admin", 0, SEED);
		let payment = 600u32.into();
		let description = vec![1; T::StringLimit::get() as usize];
		assert_is_ok(ChainlinkFeed::<T>::create_feed(
			RawOrigin::Signed(caller.clone()).into(),
			payment,
			Zero::zero(),
			(1u8.into(), 100u8.into()),
			1u8.into(),
			5u8.into(),
			description,
			Zero::zero(),
			vec![(oracle.clone(), admin)],
		));
		let feed = Zero::zero();
		let answer: T::Value = 42u8.into();
		let rounds: RoundId = 4;
		let fund_account = T::PalletId::get().into_account();
		T::Currency::make_free_balance_be(&fund_account, Zero::zero());
		for round in 1..(rounds + 1) {
			assert_is_ok(ChainlinkFeed::<T>::submit(RawOrigin::Signed(oracle.clone()).into(), feed, round, answer));
		}
		let rounds: BalanceOf<T> = rounds.into();
		let debt: BalanceOf<T> = rounds * payment;
		assert_eq!(Debt::<T>::get(), debt);
		T::Currency::make_free_balance_be(&fund_account, payment + payment);
	}: _(RawOrigin::Signed(caller.clone()), payment)
	verify {
		assert_eq!(T::Currency::free_balance(&fund_account), payment);
		assert_eq!(Debt::<T>::get(), debt - payment);
	}

	transfer_pallet_admin {
		let pallet_admin: T::AccountId = ChainlinkFeed::<T>::pallet_admin();
		whitelist_acc::<T>(&pallet_admin);
		let new_admin: T::AccountId = account("new_pallet_admin", 0, SEED);
	}: _(
		RawOrigin::Signed(pallet_admin.clone()),
		new_admin.clone()
	)
	verify {
		assert_eq!(PendingPalletAdmin::<T>::get(), Some(new_admin));
	}

	accept_pallet_admin {
		let pallet_admin: T::AccountId = ChainlinkFeed::<T>::pallet_admin();
		let new_admin: T::AccountId = whitelisted_account::<T>("new_pallet_admin", 0);
		assert_is_ok(ChainlinkFeed::<T>::transfer_pallet_admin(
			RawOrigin::Signed(pallet_admin.clone()).into(),
			new_admin.clone()
		));
	}: _(RawOrigin::Signed(new_admin.clone()))
	verify {
		assert_eq!(ChainlinkFeed::<T>::pallet_admin(), new_admin);
		assert_eq!(PendingPalletAdmin::<T>::get(), None);
	}

	set_feed_creator {
		let pallet_admin: T::AccountId = ChainlinkFeed::<T>::pallet_admin();
		whitelist_acc::<T>(&pallet_admin);
		let new_creator: T::AccountId = account("new_creator", 0, SEED);
	}: _(
		RawOrigin::Signed(pallet_admin.clone()),
		new_creator.clone()
	)
	verify {
		assert!(FeedCreators::<T>::contains_key(&new_creator));
	}

	remove_feed_creator {
		let pallet_admin: T::AccountId = ChainlinkFeed::<T>::pallet_admin();
		whitelist_acc::<T>(&pallet_admin);
		let creator: T::AccountId = account("creator", 0, SEED);
		assert_is_ok(ChainlinkFeed::<T>::set_feed_creator(
			RawOrigin::Signed(pallet_admin.clone()).into(),
			creator.clone()
		));
	}: _(RawOrigin::Signed(pallet_admin.clone()), creator.clone())
	verify {
		assert!(!FeedCreators::<T>::contains_key(&creator));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests::{new_test_ext, Test};
	use frame_support::assert_ok;

	#[test]
	fn create_feed() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_create_feed::<Test>());
		});
	}

	#[test]
	fn transfer_ownership() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_transfer_ownership::<Test>());
		});
	}

	#[test]
	fn accept_ownership() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_accept_ownership::<Test>());
		});
	}

	#[test]
	fn submit_opening_round_answers() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_submit_opening_round_answers::<Test>());
		});
	}

	#[test]
	fn submit_closing_answer() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_submit_closing_answer::<Test>());
		});
	}

	#[test]
	fn change_oracles() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_change_oracles::<Test>());
		});
	}

	#[test]
	fn update_future_rounds() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_update_future_rounds::<Test>());
		});
	}

	#[test]
	fn set_requester() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_requester::<Test>());
		});
	}

	#[test]
	fn remove_requester() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_remove_requester::<Test>());
		});
	}

	#[test]
	fn request_new_round() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_request_new_round::<Test>());
		});
	}

	#[test]
	fn withdraw_payment() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_withdraw_payment::<Test>());
		});
	}

	#[test]
	fn transfer_admin() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_transfer_admin::<Test>());
		});
	}

	#[test]
	fn accept_admin() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_accept_admin::<Test>());
		});
	}

	#[test]
	fn withdraw_funds() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_withdraw_funds::<Test>());
		});
	}

	#[test]
	fn reduce_debt() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_reduce_debt::<Test>());
		});
	}

	#[test]
	fn transfer_pallet_admin() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_transfer_pallet_admin::<Test>());
		});
	}

	#[test]
	fn accept_pallet_admin() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_accept_pallet_admin::<Test>());
		});
	}

	#[test]
	fn set_feed_creator() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_feed_creator::<Test>());
		});
	}

	#[test]
	fn remove_feed_creator() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_remove_feed_creator::<Test>());
		});
	}
}
