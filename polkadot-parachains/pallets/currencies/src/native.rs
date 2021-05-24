use frame_support::dispatch::DispatchResult;
use sp_std::{
    marker,
};
use pallet_traits::MultiCurrency;
use orml_traits::{MultiCurrency as ORMLMultiCurrency};

/// Adapt other currency traits implementation to `MultiCurrency`.
pub struct MultiCurrencyAdapter<Currency>(marker::PhantomData<Currency>);

type PalletBalanceOf<A, Currency> = <Currency as ORMLMultiCurrency<A>>::Balance;
type PalletCurrencyIdOf<A, Currency> = <Currency as ORMLMultiCurrency<A>>::CurrencyId;

impl<AccountId, Currency> MultiCurrency<AccountId> for MultiCurrencyAdapter<Currency>
    where
        Currency: ORMLMultiCurrency<AccountId>,
{
    type CurrencyId = PalletCurrencyIdOf<AccountId, Currency>;
    type Balance = PalletBalanceOf<AccountId, Currency>;

    fn minimum_balance(currency_id: Self::CurrencyId) -> Self::Balance {
        Currency::minimum_balance(currency_id)
    }

    fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance {
        Currency::total_issuance(currency_id)
    }

    fn total_balance(currency_id: Self::CurrencyId, who: &AccountId) -> Self::Balance {
        Currency::total_balance(currency_id, who)
    }

    fn free_balance(currency_id: Self::CurrencyId, who: &AccountId) -> Self::Balance {
        Currency::free_balance(currency_id, who)
    }

    fn ensure_can_withdraw(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult {
        Currency::ensure_can_withdraw(currency_id, who, amount)
    }

    fn transfer(currency_id: Self::CurrencyId, from: &AccountId, to: &AccountId, amount: Self::Balance) -> DispatchResult {
        Currency::transfer(currency_id, from, to, amount)
    }

    fn deposit(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult {
        Currency::deposit(currency_id, who, amount)
    }

    fn withdraw(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult {
        Currency::withdraw(currency_id, who, amount)
    }

    fn can_slash(currency_id: Self::CurrencyId, who: &AccountId, value: Self::Balance) -> bool {
        Currency::can_slash(currency_id, who, value)
    }

    fn slash(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::Balance) -> Self::Balance {
        Currency::slash(currency_id, who, amount)
    }
}