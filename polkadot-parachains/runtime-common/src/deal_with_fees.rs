// Copyright (c) 2019 Alain Brenzikofer
// This file is part of Encointer
//
// Encointer is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Encointer is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Encointer.  If not, see <http://www.gnu.org/licenses/>.

use frame_support::traits::{Currency, Imbalance, OnUnbalanced};
use sp_runtime::sp_std::marker::PhantomData;

/// Type alias to conveniently refer to the `Currency::NegativeImbalance` associated type.
pub type NegativeImbalance<T> = <pallet_balances::Pallet<T> as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

/// Moves all the fees to the treasury.
///
/// This does only handle the native currency. The community currencies are managed by the
/// `pallet-asset-tx-payment`.
pub struct FeesToTreasury<Runtime>(PhantomData<Runtime>);

impl<Runtime> OnUnbalanced<NegativeImbalance<Runtime>> for FeesToTreasury<Runtime>
where
	Runtime: pallet_balances::Config + pallet_treasury::Config,
{
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance<Runtime>>) {
		if let Some(mut fees) = fees_then_tips.next() {
			// no burning, add all fees and tips to the treasury

			if let Some(tips) = fees_then_tips.next() {
				tips.merge_into(&mut fees);
			}
			<FeesToTreasury<Runtime> as OnUnbalanced<_>>::on_unbalanced(fees);
		}
	}
}
