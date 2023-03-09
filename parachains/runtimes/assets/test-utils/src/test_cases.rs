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

use crate::{
	AccountIdOf, BalanceOf, ExtBuilder, RuntimeHelper, SessionKeysOf, ValidatorIdOf,
	XcmReceivedFrom,
};
use codec::Encode;
use frame_support::weights::Weight;
use parachains_common::Balance;
use xcm::latest::prelude::*;
use xcm_executor::XcmExecutor;

pub struct CollatorSessionKeys<
	Runtime: frame_system::Config + pallet_balances::Config + pallet_session::Config,
> {
	collator: AccountIdOf<Runtime>,
	validator: ValidatorIdOf<Runtime>,
	key: SessionKeysOf<Runtime>,
}

impl<Runtime: frame_system::Config + pallet_balances::Config + pallet_session::Config>
	CollatorSessionKeys<Runtime>
{
	pub fn new(
		collator: AccountIdOf<Runtime>,
		validator: ValidatorIdOf<Runtime>,
		key: SessionKeysOf<Runtime>,
	) -> Self {
		Self { collator, validator, key }
	}
	pub fn collators(&self) -> Vec<AccountIdOf<Runtime>> {
		vec![self.collator.clone()]
	}

	pub fn session_keys(
		&self,
	) -> Vec<(AccountIdOf<Runtime>, ValidatorIdOf<Runtime>, SessionKeysOf<Runtime>)> {
		vec![(self.collator.clone(), self.validator.clone(), self.key.clone())]
	}
}

/// Test-case makes sure, that `Runtime` can receive teleported native assets from relay chain
pub fn receive_teleported_asset_for_native_asset_works<Runtime, XcmConfig>(
	collator_session_keys: CollatorSessionKeys<Runtime>,
	target_account: AccountIdOf<Runtime>,
) where
	Runtime: frame_system::Config
		+ pallet_balances::Config
		+ pallet_session::Config
		+ pallet_collator_selection::Config
		+ cumulus_pallet_parachain_system::Config,
	AccountIdOf<Runtime>: Into<[u8; 32]>,
	ValidatorIdOf<Runtime>: From<AccountIdOf<Runtime>>,
	BalanceOf<Runtime>: From<Balance>,
	XcmConfig: xcm_executor::Config,
{
	ExtBuilder::<Runtime>::default()
		.with_collators(collator_session_keys.collators())
		.with_session_keys(collator_session_keys.session_keys())
		.build()
		.execute_with(|| {
			// check Balances before
			assert_eq!(
				<pallet_balances::Pallet<Runtime>>::free_balance(target_account.clone()),
				0.into()
			);

			let native_asset_id = MultiLocation::parent();

			let xcm = Xcm(vec![
				ReceiveTeleportedAsset(MultiAssets::from(vec![MultiAsset {
					id: Concrete(native_asset_id),
					fun: Fungible(10000000000000),
				}])),
				ClearOrigin,
				BuyExecution {
					fees: MultiAsset {
						id: Concrete(native_asset_id),
						fun: Fungible(10000000000000),
					},
					weight_limit: Limited(Weight::from_parts(303531000, 65536)),
				},
				DepositAsset {
					assets: Wild(AllCounted(1)),
					beneficiary: MultiLocation {
						parents: 0,
						interior: X1(AccountId32 {
							network: None,
							id: target_account.clone().into(),
						}),
					},
				},
			]);

			let hash = xcm.using_encoded(sp_io::hashing::blake2_256);

			let outcome = XcmExecutor::<XcmConfig>::execute_xcm(
				Parent,
				xcm,
				hash,
				RuntimeHelper::<Runtime>::xcm_max_weight(XcmReceivedFrom::Parent),
			);
			assert_eq!(outcome.ensure_complete(), Ok(()));

			// check Balances after
			assert_ne!(
				<pallet_balances::Pallet<Runtime>>::free_balance(target_account.clone()),
				0.into()
			);
		})
}

#[macro_export]
macro_rules! include_receive_teleported_asset_for_native_asset_works(
	(
		$runtime:path,
		$xcm_config:path,
		$collator_session_key:expr,
		$target_account:expr
	) => {
		#[test]
		fn receive_teleported_asset_for_native_asset_works() {
			asset_test_utils::test_cases::receive_teleported_asset_for_native_asset_works::<$runtime, $xcm_config>($collator_session_key, $target_account)
		}
	}
);
