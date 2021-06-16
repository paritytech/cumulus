//! # Tokens Module
//!
//! ## Overview
//!
//! The tokens module provides fungible multi-currency functionality that
//! implements `MultiCurrency` trait.
//!
//! The tokens module provides functions for:
//!
//! - Querying and setting the balance of a given account.
//! - Getting and managing total issuance.
//! - Balance transfer between accounts.
//! - Depositing and withdrawing balance.
//! - Slashing an account balance.
//!
//! ### Implementations
//!
//! The tokens module provides implementations for following traits.
//!
//! - `MultiCurrency` - Abstraction over a fungible multi-currency system.
//! - `MultiCurrencyExtended` - Extended `MultiCurrency` with additional helper
//!   types and methods, like updating balance
//! by a given signed integer amount.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `transfer` - Transfer some balance to another account.
//! - `transfer_all` - Transfer all balance to another account.
//!
//! ### Genesis Config
//!
//! The tokens module depends on the `GenesisConfig`. Endowed accounts could be
//! configured in genesis configs.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

pub use crate::imbalances::{NegativeImbalance, PositiveImbalance};

use frame_support::{
	ensure, log,
	pallet_prelude::*,
	traits::{
		BalanceStatus as Status, Currency as PalletCurrency, ExistenceRequirement, Get, Imbalance,
		LockableCurrency as PalletLockableCurrency, MaxEncodedLen, ReservableCurrency as PalletReservableCurrency,
		SignedImbalance, WithdrawReasons,
	},
	transactional, BoundedVec, PalletId,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use orml_traits::{
	arithmetic::{self, Signed},
	currency::TransferAll,
	BalanceStatus, GetByKey, LockIdentifier, MultiCurrency, MultiCurrencyExtended, MultiLockableCurrency,
	MultiReservableCurrency, OnDust,
};
use sp_runtime::{
	traits::{
		AccountIdConversion, AtLeast32BitUnsigned, Bounded, CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Member,
		Saturating, StaticLookup, Zero,
	},
	DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::{
	convert::{Infallible, TryFrom, TryInto},
	marker,
	prelude::*,
	vec::Vec,
};

mod imbalances;
mod mock;
mod tests;
mod weights;

pub use weights::WeightInfo;

pub struct TransferDust<T, GetAccountId>(marker::PhantomData<(T, GetAccountId)>);
impl<T, GetAccountId> OnDust<T::AccountId, T::CurrencyId, T::Balance> for TransferDust<T, GetAccountId>
where
	T: Config,
	GetAccountId: Get<T::AccountId>,
{
	fn on_dust(who: &T::AccountId, currency_id: T::CurrencyId, amount: T::Balance) {
		// transfer the dust to treasury account, ignore the result,
		// if failed will leave some dust which still could be recycled.
		let _ = <Pallet<T> as MultiCurrency<T::AccountId>>::transfer(currency_id, who, &GetAccountId::get(), amount);
	}
}

pub struct BurnDust<T>(marker::PhantomData<T>);
impl<T: Config> OnDust<T::AccountId, T::CurrencyId, T::Balance> for BurnDust<T> {
	fn on_dust(who: &T::AccountId, currency_id: T::CurrencyId, amount: T::Balance) {
		// burn the dust, ignore the result,
		// if failed will leave some dust which still could be recycled.
		let _ = Pallet::<T>::withdraw(currency_id, who, amount);
	}
}

/// A single lock on a balance. There can be many of these on an account and
/// they "overlap", so the same balance is frozen by multiple locks.
#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug)]
pub struct BalanceLock<Balance> {
	/// An identifier for this lock. Only one lock may be in existence for
	/// each identifier.
	pub id: LockIdentifier,
	/// The amount which the free balance may not drop below when this lock
	/// is in effect.
	pub amount: Balance,
}

/// balance information for an account.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, MaxEncodedLen, RuntimeDebug)]
pub struct AccountData<Balance> {
	/// Non-reserved part of the balance. There may still be restrictions on
	/// this, but it is the total pool what may in principle be transferred,
	/// reserved.
	///
	/// This is the only balance that matters in terms of most operations on
	/// tokens.
	pub free: Balance,
	/// Balance which is reserved and may not be used at all.
	///
	/// This can still get slashed, but gets slashed last of all.
	///
	/// This balance is a 'reserve' balance that other subsystems use in
	/// order to set aside tokens that are still 'owned' by the account
	/// holder, but which are suspendable.
	pub reserved: Balance,
	/// The amount that `free` may not drop below when withdrawing.
	pub frozen: Balance,
}

impl<Balance: Saturating + Copy + Ord> AccountData<Balance> {
	/// The amount that this account's free balance may not be reduced
	/// beyond.
	pub(crate) fn frozen(&self) -> Balance {
		self.frozen
	}
	/// The total balance in this account including any that is reserved and
	/// ignoring any frozen.
	fn total(&self) -> Balance {
		self.free.saturating_add(self.reserved)
	}
}

pub use module::*;

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The balance type
		type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;

		/// The amount type, should be signed version of `Balance`
		type Amount: Signed
			+ TryInto<Self::Balance>
			+ TryFrom<Self::Balance>
			+ Parameter
			+ Member
			+ arithmetic::SimpleArithmetic
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize;

		/// The currency ID type
		type CurrencyId: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord;

		/// Weight information for extrinsics in this module.
		type WeightInfo: WeightInfo;

		/// The minimum amount required to keep an account.
		type ExistentialDeposits: GetByKey<Self::CurrencyId, Self::Balance>;

		/// Handler to burn or transfer account's dust
		type OnDust: OnDust<Self::AccountId, Self::CurrencyId, Self::Balance>;

		type MaxLocks: Get<u32>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The balance is too low
		BalanceTooLow,
		/// This operation will cause balance to overflow
		BalanceOverflow,
		/// This operation will cause total issuance to overflow
		TotalIssuanceOverflow,
		/// Cannot convert Amount into Balance type
		AmountIntoBalanceFailed,
		/// Failed because liquidity restrictions due to locking
		LiquidityRestrictions,
		/// Failed because the maximum locks was exceeded
		MaxLocksExceeded,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Token transfer success. \[currency_id, from, to, amount\]
		Transferred(T::CurrencyId, T::AccountId, T::AccountId, T::Balance),
		/// An account was removed whose balance was non-zero but below
		/// ExistentialDeposit, resulting in an outright loss. \[account,
		/// currency_id, amount\]
		DustLost(T::AccountId, T::CurrencyId, T::Balance),
	}

	/// The total issuance of a token type.
	#[pallet::storage]
	#[pallet::getter(fn total_issuance)]
	pub type TotalIssuance<T: Config> = StorageMap<_, Twox64Concat, T::CurrencyId, T::Balance, ValueQuery>;

	/// Any liquidity locks of a token type under an account.
	/// NOTE: Should only be accessed when setting, changing and freeing a lock.
	#[pallet::storage]
	#[pallet::getter(fn locks)]
	pub type Locks<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Twox64Concat,
		T::CurrencyId,
		BoundedVec<BalanceLock<T::Balance>, T::MaxLocks>,
		ValueQuery,
	>;

	/// The balance of a token type under an account.
	///
	/// NOTE: If the total is ever zero, decrease account ref account.
	///
	/// NOTE: This is only used in the case that this module is used to store
	/// balances.
	#[pallet::storage]
	#[pallet::getter(fn accounts)]
	pub type Accounts<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Twox64Concat,
		T::CurrencyId,
		AccountData<T::Balance>,
		ValueQuery,
	>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub endowed_accounts: Vec<(T::AccountId, T::CurrencyId, T::Balance)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				endowed_accounts: vec![],
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			// ensure no duplicates exist.
			let unique_endowed_accounts = self
				.endowed_accounts
				.iter()
				.map(|(account_id, currency_id, _)| (account_id, currency_id))
				.collect::<std::collections::BTreeSet<_>>();
			assert!(
				unique_endowed_accounts.len() == self.endowed_accounts.len(),
				"duplicate endowed accounts in genesis."
			);

			self.endowed_accounts
				.iter()
				.for_each(|(account_id, currency_id, initial_balance)| {
					assert!(
						*initial_balance >= T::ExistentialDeposits::get(&currency_id),
						"the balance of any account should always be more than existential deposit.",
					);
					Pallet::<T>::mutate_account(account_id, *currency_id, |account_data, _| {
						account_data.free = *initial_balance
					});
					TotalIssuance::<T>::mutate(*currency_id, |total_issuance| {
						*total_issuance = total_issuance
							.checked_add(initial_balance)
							.expect("total issuance cannot overflow when building genesis")
					});
				});
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Transfer some balance to another account.
		///
		/// The dispatch origin for this call must be `Signed` by the
		/// transactor.
		#[pallet::weight(T::WeightInfo::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			currency_id: T::CurrencyId,
			#[pallet::compact] amount: T::Balance,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			let to = T::Lookup::lookup(dest)?;
			<Self as MultiCurrency<_>>::transfer(currency_id, &from, &to, amount)?;

			Self::deposit_event(Event::Transferred(currency_id, from, to, amount));
			Ok(().into())
		}

		/// Transfer all remaining balance to the given account.
		///
		/// The dispatch origin for this call must be `Signed` by the
		/// transactor.
		#[pallet::weight(T::WeightInfo::transfer_all())]
		pub fn transfer_all(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			currency_id: T::CurrencyId,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			let to = T::Lookup::lookup(dest)?;
			let balance = <Self as MultiCurrency<T::AccountId>>::free_balance(currency_id, &from);
			<Self as MultiCurrency<T::AccountId>>::transfer(currency_id, &from, &to, balance)?;

			Self::deposit_event(Event::Transferred(currency_id, from, to, balance));
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Check whether account_id is a module account
	pub(crate) fn is_module_account_id(account_id: &T::AccountId) -> bool {
		PalletId::try_from_account(account_id).is_some()
	}

	pub(crate) fn try_mutate_account<R, E>(
		who: &T::AccountId,
		currency_id: T::CurrencyId,
		f: impl FnOnce(&mut AccountData<T::Balance>, bool) -> sp_std::result::Result<R, E>,
	) -> sp_std::result::Result<R, E> {
		Accounts::<T>::try_mutate_exists(who, currency_id, |maybe_account| {
			let existed = maybe_account.is_some();
			let mut account = maybe_account.take().unwrap_or_default();
			f(&mut account, existed).map(move |result| {
				let mut handle_dust: Option<T::Balance> = None;
				let total = account.total();
				*maybe_account = if total.is_zero() {
					None
				} else {
					// if non_zero total is below existential deposit and the account is not a
					// module account, should handle the dust.
					if total < T::ExistentialDeposits::get(&currency_id) && !Self::is_module_account_id(who) {
						handle_dust = Some(total);
					}
					Some(account)
				};

				(existed, maybe_account.is_some(), handle_dust, result)
			})
		})
		.map(|(existed, exists, handle_dust, result)| {
			if existed && !exists {
				// If existed before, decrease account provider.
				// Ignore the result, because if it failed means that these’s remain consumers,
				// and the account storage in frame_system shouldn't be repeaded.
				let _ = frame_system::Pallet::<T>::dec_providers(who);
			} else if !existed && exists {
				// if new, increase account provider
				frame_system::Pallet::<T>::inc_providers(who);
			}

			if let Some(dust_amount) = handle_dust {
				// `OnDust` maybe get/set storage `Accounts` of `who`, trigger handler here
				// to avoid some unexpected errors.
				T::OnDust::on_dust(who, currency_id, dust_amount);
				Self::deposit_event(Event::DustLost(who.clone(), currency_id, dust_amount));
			}

			result
		})
	}

	pub(crate) fn mutate_account<R>(
		who: &T::AccountId,
		currency_id: T::CurrencyId,
		f: impl FnOnce(&mut AccountData<T::Balance>, bool) -> R,
	) -> R {
		Self::try_mutate_account(who, currency_id, |account, existed| -> Result<R, Infallible> {
			Ok(f(account, existed))
		})
		.expect("Error is infallible; qed")
	}

	/// Set free balance of `who` to a new value.
	///
	/// Note this will not maintain total issuance, and the caller is
	/// expected to do it.
	pub(crate) fn set_free_balance(currency_id: T::CurrencyId, who: &T::AccountId, amount: T::Balance) {
		Self::mutate_account(who, currency_id, |account, _| {
			account.free = amount;
		});
	}

	/// Set reserved balance of `who` to a new value.
	///
	/// Note this will not maintain total issuance, and the caller is
	/// expected to do it.
	pub(crate) fn set_reserved_balance(currency_id: T::CurrencyId, who: &T::AccountId, amount: T::Balance) {
		Self::mutate_account(who, currency_id, |account, _| {
			account.reserved = amount;
		});
	}

	/// Update the account entry for `who` under `currency_id`, given the
	/// locks.
	pub(crate) fn update_locks(
		currency_id: T::CurrencyId,
		who: &T::AccountId,
		locks: &[BalanceLock<T::Balance>],
	) -> DispatchResult {
		// update account data
		Self::mutate_account(who, currency_id, |account, _| {
			account.frozen = Zero::zero();
			for lock in locks.iter() {
				account.frozen = account.frozen.max(lock.amount);
			}
		});

		// update locks
		let existed = <Locks<T>>::contains_key(who, currency_id);
		if locks.is_empty() {
			<Locks<T>>::remove(who, currency_id);
			if existed {
				// decrease account ref count when destruct lock
				frame_system::Pallet::<T>::dec_consumers(who);
			}
		} else {
			let bounded_locks: BoundedVec<BalanceLock<T::Balance>, T::MaxLocks> =
				locks.to_vec().try_into().map_err(|_| Error::<T>::MaxLocksExceeded)?;
			<Locks<T>>::insert(who, currency_id, bounded_locks);
			if !existed {
				// increase account ref count when initialize lock
				if frame_system::Pallet::<T>::inc_consumers(who).is_err() {
					// No providers for the locks. This is impossible under normal circumstances
					// since the funds that are under the lock will themselves be stored in the
					// account and therefore will need a reference.
					log::warn!(
						"Warning: Attempt to introduce lock consumer reference, yet no providers. \
						This is unexpected but should be safe."
					);
				}
			}
		}

		Ok(())
	}
}

impl<T: Config> MultiCurrency<T::AccountId> for Pallet<T> {
	type CurrencyId = T::CurrencyId;
	type Balance = T::Balance;

	fn minimum_balance(currency_id: Self::CurrencyId) -> Self::Balance {
		T::ExistentialDeposits::get(&currency_id)
	}

	fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance {
		<TotalIssuance<T>>::get(currency_id)
	}

	fn total_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		Self::accounts(who, currency_id).total()
	}

	fn free_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		Self::accounts(who, currency_id).free
	}

	// Ensure that an account can withdraw from their free balance given any
	// existing withdrawal restrictions like locks and vesting balance.
	// Is a no-op if amount to be withdrawn is zero.
	fn ensure_can_withdraw(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}

		let new_balance = Self::free_balance(currency_id, who)
			.checked_sub(&amount)
			.ok_or(Error::<T>::BalanceTooLow)?;
		ensure!(
			new_balance >= Self::accounts(who, currency_id).frozen(),
			Error::<T>::LiquidityRestrictions
		);
		Ok(())
	}

	/// Transfer some free balance from `from` to `to`.
	/// Is a no-op if value to be transferred is zero or the `from` is the
	/// same as `to`.
	fn transfer(
		currency_id: Self::CurrencyId,
		from: &T::AccountId,
		to: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		if amount.is_zero() || from == to {
			return Ok(());
		}
		Self::ensure_can_withdraw(currency_id, from, amount)?;

		let from_balance = Self::free_balance(currency_id, from);
		let to_balance = Self::free_balance(currency_id, to)
			.checked_add(&amount)
			.ok_or(Error::<T>::BalanceOverflow)?;
		// Cannot underflow because ensure_can_withdraw check
		Self::set_free_balance(currency_id, from, from_balance - amount);
		Self::set_free_balance(currency_id, to, to_balance);

		Ok(())
	}

	/// Deposit some `amount` into the free balance of account `who`.
	///
	/// Is a no-op if the `amount` to be deposited is zero.
	fn deposit(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}

		TotalIssuance::<T>::try_mutate(currency_id, |total_issuance| -> DispatchResult {
			*total_issuance = total_issuance
				.checked_add(&amount)
				.ok_or(Error::<T>::TotalIssuanceOverflow)?;

			Self::set_free_balance(currency_id, who, Self::free_balance(currency_id, who) + amount);

			Ok(())
		})
	}

	fn withdraw(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}
		Self::ensure_can_withdraw(currency_id, who, amount)?;

		// Cannot underflow because ensure_can_withdraw check
		<TotalIssuance<T>>::mutate(currency_id, |v| *v -= amount);
		Self::set_free_balance(currency_id, who, Self::free_balance(currency_id, who) - amount);

		Ok(())
	}

	// Check if `value` amount of free balance can be slashed from `who`.
	fn can_slash(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> bool {
		if value.is_zero() {
			return true;
		}
		Self::free_balance(currency_id, who) >= value
	}

	/// Is a no-op if `value` to be slashed is zero.
	///
	/// NOTE: `slash()` prefers free balance, but assumes that reserve
	/// balance can be drawn from in extreme circumstances. `can_slash()`
	/// should be used prior to `slash()` to avoid having to draw from
	/// reserved funds, however we err on the side of punishment if things
	/// are inconsistent or `can_slash` wasn't used appropriately.
	fn slash(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> Self::Balance {
		if amount.is_zero() {
			return amount;
		}

		let account = Self::accounts(who, currency_id);
		let free_slashed_amount = account.free.min(amount);
		// Cannot underflow becuase free_slashed_amount can never be greater than amount
		let mut remaining_slash = amount - free_slashed_amount;

		// slash free balance
		if !free_slashed_amount.is_zero() {
			// Cannot underflow becuase free_slashed_amount can never be greater than
			// account.free
			Self::set_free_balance(currency_id, who, account.free - free_slashed_amount);
		}

		// slash reserved balance
		if !remaining_slash.is_zero() {
			let reserved_slashed_amount = account.reserved.min(remaining_slash);
			// Cannot underflow due to above line
			remaining_slash -= reserved_slashed_amount;
			Self::set_reserved_balance(currency_id, who, account.reserved - reserved_slashed_amount);
		}

		// Cannot underflow because the slashed value cannot be greater than total
		// issuance
		<TotalIssuance<T>>::mutate(currency_id, |v| *v -= amount - remaining_slash);
		remaining_slash
	}
}

impl<T: Config> MultiCurrencyExtended<T::AccountId> for Pallet<T> {
	type Amount = T::Amount;

	fn update_balance(currency_id: Self::CurrencyId, who: &T::AccountId, by_amount: Self::Amount) -> DispatchResult {
		if by_amount.is_zero() {
			return Ok(());
		}

		// Ensure this doesn't overflow. There isn't any traits that exposes
		// `saturating_abs` so we need to do it manually.
		let by_amount_abs = if by_amount == Self::Amount::min_value() {
			Self::Amount::max_value()
		} else {
			by_amount.abs()
		};

		let by_balance =
			TryInto::<Self::Balance>::try_into(by_amount_abs).map_err(|_| Error::<T>::AmountIntoBalanceFailed)?;
		if by_amount.is_positive() {
			Self::deposit(currency_id, who, by_balance)
		} else {
			Self::withdraw(currency_id, who, by_balance).map(|_| ())
		}
	}
}

impl<T: Config> MultiLockableCurrency<T::AccountId> for Pallet<T> {
	type Moment = T::BlockNumber;

	// Set a lock on the balance of `who` under `currency_id`.
	// Is a no-op if lock amount is zero.
	fn set_lock(
		lock_id: LockIdentifier,
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}
		let mut new_lock = Some(BalanceLock { id: lock_id, amount });
		let mut locks = Self::locks(who, currency_id)
			.into_iter()
			.filter_map(|lock| {
				if lock.id == lock_id {
					new_lock.take()
				} else {
					Some(lock)
				}
			})
			.collect::<Vec<_>>();
		if let Some(lock) = new_lock {
			locks.push(lock)
		}
		Self::update_locks(currency_id, who, &locks[..])
	}

	// Extend a lock on the balance of `who` under `currency_id`.
	// Is a no-op if lock amount is zero
	fn extend_lock(
		lock_id: LockIdentifier,
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}
		let mut new_lock = Some(BalanceLock { id: lock_id, amount });
		let mut locks = Self::locks(who, currency_id)
			.into_iter()
			.filter_map(|lock| {
				if lock.id == lock_id {
					new_lock.take().map(|nl| BalanceLock {
						id: lock.id,
						amount: lock.amount.max(nl.amount),
					})
				} else {
					Some(lock)
				}
			})
			.collect::<Vec<_>>();
		if let Some(lock) = new_lock {
			locks.push(lock)
		}
		Self::update_locks(currency_id, who, &locks[..])
	}

	fn remove_lock(lock_id: LockIdentifier, currency_id: Self::CurrencyId, who: &T::AccountId) -> DispatchResult {
		let mut locks = Self::locks(who, currency_id);
		locks.retain(|lock| lock.id != lock_id);
		let locks_vec = locks.to_vec();
		Self::update_locks(currency_id, who, &locks_vec[..])
	}
}

impl<T: Config> MultiReservableCurrency<T::AccountId> for Pallet<T> {
	/// Check if `who` can reserve `value` from their free balance.
	///
	/// Always `true` if value to be reserved is zero.
	fn can_reserve(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> bool {
		if value.is_zero() {
			return true;
		}
		Self::ensure_can_withdraw(currency_id, who, value).is_ok()
	}

	/// Slash from reserved balance, returning any amount that was unable to
	/// be slashed.
	///
	/// Is a no-op if the value to be slashed is zero.
	fn slash_reserved(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		if value.is_zero() {
			return value;
		}

		let reserved_balance = Self::reserved_balance(currency_id, who);
		let actual = reserved_balance.min(value);
		Self::set_reserved_balance(currency_id, who, reserved_balance - actual);
		<TotalIssuance<T>>::mutate(currency_id, |v| *v -= actual);
		value - actual
	}

	fn reserved_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		Self::accounts(who, currency_id).reserved
	}

	/// Move `value` from the free balance from `who` to their reserved
	/// balance.
	///
	/// Is a no-op if value to be reserved is zero.
	fn reserve(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> DispatchResult {
		if value.is_zero() {
			return Ok(());
		}
		Self::ensure_can_withdraw(currency_id, who, value)?;

		let account = Self::accounts(who, currency_id);
		Self::set_free_balance(currency_id, who, account.free - value);
		// Cannot overflow becuase total issuance is using the same balance type and
		// this doesn't increase total issuance
		Self::set_reserved_balance(currency_id, who, account.reserved + value);
		Ok(())
	}

	/// Unreserve some funds, returning any amount that was unable to be
	/// unreserved.
	///
	/// Is a no-op if the value to be unreserved is zero.
	fn unreserve(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		if value.is_zero() {
			return value;
		}

		let account = Self::accounts(who, currency_id);
		let actual = account.reserved.min(value);
		Self::set_reserved_balance(currency_id, who, account.reserved - actual);
		Self::set_free_balance(currency_id, who, account.free + actual);

		value - actual
	}

	/// Move the reserved balance of one account into the balance of
	/// another, according to `status`.
	///
	/// Is a no-op if:
	/// - the value to be moved is zero; or
	/// - the `slashed` id equal to `beneficiary` and the `status` is
	///   `Reserved`.
	fn repatriate_reserved(
		currency_id: Self::CurrencyId,
		slashed: &T::AccountId,
		beneficiary: &T::AccountId,
		value: Self::Balance,
		status: BalanceStatus,
	) -> sp_std::result::Result<Self::Balance, DispatchError> {
		if value.is_zero() {
			return Ok(value);
		}

		if slashed == beneficiary {
			return match status {
				BalanceStatus::Free => Ok(Self::unreserve(currency_id, slashed, value)),
				BalanceStatus::Reserved => Ok(value.saturating_sub(Self::reserved_balance(currency_id, slashed))),
			};
		}

		let from_account = Self::accounts(slashed, currency_id);
		let to_account = Self::accounts(beneficiary, currency_id);
		let actual = from_account.reserved.min(value);
		match status {
			BalanceStatus::Free => {
				Self::set_free_balance(currency_id, beneficiary, to_account.free + actual);
			}
			BalanceStatus::Reserved => {
				Self::set_reserved_balance(currency_id, beneficiary, to_account.reserved + actual);
			}
		}
		Self::set_reserved_balance(currency_id, slashed, from_account.reserved - actual);
		Ok(value - actual)
	}
}

pub struct CurrencyAdapter<T, GetCurrencyId>(marker::PhantomData<(T, GetCurrencyId)>);

impl<T, GetCurrencyId> PalletCurrency<T::AccountId> for CurrencyAdapter<T, GetCurrencyId>
where
	T: Config,
	GetCurrencyId: Get<T::CurrencyId>,
{
	type Balance = T::Balance;
	type PositiveImbalance = PositiveImbalance<T, GetCurrencyId>;
	type NegativeImbalance = NegativeImbalance<T, GetCurrencyId>;

	fn total_balance(who: &T::AccountId) -> Self::Balance {
		Pallet::<T>::total_balance(GetCurrencyId::get(), who)
	}

	fn can_slash(who: &T::AccountId, value: Self::Balance) -> bool {
		Pallet::<T>::can_slash(GetCurrencyId::get(), who, value)
	}

	fn total_issuance() -> Self::Balance {
		Pallet::<T>::total_issuance(GetCurrencyId::get())
	}

	fn minimum_balance() -> Self::Balance {
		Pallet::<T>::minimum_balance(GetCurrencyId::get())
	}

	fn burn(mut amount: Self::Balance) -> Self::PositiveImbalance {
		if amount.is_zero() {
			return PositiveImbalance::zero();
		}
		<TotalIssuance<T>>::mutate(GetCurrencyId::get(), |issued| {
			*issued = issued.checked_sub(&amount).unwrap_or_else(|| {
				amount = *issued;
				Zero::zero()
			});
		});
		PositiveImbalance::new(amount)
	}

	fn issue(mut amount: Self::Balance) -> Self::NegativeImbalance {
		if amount.is_zero() {
			return NegativeImbalance::zero();
		}
		<TotalIssuance<T>>::mutate(GetCurrencyId::get(), |issued| {
			*issued = issued.checked_add(&amount).unwrap_or_else(|| {
				amount = Self::Balance::max_value() - *issued;
				Self::Balance::max_value()
			})
		});
		NegativeImbalance::new(amount)
	}

	fn free_balance(who: &T::AccountId) -> Self::Balance {
		Pallet::<T>::free_balance(GetCurrencyId::get(), who)
	}

	fn ensure_can_withdraw(
		who: &T::AccountId,
		amount: Self::Balance,
		_reasons: WithdrawReasons,
		_new_balance: Self::Balance,
	) -> DispatchResult {
		Pallet::<T>::ensure_can_withdraw(GetCurrencyId::get(), who, amount)
	}

	fn transfer(
		source: &T::AccountId,
		dest: &T::AccountId,
		value: Self::Balance,
		_existence_requirement: ExistenceRequirement,
	) -> DispatchResult {
		<Pallet<T> as MultiCurrency<T::AccountId>>::transfer(GetCurrencyId::get(), &source, &dest, value)
	}

	fn slash(who: &T::AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
		if value.is_zero() {
			return (Self::NegativeImbalance::zero(), value);
		}

		let currency_id = GetCurrencyId::get();
		let account = Pallet::<T>::accounts(who, currency_id);
		let free_slashed_amount = account.free.min(value);
		let mut remaining_slash = value - free_slashed_amount;

		// slash free balance
		if !free_slashed_amount.is_zero() {
			Pallet::<T>::set_free_balance(currency_id, who, account.free - free_slashed_amount);
		}

		// slash reserved balance
		if !remaining_slash.is_zero() {
			let reserved_slashed_amount = account.reserved.min(remaining_slash);
			remaining_slash -= reserved_slashed_amount;
			Pallet::<T>::set_reserved_balance(currency_id, who, account.reserved - reserved_slashed_amount);
			(
				Self::NegativeImbalance::new(free_slashed_amount + reserved_slashed_amount),
				remaining_slash,
			)
		} else {
			(Self::NegativeImbalance::new(value), remaining_slash)
		}
	}

	fn deposit_into_existing(
		who: &T::AccountId,
		value: Self::Balance,
	) -> sp_std::result::Result<Self::PositiveImbalance, DispatchError> {
		if value.is_zero() {
			return Ok(Self::PositiveImbalance::zero());
		}
		let currency_id = GetCurrencyId::get();
		let new_total = Pallet::<T>::free_balance(currency_id, who)
			.checked_add(&value)
			.ok_or(Error::<T>::TotalIssuanceOverflow)?;
		Pallet::<T>::set_free_balance(currency_id, who, new_total);

		Ok(Self::PositiveImbalance::new(value))
	}

	fn deposit_creating(who: &T::AccountId, value: Self::Balance) -> Self::PositiveImbalance {
		Self::deposit_into_existing(who, value).unwrap_or_else(|_| Self::PositiveImbalance::zero())
	}

	fn withdraw(
		who: &T::AccountId,
		value: Self::Balance,
		_reasons: WithdrawReasons,
		_liveness: ExistenceRequirement,
	) -> sp_std::result::Result<Self::NegativeImbalance, DispatchError> {
		if value.is_zero() {
			return Ok(Self::NegativeImbalance::zero());
		}
		let currency_id = GetCurrencyId::get();
		Pallet::<T>::ensure_can_withdraw(currency_id, who, value)?;
		Pallet::<T>::set_free_balance(currency_id, who, Pallet::<T>::free_balance(currency_id, who) - value);

		Ok(Self::NegativeImbalance::new(value))
	}

	fn make_free_balance_be(
		who: &T::AccountId,
		value: Self::Balance,
	) -> SignedImbalance<Self::Balance, Self::PositiveImbalance> {
		let currency_id = GetCurrencyId::get();
		Pallet::<T>::try_mutate_account(
			who,
			currency_id,
			|account, existed| -> Result<SignedImbalance<Self::Balance, Self::PositiveImbalance>, ()> {
				// If we're attempting to set an existing account to less than ED, then
				// bypass the entire operation. It's a no-op if you follow it through, but
				// since this is an instance where we might account for a negative imbalance
				// (in the dust cleaner of set_account) before we account for its actual
				// equal and opposite cause (returned as an Imbalance), then in the
				// instance that there's no other accounts on the system at all, we might
				// underflow the issuance and our arithmetic will be off.
				let ed = T::ExistentialDeposits::get(&currency_id);
				ensure!(value.saturating_add(account.reserved) >= ed || existed, ());

				let imbalance = if account.free <= value {
					SignedImbalance::Positive(PositiveImbalance::new(value - account.free))
				} else {
					SignedImbalance::Negative(NegativeImbalance::new(account.free - value))
				};
				account.free = value;
				Ok(imbalance)
			},
		)
		.unwrap_or_else(|_| SignedImbalance::Positive(Self::PositiveImbalance::zero()))
	}
}

impl<T, GetCurrencyId> PalletReservableCurrency<T::AccountId> for CurrencyAdapter<T, GetCurrencyId>
where
	T: Config,
	GetCurrencyId: Get<T::CurrencyId>,
{
	fn can_reserve(who: &T::AccountId, value: Self::Balance) -> bool {
		Pallet::<T>::can_reserve(GetCurrencyId::get(), who, value)
	}

	fn slash_reserved(who: &T::AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
		let actual = Pallet::<T>::slash_reserved(GetCurrencyId::get(), who, value);
		(Self::NegativeImbalance::zero(), actual)
	}

	fn reserved_balance(who: &T::AccountId) -> Self::Balance {
		Pallet::<T>::reserved_balance(GetCurrencyId::get(), who)
	}

	fn reserve(who: &T::AccountId, value: Self::Balance) -> DispatchResult {
		Pallet::<T>::reserve(GetCurrencyId::get(), who, value)
	}

	fn unreserve(who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		Pallet::<T>::unreserve(GetCurrencyId::get(), who, value)
	}

	fn repatriate_reserved(
		slashed: &T::AccountId,
		beneficiary: &T::AccountId,
		value: Self::Balance,
		status: Status,
	) -> sp_std::result::Result<Self::Balance, DispatchError> {
		Pallet::<T>::repatriate_reserved(GetCurrencyId::get(), slashed, beneficiary, value, status)
	}
}

impl<T, GetCurrencyId> PalletLockableCurrency<T::AccountId> for CurrencyAdapter<T, GetCurrencyId>
where
	T: Config,
	GetCurrencyId: Get<T::CurrencyId>,
{
	type Moment = T::BlockNumber;
	type MaxLocks = ();

	fn set_lock(id: LockIdentifier, who: &T::AccountId, amount: Self::Balance, _reasons: WithdrawReasons) {
		let _ = Pallet::<T>::set_lock(id, GetCurrencyId::get(), who, amount);
	}

	fn extend_lock(id: LockIdentifier, who: &T::AccountId, amount: Self::Balance, _reasons: WithdrawReasons) {
		let _ = Pallet::<T>::extend_lock(id, GetCurrencyId::get(), who, amount);
	}

	fn remove_lock(id: LockIdentifier, who: &T::AccountId) {
		let _ = Pallet::<T>::remove_lock(id, GetCurrencyId::get(), who);
	}
}

impl<T: Config> TransferAll<T::AccountId> for Pallet<T> {
	#[transactional]
	fn transfer_all(source: &T::AccountId, dest: &T::AccountId) -> DispatchResult {
		Accounts::<T>::iter_prefix(source).try_for_each(|(currency_id, account_data)| -> DispatchResult {
			<Self as MultiCurrency<T::AccountId>>::transfer(currency_id, source, dest, account_data.free)
		})
	}
}
