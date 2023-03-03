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

pub mod foreign_creators;
pub mod fungible_conversion;
pub mod matching;
pub mod runtime_api;

use crate::matching::StartsWith;
use parachains_common::AssetIdForTrustBackedAssets;
use xcm_builder::{AsPrefixedGeneralIndex, MatchedConvertedConcreteId};
use xcm_executor::traits::JustTry;

/// `MultiLocation` vs `AssetIdForTrustBackedAssets` converter for `TrustBackedAssets`
pub type AssetIdForTrustBackedAssetsConvert<TrustBackedAssetsPalletLocation> =
	AsPrefixedGeneralIndex<TrustBackedAssetsPalletLocation, AssetIdForTrustBackedAssets, JustTry>;

/// [`MatchedConvertedConcreteId`] converter dedicated for `TrustBackedAssets`
pub type TrustBackedAssetsConvertedConcreteId<TrustBackedAssetsPalletLocation, Balance> =
	MatchedConvertedConcreteId<
		AssetIdForTrustBackedAssets,
		Balance,
		StartsWith<TrustBackedAssetsPalletLocation>,
		AssetIdForTrustBackedAssetsConvert<TrustBackedAssetsPalletLocation>,
		JustTry,
	>;

#[cfg(test)]
mod tests {

	use super::*;
	use xcm::latest::prelude::*;
	use xcm_executor::traits::{Convert, Error as MatchError, MatchesFungibles};

	#[test]
	fn asset_id_for_trust_backed_assets_convert_works() {
		frame_support::parameter_types! {
			pub TrustBackedAssetsPalletLocation: MultiLocation = MultiLocation::new(5, X1(PalletInstance(13)));
		}
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

	#[test]
	fn trust_backed_assets_match_fungibles_works() {
		frame_support::parameter_types! {
			pub TrustBackedAssetsPalletLocation: MultiLocation = MultiLocation::new(0, X1(PalletInstance(13)));
		}
		// setup convert
		type TrustBackAssetsConvert =
			TrustBackedAssetsConvertedConcreteId<TrustBackedAssetsPalletLocation, u128>;

		let test_data = vec![
			// missing GeneralIndex
			(ma_1000(0, X1(PalletInstance(13))), Err(MatchError::AssetIdConversionFailed)),
			(
				ma_1000(0, X2(PalletInstance(13), GeneralKey { data: [0; 32], length: 32 })),
				Err(MatchError::AssetIdConversionFailed),
			),
			(
				ma_1000(0, X2(PalletInstance(13), Parachain(1000))),
				Err(MatchError::AssetIdConversionFailed),
			),
			// OK
			(ma_1000(0, X2(PalletInstance(13), GeneralIndex(1234))), Ok((1234, 1000))),
			(
				ma_1000(0, X3(PalletInstance(13), GeneralIndex(1234), GeneralIndex(2222))),
				Ok((1234, 1000)),
			),
			(
				ma_1000(
					0,
					X4(
						PalletInstance(13),
						GeneralIndex(1234),
						GeneralIndex(2222),
						GeneralKey { data: [0; 32], length: 32 },
					),
				),
				Ok((1234, 1000)),
			),
			// wrong pallet instance
			(
				ma_1000(0, X2(PalletInstance(77), GeneralIndex(1234))),
				Err(MatchError::AssetNotHandled),
			),
			(
				ma_1000(0, X3(PalletInstance(77), GeneralIndex(1234), GeneralIndex(2222))),
				Err(MatchError::AssetNotHandled),
			),
			// wrong parent
			(
				ma_1000(1, X2(PalletInstance(13), GeneralIndex(1234))),
				Err(MatchError::AssetNotHandled),
			),
			(
				ma_1000(1, X3(PalletInstance(13), GeneralIndex(1234), GeneralIndex(2222))),
				Err(MatchError::AssetNotHandled),
			),
			(
				ma_1000(1, X2(PalletInstance(77), GeneralIndex(1234))),
				Err(MatchError::AssetNotHandled),
			),
			(
				ma_1000(1, X3(PalletInstance(77), GeneralIndex(1234), GeneralIndex(2222))),
				Err(MatchError::AssetNotHandled),
			),
			// wrong parent
			(
				ma_1000(2, X2(PalletInstance(13), GeneralIndex(1234))),
				Err(MatchError::AssetNotHandled),
			),
			(
				ma_1000(2, X3(PalletInstance(13), GeneralIndex(1234), GeneralIndex(2222))),
				Err(MatchError::AssetNotHandled),
			),
			// missing GeneralIndex
			(ma_1000(0, X1(PalletInstance(77))), Err(MatchError::AssetNotHandled)),
			(ma_1000(1, X1(PalletInstance(13))), Err(MatchError::AssetNotHandled)),
			(ma_1000(2, X1(PalletInstance(13))), Err(MatchError::AssetNotHandled)),
		];

		for (multi_asset, expected_result) in test_data {
			assert_eq!(
				<TrustBackAssetsConvert as MatchesFungibles<AssetIdForTrustBackedAssets, u128>>::matches_fungibles(&multi_asset),
				expected_result, "multi_asset: {:?}", multi_asset);
		}
	}

	// Create MultiAsset
	fn ma_1000(parents: u8, interior: Junctions) -> MultiAsset {
		(MultiLocation::new(parents, interior), 1000).into()
	}
}
