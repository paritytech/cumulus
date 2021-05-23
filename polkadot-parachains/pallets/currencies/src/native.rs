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
        unimplemented!()
    }

    fn total_balance(currency_id: Self::CurrencyId, who: &AccountId) -> Self::Balance {
        unimplemented!()
    }

    fn free_balance(currency_id: Self::CurrencyId, who: &AccountId) -> Self::Balance {
        unimplemented!()
    }

    fn ensure_can_withdraw(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult {
        unimplemented!()
    }

    fn transfer(currency_id: Self::CurrencyId, from: &AccountId, to: &AccountId, amount: Self::Balance) -> DispatchResult {
        unimplemented!()
    }

    fn deposit(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult {
        unimplemented!()
    }

    fn withdraw(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult {
        unimplemented!()
    }

    fn can_slash(currency_id: Self::CurrencyId, who: &AccountId, value: Self::Balance) -> bool {
        unimplemented!()
    }

    fn slash(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::Balance) -> Self::Balance {
        unimplemented!()
    }
}