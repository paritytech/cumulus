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

//! XCM-Token module that handles cross chain related issues.
//! Currently still in test phase

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

    /* -------- Substrate Libs ------- */
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    /* ------- Local Libs -------- */
    use pallet_traits::{MultiCurrency, CrossChainTransfer};
    use xcm::v0::{ExecuteXcm, Xcm, MultiAsset, MultiLocation, Order};
    use frame_support::sp_runtime::traits::{Convert};
    use xcm_executor::traits::WeightBounds;
    use sp_std::{vec};
    use polkadot_parachain_primitives::ParachainId;

    pub(crate) type BalanceOf<T> =
    <<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
    pub type CurrencyIdOf<T> =
    <<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Something to execute an XCM message.
        type XcmExecutor: ExecuteXcm<Self::Call>;
        /// Means of measuring the weight consumed by an XCM message locally.
        type Weigher: WeightBounds<Self::Call>;
        type XCMAssetConverter: Convert<(CurrencyIdOf<Self>, BalanceOf<Self>), Option<MultiAsset>>;
        type XCMSelfLocConverter: Convert<Self::AccountId, MultiLocation>;
        type XCMAccountConverter: Convert<(ParachainId, Self::AccountId), Option<MultiLocation>>;
        type XCMDestinationConverter: Convert<ParachainId, Option<MultiLocation>>;
        //TODO: remove this when ready
        type MultiCurrency: MultiCurrency<Self::AccountId>;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::error]
    pub enum Error<T> {
        /// The weight for the message is invalid
        InvalidWeightForMessage,
        /// Invalid currency id
        InvalidCurrencyId,
        /// Invalid account for xcm
        AccountNotSupported,
        /// Currency not supported for xcm
        CurrencyNotSupported,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    impl <T: Config> CrossChainTransfer<T::AccountId> for Pallet<T> {
        type CurrencyId = CurrencyIdOf<T>;
        type Balance = BalanceOf<T>;

        fn transfer(chain_id: ParachainId, currency_id: Self::CurrencyId, from: &T::AccountId, to: &T::AccountId, amount: Self::Balance) -> DispatchResult {
            let origin = T::XCMSelfLocConverter::convert(from.clone());
            let asset = T::XCMAssetConverter::convert((currency_id, amount)).ok_or(Error::<T>::InvalidCurrencyId)?;
            let dest = T::XCMDestinationConverter::convert(chain_id).ok_or(Error::<T>::CurrencyNotSupported)?;
            let beneficiary = T::XCMAccountConverter::convert((chain_id, to.clone())).ok_or(Error::<T>::AccountNotSupported)?;
            let mut message = Xcm::WithdrawAsset {
                assets: vec![asset.clone()],
                effects: vec![
                    Order::InitiateTeleport {
                        assets: vec![ MultiAsset::All ],
                        dest,
                        effects: vec![
                            Order::DepositAsset { assets: vec![asset], dest: beneficiary },
                        ],
                    },
                ],
            };
            let weight = T::Weigher::weight(&mut message)
                .map_err(|()| Error::<T>::InvalidWeightForMessage)?;
            let _outcome = T::XcmExecutor::execute_xcm_in_credit(origin, message, weight, weight);
            Ok(())
        }
    }

    //
    // impl <T: Config> MultiCurrency<T::AccountId> for Pallet<T> {
    //     type CurrencyId = CurrencyIdOf<T>;
    //     type Balance = BalanceOf<T>;
    //
    //     fn minimum_balance(_currency_id: Self::CurrencyId) -> Self::Balance {
    //         unimplemented!()
    //     }
    //
    //     fn total_issuance(_currency_id: Self::CurrencyId) -> Self::Balance {
    //         unimplemented!()
    //     }
    //
    //     fn total_balance(_currency_id: Self::CurrencyId, _who: &T::AccountId) -> Self::Balance {
    //         unimplemented!()
    //     }
    //
    //     fn free_balance(_currency_id: Self::CurrencyId, _who: &T::AccountId) -> Self::Balance {
    //         unimplemented!()
    //     }
    //
    //     fn ensure_can_withdraw(_currency_id: Self::CurrencyId, _who: &T::AccountId, _amount: Self::Balance) -> DispatchResult {
    //         unimplemented!()
    //     }
    //
    //     fn transfer(currency_id: Self::CurrencyId, from: &T::AccountId, to: &T::AccountId, amount: Self::Balance) -> DispatchResult {
    //         let origin = T::XCMSelfLocConverter::convert(from.clone());
    //         let asset = T::XCMAssetConverter::convert((currency_id, amount)).ok_or(Error::<T>::InvalidCurrencyId)?;
    //         let dest = T::XCMDestinationConverter::convert(currency_id).ok_or(Error::<T>::CurrencyNotSupported)?;
    //         let beneficiary = T::XCMAccountConverter::convert((currency_id, to.clone())).ok_or(Error::<T>::AccountNotSupported)?;
    //         let mut message = Xcm::WithdrawAsset {
    //             assets: vec![asset.clone()],
    //             effects: vec![
    //                 Order::InitiateTeleport {
    //                     assets: vec![ MultiAsset::All ],
    //                     dest,
    //                     effects: vec![
    //                         Order::DepositAsset { assets: vec![asset], dest: beneficiary },
    //                     ],
    //                 },
    //             ],
    //         };
    //         let weight = T::Weigher::weight(&mut message)
    //             .map_err(|()| Error::<T>::InvalidWeightForMessage)?;
    //         let _outcome = T::XcmExecutor::execute_xcm_in_credit(origin, message, weight, weight);
    //         Ok(())
    //     }
    //
    //     fn deposit(_currency_id: Self::CurrencyId, _who: &T::AccountId, _amount: Self::Balance) -> DispatchResult {
    //         unimplemented!()
    //     }
    //
    //     fn withdraw(_currency_id: Self::CurrencyId, _who: &T::AccountId, _amount: Self::Balance) -> DispatchResult {
    //         unimplemented!()
    //     }
    //
    //     fn can_slash(_currency_id: Self::CurrencyId, _who: &T::AccountId, _value: Self::Balance) -> bool {
    //         unimplemented!()
    //     }
    //
    //     fn slash(_currency_id: Self::CurrencyId, _who: &T::AccountId, _amount: Self::Balance) -> Self::Balance {
    //         unimplemented!()
    //     }
    // }
}