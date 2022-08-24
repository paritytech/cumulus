// Copyright 2022 Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Cumulus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

mod pallet_xcm_benchmarks_fungible;
mod pallet_xcm_benchmarks_generic;

use crate::Runtime;
use frame_support::weights::Weight;
use sp_std::prelude::*;
use xcm::{latest::prelude::*, DoubleEncoded};

use pallet_xcm_benchmarks_fungible::WeightInfo as XcmFungibleWeight;
use pallet_xcm_benchmarks_generic::WeightInfo as XcmGeneric;

trait WeighMultiAssets {
	fn weigh_multi_assets(&self, weight: Weight) -> Weight;
}

const MAX_ASSETS: u32 = 100;

impl WeighMultiAssets for MultiAssetFilter {
	fn weigh_multi_assets(&self, weight: Weight) -> Weight {
		match self {
			Self::Definite(assets) =>
				(assets.inner().into_iter().count() as Weight).saturating_mul(weight),
			Self::Wild(_) => (MAX_ASSETS as Weight).saturating_mul(weight),
		}
	}
}

impl WeighMultiAssets for MultiAssets {
	fn weigh_multi_assets(&self, weight: Weight) -> Weight {
		(self.inner().into_iter().count() as Weight).saturating_mul(weight)
	}
}

pub struct StatemintXcmWeight<Call>(core::marker::PhantomData<Call>);
impl<Call> XcmWeightInfo<Call> for StatemintXcmWeight<Call> {
	fn withdraw_asset(assets: &MultiAssets) -> Weight {
		assets.weigh_multi_assets(XcmFungibleWeight::<Runtime>::withdraw_asset())
	}
	// Currently there is no trusted reserve
	fn reserve_asset_deposited(_assets: &MultiAssets) -> Weight {
		unimplemented!()
	}
	fn receive_teleported_asset(assets: &MultiAssets) -> Weight {
		assets.weigh_multi_assets(XcmFungibleWeight::<Runtime>::receive_teleported_asset())
	}
	fn query_response(_query_id: &u64, _response: &Response, _max_weight: &u64) -> Weight {
		XcmGeneric::<Runtime>::query_response()
	}
	fn transfer_asset(assets: &MultiAssets, _dest: &MultiLocation) -> Weight {
		assets.weigh_multi_assets(XcmFungibleWeight::<Runtime>::transfer_asset())
	}
	fn transfer_reserve_asset(
		assets: &MultiAssets,
		_dest: &MultiLocation,
		_xcm: &Xcm<()>,
	) -> Weight {
		assets.weigh_multi_assets(XcmFungibleWeight::<Runtime>::transfer_reserve_asset())
	}
	fn transact(
		_origin_type: &OriginKind,
		_require_weight_at_most: &u64,
		_call: &DoubleEncoded<Call>,
	) -> Weight {
		XcmGeneric::<Runtime>::transact()
	}
	fn hrmp_new_channel_open_request(
		_sender: &u32,
		_max_message_size: &u32,
		_max_capacity: &u32,
	) -> Weight {
		// XCM Executor does not currently support HRMP channel operations
		Weight::MAX
	}
	fn hrmp_channel_accepted(_recipient: &u32) -> Weight {
		// XCM Executor does not currently support HRMP channel operations
		Weight::MAX
	}
	fn hrmp_channel_closing(_initiator: &u32, _sender: &u32, _recipient: &u32) -> Weight {
		// XCM Executor does not currently support HRMP channel operations
		Weight::MAX
	}
	fn clear_origin() -> Weight {
		XcmGeneric::<Runtime>::clear_origin()
	}
	fn descend_origin(_who: &InteriorMultiLocation) -> Weight {
		XcmGeneric::<Runtime>::descend_origin()
	}
	fn report_error(
		_query_id: &QueryId,
		_dest: &MultiLocation,
		_max_response_weight: &u64,
	) -> Weight {
		XcmGeneric::<Runtime>::report_error()
	}

	fn deposit_asset(
		assets: &MultiAssetFilter,
		_max_assets: &u32, // TODO use max assets?
		_dest: &MultiLocation,
	) -> Weight {
		assets.weigh_multi_assets(XcmFungibleWeight::<Runtime>::deposit_asset())
	}
	fn deposit_reserve_asset(
		assets: &MultiAssetFilter,
		_max_assets: &u32, // TODO use max assets?
		_dest: &MultiLocation,
		_xcm: &Xcm<()>,
	) -> Weight {
		assets.weigh_multi_assets(XcmFungibleWeight::<Runtime>::deposit_reserve_asset())
	}
	fn exchange_asset(_give: &MultiAssetFilter, _receive: &MultiAssets) -> Weight {
		Weight::MAX // todo fix
	}
	fn initiate_reserve_withdraw(
		assets: &MultiAssetFilter,
		_reserve: &MultiLocation,
		_xcm: &Xcm<()>,
	) -> Weight {
		assets.weigh_multi_assets(XcmGeneric::<Runtime>::initiate_reserve_withdraw())
	}
	fn initiate_teleport(
		assets: &MultiAssetFilter,
		_dest: &MultiLocation,
		_xcm: &Xcm<()>,
	) -> Weight {
		assets.weigh_multi_assets(XcmFungibleWeight::<Runtime>::initiate_teleport())
	}
	fn query_holding(
		_query_id: &u64,
		_dest: &MultiLocation,
		_assets: &MultiAssetFilter,
		_max_response_weight: &u64,
	) -> Weight {
		XcmGeneric::<Runtime>::query_holding()
	}
	fn buy_execution(_fees: &MultiAsset, _weight_limit: &WeightLimit) -> Weight {
		XcmGeneric::<Runtime>::buy_execution()
	}
	fn refund_surplus() -> Weight {
		XcmGeneric::<Runtime>::refund_surplus()
	}
	fn set_error_handler(_xcm: &Xcm<Call>) -> Weight {
		XcmGeneric::<Runtime>::set_error_handler()
	}
	fn set_appendix(_xcm: &Xcm<Call>) -> Weight {
		XcmGeneric::<Runtime>::set_appendix()
	}
	fn clear_error() -> Weight {
		XcmGeneric::<Runtime>::clear_error()
	}
	fn claim_asset(_assets: &MultiAssets, _ticket: &MultiLocation) -> Weight {
		XcmGeneric::<Runtime>::claim_asset()
	}
	fn trap(_code: &u64) -> Weight {
		XcmGeneric::<Runtime>::trap()
	}
	fn subscribe_version(_query_id: &QueryId, _max_response_weight: &u64) -> Weight {
		XcmGeneric::<Runtime>::subscribe_version()
	}
	fn unsubscribe_version() -> Weight {
		XcmGeneric::<Runtime>::unsubscribe_version()
	}
}
