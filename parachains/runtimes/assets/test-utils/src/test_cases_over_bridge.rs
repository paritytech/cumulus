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

//! Module contains predefined test-case scenarios for `Runtime` with various assets over bridge transfer.

use codec::Encode;
use cumulus_primitives_core::XcmpMessageSource;
use frame_support::{
	assert_ok,
	traits::{Currency, OriginTrait, ProcessMessageError},
};
use pallet_xcm::destination_fees::{DestinationFeesManager, DestinationFeesSetup};
use parachains_common::Balance;
use parachains_runtimes_test_utils::{
	mock_open_hrmp_channel, AccountIdOf, BalanceOf, CollatorSessionKeys, ExtBuilder, RuntimeHelper,
	ValidatorIdOf,
};
use sp_runtime::traits::StaticLookup;
use xcm::{latest::prelude::*, VersionedMultiAssets};
use xcm_builder::{CreateMatcher, MatchXcm};
use xcm_executor::traits::ConvertLocation;

pub struct TestBridgingConfig {
	pub bridged_network: NetworkId,
	pub local_bridge_hub_para_id: u32,
	pub local_bridge_hub_location: MultiLocation,
	pub bridged_target_location: MultiLocation,
}

/// Test-case makes sure that `Runtime` can initiate transfer of assets via bridge - `TransferKind::ReserveBased`
pub fn transfer_asset_via_bridge_initiate_reserve_based_for_native_asset_works<
	Runtime,
	XcmConfig,
	HrmpChannelOpener,
	HrmpChannelSource,
	LocationToAccountId,
>(
	collator_session_keys: CollatorSessionKeys<Runtime>,
	existential_deposit: BalanceOf<Runtime>,
	alice_account: AccountIdOf<Runtime>,
	unwrap_pallet_xcm_event: Box<dyn Fn(Vec<u8>) -> Option<pallet_xcm::Event<Runtime>>>,
	unwrap_xcmp_queue_event: Box<
		dyn Fn(Vec<u8>) -> Option<cumulus_pallet_xcmp_queue::Event<Runtime>>,
	>,
	ensure_configuration: fn() -> TestBridgingConfig,
) where
	Runtime: frame_system::Config
		+ pallet_balances::Config
		+ pallet_session::Config
		+ pallet_xcm::Config
		+ parachain_info::Config
		+ pallet_collator_selection::Config
		+ cumulus_pallet_parachain_system::Config
		+ cumulus_pallet_xcmp_queue::Config,
	AccountIdOf<Runtime>: Into<[u8; 32]>,
	ValidatorIdOf<Runtime>: From<AccountIdOf<Runtime>>,
	BalanceOf<Runtime>: From<Balance>,
	<Runtime as pallet_balances::Config>::Balance: From<Balance> + Into<u128>,
	XcmConfig: xcm_executor::Config,
	LocationToAccountId: ConvertLocation<AccountIdOf<Runtime>>,
	<Runtime as frame_system::Config>::AccountId:
		Into<<<Runtime as frame_system::Config>::RuntimeOrigin as OriginTrait>::AccountId>,
	<<Runtime as frame_system::Config>::Lookup as StaticLookup>::Source:
		From<<Runtime as frame_system::Config>::AccountId>,
	HrmpChannelOpener: frame_support::inherent::ProvideInherent<
		Call = cumulus_pallet_parachain_system::Call<Runtime>,
	>,
	HrmpChannelSource: XcmpMessageSource,
{
	let runtime_para_id = 1000;
	ExtBuilder::<Runtime>::default()
		.with_collators(collator_session_keys.collators())
		.with_session_keys(collator_session_keys.session_keys())
		.with_tracing()
		.with_safe_xcm_version(3)
		.with_para_id(runtime_para_id.into())
		.build()
		.execute_with(|| {
			// prepare bridge config
			let TestBridgingConfig {
				bridged_network,
				local_bridge_hub_para_id,
				bridged_target_location: target_location_from_different_consensus,
				..
			} = ensure_configuration();

			let reserve_account =
				LocationToAccountId::convert_location(&target_location_from_different_consensus)
					.expect("Sovereign account for reserves");
			let balance_to_transfer = 1_000_000_000_000_u128;
			let native_asset = MultiLocation::parent();

			// open HRMP to bridge hub
			mock_open_hrmp_channel::<Runtime, HrmpChannelOpener>(
				runtime_para_id.into(),
				local_bridge_hub_para_id.into(),
			);

			// drip ED to account
			let alice_account_init_balance = existential_deposit + balance_to_transfer.into();
			let _ = <pallet_balances::Pallet<Runtime>>::deposit_creating(
				&alice_account,
				alice_account_init_balance,
			);
			// SA of target location needs to have at least ED, anyway making reserve fails
			let _ = <pallet_balances::Pallet<Runtime>>::deposit_creating(
				&reserve_account,
				existential_deposit,
			);

			// we just check here, that user remains enough balances after withdraw
			// and also we check if `balance_to_transfer` is more than `existential_deposit`,
			assert!(
				(<pallet_balances::Pallet<Runtime>>::free_balance(&alice_account) -
					balance_to_transfer.into()) >=
					existential_deposit
			);
			// SA has just ED
			assert_eq!(
				<pallet_balances::Pallet<Runtime>>::free_balance(&reserve_account),
				existential_deposit
			);

			// local native asset (pallet_balances)
			let asset_to_transfer = MultiAsset {
				fun: Fungible(balance_to_transfer.into()),
				id: Concrete(native_asset),
			};

			// check other accounts
			let destination_fees_setup =
				<Runtime as pallet_xcm::Config>::DestinationFeesManager::decide_for(
					&target_location_from_different_consensus,
					&asset_to_transfer.id,
				);
			if let DestinationFeesSetup::ByUniversalLocation { local_account } =
				destination_fees_setup
			{
				let local_account = LocationToAccountId::convert_location(&local_account)
					.expect("Sovereign account for fee");
				let _ = <pallet_balances::Pallet<Runtime>>::deposit_creating(
					&local_account,
					existential_deposit,
				);
				assert_eq!(
					<pallet_balances::Pallet<Runtime>>::free_balance(&local_account),
					existential_deposit
				);
			}

			// destination is (some) account relative to the destination different consensus
			let target_destination_account = MultiLocation {
				parents: 0,
				interior: X1(AccountId32 {
					network: Some(bridged_network),
					id: sp_runtime::AccountId32::new([3; 32]).into(),
				}),
			};

			// do pallet_xcm call reserve transfer
			assert_ok!(<pallet_xcm::Pallet<Runtime>>::reserve_transfer_assets(
				RuntimeHelper::<Runtime>::origin_of(alice_account.clone()),
				Box::new(target_location_from_different_consensus.clone().into_versioned()),
				Box::new(target_destination_account.clone().into_versioned()),
				Box::new(VersionedMultiAssets::from(MultiAssets::from(asset_to_transfer))),
				0,
			));

			// check alice account decreased about all balance_to_transfer
			assert_eq!(
				<pallet_balances::Pallet<Runtime>>::free_balance(&alice_account),
				alice_account_init_balance - balance_to_transfer.into()
			);

			// check reserve account
			if let DestinationFeesSetup::ByUniversalLocation { local_account } =
				destination_fees_setup
			{
				// partial fees goes here
				let local_account = LocationToAccountId::convert_location(&local_account)
					.expect("Sovereign account for fee");
				let local_account_balance =
					<pallet_balances::Pallet<Runtime>>::free_balance(&local_account);
				assert_ne!(
					<pallet_balances::Pallet<Runtime>>::free_balance(&local_account),
					existential_deposit
				);
				let additional_fee = local_account_balance - existential_deposit;

				// check reserve account increased about all balance_to_transfer
				assert_eq!(
					<pallet_balances::Pallet<Runtime>>::free_balance(&reserve_account),
					existential_deposit + balance_to_transfer.into() - additional_fee
				);
			} else {
				// check reserve account increased about all balance_to_transfer
				assert_eq!(
					<pallet_balances::Pallet<Runtime>>::free_balance(&reserve_account),
					existential_deposit + balance_to_transfer.into()
				);
			}

			// check events
			// check pallet_xcm attempted
			RuntimeHelper::<Runtime>::assert_pallet_xcm_event_outcome(
				&unwrap_pallet_xcm_event,
				|outcome| {
					assert_ok!(outcome.ensure_complete());
				},
			);

			// check that xcm was sent
			let xcm_sent_message_hash = <frame_system::Pallet<Runtime>>::events()
				.into_iter()
				.filter_map(|e| unwrap_xcmp_queue_event(e.event.encode()))
				.find_map(|e| match e {
					cumulus_pallet_xcmp_queue::Event::XcmpMessageSent { message_hash } =>
						Some(message_hash),
					_ => None,
				});

			// read xcm
			let xcm_sent =
				RuntimeHelper::<HrmpChannelSource>::take_xcm(local_bridge_hub_para_id.into())
					.unwrap();
			println!("xcm_sent: {:?}", xcm_sent);

			assert_eq!(
				xcm_sent_message_hash,
				Some(xcm_sent.using_encoded(sp_io::hashing::blake2_256))
			);
			let mut xcm_sent: Xcm<()> = xcm_sent.try_into().expect("versioned xcm");

			// check sent XCM ExportMessage to bridge-hub
			xcm_sent
				.0
				.matcher()
				.match_next_inst(|instr| match instr {
					// first instruction is UNpai (because we have explicit unpaid execution on bridge-hub now)
					UnpaidExecution { weight_limit, check_origin }
						if weight_limit == &Unlimited && check_origin.is_none() =>
						Ok(()),
					_ => Err(ProcessMessageError::BadFormat),
				})
				.expect("contains UnpaidExecution")
				.match_next_inst(|instr| match instr {
					// second instruction is ExportMessage
					ExportMessage { network, destination, xcm: _inner_xcm } => {
						assert_eq!(network, &bridged_network);
						let (_, target_location_junctions_without_global_consensus) =
							target_location_from_different_consensus
								.interior
								.split_global()
								.expect("split works");
						assert_eq!(
							destination,
							&target_location_junctions_without_global_consensus
						);
						Ok(())
					},
					_ => Err(ProcessMessageError::BadFormat),
				})
				.expect("contains ExportMessage");
		})
}
