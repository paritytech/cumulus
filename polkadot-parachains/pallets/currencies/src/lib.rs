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

mod types;
pub use types::BasicCurrencyAdapter;

#[frame_support::pallet]
pub mod pallet {

    /* -------- Substrate Libs ------- */
    use frame_support::{
        pallet_prelude::*,
        sp_runtime::traits::{One},
    };
    use frame_system::{pallet_prelude::*};
    use sp_runtime::{ DispatchResult };
    use sp_std::{vec::Vec};

    /* ------- Local Libs -------- */
    use polkadot_parachain_primitives::{Currency, Price, PriceValue, CurrencyId};
    use pallet_traits::{PriceProvider, MultiCurrency, PriceSetter};
    use frame_support::sp_runtime::traits::Zero;
    use crate::types::BasicCurrency;

    pub(crate) type BalanceOf<T> =
    <<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_sudo::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// TODO: figure out the syntax here: 'Balance = BalanceOf<Self>'
        type BasicCurrency: BasicCurrency<Self::AccountId, Balance = BalanceOf<Self>>;
        type MultiCurrency: MultiCurrency<Self::AccountId>;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::error]
    pub enum Error<T> {
        /// The currency has already been listed, cannot list again.
        CurrencyAlreadyListed,
        /// The currency does not exist.
        CurrencyDoesNotExist,
        /// Cannot set the price of the currency.
        CannotSetPrice,
        /// Not authorized to perform the action
        NotAuthorized,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The currency is listed. [currency_id]
        CurrencyListed(CurrencyId),
        /// The price of the currency was updated. [currency_id, price_value]
        CurrencyPriceUpdated(CurrencyId, PriceValue),
    }

    /* ------- Storage Related ------- */
    #[pallet::storage]
    #[pallet::getter(fn next_currency_id)]
    pub(super) type NextCurrencyId<T: Config> = StorageValue<_, CurrencyId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn currency_name)]
    pub(super) type CurrencyNameStorage<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, CurrencyId>;

    #[pallet::storage]
    #[pallet::getter(fn currency)]
    pub(super) type CurrencyStorage<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Currency>;

    #[pallet::storage]
    #[pallet::getter(fn price)]
    pub(super) type PriceStorage<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Price<T>>;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1)]
        pub fn transfer(
            origin: OriginFor<T>,
            dest: T::AccountId,
            currency_id: CurrencyId,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;
            <Self as MultiCurrency<T::AccountId>>::transfer(currency_id, &from, &dest, amount)?;
            Ok(().into())
        }

        #[pallet::weight(1)]
        pub fn list_new(
            origin: OriginFor<T>,
            name: Vec<u8>,
            currency_type: u64,
            address: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            // Checks
            // TODO: enable this after figuring out why this won't work
            // ensure_root(origin)?;

            // We only allow root to perform this action
            let origin = ensure_signed(origin)?;
            if origin != <pallet_sudo::Pallet<T>>::key() {
                return Err(Error::<T>::NotAuthorized.into());
            }

            if <CurrencyNameStorage<T>>::contains_key(name.clone()) {
                return Err(Error::<T>::CurrencyAlreadyListed.into());
            }

            // Insert the currency into DB
            let id = <NextCurrencyId<T>>::get();
            let currency = Currency::new(
                currency_type,
                0,
                name.clone(),
                address
            );
            <CurrencyStorage<T>>::insert(id, currency);
            <CurrencyNameStorage<T>>::insert(name.clone(), id);

            <NextCurrencyId<T>>::mutate(|id| *id += CurrencyId::one());

            Self::deposit_event(Event::CurrencyListed(id));

            Ok(().into())
        }
    }

    impl<T: Config> PriceProvider<T> for Pallet<T> {
        fn price(currency_id: CurrencyId) -> Price<T> {
            <PriceStorage::<T>>::get(currency_id).unwrap_or(Price::<T>::invalid_price())
        }
    }

    impl<T: Config> PriceSetter<T> for Pallet<T> {
        fn set_price_val(currency_id: CurrencyId, price_val: PriceValue, block_number: T::BlockNumber) -> DispatchResultWithPostInfo {
            let price = Price::new(price_val, block_number);
            Self::set_price(currency_id, price)
        }

        fn set_price(currency_id: CurrencyId, price: Price<T>) -> DispatchResultWithPostInfo {
            <PriceStorage::<T>>::insert(currency_id, price.clone());
            log::debug!("update price of currency {:?} to price {:?}", currency_id, price.clone().value());
            Ok(().into())
        }
    }

    impl<T: Config> MultiCurrency<T::AccountId> for Pallet<T> {
        type Balance = BalanceOf<T>;

        fn minimum_balance(_currency_id: CurrencyId) -> Self::Balance {
            unimplemented!()
        }

        fn total_issuance(_currency_id: CurrencyId) -> Self::Balance {
            unimplemented!()
        }

        fn total_balance(_currency_id: CurrencyId, _who: &T::AccountId) -> Self::Balance {
            unimplemented!()
        }

        fn free_balance(_currency_id: CurrencyId, _who: &T::AccountId) -> Self::Balance {
            unimplemented!()
        }

        // Ensure that an account can withdraw from their free balance
        // Is a no-op if amount to be withdrawn is zero.
        fn ensure_can_withdraw(_currency_id: CurrencyId, _who: &T::AccountId, _amount: Self::Balance) -> DispatchResult {
            unimplemented!()
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

            let currency = <CurrencyStorage<T>>::get(currency_id)
                .ok_or(Error::<T>::CurrencyDoesNotExist)?;

            if currency.is_basic_currency() {
                T::BasicCurrency::transfer(from, to, amount)?;
            } else if currency.is_polkadot_currency() {
                T::MultiCurrency::transfer(currency_id, from, to, amount)?;
            } else {
                unimplemented!()
            }

            Ok(().into())
        }

        /// Deposit some `amount` into the free balance of account `who`.
        ///
        /// Is a no-op if the `amount` to be deposited is zero.
        fn deposit(_currency_id: CurrencyId, _who: &T::AccountId, _amount: Self::Balance) -> DispatchResult {
            unimplemented!()
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
}