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
use pallet_xcm_benchmarks_fungible::WeightInfo as XcmFungibleWeight;
use pallet_xcm_benchmarks_generic::WeightInfo as XcmGeneric;
use sp_std::{cmp, prelude::*};
use xcm::{
	latest::{prelude::*, Weight as XCMWeight},
	DoubleEncoded,
};

trait WeighMultiAssets {
	fn weigh_multi_assets(&self, weight: Weight) -> XCMWeight;
}

const MAX_ASSETS: u32 = 100;

impl WeighMultiAssets for MultiAssetFilter {
	fn weigh_multi_assets(&self, weight: Weight) -> XCMWeight {
		let weight = match self {
			Self::Definite(assets) =>
				weight.saturating_mul(assets.inner().into_iter().count() as u64),
			Self::Wild(_) => weight.saturating_mul(MAX_ASSETS as u64),
		};
		weight.ref_time()
	}
}

impl WeighMultiAssets for MultiAssets {
	fn weigh_multi_assets(&self, weight: Weight) -> XCMWeight {
		weight.saturating_mul(self.inner().into_iter().count() as u64).ref_time()
	}
}

pub struct StatemineXcmWeight<Call>(core::marker::PhantomData<Call>);
impl<Call> XcmWeightInfo<Call> for StatemineXcmWeight<Call> {
	fn withdraw_asset(assets: &MultiAssets) -> XCMWeight {
		assets.weigh_multi_assets(XcmFungibleWeight::<Runtime>::withdraw_asset())
	}
	// Currently there is no trusted reserve
	fn reserve_asset_deposited(_assets: &MultiAssets) -> XCMWeight {
		u64::MAX
	}
	fn receive_teleported_asset(assets: &MultiAssets) -> XCMWeight {
		assets.weigh_multi_assets(XcmFungibleWeight::<Runtime>::receive_teleported_asset())
	}
	fn query_response(
		_query_id: &u64,
		_response: &Response,
		_max_weight: &u64,
		_querier: &Option<MultiLocation>,
	) -> XCMWeight {
		XcmGeneric::<Runtime>::query_response().ref_time()
	}
	fn transfer_asset(assets: &MultiAssets, _dest: &MultiLocation) -> XCMWeight {
		assets.weigh_multi_assets(XcmFungibleWeight::<Runtime>::transfer_asset())
	}
	fn transfer_reserve_asset(
		assets: &MultiAssets,
		_dest: &MultiLocation,
		_xcm: &Xcm<()>,
	) -> XCMWeight {
		assets.weigh_multi_assets(XcmFungibleWeight::<Runtime>::transfer_reserve_asset())
	}
	fn transact(
		_origin_type: &OriginKind,
		_require_weight_at_most: &u64,
		_call: &DoubleEncoded<Call>,
	) -> XCMWeight {
		XcmGeneric::<Runtime>::transact().ref_time()
	}
	fn hrmp_new_channel_open_request(
		_sender: &u32,
		_max_message_size: &u32,
		_max_capacity: &u32,
	) -> XCMWeight {
		// XCM Executor does not currently support HRMP channel operations
		Weight::MAX.ref_time()
	}
	fn hrmp_channel_accepted(_recipient: &u32) -> XCMWeight {
		// XCM Executor does not currently support HRMP channel operations
		Weight::MAX.ref_time()
	}
	fn hrmp_channel_closing(_initiator: &u32, _sender: &u32, _recipient: &u32) -> XCMWeight {
		// XCM Executor does not currently support HRMP channel operations
		Weight::MAX.ref_time()
	}
	fn clear_origin() -> XCMWeight {
		XcmGeneric::<Runtime>::clear_origin().ref_time()
	}
	fn descend_origin(_who: &InteriorMultiLocation) -> XCMWeight {
		XcmGeneric::<Runtime>::descend_origin().ref_time()
	}
	fn report_error(_query_response_info: &QueryResponseInfo) -> XCMWeight {
		XcmGeneric::<Runtime>::report_error().ref_time()
	}

	fn deposit_asset(assets: &MultiAssetFilter, _dest: &MultiLocation) -> XCMWeight {
		// Hardcoded till the XCM pallet is fixed
		let hardcoded_weight = Weight::from_ref_time(1_000_000_000 as u64).ref_time();
		let weight = assets.weigh_multi_assets(XcmFungibleWeight::<Runtime>::deposit_asset());
		cmp::min(hardcoded_weight, weight)
	}
	fn deposit_reserve_asset(
		assets: &MultiAssetFilter,
		_dest: &MultiLocation,
		_xcm: &Xcm<()>,
	) -> XCMWeight {
		assets.weigh_multi_assets(XcmFungibleWeight::<Runtime>::deposit_reserve_asset())
	}
	fn exchange_asset(
		_give: &MultiAssetFilter,
		_receive: &MultiAssets,
		_maximal: &bool,
	) -> XCMWeight {
		Weight::MAX.ref_time()
	}
	fn initiate_reserve_withdraw(
		assets: &MultiAssetFilter,
		_reserve: &MultiLocation,
		_xcm: &Xcm<()>,
	) -> XCMWeight {
		assets.weigh_multi_assets(XcmGeneric::<Runtime>::initiate_reserve_withdraw())
	}
	fn initiate_teleport(
		assets: &MultiAssetFilter,
		_dest: &MultiLocation,
		_xcm: &Xcm<()>,
	) -> XCMWeight {
		// Hardcoded till the XCM pallet is fixed
		let hardcoded_weight = Weight::from_ref_time(200_000_000 as u64).ref_time();
		let weight = assets.weigh_multi_assets(XcmFungibleWeight::<Runtime>::initiate_teleport());
		cmp::min(hardcoded_weight, weight)
	}
	fn report_holding(_response_info: &QueryResponseInfo, _assets: &MultiAssetFilter) -> XCMWeight {
		XcmGeneric::<Runtime>::report_holding().ref_time()
	}
	fn buy_execution(_fees: &MultiAsset, _weight_limit: &WeightLimit) -> XCMWeight {
		XcmGeneric::<Runtime>::buy_execution().ref_time()
	}
	fn refund_surplus() -> XCMWeight {
		XcmGeneric::<Runtime>::refund_surplus().ref_time()
	}
	fn set_error_handler(_xcm: &Xcm<Call>) -> XCMWeight {
		XcmGeneric::<Runtime>::set_error_handler().ref_time()
	}
	fn set_appendix(_xcm: &Xcm<Call>) -> XCMWeight {
		XcmGeneric::<Runtime>::set_appendix().ref_time()
	}
	fn clear_error() -> XCMWeight {
		XcmGeneric::<Runtime>::clear_error().ref_time()
	}
	fn claim_asset(_assets: &MultiAssets, _ticket: &MultiLocation) -> XCMWeight {
		XcmGeneric::<Runtime>::claim_asset().ref_time()
	}
	fn trap(_code: &u64) -> XCMWeight {
		XcmGeneric::<Runtime>::trap().ref_time()
	}
	fn subscribe_version(_query_id: &QueryId, _max_response_weight: &u64) -> XCMWeight {
		XcmGeneric::<Runtime>::subscribe_version().ref_time()
	}
	fn unsubscribe_version() -> XCMWeight {
		XcmGeneric::<Runtime>::unsubscribe_version().ref_time()
	}
	fn burn_asset(assets: &MultiAssets) -> XCMWeight {
		assets.weigh_multi_assets(XcmGeneric::<Runtime>::burn_asset())
	}
	fn expect_asset(assets: &MultiAssets) -> XCMWeight {
		assets.weigh_multi_assets(XcmGeneric::<Runtime>::expect_asset())
	}
	fn expect_origin(_origin: &Option<MultiLocation>) -> XCMWeight {
		XcmGeneric::<Runtime>::expect_origin().ref_time()
	}
	fn expect_error(_error: &Option<(u32, XcmError)>) -> XCMWeight {
		XcmGeneric::<Runtime>::expect_error().ref_time()
	}
	fn query_pallet(_module_name: &Vec<u8>, _response_info: &QueryResponseInfo) -> XCMWeight {
		XcmGeneric::<Runtime>::query_pallet().ref_time()
	}
	fn expect_pallet(
		_index: &u32,
		_name: &Vec<u8>,
		_module_name: &Vec<u8>,
		_crate_major: &u32,
		_min_crate_minor: &u32,
	) -> XCMWeight {
		XcmGeneric::<Runtime>::expect_pallet().ref_time()
	}
	fn report_transact_status(_response_info: &QueryResponseInfo) -> XCMWeight {
		XcmGeneric::<Runtime>::report_transact_status().ref_time()
	}
	fn clear_transact_status() -> XCMWeight {
		XcmGeneric::<Runtime>::clear_transact_status().ref_time()
	}
	fn universal_origin(_: &Junction) -> XCMWeight {
		Weight::MAX.ref_time()
	}
	fn export_message(_: &NetworkId, _: &Junctions, _: &Xcm<()>) -> XCMWeight {
		Weight::MAX.ref_time()
	}
	fn lock_asset(_: &MultiAsset, _: &MultiLocation) -> XCMWeight {
		Weight::MAX.ref_time()
	}
	fn unlock_asset(_: &MultiAsset, _: &MultiLocation) -> XCMWeight {
		Weight::MAX.ref_time()
	}
	fn note_unlockable(_: &MultiAsset, _: &MultiLocation) -> XCMWeight {
		Weight::MAX.ref_time()
	}
	fn request_unlock(_: &MultiAsset, _: &MultiLocation) -> XCMWeight {
		Weight::MAX.ref_time()
	}
	fn set_fees_mode(_: &bool) -> XCMWeight {
		XcmGeneric::<Runtime>::set_fees_mode().ref_time()
	}
	fn set_topic(_topic: &[u8; 32]) -> XCMWeight {
		XcmGeneric::<Runtime>::set_topic().ref_time()
	}
	fn clear_topic() -> XCMWeight {
		XcmGeneric::<Runtime>::clear_topic().ref_time()
	}
	fn alias_origin(_: &MultiLocation) -> XCMWeight {
		// XCM Executor does not currently support alias origin operations
		Weight::MAX.ref_time()
	}
	fn unpaid_execution(_: &WeightLimit, _: &Option<MultiLocation>) -> XCMWeight {
		XcmGeneric::<Runtime>::unpaid_execution().ref_time()
	}
}
