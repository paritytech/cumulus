// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// &Licensed under the Apache License, Version 2.0 (the "License");
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

use frame_support::{
	log,
	traits::{Currency, Get, Imbalance, OnUnbalanced, OriginTrait},
};
use sp_std::{boxed::Box, marker::PhantomData};
use xcm::latest::{Fungibility, Junction, NetworkId, Parent};

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

type NegativeImbalanceOf<T, I> = <<T as pallet_alliance::Config<I>>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

type CurrencyOf<T, I> = <T as pallet_alliance::Config<I>>::Currency;

type BalanceOf<T, I> = <<T as pallet_alliance::Config<I>>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

pub struct ToParentTreasury<TreasuryAcc, TempAcc, T, I = ()>(
	PhantomData<(TreasuryAcc, TempAcc, T, I)>,
);

impl<TreasuryAcc, TempAcc, T, I: 'static> OnUnbalanced<NegativeImbalanceOf<T, I>>
	for ToParentTreasury<TreasuryAcc, TempAcc, T, I>
where
	TreasuryAcc: Get<AccountIdOf<T>>,
	TempAcc: Get<AccountIdOf<T>>,
	T: pallet_xcm::Config + frame_system::Config + pallet_alliance::Config<I>,
	[u8; 32]: From<AccountIdOf<T>>,
	BalanceOf<T, I>: Into<Fungibility>,
	<<T as frame_system::Config>::Origin as OriginTrait>::AccountId: From<AccountIdOf<T>>,
{
	fn on_unbalanced(amount: NegativeImbalanceOf<T, I>) {
		let temp_account: AccountIdOf<T> = TempAcc::get();
		let treasury_acc: AccountIdOf<T> = TreasuryAcc::get();
		let imbalance = amount.peek();

		<CurrencyOf<T, I>>::resolve_creating(&temp_account, amount);

		let result = pallet_xcm::Pallet::<T>::teleport_assets(
			<T as frame_system::Config>::Origin::signed(temp_account.into()),
			Box::new(Parent.into()),
			Box::new(
				Junction::AccountId32 { network: NetworkId::Any, id: treasury_acc.into() }
					.into()
					.into(),
			),
			Box::new((Parent, imbalance).into()),
			0,
		);

		match result {
			Err(err) => log::warn!("Failed to teleport slashed assets: {:?}", err),
			_ => (),
		};
	}
}
