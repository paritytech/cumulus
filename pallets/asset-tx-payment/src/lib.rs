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

//! # Asset Transaction Payment Pallet
//!
//! This pallet allows runtimes that include it to pay for transactions in assets other than the
//! main token of the chain.
//!
//! ## Overview
//! It does this by extending transactions to include an optional `AssetId` that specifies the asset
//! to be used for payment (defaulting to the native token on `None`). It expects an
//! `OnChargeAssetTransaction` implementation analogously to `pallet-transaction-payment`. The
//! included `FungiblesAdapter` (implementing `OnChargeAssetTransaction`) determines the fee amount
//! by converting the fee calculated by `pallet-transaction-payment` into the desired asset.
//!
//! ## Integration
//! This pallet wraps FRAME's transaction payment pallet and functions as a replacement. This means
//! you should include both pallets in your `construct_runtime` macro, but only include this
//! pallet's `SignedExtension` (`ChargeAssetTxPayment`).

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;

use codec::{Decode, Encode};
use frame_support::{
	dispatch::DispatchResult,
	traits::{
		tokens::{
			fungibles::{Balanced, CreditOf, Inspect},
			WithdrawConsequence,
		},
		Get, IsType,
	},
	weights::{DispatchClass, DispatchInfo, PostDispatchInfo},
	DefaultNoBound,
};
use pallet_transaction_payment::OnChargeTransaction;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{
		DispatchInfoOf, Dispatchable, PostDispatchInfoOf, SaturatedConversion, Saturating,
		SignedExtension, Zero,
	},
	transaction_validity::{
		InvalidTransaction, TransactionPriority, TransactionValidity, TransactionValidityError,
		ValidTransaction,
	},
	FixedPointOperand,
};

#[cfg(test)]
mod tests;

mod payment;
pub use payment::*;

// Type aliases used for interaction with `OnChargeTransaction`.
pub(crate) type OnChargeTransactionOf<T> =
	<T as pallet_transaction_payment::Config>::OnChargeTransaction;
// Balance type alias.
pub(crate) type BalanceOf<T> = <OnChargeTransactionOf<T> as OnChargeTransaction<T>>::Balance;
// Liquity info type alias.
pub(crate) type LiquidityInfoOf<T> =
	<OnChargeTransactionOf<T> as OnChargeTransaction<T>>::LiquidityInfo;

// Type alias used for interaction with fungibles (assets).
// Balance type alias.
pub(crate) type AssetBalanceOf<T> =
	<<T as Config>::Fungibles as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
/// Asset id type alias.
pub(crate) type AssetIdOf<T> =
	<<T as Config>::Fungibles as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;

// Type aliases used for interaction with `OnChargeAssetTransaction`.
// Balance type alias.
pub(crate) type ChargeAssetBalanceOf<T> =
	<<T as Config>::OnChargeAssetTransaction as OnChargeAssetTransaction<T>>::Balance;
// Asset id type alias.
pub(crate) type ChargeAssetIdOf<T> =
	<<T as Config>::OnChargeAssetTransaction as OnChargeAssetTransaction<T>>::AssetId;
// Liquity info type alias.
pub(crate) type ChargeAssetLiquidityOf<T> =
	<<T as Config>::OnChargeAssetTransaction as OnChargeAssetTransaction<T>>::LiquidityInfo;

#[derive(Encode, Decode, DefaultNoBound, TypeInfo)]
pub enum InitialPayment<T: Config> {
	Nothing,
	Native(LiquidityInfoOf<T>),
	Asset(CreditOf<T::AccountId, T::Fungibles>),
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_transaction_payment::Config {
		/// The fungibles instance used to pay for transactions in assets.
		type Fungibles: Balanced<Self::AccountId>;
		/// The actual transaction charging logic that charges the fees.
		type OnChargeAssetTransaction: OnChargeAssetTransaction<Self>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

/// Require the transactor pay for themselves and maybe include a tip to gain additional priority
/// in the queue. Allows paying via both `Currency` as well as `fungibles::Balanced`.
///
/// Wraps the transaction logic in `pallet_transaction_payment` and extends it with assets.
/// An asset id of `None` falls back to the underlying transaction payment via the native currency.
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ChargeAssetTxPayment<T: Config>(
	#[codec(compact)] BalanceOf<T>,
	Option<ChargeAssetIdOf<T>>,
);

impl<T: Config> ChargeAssetTxPayment<T>
where
	T::Call: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
	AssetBalanceOf<T>: Send + Sync + FixedPointOperand,
	BalanceOf<T>: Send + Sync + FixedPointOperand + IsType<ChargeAssetBalanceOf<T>>,
	ChargeAssetIdOf<T>: Send + Sync,
	CreditOf<T::AccountId, T::Fungibles>: IsType<ChargeAssetLiquidityOf<T>>,
{
	/// utility constructor. Used only in client/factory code.
	pub fn from(fee: BalanceOf<T>, asset_id: Option<ChargeAssetIdOf<T>>) -> Self {
		Self(fee, asset_id)
	}

	/// Fee withdrawal logic that dispatches to either `OnChargeAssetTransaction` or `OnChargeTransaction`.
	fn withdraw_fee(
		&self,
		who: &T::AccountId,
		call: &T::Call,
		info: &DispatchInfoOf<T::Call>,
		len: usize,
	) -> Result<(BalanceOf<T>, InitialPayment<T>), TransactionValidityError> {
		let tip = self.0;
		let fee = pallet_transaction_payment::Pallet::<T>::compute_fee(len as u32, info, tip);

		debug_assert!(tip <= fee, "tip should be included in the computed fee");

		if fee.is_zero() {
			return Ok((fee, InitialPayment::Nothing))
		}

		let maybe_asset_id = self.1;
		if let Some(asset_id) = maybe_asset_id {
			T::OnChargeAssetTransaction::withdraw_fee(
				who,
				call,
				info,
				asset_id,
				fee.into(),
				tip.into(),
			)
			.map(|i| (fee, InitialPayment::Asset(i.into())))
		} else {
			<OnChargeTransactionOf<T> as OnChargeTransaction<T>>::withdraw_fee(
				who, call, info, fee, tip,
			)
			.map(|i| (fee, InitialPayment::Native(i)))
			.map_err(|_| -> TransactionValidityError { InvalidTransaction::Payment.into() })
		}
	}

	/// Get an appropriate priority for a transaction with the given length and info.
	///
	/// This will try and optimise the `fee/weight` `fee/length`, whichever is consuming more of the
	/// maximum corresponding limit.
	///
	/// For example, if a transaction consumed 1/4th of the block length and half of the weight, its
	/// final priority is `fee * min(2, 4) = fee * 2`. If it consumed `1/4th` of the block length
	/// and the entire block weight `(1/1)`, its priority is `fee * min(1, 4) = fee * 1`. This means
	///  that the transaction which consumes more resources (either length or weight) with the same
	/// `fee` ends up having lower priority.
	// NOTE: copied from `pallet_transaction_payment`
	fn get_priority(
		len: usize,
		info: &DispatchInfoOf<T::Call>,
		final_fee: BalanceOf<T>,
	) -> TransactionPriority {
		let weight_saturation = T::BlockWeights::get().max_block / info.weight.max(1);
		let max_block_length = *T::BlockLength::get().max.get(DispatchClass::Normal);
		let len_saturation = max_block_length as u64 / (len as u64).max(1);
		let coefficient: BalanceOf<T> =
			weight_saturation.min(len_saturation).saturated_into::<BalanceOf<T>>();
		final_fee.saturating_mul(coefficient).saturated_into::<TransactionPriority>()
	}
}

impl<T: Config> sp_std::fmt::Debug for ChargeAssetTxPayment<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "ChargeAssetTxPayment<{:?}, {:?}>", self.0, self.1.encode())
	}
	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl<T: Config> SignedExtension for ChargeAssetTxPayment<T>
where
	T::Call: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
	AssetBalanceOf<T>: Send + Sync + FixedPointOperand,
	BalanceOf<T>: Send + Sync + From<u64> + FixedPointOperand + IsType<ChargeAssetBalanceOf<T>>,
	ChargeAssetIdOf<T>: Send + Sync,
	CreditOf<T::AccountId, T::Fungibles>: IsType<ChargeAssetLiquidityOf<T>>,
{
	const IDENTIFIER: &'static str = "ChargeAssetTxPayment";
	type AccountId = T::AccountId;
	type Call = T::Call;
	type AdditionalSigned = ();
	type Pre = (
		// tip
		BalanceOf<T>,
		// who paid the fee
		Self::AccountId,
		// imbalance resulting from withdrawing the fee
		InitialPayment<T>,
	);
	fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
		Ok(())
	}

	fn validate(
		&self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> TransactionValidity {
		let (fee, _) = self.withdraw_fee(who, call, info, len)?;
		Ok(ValidTransaction { priority: Self::get_priority(len, info, fee), ..Default::default() })
	}

	fn pre_dispatch(
		self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> Result<Self::Pre, TransactionValidityError> {
		let (_fee, initial_payment) = self.withdraw_fee(who, call, info, len)?;
		Ok((self.0, who.clone(), initial_payment))
	}

	fn post_dispatch(
		pre: Self::Pre,
		info: &DispatchInfoOf<Self::Call>,
		post_info: &PostDispatchInfoOf<Self::Call>,
		len: usize,
		_result: &DispatchResult,
	) -> Result<(), TransactionValidityError> {
		let (tip, who, initial_payment) = pre;
		let actual_fee = pallet_transaction_payment::Pallet::<T>::compute_actual_fee(
			len as u32, info, post_info, tip,
		);
		match initial_payment {
			InitialPayment::Native(already_withdrawn) => {
				<OnChargeTransactionOf<T> as OnChargeTransaction<T>>::correct_and_deposit_fee(
					&who,
					info,
					post_info,
					actual_fee,
					tip,
					already_withdrawn,
				)?;
			},
			InitialPayment::Asset(already_withdrawn) => {
				T::OnChargeAssetTransaction::correct_and_deposit_fee(
					&who,
					info,
					post_info,
					actual_fee.into(),
					tip.into(),
					already_withdrawn.into(),
				)?;
			},
			InitialPayment::Nothing => {
				debug_assert!(
					actual_fee.is_zero(),
					"actual fee should be zero if initial fee was zero."
				);
				debug_assert!(tip.is_zero(), "tip should be zero if initial fee was zero.");
			},
		}

		Ok(())
	}
}
