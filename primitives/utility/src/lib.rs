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
	traits::tokens::{fungibles, fungibles::Inspect},
	weights::Weight,
};
use sp_runtime::{
	traits::{Saturating, Zero},
	SaturatedConversion,
};

use sp_std::marker::PhantomData;
use xcm::{latest::prelude::*, WrapVersion};
use xcm_builder::TakeRevenue;
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

/// Contains information to handle refund/payment for xcm-execution
#[derive(Clone, Eq, PartialEq, Debug)]
struct AssetTraderRefunder<Balance, AssetId> {
	// The amount of weight bought minus the weigh already refunded
	weight: Weight,
	// The balance paid for the weight bought minus the balance already refunded
	balance: Balance,
	// The multilocation representing the fungible asset in which fees were paid
	location: MultiLocation,
	// The local assetId the aforementioned location corresponds to
	asset: AssetId,
}

/// Charges for exercution in the first asset of those selected for fee payment
/// First tries to convert the this MultiAsset into a local assetId
/// Then charges for this assetId as described by FeeCharger
/// Weight, paid balance, local asset Id and the multilocation is stored for
/// later refund purposes
/// Important: Errors if the Trader is being called twice by 2 BuyExecution instructions
/// Alternatively we could just return payment in the aforementioned case
pub struct TakeFirstAssetTrader<
	AccountId,
	FeeCharger: ChargeWeightInFungibles<AccountId, Assets>,
	Matcher: MatchesFungibles<Assets::AssetId, Assets::Balance>,
	Assets: fungibles::Mutate<AccountId> + fungibles::Transfer<AccountId> + fungibles::Balanced<AccountId>,
	HandleRefund: TakeRevenue,
>(
	Option<AssetTraderRefunder<Assets::Balance, Assets::AssetId>>,
	PhantomData<(AccountId, FeeCharger, Matcher, Assets, HandleRefund)>,
);
impl<
		AccountId,
		FeeCharger: ChargeWeightInFungibles<AccountId, Assets>,
		Matcher: MatchesFungibles<Assets::AssetId, Assets::Balance>,
		Assets: fungibles::Mutate<AccountId>
			+ fungibles::Transfer<AccountId>
			+ fungibles::Balanced<AccountId>,
		HandleRefund: TakeRevenue,
	> WeightTrader for TakeFirstAssetTrader<AccountId, FeeCharger, Matcher, Assets, HandleRefund>
{
	fn new() -> Self {
		Self(None, PhantomData)
	}
	// We take first asset
	// Check whether we can convert fee to asset_fee (is_sufficient, min_deposit)
	// If everything goes well, we charge.
	fn buy_weight(
		&mut self,
		weight: Weight,
		payment: xcm_executor::Assets,
	) -> Result<xcm_executor::Assets, XcmError> {
		log::trace!(target: "xcm::weight", "TakeFirstAssetTrader::buy_weight weight: {:?}, payment: {:?}", weight, payment);

		// Make sure we dont enter twice
		if self.0.is_some() {
			return Err(XcmError::NotWithdrawable)
		}

		// We take the very first multiasset from payment
		let multiassets: MultiAssets = payment.clone().into();

		// Take the first asset from the selected MultiAssets
		let first = multiassets.get(0).ok_or(XcmError::TooExpensive)?;

		// Get the local asset id in which we can pay for fees
		let (local_asset_id, _) =
			Matcher::matches_fungibles(&first).map_err(|_| XcmError::AssetNotFound)?;

		// Calculate how much we should charge in the asset_id for such amount of weight
		let asset_balance = FeeCharger::charge_weight_in_fungibles(local_asset_id, weight)?;

		match first {
			// Not relevant match, as matches_fungibles should have already verified this above
			MultiAsset {
				id: xcm::latest::AssetId::Concrete(location),
				fun: Fungibility::Fungible(_),
			} => {
				// Convert asset_balance to u128
				let u128_amount: u128 = asset_balance.try_into().map_err(|_| XcmError::Overflow)?;
				// Construct required payment
				let required = (Concrete(location.clone()), u128_amount).into();
				// Substract payment
				let unused = payment.checked_sub(required).map_err(|_| XcmError::TooExpensive)?;

				// record weight, balance, multilocation and assetId
				self.0 = Some(AssetTraderRefunder {
					weight,
					balance: asset_balance,
					location: location.clone(),
					asset: local_asset_id,
				});
				Ok(unused)
			},
			_ => return Err(XcmError::TooExpensive),
		}
	}

	fn refund_weight(&mut self, weight: Weight) -> Option<MultiAsset> {
		log::trace!(target: "xcm::weight", "TakeFirstAssetTrader::refund_weight weight: {:?}", weight);
		if let Some(mut asset_trader) = self.0.clone() {
			let weight = weight.min(asset_trader.weight);

			// Calculate asset_balance
			// This read should have already be cached in buy_weight
			let asset_balance =
				FeeCharger::charge_weight_in_fungibles(asset_trader.asset, weight).ok()?;

			// Substract from existing weight and balance
			asset_trader.weight = asset_trader.weight.saturating_sub(weight);
			asset_trader.balance = asset_trader.balance.saturating_sub(asset_balance);
			self.0 = Some(asset_trader.clone());

			let asset_balance: u128 = asset_balance.saturated_into();
			if asset_balance > 0 {
				Some((asset_trader.location, asset_balance).into())
			} else {
				None
			}
		} else {
			None
		}
	}
}

impl<
		AccountId,
		FeeCharger: ChargeWeightInFungibles<AccountId, Assets>,
		Matcher: MatchesFungibles<Assets::AssetId, Assets::Balance>,
		Assets: fungibles::Mutate<AccountId>
			+ fungibles::Transfer<AccountId>
			+ fungibles::Balanced<AccountId>,
		HandleRefund: TakeRevenue,
	> Drop for TakeFirstAssetTrader<AccountId, FeeCharger, Matcher, Assets, HandleRefund>
{
	fn drop(&mut self) {
		if let Some(asset_trader) = self.0.clone() {
			let u128_amount: u128 =
				asset_trader.balance.try_into().map_err(|_| XcmError::Overflow).unwrap();
			HandleRefund::take_revenue(MultiAsset {
				id: asset_trader.location.clone().into(),
				fun: Fungibility::Fungible(u128_amount),
			});
		}
	}
}

/// XCM fee depositor to which we implement the TakeRevenue trait
/// It receives a fungibles::Mutate implemented argument, a matcher to convert MultiAsset into
/// AssetId and amount, and the fee receiver account
pub struct XcmFeesToAccount<Assets, Matcher, AccountId, ReceiverAccount>(
	PhantomData<(Assets, Matcher, AccountId, ReceiverAccount)>,
);
impl<
		Assets: fungibles::Mutate<AccountId>,
		Matcher: MatchesFungibles<Assets::AssetId, Assets::Balance>,
		AccountId: Clone,
		ReceiverAccount: frame_support::traits::Get<Option<AccountId>>,
	> TakeRevenue for XcmFeesToAccount<Assets, Matcher, AccountId, ReceiverAccount>
{
	fn take_revenue(revenue: MultiAsset) {
		match Matcher::matches_fungibles(&revenue) {
			Ok((asset_id, amount)) =>
				if let Some(receiver) = ReceiverAccount::get() {
					if !amount.is_zero() {
						let ok = Assets::mint_into(asset_id, &receiver, amount).is_ok();
						debug_assert!(ok, "`mint_into` cannot generally fail; qed");
					}
				},
			Err(_) => log::debug!(
				target: "xcm",
				"take revenue failed matching fungible"
			),
		}
	}
}

/// ChargeWeightInFungibles trait, which converts a given amount of weight
/// and an assetId, and it returns the balance amount that should be charged
/// in such assetId for that amount of weight
pub trait ChargeWeightInFungibles<AccountId, Assets: fungibles::Inspect<AccountId>> {
	fn charge_weight_in_fungibles(
		asset_id: <Assets as Inspect<AccountId>>::AssetId,
		weight: Weight,
	) -> Result<<Assets as Inspect<AccountId>>::Balance, XcmError>;
}
