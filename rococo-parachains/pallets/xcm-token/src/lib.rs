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

#[frame_support::pallet]
pub mod pallet {

    /* -------- Substrate Libs ------- */
    use frame_support::{pallet_prelude::*};
    use frame_system::{pallet_prelude::*};
    use sp_runtime::{ DispatchResult };
    use frame_support::sp_runtime::traits::{AtLeast32BitUnsigned};
    use frame_support::sp_runtime::FixedPointOperand;

    use xcm::v0::{
        MultiAsset, MultiLocation, Order,
        Order::*,
        Xcm::{self, *},
    };

    /* ------- Local Libs -------- */
    use rococo_parachain_primitives::{CurrencyId};
    use pallet_traits::{MultiCurrency};

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Balance: Parameter + Member + FixedPointOperand + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::error]
    pub enum Error<T> {
    }

    #[pallet::event]
    // #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
    }

    impl<T: Config> MultiCurrency<T::AccountId> for Pallet<T> {
        type Balance = T::Balance;

        fn minimum_balance(_currency_id: CurrencyId) -> Self::Balance {
            unimplemented!()
        }

        fn total_issuance(_currency_id: u64) -> Self::Balance {
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
            _currency_id: CurrencyId,
            _from: &T::AccountId,
            _to: &T::AccountId,
            _amount: Self::Balance,
        ) -> DispatchResult {
            unimplemented!()
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