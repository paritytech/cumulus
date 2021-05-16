use codec::{FullCodec};
use frame_support::traits::{ Currency as PalletCurrency, ExistenceRequirement, WithdrawReasons};
use frame_support::dispatch::DispatchResult;
use frame_support::sp_runtime::traits::AtLeast32BitUnsigned;
use frame_support::pallet_prelude::MaybeSerializeDeserialize;
use sp_core::sp_std::fmt::Debug;
use sp_std::{
    marker,
};
use sp_runtime::traits::CheckedSub;
use rococo_parachain_primitives::InvalidParameters;

/// Abstraction over a fungible (single) currency system.
pub trait BasicCurrency<AccountId> {
    /// The balance of an account.
    type Balance: AtLeast32BitUnsigned + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;

    // Public immutables

    /// Existential deposit.
    fn minimum_balance() -> Self::Balance;

    /// The total amount of issuance.
    fn total_issuance() -> Self::Balance;

    /// The combined balance of `who`.
    fn total_balance(who: &AccountId) -> Self::Balance;

    /// The free balance of `who`.
    fn free_balance(who: &AccountId) -> Self::Balance;

    /// A dry-run of `withdraw`. Returns `Ok` iff the account is able to make a
    /// withdrawal of the given amount.
    fn ensure_can_withdraw(who: &AccountId, amount: Self::Balance) -> DispatchResult;

    // Public mutables

    /// Transfer some amount from one account to another.
    fn transfer(from: &AccountId, to: &AccountId, amount: Self::Balance) -> DispatchResult;

    /// Add `amount` to the balance of `who` and increase total issuance.
    fn deposit(who: &AccountId, amount: Self::Balance) -> DispatchResult;

    /// Remove `amount` from the balance of `who` and reduce total issuance.
    fn withdraw(who: &AccountId, amount: Self::Balance) -> DispatchResult;

    /// Same result as `slash(who, value)` (but without the side-effects)
    /// assuming there are no balance changes in the meantime and only the
    /// reserved balance is not taken into account.
    fn can_slash(who: &AccountId, value: Self::Balance) -> bool;

    /// Deduct the balance of `who` by up to `amount`.
    ///
    /// As much funds up to `amount` will be deducted as possible. If this is
    /// less than `amount`,then a non-zero value will be returned.
    fn slash(who: &AccountId, amount: Self::Balance) -> Self::Balance;
}


/// Adapt other currency traits implementation to `BasicCurrency`.
pub struct BasicCurrencyAdapter<Currency>(marker::PhantomData<Currency>);

type PalletBalanceOf<A, Currency> = <Currency as PalletCurrency<A>>::Balance;

impl<AccountId, Currency> BasicCurrency<AccountId>
for BasicCurrencyAdapter<Currency>
    where
        Currency: PalletCurrency<AccountId>,
{
    type Balance = PalletBalanceOf<AccountId, Currency>;

    fn minimum_balance() -> Self::Balance {
        Currency::minimum_balance()
    }

    fn total_issuance() -> Self::Balance {
        Currency::total_issuance()
    }

    fn total_balance(who: &AccountId) -> Self::Balance {
        Currency::total_balance(who)
    }

    fn free_balance(who: &AccountId) -> Self::Balance {
        Currency::free_balance(who)
    }

    fn ensure_can_withdraw(who: &AccountId, amount: Self::Balance) -> DispatchResult {
        let new_balance = Self::free_balance(who)
            .checked_sub(&amount)
            .ok_or(InvalidParameters{})?;

        Currency::ensure_can_withdraw(who, amount, WithdrawReasons::all(), new_balance)
    }

    fn transfer(from: &AccountId, to: &AccountId, amount: Self::Balance) -> DispatchResult {
        Currency::transfer(from, to, amount, ExistenceRequirement::AllowDeath)
    }

    fn deposit(who: &AccountId, amount: Self::Balance) -> DispatchResult {
        let _ = Currency::deposit_creating(who, amount);
        Ok(())
    }

    fn withdraw(who: &AccountId, amount: Self::Balance) -> DispatchResult {
        Currency::withdraw(who, amount, WithdrawReasons::all(), ExistenceRequirement::AllowDeath).map(|_| ())
    }

    fn can_slash(who: &AccountId, amount: Self::Balance) -> bool {
        Currency::can_slash(who, amount)
    }

    fn slash(who: &AccountId, amount: Self::Balance) -> Self::Balance {
        let (_, gap) = Currency::slash(who, amount);
        gap
    }
}