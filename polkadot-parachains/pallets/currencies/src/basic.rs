use frame_support::traits::{ Currency as PalletCurrency, ExistenceRequirement, WithdrawReasons};
use frame_support::dispatch::DispatchResult;
use sp_std::{
    marker,
};
use sp_runtime::traits::CheckedSub;
use polkadot_parachain_primitives::InvalidParameters;
use pallet_traits::BasicCurrency;


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