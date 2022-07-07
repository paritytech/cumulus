// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::constants::POLKADOT_TREASURY_PALLET_INDEX;
use frame_support::{
	log,
	traits::{Currency, Imbalance, OnUnbalanced, OriginTrait},
};
use sp_std::{boxed::Box, marker::PhantomData};
use xcm::latest::{Fungibility, Junction, Parent};

type NegativeImbalanceOf<T, I> = <<T as pallet_alliance::Config<I>>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

type BalanceOf<T, I> = <<T as pallet_alliance::Config<I>>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;
pub struct ToParentTreasury<T, I = ()>(PhantomData<(T, I)>);
impl<T, I: 'static> OnUnbalanced<NegativeImbalanceOf<T, I>> for ToParentTreasury<T, I>
where
	T: pallet_xcm::Config + frame_system::Config + pallet_alliance::Config<I>,
	BalanceOf<T, I>: Into<Fungibility>,
{
	fn on_unbalanced(amount: NegativeImbalanceOf<T, I>) {
		let result = pallet_xcm::Pallet::<T>::teleport_assets(
			<T as frame_system::Config>::Origin::root(),
			Box::new(Parent.into()),
			Box::new(Junction::PalletInstance(POLKADOT_TREASURY_PALLET_INDEX).into().into()),
			Box::new((Parent, amount.peek()).into()),
			0,
		);
		match result {
			Err(err) => log::warn!("Failed to teleport slashed assets: {:?}", err),
			_ => (),
		};
	}
}
