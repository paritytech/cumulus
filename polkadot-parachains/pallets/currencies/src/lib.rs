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

//! Currencies module.
//! This pallet handles the lifecycle of the currency.
//! This includes getting and setting the prices. The actual price is fetched by the Oracle module.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

mod basic;
mod native;
mod mock;
mod tests;

pub use basic::BasicCurrencyAdapter;
pub use native::MultiCurrencyAdapter;

#[frame_support::pallet]
pub mod pallet {

    /* -------- Substrate Libs ------- */
    use frame_support::{pallet_prelude::*};
    use frame_system::{pallet_prelude::*};
    use sp_runtime::{ DispatchResult };

    /* ------- Local Libs -------- */
    use frame_support::sp_runtime::traits::{Zero, CheckedSub};
    use pallet_traits::{MultiCurrency, BasicCurrency, CrossChainTransfer};
    use frame_support::traits::{SignedImbalance, ExistenceRequirement, WithdrawReasons, Contains};
    use polkadot_parachain_primitives::ParachainId;

    pub(crate) type BalanceOf<T> =
    <<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type PositiveImbalanceOf<T> =
    <<T as Config>::BasicCurrency as BasicCurrency<<T as frame_system::Config>::AccountId>>::PositiveImbalance;
    pub(crate) type NegativeImbalanceOf<T> =
    <<T as Config>::BasicCurrency as BasicCurrency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;
    pub type CurrencyIdOf<T> =
    <<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type GetBasicCurrencyId: Get<CurrencyIdOf<Self>>;
        type IsCrossCurrencyId: Contains<CurrencyIdOf<Self>>;
        type BasicCurrency: BasicCurrency<Self::AccountId, Balance = BalanceOf<Self>>;
        type MultiCurrency: MultiCurrency<Self::AccountId>;
        type CrossCurrency: CrossChainTransfer<Self::AccountId, CurrencyId = CurrencyIdOf<Self>, Balance = BalanceOf<Self>>;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::error]
    pub enum Error<T> {
        /// The currency does not exist.
        CurrencyDoesNotExist,
        NotEnoughBalance,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Token transfer success. \[currency_id, from, to, amount\]
        Transferred(CurrencyIdOf<T>, T::AccountId, T::AccountId, BalanceOf<T>),
    }

    /* ------- Storage Related ------- */
    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1)]
        pub fn transfer(
            origin: OriginFor<T>,
            dest: T::AccountId,
            currency_id: CurrencyIdOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;
            <Self as MultiCurrency<T::AccountId>>::transfer(currency_id, &from, &dest, amount)?;
            Self::deposit_event(Event::Transferred(currency_id, from, dest, amount));

            Ok(().into())
        }

        #[pallet::weight(1000000000)]
        pub fn cross_chain_transfer(
            origin: OriginFor<T>,
            chain_id: ParachainId,
            dest: T::AccountId,
            currency_id: CurrencyIdOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;
            T::CrossCurrency::transfer(chain_id, currency_id, &from, &dest, amount)?;
            Self::deposit_event(Event::Transferred(currency_id, from, dest, amount));

            Ok(().into())
        }
    }

    impl <T: Config> frame_support::traits::Currency<T::AccountId> for Pallet<T> {
        type Balance = BalanceOf<T>;
        type PositiveImbalance = PositiveImbalanceOf<T>;
        type NegativeImbalance = NegativeImbalanceOf<T>;

        fn total_balance(who: &T::AccountId) -> Self::Balance {
            T::BasicCurrency::total_balance(who)
        }

        fn can_slash(who: &T::AccountId, value: Self::Balance) -> bool {
            T::BasicCurrency::can_slash(who, value)
        }

        fn total_issuance() -> Self::Balance {
            T::BasicCurrency::total_issuance()
        }

        fn minimum_balance() -> Self::Balance {
            T::BasicCurrency::minimum_balance()
        }

        fn burn(amount: Self::Balance) -> Self::PositiveImbalance {
            T::BasicCurrency::burn(amount)
        }

        fn issue(amount: Self::Balance) -> Self::NegativeImbalance {
            T::BasicCurrency::issue(amount)
        }

        fn free_balance(who: &T::AccountId) -> Self::Balance {
            T::BasicCurrency::free_balance(who)
        }

        fn ensure_can_withdraw(
            who: &T::AccountId,
            amount: Self::Balance,
            reasons: WithdrawReasons,
            new_balance: Self::Balance,
        ) -> DispatchResult {
            T::BasicCurrency::ensure_can_withdraw(who, amount, reasons, new_balance)
        }

        fn transfer(source: &T::AccountId, dest: &T::AccountId, value: Self::Balance, existence_requirement: ExistenceRequirement) -> DispatchResult {
            T::BasicCurrency::transfer(source, dest, value, existence_requirement)
        }

        fn slash(who: &T::AccountId, amount: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
            T::BasicCurrency::slash(who, amount)
        }

        fn deposit_into_existing(who: &T::AccountId, value: Self::Balance) -> Result<Self::PositiveImbalance, DispatchError> {
            T::BasicCurrency::deposit_into_existing(who, value)
        }

        fn deposit_creating(who: &T::AccountId, value: Self::Balance) -> Self::PositiveImbalance {
            T::BasicCurrency::deposit_creating(who, value)
        }

        fn withdraw(who: &T::AccountId, value: Self::Balance, reasons: WithdrawReasons, liveness: ExistenceRequirement) -> Result<Self::NegativeImbalance, DispatchError> {
            T::BasicCurrency::withdraw(who, value, reasons, liveness)
        }

        fn make_free_balance_be(who: &T::AccountId, balance: Self::Balance) -> SignedImbalance<Self::Balance, Self::PositiveImbalance> {
            T::BasicCurrency::make_free_balance_be(who, balance)
        }
    }

    /// Currently we only need `transfer`, we will slowly implement as the use cases are introduced
    impl<T: Config> MultiCurrency<T::AccountId> for Pallet<T> {
        type CurrencyId = CurrencyIdOf<T>;
        type Balance = BalanceOf<T>;

        fn minimum_balance(_currency_id: Self::CurrencyId) -> Self::Balance {
            unimplemented!()
        }

        fn total_issuance(_currency_id: Self::CurrencyId) -> Self::Balance {
            unimplemented!()
        }

        fn total_balance(_currency_id: Self::CurrencyId, _who: &T::AccountId) -> Self::Balance {
            unimplemented!()
        }

        fn free_balance(_currency_id: Self::CurrencyId, _who: &T::AccountId) -> Self::Balance {
            unimplemented!()
        }

        // Ensure that an account can withdraw from their free balance
        // Is a no-op if amount to be withdrawn is zero.
        fn ensure_can_withdraw(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
            if amount.is_zero() {
                return Ok(());
            }


            // TODO: check ensure the currency exists

            match currency_id {
                id if id == T::GetBasicCurrencyId::get() => {
                    let total = T::BasicCurrency::total_balance(who);
                    let new_balance = total.checked_sub(&amount).ok_or(Error::<T>::NotEnoughBalance)?;
                    T::BasicCurrency::ensure_can_withdraw(who, amount, WithdrawReasons::TRANSFER, new_balance)?
                },
                _ => T::MultiCurrency::ensure_can_withdraw(currency_id, who, amount)?
            }

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
            // TODO: check ensure the currency exists

            match currency_id {
                id if id == T::GetBasicCurrencyId::get() => T::BasicCurrency::transfer(from, to, amount, ExistenceRequirement::AllowDeath)?,
                _ => {
                    T::MultiCurrency::transfer(currency_id, from, to, amount)?
                }
            }

            Ok(())
        }

        /// Deposit some `amount` into the free balance of account `who`.
        ///
        /// Is a no-op if the `amount` to be deposited is zero.
        fn deposit(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
            if amount.is_zero() {
                return Ok(());
            }

            // TODO: check ensure the currency exists
            match currency_id {
                id if id == T::GetBasicCurrencyId::get() => {
                    T::BasicCurrency::deposit_creating(who, amount);
                },
                _ => {
                    T::MultiCurrency::deposit(currency_id, who, amount).unwrap();
                }
            }

            Ok(())
        }

        fn withdraw(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
            if amount.is_zero() {
                return Ok(());
            }

            // TODO: check ensure the currency exists
            match currency_id {
                id if id == T::GetBasicCurrencyId::get() => {
                    T::BasicCurrency::withdraw(who, amount, WithdrawReasons::TRANSFER, ExistenceRequirement::AllowDeath)?;
                },
                _ => {
                    T::MultiCurrency::withdraw(currency_id, who, amount)?;
                }
            }

            Ok(())
        }

        // Check if `value` amount of free balance can be slashed from `who`.
        fn can_slash(_currency_id: Self::CurrencyId, _who: &T::AccountId, _value: Self::Balance) -> bool {
            unimplemented!()
        }

        fn slash(_currency_id: Self::CurrencyId, _who: &T::AccountId, _amount: Self::Balance) -> Self::Balance {
            unimplemented!()
        }
    }
}