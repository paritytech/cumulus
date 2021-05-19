// // This file is part of Konomi.
//
// // Copyright (C) 2020-2021 Konomi Foundation.
// // SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
//
// // This program is free software: you can redistribute it and/or modify
// // it under the terms of the GNU General Public License as published by
// // the Free Software Foundation, either version 3 of the License, or
// // (at your option) any later version.
//
// // This program is distributed in the hope that it will be useful,
// // but WITHOUT ANY WARRANTY; without even the implied warranty of
// // MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// // GNU General Public License for more details.
//
// // You should have received a copy of the GNU General Public License
// // along with this program. If not, see <https://www.gnu.org/licenses/>.
//
// //! Currencies module.
// //! This pallet handles the lifecycle of the currency.
// //! This includes getting and setting the prices. The actual price is fetched by the Oracle module.
//
// #![cfg_attr(not(feature = "std"), no_std)]
//
// pub use pallet::*;
//
// #[frame_support::pallet]
// pub mod pallet {
//
//     /* -------- Substrate Libs ------- */
//     use frame_support::{
//         pallet_prelude::*,
//     };
//     use frame_system::{pallet_prelude::*};
//
//     /* ------- Local Libs -------- */
//     use primitives::{Currency, CurrencyId};
//     use pallet_chainlink_feed::{FeedOracle, RoundData};
//
//     #[pallet::config]
//     pub trait Config: frame_system::Config {
//         type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
//         /// TODO: figure out the syntax here: 'Balance = BalanceOf<Self>'
//         type Oracle: FeedOracle<Self>;
//     }
//
//     #[pallet::hooks]
//     impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
//
//     #[pallet::error]
//     pub enum Error<T> {
//         FeedMissing,
//     }
//
//     #[pallet::event]
//     #[pallet::generate_deposit(pub(crate) fn deposit_event)]
//     pub enum Event<T: Config> {
//     }
//
//     #[pallet::pallet]
//     pub struct Pallet<T>(_);
//
//     #[pallet::call]
//     impl<T: Config> Pallet<T> {
//         #[pallet::weight(1)]
//         pub fn test(
//             origin: OriginFor<T>,
//             dest: T::AccountId,
//             currency_id: CurrencyId,
//         ) -> DispatchResultWithPostInfo {
//             let from = ensure_signed(origin)?;
//             let feed = T::Oracle::feed(0.into()).ok_or(Error::<T>::FeedMissing)?;
//             let RoundData { answer, .. } = feed.latest_data();
//
//             log::info!("{:?}", answer);
//             Ok(().into())
//         }
//     }
// }