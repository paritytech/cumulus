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

use xcm::latest::MultiAsset;
use xcm_executor::traits::{Error, MatchesFungibles};

// TODO:check-parameter - workaround for ConvertedConcreteId/MatchesFungibles and multiple FungiblesAdapter for AssetTransactor tuple (find a nicer way)
// TODO:check-parameter - waiting for https://github.com/paritytech/polkadot/pull/6739
//
// Problem is combination [`(FungiblesAdapter<Assets1, ConvertedConcreteId<..>, ...>, FungiblesAdapter<Assets2, ConvertedConcreteId<..>, ...>)`]
// where ConvertedConcreteId for ConvertAssetId returns AssetIdConversionFailed,
// but [`impl TransactAsset for Tuple {`] does continuation only for cases: `Err(XcmError::AssetNotFound) | Err(XcmError::Unimplemented) => ()`,
// so if first [`(FungiblesAdapter<Assets1, ConvertedConcreteId<..>, ...>`] fails on [`AssetIdConversionFailed`], then we dont give chance to other Adapter/FungiblesAdapter in tuple chain
// so it is not possible to have multiple pallet_assets instances with its own FungiblesAdapter
//
// so this wrapper just translate AssetIdConversionFailed->AssetNotFound and can give chance to other Adapter in tuple chain for AssetTransactor
//
// Possible fixes:
// 	1. extend cases `Err(XcmError::AssetNotFound) | Err(XcmError::Unimplemented) => ()` with `Err(XcmError::AssetIdConversionFailed)`
//  2. just rename nicely this wrapper
//
pub struct AssetIdConversionFailedToAssetNotFoundWrapper<Matcher>(
	sp_std::marker::PhantomData<Matcher>,
);

impl<Matcher: MatchesFungibles<AssetId, Balance>, AssetId: Clone, Balance: Clone>
	MatchesFungibles<AssetId, Balance> for AssetIdConversionFailedToAssetNotFoundWrapper<Matcher>
{
	fn matches_fungibles(asset: &MultiAsset) -> Result<(AssetId, Balance), Error> {
		match Matcher::matches_fungibles(asset) {
			Err(Error::AssetIdConversionFailed) => Err(Error::AssetNotHandled),
			result => result,
		}
	}
}
