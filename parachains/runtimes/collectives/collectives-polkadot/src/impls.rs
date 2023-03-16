// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{AccountId, Balance, OriginCaller};
use frame_support::{
	dispatch::{DispatchError, DispatchResultWithPostInfo},
	log,
	pallet_prelude::{DispatchResult, StorageMap, ValueQuery},
	traits::{
		fungible, Currency, ExistenceRequirement, Get, Imbalance, LockIdentifier, LockableCurrency,
		OnUnbalanced, OriginTrait, PrivilegeCmp, ReservableCurrency, SignedImbalance,
		StorageInstance, WithdrawReasons,
	},
	weights::Weight,
	Blake2_128Concat, BoundedVec,
};
use pallet_alliance::{ProposalIndex, ProposalProvider};
use parachains_common::impls::NegativeImbalance;
use sp_core::ConstU32;
use sp_runtime::traits::Zero;
use sp_std::{cmp::Ordering, marker::PhantomData, prelude::*};
use xcm::latest::{AssetId, Fungibility, Junction, Parent};

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

type ProposalOf<T, I> = <T as pallet_collective::Config<I>>::Proposal;

type HashOf<T> = <T as frame_system::Config>::Hash;

/// Type alias to conveniently refer to the `Currency::Balance` associated type.
pub type BalanceOf<T> =
	<pallet_balances::Pallet<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Implements `OnUnbalanced::on_unbalanced` to teleport slashed assets to relay chain treasury account.
pub struct ToParentTreasury<TreasuryAccount, PalletAccount, T>(
	PhantomData<(TreasuryAccount, PalletAccount, T)>,
);

impl<TreasuryAccount, PalletAccount, T> OnUnbalanced<NegativeImbalance<T>>
	for ToParentTreasury<TreasuryAccount, PalletAccount, T>
where
	T: pallet_balances::Config + pallet_xcm::Config + frame_system::Config,
	<<T as frame_system::Config>::RuntimeOrigin as OriginTrait>::AccountId: From<AccountIdOf<T>>,
	[u8; 32]: From<<T as frame_system::Config>::AccountId>,
	TreasuryAccount: Get<AccountIdOf<T>>,
	PalletAccount: Get<AccountIdOf<T>>,
	BalanceOf<T>: Into<Fungibility>,
{
	fn on_unbalanced(amount: NegativeImbalance<T>) {
		let amount = match amount.drop_zero() {
			Ok(..) => return,
			Err(amount) => amount,
		};
		let imbalance = amount.peek();
		let pallet_acc: AccountIdOf<T> = PalletAccount::get();
		let treasury_acc: AccountIdOf<T> = TreasuryAccount::get();

		<pallet_balances::Pallet<T>>::resolve_creating(&pallet_acc.clone(), amount);

		let result = <pallet_xcm::Pallet<T>>::teleport_assets(
			<<T as frame_system::Config>::RuntimeOrigin>::signed(pallet_acc.into()),
			Box::new(Parent.into()),
			Box::new(
				Junction::AccountId32 { network: None, id: treasury_acc.into() }
					.into_location()
					.into(),
			),
			Box::new((Parent, imbalance).into()),
			0,
		);

		match result {
			Err(err) => log::warn!("Failed to teleport slashed assets: {:?}", err),
			_ => (),
		};
	}
}

/// Proposal provider for alliance pallet.
/// Adapter from collective pallet to alliance proposal provider trait.
pub struct AllianceProposalProvider<T, I = ()>(PhantomData<(T, I)>);

impl<T, I> ProposalProvider<AccountIdOf<T>, HashOf<T>, ProposalOf<T, I>>
	for AllianceProposalProvider<T, I>
where
	T: pallet_collective::Config<I> + frame_system::Config,
	I: 'static,
{
	fn propose_proposal(
		who: AccountIdOf<T>,
		threshold: u32,
		proposal: Box<ProposalOf<T, I>>,
		length_bound: u32,
	) -> Result<(u32, u32), DispatchError> {
		pallet_collective::Pallet::<T, I>::do_propose_proposed(
			who,
			threshold,
			proposal,
			length_bound,
		)
	}

	fn vote_proposal(
		who: AccountIdOf<T>,
		proposal: HashOf<T>,
		index: ProposalIndex,
		approve: bool,
	) -> Result<bool, DispatchError> {
		pallet_collective::Pallet::<T, I>::do_vote(who, proposal, index, approve)
	}

	fn close_proposal(
		proposal_hash: HashOf<T>,
		proposal_index: ProposalIndex,
		proposal_weight_bound: Weight,
		length_bound: u32,
	) -> DispatchResultWithPostInfo {
		pallet_collective::Pallet::<T, I>::do_close(
			proposal_hash,
			proposal_index,
			proposal_weight_bound,
			length_bound,
		)
	}

	fn proposal_of(proposal_hash: HashOf<T>) -> Option<ProposalOf<T, I>> {
		pallet_collective::Pallet::<T, I>::proposal_of(proposal_hash)
	}
}

/// Used to compare the privilege of an origin inside the scheduler.
pub struct EqualOrGreatestRootCmp;

impl PrivilegeCmp<OriginCaller> for EqualOrGreatestRootCmp {
	fn cmp_privilege(left: &OriginCaller, right: &OriginCaller) -> Option<Ordering> {
		if left == right {
			return Some(Ordering::Equal)
		}
		match (left, right) {
			// Root is greater than anything.
			(OriginCaller::system(frame_system::RawOrigin::Root), _) => Some(Ordering::Greater),
			_ => None,
		}
	}
}

pub struct RemoteFungible<T, B, L, A>(PhantomData<(T, B, L, A)>);

impl<T, B, L, A> StorageInstance for RemoteFungible<T, B, L, A> {
	const STORAGE_PREFIX: &'static str = "freezes";
	fn pallet_prefix() -> &'static str {
		"remote_fungible"
	}
}

type Freezes<T, B, L, A> = StorageMap<
	RemoteFungible<T, B, L, A>,
	Blake2_128Concat,
	<T as frame_system::Config>::AccountId,
	BoundedVec<
		(LockIdentifier, <B as Currency<<T as frame_system::Config>::AccountId>>::Balance),
		ConstU32<{ u32::MAX }>,
	>,
	ValueQuery,
>;

use xcm_executor::traits::{AssetLockInspect, LockUsersInspect, LockUsersMutate};

impl<T, B, L, A> LockableCurrency<T::AccountId> for RemoteFungible<T, B, L, A>
where
	T: frame_system::Config,
	B: LockableCurrency<T::AccountId>,
	B::Balance: From<u128> + 'static,
	L: AssetLockInspect + LockUsersMutate,
	A: Get<AssetId>,
	[u8; 32]: From<T::AccountId>,
{
	type Moment = B::Moment;
	type MaxLocks = B::MaxLocks;

	fn set_lock(
		id: LockIdentifier,
		who: &T::AccountId,
		amount: Self::Balance,
		reasons: WithdrawReasons,
	) {
		// todo constructor for Junction::AccountId32
		let owner = Junction::AccountId32 { network: None, id: who.clone().to_owned().into() }
			.into_location();
		let lockable: B::Balance = match L::balance_locked(A::get(), &owner).unwrap().fun {
			Fungibility::Fungible(a) => a.into(),
			Fungibility::NonFungible(_) => 0.into(), // TODO verify
		};
		if amount > lockable {
			B::set_lock(id, who, amount, reasons)
		}
		if amount.is_zero() {
			return Self::remove_lock(id, who)
		}
		let mut locks = Freezes::<T, B, L, A>::get(who);
		if let Some((_, old_amount)) = locks.iter_mut().find(|(lid, _)| lid == &id) {
			*old_amount = amount.clone();
		} else {
			L::inc_users(A::get(), &owner);
			locks.force_push((id, amount)); // todo Q
		}
		Freezes::<T, B, L, A>::insert(who, locks);
	}

	fn extend_lock(
		id: LockIdentifier,
		who: &T::AccountId,
		amount: Self::Balance,
		reasons: WithdrawReasons,
	) {
		let owner = Junction::AccountId32 { network: None, id: who.clone().to_owned().into() }
			.into_location();
		let lockable: B::Balance = match L::balance_locked(A::get(), &owner).unwrap().fun {
			Fungibility::Fungible(a) => a.into(),
			Fungibility::NonFungible(_) => 0.into(), // TODO verify
		};
		if amount > lockable {
			// TODO verify, if there is already lock on remote assets
			// is if valid to extend the lock on local asset
			B::extend_lock(id, who, amount, reasons)
		}
		if amount.is_zero() {
			return
		}
		let mut locks = Freezes::<T, B, L, A>::get(who);
		if let Some((_, old_amount)) = locks.iter_mut().find(|(lid, _)| lid == &id) {
			*old_amount = *old_amount.max(&mut amount.clone());
		} else {
			L::inc_users(A::get(), &owner);
			locks.force_push((id, amount)); // todo Q
		}
		Freezes::<T, B, L, A>::insert(who, locks);
	}

	fn remove_lock(id: LockIdentifier, who: &T::AccountId) {
		let mut locks = Freezes::<T, B, L, A>::get(who);
		if let Some(i) = locks.iter().position(|(lid, _)| lid == &id) {
			locks.remove(i);
			let owner = Junction::AccountId32 { network: None, id: who.clone().to_owned().into() }
				.into_location();
			L::inc_users(A::get(), &owner);
			// todo if user count == 0 maybe RequestUnlock
			if locks.len() > 0 {
				Freezes::<T, B, L, A>::insert(who, locks);
			} else {
				Freezes::<T, B, L, A>::remove(who);
			}
		} else {
			B::remove_lock(id, who)
		}
	}
}

impl<T: frame_system::Config, B: Currency<T::AccountId>, L, A> Currency<T::AccountId>
	for RemoteFungible<T, B, L, A>
{
	type Balance = B::Balance;
	type PositiveImbalance = B::PositiveImbalance;
	type NegativeImbalance = B::NegativeImbalance;

	fn total_balance(who: &T::AccountId) -> Self::Balance {
		B::total_balance(who)
	}

	fn can_slash(who: &T::AccountId, value: Self::Balance) -> bool {
		B::can_slash(who, value)
	}

	fn total_issuance() -> Self::Balance {
		B::total_issuance()
	}

	fn active_issuance() -> Self::Balance {
		B::active_issuance()
	}

	fn deactivate(amount: Self::Balance) {
		B::deactivate(amount)
	}

	fn reactivate(amount: Self::Balance) {
		B::reactivate(amount)
	}

	fn minimum_balance() -> Self::Balance {
		B::minimum_balance()
	}

	fn burn(mut amount: Self::Balance) -> Self::PositiveImbalance {
		B::burn(amount)
	}

	fn issue(mut amount: Self::Balance) -> Self::NegativeImbalance {
		B::issue(amount)
	}

	fn free_balance(who: &T::AccountId) -> Self::Balance {
		// query lock amount from XCM
		B::free_balance(who)
	}

	fn ensure_can_withdraw(
		who: &T::AccountId,
		amount: Self::Balance,
		reasons: WithdrawReasons,
		new_balance: Self::Balance,
	) -> DispatchResult {
		B::ensure_can_withdraw(who, amount, reasons, new_balance)
	}

	fn transfer(
		transactor: &T::AccountId,
		dest: &T::AccountId,
		value: Self::Balance,
		existence_requirement: ExistenceRequirement,
	) -> DispatchResult {
		B::transfer(transactor, dest, value, existence_requirement)
	}

	fn slash(who: &T::AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
		B::slash(who, value)
	}

	fn deposit_into_existing(
		who: &T::AccountId,
		value: Self::Balance,
	) -> Result<Self::PositiveImbalance, DispatchError> {
		B::deposit_into_existing(who, value)
	}

	fn deposit_creating(who: &T::AccountId, value: Self::Balance) -> Self::PositiveImbalance {
		B::deposit_creating(who, value)
	}

	fn withdraw(
		who: &T::AccountId,
		value: Self::Balance,
		reasons: WithdrawReasons,
		liveness: ExistenceRequirement,
	) -> Result<Self::NegativeImbalance, DispatchError> {
		B::withdraw(who, value, reasons, liveness)
	}

	fn make_free_balance_be(
		who: &T::AccountId,
		value: Self::Balance,
	) -> SignedImbalance<Self::Balance, Self::PositiveImbalance> {
		B::make_free_balance_be(who, value)
	}
}

impl<T: frame_system::Config, B: ReservableCurrency<T::AccountId>, L, A>
	ReservableCurrency<T::AccountId> for RemoteFungible<T, B, L, A>
{
	fn can_reserve(who: &T::AccountId, value: Self::Balance) -> bool {
		B::can_reserve(who, value)
	}

	fn repatriate_reserved(
		slashed: &T::AccountId,
		beneficiary: &T::AccountId,
		value: Self::Balance,
		status: frame_support::traits::BalanceStatus,
	) -> Result<Self::Balance, DispatchError> {
		B::repatriate_reserved(slashed, beneficiary, value, status)
	}

	fn reserve(who: &T::AccountId, value: Self::Balance) -> DispatchResult {
		B::reserve(who, value)
	}

	fn reserved_balance(who: &T::AccountId) -> Self::Balance {
		B::reserved_balance(who)
	}

	fn slash_reserved(
		who: &T::AccountId,
		value: Self::Balance,
	) -> (Self::NegativeImbalance, Self::Balance) {
		B::slash_reserved(who, value)
	}

	fn unreserve(who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		B::unreserve(who, value)
	}
}

impl<T: frame_system::Config, B: fungible::Inspect<T::AccountId>, L, A>
	fungible::Inspect<T::AccountId> for RemoteFungible<T, B, L, A>
{
	type Balance = B::Balance;

	fn total_issuance() -> Self::Balance {
		B::total_issuance()
	}

	fn active_issuance() -> Self::Balance {
		B::active_issuance()
	}

	fn balance(who: &T::AccountId) -> Self::Balance {
		B::balance(who)
	}

	fn can_deposit(
		who: &T::AccountId,
		amount: Self::Balance,
		mint: bool,
	) -> frame_support::traits::tokens::DepositConsequence {
		B::can_deposit(who, amount, mint)
	}

	fn can_withdraw(
		who: &T::AccountId,
		amount: Self::Balance,
	) -> frame_support::traits::tokens::WithdrawConsequence<Self::Balance> {
		B::can_withdraw(who, amount)
	}

	fn minimum_balance() -> Self::Balance {
		B::minimum_balance()
	}

	fn reducible_balance(who: &T::AccountId, keep_alive: bool) -> Self::Balance {
		B::reducible_balance(who, keep_alive)
	}
}
