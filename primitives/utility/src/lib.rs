// Copyright 2020-2021 Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

//! Helper datatypes for cumulus. This includes the [`ParentAsUmp`] routing type which will route
//! messages into an [`UpwardMessageSender`] if the destination is `Parent`.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use cumulus_primitives_core::UpwardMessageSender;
use frame_support::{
	pallet_prelude::Get,
	traits::tokens::{currency::Currency as CurrencyT, fungibles, BalanceConversion},
	weights::{Weight, WeightToFeePolynomial},
};
use pallet_asset_tx_payment::HandleCredit as HandleCreditT;
use sp_runtime::{
	traits::{Saturating, Zero},
	SaturatedConversion,
};
use sp_std::marker::PhantomData;
use xcm::{latest::prelude::*, WrapVersion};
use xcm_executor::traits::{MatchesFungibles, WeightTrader};
/// Xcm router which recognises the `Parent` destination and handles it by sending the message into
/// the given UMP `UpwardMessageSender` implementation. Thus this essentially adapts an
/// `UpwardMessageSender` trait impl into a `SendXcm` trait impl.
///
/// NOTE: This is a pretty dumb "just send it" router; we will probably want to introduce queuing
/// to UMP eventually and when we do, the pallet which implements the queuing will be responsible
/// for the `SendXcm` implementation.
pub struct ParentAsUmp<T, W>(PhantomData<(T, W)>);
impl<T: UpwardMessageSender, W: WrapVersion> SendXcm for ParentAsUmp<T, W> {
	fn send_xcm(dest: impl Into<MultiLocation>, msg: Xcm<()>) -> Result<(), SendError> {
		let dest = dest.into();

		if dest.contains_parents_only(1) {
			// An upward message for the relay chain.
			let versioned_xcm =
				W::wrap_version(&dest, msg).map_err(|()| SendError::DestinationUnsupported)?;
			let data = versioned_xcm.encode();

			T::send_upward_message(data).map_err(|e| SendError::Transport(e.into()))?;

			Ok(())
		} else {
			// Anything else is unhandled. This includes a message this is meant for us.
			Err(SendError::CannotReachDestination(dest, msg))
		}
	}
}

pub struct TakeFirstAssetTrader<
	WeightToFee: WeightToFeePolynomial<Balance = Currency::Balance>,
	AccountId,
	Currency: CurrencyT<AccountId>,
	CON: BalanceConversion<Currency::Balance, Assets::AssetId, Assets::Balance>,
	Matcher: MatchesFungibles<Assets::AssetId, Assets::Balance>,
	Assets: fungibles::Mutate<AccountId> + fungibles::Transfer<AccountId> + fungibles::Balanced<AccountId>,
	HandleCredit: HandleCreditT<AccountId, Assets>,
>(
	Weight,
	Assets::Balance,
	Option<(MultiLocation, Assets::AssetId)>,
	PhantomData<(WeightToFee, AccountId, Currency, CON, Matcher, Assets, HandleCredit)>,
);
impl<
		WeightToFee: WeightToFeePolynomial<Balance = Currency::Balance>,
		AccountId,
		Currency: CurrencyT<AccountId>,
		CON: BalanceConversion<Currency::Balance, Assets::AssetId, Assets::Balance>,
		Matcher: MatchesFungibles<Assets::AssetId, Assets::Balance>,
		Assets: fungibles::Mutate<AccountId>
			+ fungibles::Transfer<AccountId>
			+ fungibles::Balanced<AccountId>,
		HandleCredit: HandleCreditT<AccountId, Assets>,
	> WeightTrader
	for TakeFirstAssetTrader<
		WeightToFee,
		AccountId,
		Currency,
		CON,
		Matcher,
		Assets,
		HandleCredit,
	>
{
	fn new() -> Self {
		Self(0, Zero::zero(), None, PhantomData)
	}

	// We take first asset
	// TODO: do we want to be iterating over all potential assets in payments?
	// Check whether we can convert fee to asset_fee (is_sufficient, min_deposit)
	// If everything goes well, we charge.
	fn buy_weight(
		&mut self,
		weight: Weight,
		payment: xcm_executor::Assets,
	) -> Result<xcm_executor::Assets, XcmError> {
		log::trace!(target: "xcm::weight", "TakeFirstAssetTrader::buy_weight weight: {:?}, payment: {:?}", weight, payment);
		// Based on weight bought, calculate how much native token fee we need to pay
		let amount = WeightToFee::calc(&weight);
		// We take the very first multiasset from payment
		// TODO: revisit this clone
		let multiassets: MultiAssets = payment.clone().into();
		let first = multiassets.get(0).ok_or(XcmError::TooExpensive)?;
		// We get the local asset id in which we can pay for fees
		let (local_asset_id, _) =
			Matcher::matches_fungibles(&first).map_err(|_| XcmError::TooExpensive)?;
		// it reads the min_balance of the asset
		// potential db read
		// Already should have been read with WithdrawAsset in case of pallet-assets
		let asset_balance =
			CON::to_asset_balance(amount, local_asset_id).map_err(|_| XcmError::TooExpensive)?;

		match first {
			// Not relevant match, as matches_fungibles should have already verified this above
			MultiAsset {
				id: xcm::latest::AssetId::Concrete(location),
				fun: Fungibility::Fungible(_),
			} => {
				// convert asset_balance to u128
				let u128_amount: u128 = asset_balance.try_into().map_err(|_| XcmError::Overflow)?;
				// Construct required payment
				let required = (Concrete(location.clone()), u128_amount).into();
				// Substract payment
				let unused = payment.checked_sub(required).map_err(|_| XcmError::TooExpensive)?;
				// record weight
				self.0 = self.0.saturating_add(weight);
				// record amount
				self.1 = self.1.saturating_add(asset_balance);
				// record multilocation and local_asset_id
				self.2 = Some((location.clone(), local_asset_id));
				Ok(unused)
			},
			_ => return Err(XcmError::TooExpensive),
		}
	}

	fn refund_weight(&mut self, weight: Weight) -> Option<MultiAsset> {
		log::trace!(target: "xcm::weight", "UsingComponents::refund_weight weight: {:?}", weight);
		let weight = weight.min(self.0);
		let amount = WeightToFee::calc(&weight);
		// If not None
		if let Some((asset_location, local_asset_id)) = self.2.clone() {
			// Calculate asset_balance
			// This read should have already be cached in buy_weight
			let asset_balance = CON::to_asset_balance(amount, local_asset_id).ok()?;
			self.0 -= weight;
			self.1 = self.1.saturating_sub(asset_balance);

			let asset_balance: u128 = asset_balance.saturated_into();
			if asset_balance > 0 {
				Some((asset_location, asset_balance).into())
			} else {
				None
			}
		} else {
			None
		}
	}
}

impl<
		WeightToFee: WeightToFeePolynomial<Balance = Currency::Balance>,
		AccountId,
		Currency: CurrencyT<AccountId>,
		CON: BalanceConversion<Currency::Balance, Assets::AssetId, Assets::Balance>,
		Matcher: MatchesFungibles<Assets::AssetId, Assets::Balance>,
		Assets: fungibles::Mutate<AccountId>
			+ fungibles::Transfer<AccountId>
			+ fungibles::Balanced<AccountId>,
		HandleCredit: HandleCreditT<AccountId, Assets>,
	> Drop
	for TakeFirstAssetTrader<
		WeightToFee,
		AccountId,
		Currency,
		CON,
		Matcher,
		Assets,
		HandleCredit,
	>
{
	fn drop(&mut self) {
		if let Some((_, local_asset_id)) = self.2 {
			HandleCredit::handle_credit(Assets::issue(local_asset_id, self.1));
		}
	}
}
