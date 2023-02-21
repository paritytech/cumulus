// This file is part of Substrate.

// Copyright (C) 2018-2022 Parity Technologies (UK) Ltd.
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

//! Runtime API definition for assets.

use codec::{Codec, Decode, Encode};
use frame_support::RuntimeDebug;
use sp_std::vec::Vec;
use xcm::latest::{MultiAsset, MultiLocation};
use xcm_executor::traits::Convert;

/// The possible errors that can happen querying the storage of assets.
#[derive(Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum AssetsAccessError {
	/// `MultiLocation` to `AssetId`/`ClassId` conversion failed.
	AssetIdConversionFailed,
	/// `u128` amount to currency `Balance` conversion failed.
	AmountToBalanceConversionFailed,
}

/// Helper function to convert collections with (`AssetId`, 'Balance') to (`MultiAsset`)
pub fn convert_asset<AssetId: Clone, Balance: Clone, ConvertAssetId, ConvertBalance>(
	assets_balances: Vec<(AssetId, Balance)>,
) -> Result<Vec<MultiAsset>, AssetsAccessError>
where
	ConvertAssetId: Convert<MultiLocation, AssetId>,
	ConvertBalance: Convert<u128, Balance>,
{
	assets_balances
		.into_iter()
		.map(|(asset_id, balance)| match ConvertAssetId::reverse_ref(asset_id) {
			Ok(asset_id_as_multilocation) => match ConvertBalance::reverse_ref(balance) {
				Ok(amount) => Ok((asset_id_as_multilocation, amount).into()),
				Err(_) => Err(AssetsAccessError::AmountToBalanceConversionFailed),
			},
			Err(_) => Err(AssetsAccessError::AssetIdConversionFailed),
		})
		.collect()
}

sp_api::decl_runtime_apis! {
	pub trait AssetsApi<AccountId>
	where
		AccountId: Codec,
	{
		/// Returns the list of `AssetId`s and corresponding balance that an `AccountId` has.
		fn query_account_balances(account: AccountId) -> Result<Vec<MultiAsset>, AssetsAccessError>;
	}
}
