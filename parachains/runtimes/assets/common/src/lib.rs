// Copyright (C) 2023 Parity Technologies (UK) Ltd.
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

#![cfg_attr(not(feature = "std"), no_std)]

pub mod assets_api;

use parachains_common::AssetIdForTrustBackedAssets;
use sp_std::prelude::Vec;
use xcm::latest::MultiLocation;
use xcm_builder::AsPrefixedGeneralIndex;
use xcm_executor::traits::{Convert, JustTry};

/// [`MultiLocation`] vs [`AssetIdForTrustBackedAssets`] converter for `TrustBackedAssets`
pub type AssetIdForTrustBackedAssetsConvert<TrustBackedAssetsPalletLocation> =
	AsPrefixedGeneralIndex<TrustBackedAssetsPalletLocation, AssetIdForTrustBackedAssets, JustTry>;

/// Helper function to convert collections with (`AssetId`, 'Balance') to (`MultiLocation`, 'Balance')
pub fn convert_asset_id<AssetId: Clone, Balance, ConvertAssetId>(
	assets_balances: Vec<(AssetId, Balance)>,
) -> Result<Vec<(MultiLocation, Balance)>, assets_api::AssetsAccessError>
where
	ConvertAssetId: Convert<MultiLocation, AssetId>,
{
	assets_balances
		.into_iter()
		.map(|(asset_id, balance)| {
			ConvertAssetId::reverse_ref(asset_id)
				.map(|converted_asset_id| (converted_asset_id, balance))
				.map_err(|_| assets_api::AssetsAccessError::AssetIdConversionFailed)
		})
		.collect()
}

#[cfg(test)]
mod tests {

	use super::*;
	use xcm::latest::prelude::*;

	frame_support::parameter_types! {
		pub TrustBackedAssetsPalletLocation: MultiLocation = MultiLocation::new(5, X1(PalletInstance(13)));
	}

	#[test]
	fn asset_id_for_trust_backed_assets_convert_works() {
		let local_asset_id = 123456789 as AssetIdForTrustBackedAssets;
		let expected_reverse_ref =
			MultiLocation::new(5, X2(PalletInstance(13), GeneralIndex(local_asset_id.into())));

		assert_eq!(
			AssetIdForTrustBackedAssetsConvert::<TrustBackedAssetsPalletLocation>::reverse_ref(
				local_asset_id
			)
			.unwrap(),
			expected_reverse_ref
		);
		assert_eq!(
			AssetIdForTrustBackedAssetsConvert::<TrustBackedAssetsPalletLocation>::convert_ref(
				expected_reverse_ref
			)
			.unwrap(),
			local_asset_id
		);
	}
}
