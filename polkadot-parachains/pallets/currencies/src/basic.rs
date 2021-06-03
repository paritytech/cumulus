use frame_support::traits::{Currency as PalletCurrency, ExistenceRequirement, WithdrawReasons, SignedImbalance};
use frame_support::dispatch::{DispatchResult, DispatchError};
use sp_std::{
    marker,
};
use pallet_traits::BasicCurrency;


/// Adapt other currency traits implementation to `BasicCurrency`.
pub struct BasicCurrencyAdapter<Currency>(marker::PhantomData<Currency>);

type PalletBalanceOf<A, Currency> = <Currency as PalletCurrency<A>>::Balance;
type PalletPositiveImbalanceOf<A, Currency> = <Currency as PalletCurrency<A>>::PositiveImbalance;
type PalletNegativeImbalanceOf<A, Currency> = <Currency as PalletCurrency<A>>::NegativeImbalance;

impl<AccountId, Currency> BasicCurrency<AccountId>
for BasicCurrencyAdapter<Currency>
    where
        Currency: PalletCurrency<AccountId>,
{
    type Balance = PalletBalanceOf<AccountId, Currency>;
    type PositiveImbalance = PalletPositiveImbalanceOf<AccountId, Currency>;
    type NegativeImbalance = PalletNegativeImbalanceOf<AccountId, Currency>;

    fn total_balance(who: &AccountId) -> Self::Balance {
        Currency::total_balance(who)
    }

    fn can_slash(who: &AccountId, amount: Self::Balance) -> bool {
        Currency::can_slash(who, amount)
    }

    fn total_issuance() -> Self::Balance {
        Currency::total_issuance()
    }

    fn minimum_balance() -> Self::Balance {
        Currency::minimum_balance()
    }

    fn burn(amount: Self::Balance) -> Self::PositiveImbalance { Currency::burn(amount) }

    fn issue(amount: Self::Balance) -> Self::NegativeImbalance {
        Currency::issue(amount)
    }

    fn free_balance(who: &AccountId) -> Self::Balance {
        Currency::free_balance(who)
    }

    fn ensure_can_withdraw(
        who: &AccountId,
        amount: Self::Balance,
        reasons: WithdrawReasons,
        new_balance: Self::Balance,
    ) -> DispatchResult {
        Currency::ensure_can_withdraw(who, amount, reasons, new_balance)
    }

    fn transfer(source: &AccountId, dest: &AccountId, value: Self::Balance, existence_requirement: ExistenceRequirement) -> DispatchResult {
        Currency::transfer(source, dest, value, existence_requirement)
    }

    fn slash(who: &AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
        Currency::slash(who, value)
    }

    fn deposit_into_existing(who: &AccountId, value: Self::Balance) -> Result<Self::PositiveImbalance, DispatchError> {
        Currency::deposit_into_existing(who, value)
    }

    fn deposit_creating(who: &AccountId, value: Self::Balance) -> Self::PositiveImbalance {
        Currency::deposit_creating(who, value)
    }

    fn withdraw(who: &AccountId, value: Self::Balance, reasons: WithdrawReasons, liveness: ExistenceRequirement) -> Result<Self::NegativeImbalance, DispatchError> {
        Currency::withdraw(who, value, reasons, liveness)
    }

    fn make_free_balance_be(who: &AccountId, balance: Self::Balance) -> SignedImbalance<Self::Balance, Self::PositiveImbalance> {
        Currency::make_free_balance_be(who, balance)
    }
}