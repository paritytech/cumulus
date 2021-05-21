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
    use frame_support::{
        pallet_prelude::*,
    };
    use frame_system::{pallet_prelude::*};
    use sp_runtime::{
        traits::{Convert, Zero}, FixedU128, FixedPointNumber, sp_std::convert::TryInto
    };

    // Chainlink
    use pallet_chainlink_feed::{FeedOracle, FeedInterface};

    /* ------- Local Libs -------- */
    use rococo_parachain_primitives::{CurrencyId, Price};
    use pallet_traits::PriceProvider;

    type FeedIdOf<T> = <<T as Config>::Oracle as FeedOracle<T>>::FeedId;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Oracle: FeedOracle<Self>;
        type CurrencyFeedConvertor: Convert<CurrencyId, Option<FeedIdOf<Self>>>;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::error]
    pub enum Error<T> {
        /// The price feed is missing
        FeedMissing,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
    }

    impl<T: Config> PriceProvider<T> for Pallet<T> {
        fn price(currency_id: u64) -> Price<T> {
            if let Some(feed_id) = T::CurrencyFeedConvertor::convert(currency_id) {
                return match T::Oracle::feed(feed_id) {
                    Some(feed) => {
                        let round_data = feed.latest_data();

                        // Convert data to proper FixedU128
                        let raw = TryInto::<u128>::try_into(round_data.answer).ok().unwrap_or(0);
                        let val = FixedU128::checked_from_rational(
                            raw,
                            u128::pow(10, feed.decimals().into())
                        ).unwrap_or(FixedU128::zero());

                        return Price::new(
                            val,
                            round_data.updated_at.clone()
                        );
                    },
                    None => Price::invalid_price()
                }

            }
            Price::invalid_price()
        }
    }
}