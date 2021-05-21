// This file is part of Konomi.

// Copyright (C) 2020-2021 Konomi Foundation.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

pub use pallet::*;


#[frame_support::pallet]
pub mod pallet {
    use frame_system::pallet_prelude::*;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, PalletId};
    use frame_support::sp_runtime::traits::{AtLeast32BitUnsigned, Zero, CheckedSub, CheckedAdd, AccountIdConversion};
    use frame_support::sp_runtime::{FixedPointOperand};
    use frame_support::dispatch::DispatchResult;
    use sp_std::{convert::{Infallible}};
    use frame_support::{sp_runtime::traits::Saturating};
    use codec::{Decode, Encode};
    use frame_support::pallet_prelude::Get;
    use sp_std::{
        marker,
    };

    use pallet_traits::{MultiCurrency, GetByKey, OnDust};
    use polkadot_parachain_primitives::CurrencyId;

    /// balance information for an account.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
    pub struct AccountData<Balance> {
        /// Non-reserved part of the balance. There may still be restrictions on
        /// this, but it is the total floating-rate-pool what may in principle be transferred,
        /// reserved.
        ///
        /// This is the only balance that matters in terms of most operations on
        /// xcm-token.
        pub free: Balance,
        /// Balance which is reserved and may not be used at all.
        ///
        /// This can still get slashed, but gets slashed last of all.
        ///
        /// This balance is a 'reserve' balance that other subsystems use in
        /// order to set aside xcm-token that are still 'owned' by the account
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

    pub struct TransferDust<T, GetAccountId>(marker::PhantomData<(T, GetAccountId)>);
    impl<T, GetAccountId> OnDust<T::AccountId, CurrencyId, T::Balance> for TransferDust<T, GetAccountId>
        where
            T: Config,
            GetAccountId: Get<T::AccountId>,
    {
        fn on_dust(who: &T::AccountId, currency_id: CurrencyId, amount: T::Balance) {
            // transfer the dust to treasury account, ignore the result,
            // if failed will leave some dust which still could be recycled.
            let _ = <Pallet<T> as MultiCurrency<T::AccountId>>::transfer(currency_id, who, &GetAccountId::get(), amount);
        }
    }

    pub struct BurnDust<T>(marker::PhantomData<T>);
    impl<T: Config> OnDust<T::AccountId, CurrencyId, T::Balance> for BurnDust<T> {
        fn on_dust(who: &T::AccountId, currency_id: CurrencyId, amount: T::Balance) {
            // burn the dust, ignore the result,
            // if failed will leave some dust which still could be recycled.
            let _ = Pallet::<T>::withdraw(currency_id, who, amount);
        }
    }

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Balance: Member + Parameter + FixedPointOperand + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;
        /// The minimum amount required to keep an account.
        type ExistentialDeposits: GetByKey<CurrencyId, Self::Balance>;
        /// Handler to burn or transfer account's dust
        type OnDust: OnDust<Self::AccountId, CurrencyId, Self::Balance>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
        /// The balance is too low
        BalanceTooLow,
        /// This operation will cause balance to overflow
        BalanceOverflow,
        /// This operation will cause total issuance to overflow
        TotalIssuanceOverflow,
        /// Failed because liquidity restrictions due to locking
        LiquidityRestrictions,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Token transfer success. \[currency_id, from, to, amount\]
        Transferred(CurrencyId, T::AccountId, T::AccountId, T::Balance),
        /// An account was removed whose balance was non-zero but below
        /// ExistentialDeposit, resulting in an outright loss. \[account,
        /// currency_id, amount\]
        DustLost(T::AccountId, CurrencyId, T::Balance),
    }

    /// The total issuance of a token type.
    #[pallet::storage]
    #[pallet::getter(fn total_issuance)]
    pub type TotalIssuance<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, T::Balance, ValueQuery>;

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
        CurrencyId,
        AccountData<T::Balance>,
        ValueQuery,
    >;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub endowed_accounts: Vec<(T::AccountId, CurrencyId, T::Balance)>,
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

    #[pallet::call]
    impl<T:Config> Pallet<T> {
        /// Transfer some balance to another account.
        ///
        /// The dispatch origin for this call must be `Signed` by the
        /// transactor.
        #[pallet::weight(1)]
        pub fn transfer(
            origin: OriginFor<T>,
            dest: T::AccountId,
            currency_id: CurrencyId,
            #[pallet::compact] amount: T::Balance,
        ) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;
            <Self as MultiCurrency<_>>::transfer(currency_id, &from, &dest, amount)?;

            Self::deposit_event(Event::Transferred(currency_id, from, dest, amount));
            Ok(().into())
        }
    }

    impl<T: Config> MultiCurrency<T::AccountId> for Pallet<T> {
        type Balance = T::Balance;

        fn minimum_balance(currency_id: CurrencyId) -> Self::Balance {
            T::ExistentialDeposits::get(&currency_id)
        }

        fn total_issuance(currency_id: CurrencyId) -> Self::Balance {
            <TotalIssuance<T>>::get(currency_id)
        }

        fn total_balance(currency_id: CurrencyId, who: &T::AccountId) -> Self::Balance {
            Accounts::<T>::get(who, currency_id).total()
        }

        fn free_balance(currency_id: CurrencyId, who: &T::AccountId) -> Self::Balance {
            Accounts::<T>::get(who, currency_id).free
        }

        // Ensure that an account can withdraw from their free balance given any
        // existing withdrawal restrictions like locks and vesting balance.
        // Is a no-op if amount to be withdrawn is zero.
        fn ensure_can_withdraw(currency_id: CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
            if amount.is_zero() {
                return Ok(());
            }

            let new_balance = Self::free_balance(currency_id, who)
                .checked_sub(&amount)
                .ok_or(Error::<T>::BalanceTooLow)?;
            ensure!(
                new_balance >= Accounts::<T>::get(who, currency_id).frozen(),
                Error::<T>::LiquidityRestrictions
            );
            Ok(())
        }

        /// Transfer some free balance from `from` to `to`.
        /// Is a no-op if value to be transferred is zero or the `from` is the
        /// same as `to`.
        fn transfer(
            currency_id: CurrencyId,
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
        fn deposit(currency_id: CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
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

        fn withdraw(_currency_id: CurrencyId, _who: &T::AccountId, _amount: Self::Balance) -> DispatchResult {
            unimplemented!()
        }

        // Check if `value` amount of free balance can be slashed from `who`.
        fn can_slash(_currency_id: CurrencyId, _who: &T::AccountId, _value: Self::Balance) -> bool {
            unimplemented!()
        }

        fn slash(_currency_id: CurrencyId, _who: &T::AccountId, _amount: Self::Balance) -> Self::Balance {
            unimplemented!()
        }
    }

    impl<T: Config> Pallet<T> {
        /// Check whether account_id is a module account
        pub(crate) fn is_module_account_id(account_id: &T::AccountId) -> bool {
            PalletId::try_from_account(account_id).is_some()
        }

        pub(crate) fn try_mutate_account<R, E>(
            who: &T::AccountId,
            currency_id: CurrencyId,
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
            currency_id: CurrencyId,
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
        pub(crate) fn set_free_balance(currency_id: CurrencyId, who: &T::AccountId, amount: T::Balance) {
            Self::mutate_account(who, currency_id, |account, _| {
                account.free = amount;
            });
        }
    }
}
