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
	traits::{Currency, Get, OriginTrait, ProcessMessageError},
};
use pallet_bridge_transfer_primitives::MaybePaidLocation;
use parachains_common::Balance;
use parachains_runtimes_test_utils::{
	mock_open_hrmp_channel, AccountIdOf, BalanceOf, CollatorSessionKeys, ExtBuilder, RuntimeHelper,
	ValidatorIdOf, XcmReceivedFrom,
};
use sp_runtime::traits::StaticLookup;
use xcm::{latest::prelude::*, VersionedMultiAssets, VersionedMultiLocation};
use xcm_builder::{CreateMatcher, MatchXcm};
use xcm_executor::{traits::ConvertLocation, XcmExecutor};

pub struct TestBridgingConfig {
	pub bridged_network: NetworkId,
	pub local_bridge_hub_para_id: u32,
	pub local_bridge_hub_location: MaybePaidLocation,
	pub bridged_target_location: MaybePaidLocation,
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
	unwrap_pallet_bridge_transfer_event: Box<
		dyn Fn(Vec<u8>) -> Option<pallet_bridge_transfer::Event<Runtime>>,
	>,
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
		+ pallet_bridge_transfer::Config
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
				bridged_target_location:
					MaybePaidLocation {
						location: target_location_from_different_consensus,
						maybe_fee: target_location_fee,
					},
				..
			} = ensure_configuration();

			// we expect paid target execution
			let target_location_fee = target_location_fee.unwrap();

			let reserve_account =
				LocationToAccountId::convert_location(&target_location_from_different_consensus)
					.expect("Sovereign account for reserves");
			let balance_to_transfer = 1000_u128;
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
			let assets = MultiAssets::from(MultiAsset {
				fun: Fungible(balance_to_transfer.into()),
				id: Concrete(native_asset),
			});

			// destination is (some) account from different consensus
			let target_destination_account = target_location_from_different_consensus
				.appended_with(AccountId32 {
					network: Some(bridged_network),
					id: sp_runtime::AccountId32::new([3; 32]).into(),
				})
				.unwrap();

			// trigger asset transfer
			assert_ok!(<pallet_bridge_transfer::Pallet<Runtime>>::transfer_asset_via_bridge(
				RuntimeHelper::<Runtime>::origin_of(alice_account.clone()),
				Box::new(VersionedMultiAssets::from(assets.clone())),
				Box::new(VersionedMultiLocation::from(target_destination_account)),
			));

			// check alice account decreased
			assert_eq!(
				<pallet_balances::Pallet<Runtime>>::free_balance(&alice_account),
				alice_account_init_balance - balance_to_transfer.into()
			);
			// check reserve account increased
			assert_eq!(
				<pallet_balances::Pallet<Runtime>>::free_balance(&reserve_account),
				existential_deposit + balance_to_transfer.into()
			);

			// check events
			let mut bridge_transfer_events = <frame_system::Pallet<Runtime>>::events()
				.into_iter()
				.filter_map(|e| unwrap_pallet_bridge_transfer_event(e.event.encode()));
			assert!(bridge_transfer_events.any(|r| matches!(
				r,
				pallet_bridge_transfer::Event::ReserveAssetsDeposited { .. }
			)));
			let transfer_initiated_event = bridge_transfer_events.find_map(|e| match e {
				pallet_bridge_transfer::Event::TransferInitiated {
					message_id,
					forwarded_message_id,
					sender_cost,
				} => Some((message_id, forwarded_message_id, sender_cost)),
				_ => None,
			});
			assert!(transfer_initiated_event.is_some());
			let (message_id, forwarded_message_id, sender_cost) = transfer_initiated_event.unwrap();
			// we expect UnpaidRemoteExporter
			assert!(sender_cost.is_none());

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
			assert!(xcm_sent
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
					ExportMessage { network, destination, xcm: inner_xcm } => {
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

						let mut reanchored_assets = assets.clone();
						reanchored_assets
							.reanchor(
								&target_location_from_different_consensus,
								XcmConfig::UniversalLocation::get(),
							)
							.expect("reanchored assets");
						let mut reanchored_destination_account = target_destination_account;
						reanchored_destination_account
							.reanchor(
								&target_location_from_different_consensus,
								XcmConfig::UniversalLocation::get(),
							)
							.expect("reanchored destination account");
						let universal_location_as_sovereign_account_on_target =
							<Runtime as pallet_bridge_transfer::Config>::UniversalLocation::get()
								.invert_target(&target_location_from_different_consensus)
								.expect("invert_target Universal Location");

						// match inner xcm
						assert!(inner_xcm
							.0
							.matcher()
							.match_next_inst(|next_instr| match next_instr {
								WithdrawAsset(fees)
									if fees == &MultiAssets::from(target_location_fee.clone()) =>
									Ok(()),
								_ => Err(ProcessMessageError::BadFormat),
							})
							.expect("contains WithdrawAsset")
							.match_next_inst(|next_instr| match next_instr {
								BuyExecution { ref fees, ref weight_limit }
									if fees == &target_location_fee &&
										weight_limit == &Unlimited =>
									Ok(()),
								_ => Err(ProcessMessageError::BadFormat),
							})
							.expect("contains BuyExecution")
							.match_next_inst(|inner_xcm_instr| match inner_xcm_instr {
								ReserveAssetDeposited(ref deposited)
									if deposited.eq(&reanchored_assets) =>
									Ok(()),
								_ => Err(ProcessMessageError::BadFormat),
							})
							.expect("contains ReserveAssetDeposited")
							.match_next_inst(|inner_xcm_instr| match inner_xcm_instr {
								DepositAsset { assets: filter, ref beneficiary }
									if filter ==
										&MultiAssetFilter::from(reanchored_assets.clone()) &&
										beneficiary.eq(&reanchored_destination_account) =>
									Ok(()),
								_ => Err(ProcessMessageError::BadFormat),
							})
							.expect("contains DepositAsset")
							.match_next_inst(|inner_xcm_instr| match inner_xcm_instr {
								RefundSurplus => Ok(()),
								_ => Err(ProcessMessageError::BadFormat),
							})
							.expect("contains RefundSurplus")
							.match_next_inst(|inner_xcm_instr| {
								match inner_xcm_instr {
									DepositAsset { assets: filter, ref beneficiary }
										if filter ==
											&MultiAssetFilter::from(
												target_location_fee.clone(),
											) && beneficiary.eq(
											&universal_location_as_sovereign_account_on_target,
										) =>
										Ok(()),
									_ => Err(ProcessMessageError::BadFormat),
								}
							})
							.expect("contains DepositAsset")
							.match_next_inst(|instr| match instr {
								SetTopic(ref topic) if topic.eq(&message_id) => Ok(()),
								_ => Err(ProcessMessageError::BadFormat),
							})
							.expect("contains SetTopic")
							.assert_remaining_insts(0)
							.is_ok());
						Ok(())
					},
					_ => Err(ProcessMessageError::BadFormat),
				})
				.expect("contains ExportMessage")
				.match_next_inst(|instr| match instr {
					SetTopic(ref topic) if topic.eq(&forwarded_message_id) => Ok(()),
					_ => Err(ProcessMessageError::BadFormat),
				})
				.expect("contains SetTopic")
				.assert_remaining_insts(0)
				.is_ok());
		})
}

/// Test-case makes sure that `Runtime` can initiate transfer of assets via bridge - `TransferKind::WithdrawReserve`
pub fn transfer_asset_via_bridge_initiate_withdraw_reserve_for_native_asset_works<
	Runtime,
	XcmConfig,
	HrmpChannelOpener,
	HrmpChannelSource,
	LocationToAccountId,
	ForeignAssetsPalletInstance,
>(
	collator_session_keys: CollatorSessionKeys<Runtime>,
	existential_deposit: BalanceOf<Runtime>,
	alice_account: AccountIdOf<Runtime>,
	unwrap_pallet_bridge_transfer_event: Box<
		dyn Fn(Vec<u8>) -> Option<pallet_bridge_transfer::Event<Runtime>>,
	>,
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
		+ pallet_assets::Config<ForeignAssetsPalletInstance>
		+ pallet_bridge_transfer::Config
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
	<Runtime as pallet_assets::Config<ForeignAssetsPalletInstance>>::AssetId:
		From<MultiLocation> + Into<MultiLocation>,
	<Runtime as pallet_assets::Config<ForeignAssetsPalletInstance>>::AssetIdParameter:
		From<MultiLocation> + Into<MultiLocation>,
	<Runtime as pallet_assets::Config<ForeignAssetsPalletInstance>>::Balance:
		From<Balance> + Into<u128>,
	ForeignAssetsPalletInstance: 'static,
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
				bridged_target_location:
					MaybePaidLocation {
						location: target_location_from_different_consensus,
						maybe_fee: target_location_fee,
					},
				..
			} = ensure_configuration();

			// we expect paid target execution
			let target_location_fee = target_location_fee.unwrap();

			let foreign_asset_id_multilocation =
				MultiLocation::new(2, X1(GlobalConsensus(bridged_network)));

			let reserve_account =
				LocationToAccountId::convert_location(&target_location_from_different_consensus)
					.expect("Sovereign account for reserves");
			let balance_to_transfer = 1000_u128;
			let asset_minimum_asset_balance = 1_000_000_u128;

			// open HRMP to bridge hub
			mock_open_hrmp_channel::<Runtime, HrmpChannelOpener>(
				runtime_para_id.into(),
				local_bridge_hub_para_id.into(),
			);

			// drip ED to account
			let _ = <pallet_balances::Pallet<Runtime>>::deposit_creating(
				&alice_account,
				existential_deposit,
			);
			// SA of target location needs to have at least ED, anyway making reserve fails
			let _ = <pallet_balances::Pallet<Runtime>>::deposit_creating(
				&reserve_account,
				existential_deposit,
			);

			// user already received native tokens from bridged chain, which are stored in `ForeignAssets`
			{
				//1. create foreign asset
				assert_ok!(
					<pallet_assets::Pallet<Runtime, ForeignAssetsPalletInstance>>::force_create(
						RuntimeHelper::<Runtime>::root_origin(),
						foreign_asset_id_multilocation.into(),
						reserve_account.clone().into(),
						false,
						asset_minimum_asset_balance.into()
					)
				);

				// 2. drip asset to alice
				assert_ok!(<pallet_assets::Pallet<Runtime, ForeignAssetsPalletInstance>>::mint(
					RuntimeHelper::<Runtime>::origin_of(reserve_account.clone()),
					foreign_asset_id_multilocation.into(),
					alice_account.clone().into(),
					(asset_minimum_asset_balance + balance_to_transfer).into()
				));
			}

			assert_eq!(
				<pallet_balances::Pallet<Runtime>>::free_balance(&alice_account),
				existential_deposit
			);
			assert_eq!(
				<pallet_balances::Pallet<Runtime>>::free_balance(&reserve_account),
				existential_deposit
			);
			assert_eq!(
				<pallet_assets::Pallet<Runtime, ForeignAssetsPalletInstance>>::balance(
					foreign_asset_id_multilocation.into(),
					alice_account.clone()
				),
				(asset_minimum_asset_balance + balance_to_transfer).into()
			);

			// lets withdraw previously reserve asset deposited from `ForeignAssets`
			let assets = MultiAssets::from(MultiAsset {
				fun: Fungible(balance_to_transfer.into()),
				id: Concrete(foreign_asset_id_multilocation),
			});

			// destination is (some) account from different consensus
			let target_destination_account = target_location_from_different_consensus
				.appended_with(AccountId32 {
					network: Some(bridged_network),
					id: sp_runtime::AccountId32::new([3; 32]).into(),
				})
				.unwrap();

			// trigger asset transfer
			assert_ok!(<pallet_bridge_transfer::Pallet<Runtime>>::transfer_asset_via_bridge(
				RuntimeHelper::<Runtime>::origin_of(alice_account.clone()),
				Box::new(VersionedMultiAssets::from(assets.clone())),
				Box::new(VersionedMultiLocation::from(target_destination_account)),
			));

			// check alice account (balances not changed)
			assert_eq!(
				<pallet_balances::Pallet<Runtime>>::free_balance(&alice_account),
				existential_deposit
			);
			// check reserve account (balances not changed)
			assert_eq!(
				<pallet_balances::Pallet<Runtime>>::free_balance(&reserve_account),
				existential_deposit
			);
			// `ForeignAssets` for alice account is decressed
			assert_eq!(
				<pallet_assets::Pallet<Runtime, ForeignAssetsPalletInstance>>::balance(
					foreign_asset_id_multilocation.into(),
					alice_account.clone()
				),
				asset_minimum_asset_balance.into()
			);

			// check events
			let mut bridge_transfer_events = <frame_system::Pallet<Runtime>>::events()
				.into_iter()
				.filter_map(|e| unwrap_pallet_bridge_transfer_event(e.event.encode()));
			assert!(bridge_transfer_events
				.any(|r| matches!(r, pallet_bridge_transfer::Event::AssetsWithdrawn { .. })));
			let transfer_initiated_event = bridge_transfer_events.find_map(|e| match e {
				pallet_bridge_transfer::Event::TransferInitiated {
					message_id,
					forwarded_message_id,
					sender_cost,
				} => Some((message_id, forwarded_message_id, sender_cost)),
				_ => None,
			});
			assert!(transfer_initiated_event.is_some());
			let (message_id, forwarded_message_id, sender_cost) = transfer_initiated_event.unwrap();
			// we expect UnpaidRemoteExporter
			assert!(sender_cost.is_none());

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
			assert!(xcm_sent
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
					ExportMessage { network, destination, xcm: inner_xcm } => {
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

						let mut reanchored_assets = assets.clone();
						reanchored_assets
							.reanchor(
								&target_location_from_different_consensus,
								XcmConfig::UniversalLocation::get(),
							)
							.expect("reanchored assets");
						let mut reanchored_destination_account = target_destination_account;
						reanchored_destination_account
							.reanchor(
								&target_location_from_different_consensus,
								XcmConfig::UniversalLocation::get(),
							)
							.expect("reanchored destination account");
						let universal_location_as_sovereign_account_on_target =
							<Runtime as pallet_bridge_transfer::Config>::UniversalLocation::get()
								.invert_target(&target_location_from_different_consensus)
								.expect("invert_target Universal Location");

						// match inner xcm
						assert!(inner_xcm
							.0
							.matcher()
							.match_next_inst(|next_instr| match next_instr {
								WithdrawAsset(fees)
									if fees == &MultiAssets::from(target_location_fee.clone()) =>
									Ok(()),
								_ => Err(ProcessMessageError::BadFormat),
							})
							.expect("contains WithdrawAsset")
							.match_next_inst(|next_instr| match next_instr {
								BuyExecution { ref fees, ref weight_limit }
									if fees == &target_location_fee &&
										weight_limit == &Unlimited =>
									Ok(()),
								_ => Err(ProcessMessageError::BadFormat),
							})
							.expect("contains BuyExecution")
							.match_next_inst(|inner_xcm_instr| match inner_xcm_instr {
								WithdrawAsset(ref deposited)
									if deposited.eq(&reanchored_assets) =>
									Ok(()),
								_ => Err(ProcessMessageError::BadFormat),
							})
							.expect("contains WithdrawAsset")
							.match_next_inst(|inner_xcm_instr| match inner_xcm_instr {
								DepositAsset { assets: filter, ref beneficiary }
									if filter ==
										&MultiAssetFilter::from(reanchored_assets.clone()) &&
										beneficiary.eq(&reanchored_destination_account) =>
									Ok(()),
								_ => Err(ProcessMessageError::BadFormat),
							})
							.expect("contains DepositAsset")
							.match_next_inst(|inner_xcm_instr| match inner_xcm_instr {
								RefundSurplus => Ok(()),
								_ => Err(ProcessMessageError::BadFormat),
							})
							.expect("contains RefundSurplus")
							.match_next_inst(|inner_xcm_instr| {
								match inner_xcm_instr {
									DepositAsset { assets: filter, ref beneficiary }
										if filter ==
											&MultiAssetFilter::from(
												target_location_fee.clone(),
											) && beneficiary.eq(
											&universal_location_as_sovereign_account_on_target,
										) =>
										Ok(()),
									_ => Err(ProcessMessageError::BadFormat),
								}
							})
							.expect("contains DepositAsset")
							.match_next_inst(|instr| match instr {
								SetTopic(ref topic) if topic.eq(&message_id) => Ok(()),
								_ => Err(ProcessMessageError::BadFormat),
							})
							.expect("contains SetTopic")
							.assert_remaining_insts(0)
							.is_ok());
						Ok(())
					},
					_ => Err(ProcessMessageError::BadFormat),
				})
				.expect("contains ExportMessage")
				.match_next_inst(|instr| match instr {
					SetTopic(ref topic) if topic.eq(&forwarded_message_id) => Ok(()),
					_ => Err(ProcessMessageError::BadFormat),
				})
				.expect("contains SetTopic")
				.assert_remaining_insts(0)
				.is_ok());
		})
}

/// Test-case makes sure that `Runtime` can process `ReserveAssetDeposited`.
pub fn receive_reserve_asset_deposited_from_different_consensus_over_bridge_works<
	Runtime,
	XcmConfig,
	LocationToAccountId,
	ForeignAssetsPalletInstance,
>(
	collator_session_keys: CollatorSessionKeys<Runtime>,
	existential_deposit: BalanceOf<Runtime>,
	target_account: AccountIdOf<Runtime>,
	unwrap_pallet_xcm_event: Box<dyn Fn(Vec<u8>) -> Option<pallet_xcm::Event<Runtime>>>,
	ensure_configuration: fn() -> TestBridgingConfig,
) where
	Runtime: frame_system::Config
		+ pallet_balances::Config
		+ pallet_session::Config
		+ pallet_xcm::Config
		+ parachain_info::Config
		+ pallet_collator_selection::Config
		+ cumulus_pallet_parachain_system::Config
		+ cumulus_pallet_xcmp_queue::Config
		+ pallet_assets::Config<ForeignAssetsPalletInstance>
		+ pallet_bridge_transfer::Config,
	AccountIdOf<Runtime>: Into<[u8; 32]>,
	ValidatorIdOf<Runtime>: From<AccountIdOf<Runtime>>,
	BalanceOf<Runtime>: From<Balance>,
	<Runtime as frame_system::Config>::AccountId:
		Into<<<Runtime as frame_system::Config>::RuntimeOrigin as OriginTrait>::AccountId>,
	<<Runtime as frame_system::Config>::Lookup as StaticLookup>::Source:
		From<<Runtime as frame_system::Config>::AccountId>,
	XcmConfig: xcm_executor::Config,
	<Runtime as pallet_assets::Config<ForeignAssetsPalletInstance>>::AssetId:
		From<MultiLocation> + Into<MultiLocation>,
	<Runtime as pallet_assets::Config<ForeignAssetsPalletInstance>>::AssetIdParameter:
		From<MultiLocation> + Into<MultiLocation>,
	<Runtime as pallet_assets::Config<ForeignAssetsPalletInstance>>::Balance:
		From<Balance> + Into<u128>,
	LocationToAccountId: ConvertLocation<AccountIdOf<Runtime>>,
	ForeignAssetsPalletInstance: 'static,
{
	ExtBuilder::<Runtime>::default()
		.with_collators(collator_session_keys.collators())
		.with_session_keys(collator_session_keys.session_keys())
		.with_balances(vec![(target_account.clone(), existential_deposit)])
		.with_tracing()
		.build()
		.execute_with(|| {
			// prepare bridge config
			let TestBridgingConfig {
				bridged_network: remote_network_id,
				local_bridge_hub_location:
					MaybePaidLocation { location: local_bridge_hub_location, .. },
				bridged_target_location:
					MaybePaidLocation { location: remote_parachain_as_origin, .. },
				..
			} = ensure_configuration();

			let foreign_asset_id_multilocation =
				MultiLocation { parents: 2, interior: X1(GlobalConsensus(remote_network_id)) };

			let buy_execution_fee_amount = 50000000000;
			let reserve_asset_deposisted = 100_000_000;

			// drip SA for remote global parachain origin
			let remote_parachain_sovereign_account =
				LocationToAccountId::convert_location(&remote_parachain_as_origin)
					.expect("Sovereign account works");
			assert_ok!(<pallet_balances::Pallet<Runtime>>::force_set_balance(
				RuntimeHelper::<Runtime>::root_origin(),
				remote_parachain_sovereign_account.clone().into(),
				existential_deposit + buy_execution_fee_amount.into(),
			));

			// create foreign asset
			let asset_minimum_asset_balance = 1_000_000_u128;
			assert_ok!(
				<pallet_assets::Pallet<Runtime, ForeignAssetsPalletInstance>>::force_create(
					RuntimeHelper::<Runtime>::root_origin(),
					foreign_asset_id_multilocation.into(),
					remote_parachain_sovereign_account.clone().into(),
					false,
					asset_minimum_asset_balance.into()
				)
			);

			// we assume here that BuyExecution fee goes to staking pot
			let staking_pot_account_id = <pallet_collator_selection::Pallet<Runtime>>::account_id();
			assert_ok!(<pallet_balances::Pallet<Runtime>>::force_set_balance(
				RuntimeHelper::<Runtime>::root_origin(),
				staking_pot_account_id.clone().into(),
				existential_deposit,
			));

			let local_bridge_hub_multilocation_as_account_id =
				LocationToAccountId::convert_location(&local_bridge_hub_location)
					.expect("Correct AccountId");

			// check before
			let remote_parachain_sovereign_account_balance_before =
				<pallet_balances::Pallet<Runtime>>::free_balance(
					&remote_parachain_sovereign_account,
				);
			assert_eq!(
				remote_parachain_sovereign_account_balance_before,
				existential_deposit + buy_execution_fee_amount.into()
			);
			assert_eq!(
				<pallet_balances::Pallet<Runtime>>::free_balance(&target_account),
				existential_deposit
			);
			assert_eq!(
				<pallet_balances::Pallet<Runtime>>::free_balance(
					&local_bridge_hub_multilocation_as_account_id
				),
				0.into()
			);
			assert_eq!(
				<pallet_balances::Pallet<Runtime>>::free_balance(&staking_pot_account_id),
				existential_deposit
			);
			assert_eq!(
				<pallet_assets::Pallet<Runtime, ForeignAssetsPalletInstance>>::balance(
					foreign_asset_id_multilocation.into(),
					&target_account
				),
				0.into()
			);

			// xcm
			let xcm = Xcm(vec![
				UniversalOrigin(GlobalConsensus(remote_network_id)),
				DescendOrigin(X1(Parachain(1000))),
				// buying execution as sovereign account `remote_parachain_sovereign_account` in *native asset on receiving runtime*
				WithdrawAsset(MultiAssets::from(vec![MultiAsset {
					id: Concrete(MultiLocation { parents: 1, interior: Here }),
					fun: Fungible(buy_execution_fee_amount),
				}])),
				BuyExecution {
					fees: MultiAsset {
						id: Concrete(MultiLocation { parents: 1, interior: Here }),
						fun: Fungible(buy_execution_fee_amount),
					},
					weight_limit: Unlimited,
				},
				// reserve deposited - assets transferred through bridge -  *native asset on sending runtime*
				ReserveAssetDeposited(MultiAssets::from(vec![MultiAsset {
					id: Concrete(MultiLocation {
						parents: 2,
						interior: X1(GlobalConsensus(remote_network_id)),
					}),
					fun: Fungible(reserve_asset_deposisted),
				}])),
				DepositAsset {
					assets: Definite(MultiAssets::from(vec![MultiAsset {
						id: Concrete(MultiLocation {
							parents: 2,
							interior: X1(GlobalConsensus(remote_network_id)),
						}),
						fun: Fungible(reserve_asset_deposisted),
					}])),
					beneficiary: MultiLocation {
						parents: 0,
						interior: X1(AccountId32 {
							network: None,
							id: target_account.clone().into(),
						}),
					},
				},
				// return unspent weight back to SA of caller
				RefundSurplus,
				DepositAsset {
					assets: Definite(MultiAssets::from(vec![MultiAsset {
						id: Concrete(MultiLocation { parents: 1, interior: Here }),
						fun: Fungible(buy_execution_fee_amount),
					}])),
					beneficiary: remote_parachain_as_origin,
				},
			]);

			// origin as BridgeHub
			let origin = local_bridge_hub_location;

			let hash = xcm.using_encoded(sp_io::hashing::blake2_256);

			// execute xcm as XcmpQueue would do
			let outcome = XcmExecutor::<XcmConfig>::execute_xcm(
				origin,
				xcm,
				hash,
				RuntimeHelper::<Runtime>::xcm_max_weight(XcmReceivedFrom::Sibling),
			);
			assert_eq!(outcome.ensure_complete(), Ok(()));

			// check after
			let expected_buy_execution_fee =
				<pallet_balances::Pallet<Runtime>>::free_balance(&staking_pot_account_id) -
					existential_deposit;
			assert_eq!(
				<pallet_balances::Pallet<Runtime>>::free_balance(
					&remote_parachain_sovereign_account
				),
				remote_parachain_sovereign_account_balance_before - expected_buy_execution_fee
			);
			assert_eq!(
				<pallet_balances::Pallet<Runtime>>::free_balance(&target_account),
				existential_deposit
			);
			assert_eq!(
				<pallet_balances::Pallet<Runtime>>::free_balance(
					&local_bridge_hub_multilocation_as_account_id
				),
				0.into()
			);
			assert_ne!(
				<pallet_balances::Pallet<Runtime>>::free_balance(&staking_pot_account_id),
				0.into()
			);
			assert_eq!(
				<pallet_assets::Pallet<Runtime, ForeignAssetsPalletInstance>>::balance(
					foreign_asset_id_multilocation.into(),
					&target_account
				),
				reserve_asset_deposisted.into()
			);

			// check NO asset trap occurred
			assert_eq!(
				false,
				<frame_system::Pallet<Runtime>>::events()
					.into_iter()
					.filter_map(|e| unwrap_pallet_xcm_event(e.event.encode()))
					.any(|e| matches!(e, pallet_xcm::Event::AssetsTrapped { .. }))
			);
		})
}

/// Test-case makes sure that `Runtime` can process reserve withdraw which was sent over bridge.
pub fn withdraw_reserve_asset_deposited_from_different_consensus_over_bridge_works<
	Runtime,
	XcmConfig,
	LocationToAccountId,
>(
	collator_session_keys: CollatorSessionKeys<Runtime>,
	existential_deposit: BalanceOf<Runtime>,
	target_account: AccountIdOf<Runtime>,
	unwrap_pallet_xcm_event: Box<dyn Fn(Vec<u8>) -> Option<pallet_xcm::Event<Runtime>>>,
	ensure_configuration: fn() -> TestBridgingConfig,
) where
	Runtime: frame_system::Config
		+ pallet_balances::Config
		+ pallet_session::Config
		+ pallet_xcm::Config
		+ parachain_info::Config
		+ pallet_collator_selection::Config
		+ cumulus_pallet_parachain_system::Config
		+ cumulus_pallet_xcmp_queue::Config
		+ pallet_bridge_transfer::Config,
	AccountIdOf<Runtime>: Into<[u8; 32]>,
	ValidatorIdOf<Runtime>: From<AccountIdOf<Runtime>>,
	BalanceOf<Runtime>: From<Balance>,
	<Runtime as frame_system::Config>::AccountId:
		Into<<<Runtime as frame_system::Config>::RuntimeOrigin as OriginTrait>::AccountId>,
	<<Runtime as frame_system::Config>::Lookup as StaticLookup>::Source:
		From<<Runtime as frame_system::Config>::AccountId>,
	XcmConfig: xcm_executor::Config,
	LocationToAccountId: ConvertLocation<AccountIdOf<Runtime>>,
{
	ExtBuilder::<Runtime>::default()
		.with_collators(collator_session_keys.collators())
		.with_session_keys(collator_session_keys.session_keys())
		.with_balances(vec![(target_account.clone(), existential_deposit)])
		.with_tracing()
		.build()
		.execute_with(|| {
			// prepare bridge config
			let TestBridgingConfig {
				bridged_network: remote_network_id,
				local_bridge_hub_location:
					MaybePaidLocation { location: local_bridge_hub_location, .. },
				bridged_target_location:
					MaybePaidLocation { location: remote_parachain_as_origin, .. },
				..
			} = ensure_configuration();

			let buy_execution_fee_amount = 50000000000;
			let reserve_asset_deposisted = 100_000_000;

			// add reserved assets to SA for remote global parachain origin (this is how reserve was done, when reserve_asset_deposisted was transferred out)
			let remote_parachain_sovereign_account =
				LocationToAccountId::convert_location(&remote_parachain_as_origin)
					.expect("Sovereign account works");
			assert_ok!(<pallet_balances::Pallet<Runtime>>::force_set_balance(
				RuntimeHelper::<Runtime>::root_origin(),
				remote_parachain_sovereign_account.clone().into(),
				existential_deposit +
					buy_execution_fee_amount.into() +
					reserve_asset_deposisted.into(),
			));

			// we assume here that BuyExecution fee goes to staking pot
			let staking_pot_account_id = <pallet_collator_selection::Pallet<Runtime>>::account_id();
			assert_ok!(<pallet_balances::Pallet<Runtime>>::force_set_balance(
				RuntimeHelper::<Runtime>::root_origin(),
				staking_pot_account_id.clone().into(),
				existential_deposit,
			));

			let local_bridge_hub_multilocation_as_account_id =
				LocationToAccountId::convert_location(&local_bridge_hub_location)
					.expect("Correct AccountId");

			// check before
			let remote_parachain_sovereign_account_balance_before =
				<pallet_balances::Pallet<Runtime>>::free_balance(
					&remote_parachain_sovereign_account,
				);
			assert_eq!(
				remote_parachain_sovereign_account_balance_before,
				existential_deposit +
					buy_execution_fee_amount.into() +
					reserve_asset_deposisted.into()
			);
			assert_eq!(
				<pallet_balances::Pallet<Runtime>>::free_balance(&target_account),
				existential_deposit
			);
			assert_eq!(
				<pallet_balances::Pallet<Runtime>>::free_balance(
					&local_bridge_hub_multilocation_as_account_id
				),
				0.into()
			);
			assert_eq!(
				<pallet_balances::Pallet<Runtime>>::free_balance(&staking_pot_account_id),
				existential_deposit
			);

			// xcm
			let xcm = Xcm(vec![
				UniversalOrigin(GlobalConsensus(remote_network_id)),
				DescendOrigin(X1(Parachain(1000))),
				// buying execution as sovereign account `remote_parachain_sovereign_account` in *native asset on receiving runtime*
				WithdrawAsset(MultiAssets::from(vec![MultiAsset {
					id: Concrete(MultiLocation { parents: 1, interior: Here }),
					fun: Fungible(buy_execution_fee_amount),
				}])),
				BuyExecution {
					fees: MultiAsset {
						id: Concrete(MultiLocation { parents: 1, interior: Here }),
						fun: Fungible(buy_execution_fee_amount),
					},
					weight_limit: Unlimited,
				},
				// we are returning reserve deposited - assets transferred through bridge -  *native asset on receiving runtime*
				WithdrawAsset(MultiAssets::from(vec![MultiAsset {
					id: Concrete(MultiLocation { parents: 1, interior: Here }),
					fun: Fungible(reserve_asset_deposisted),
				}])),
				DepositAsset {
					assets: Definite(MultiAssets::from(vec![MultiAsset {
						id: Concrete(MultiLocation { parents: 1, interior: Here }),
						fun: Fungible(reserve_asset_deposisted),
					}])),
					beneficiary: MultiLocation {
						parents: 0,
						interior: X1(AccountId32 {
							network: None,
							id: target_account.clone().into(),
						}),
					},
				},
				// return unspent weight back to SA of caller
				RefundSurplus,
				DepositAsset {
					assets: Definite(MultiAssets::from(vec![MultiAsset {
						id: Concrete(MultiLocation { parents: 1, interior: Here }),
						fun: Fungible(buy_execution_fee_amount),
					}])),
					beneficiary: remote_parachain_as_origin,
				},
			]);

			// origin as BridgeHub
			let origin = local_bridge_hub_location;

			let hash = xcm.using_encoded(sp_io::hashing::blake2_256);

			// execute xcm as XcmpQueue would do
			let outcome = XcmExecutor::<XcmConfig>::execute_xcm(
				origin,
				xcm,
				hash,
				RuntimeHelper::<Runtime>::xcm_max_weight(XcmReceivedFrom::Sibling),
			);
			assert_eq!(outcome.ensure_complete(), Ok(()));

			// check after
			let expected_buy_execution_fee =
				<pallet_balances::Pallet<Runtime>>::free_balance(&staking_pot_account_id) -
					existential_deposit;
			// check if SA reserve was withdrawn
			assert_eq!(
				<pallet_balances::Pallet<Runtime>>::free_balance(
					&remote_parachain_sovereign_account
				),
				remote_parachain_sovereign_account_balance_before -
					expected_buy_execution_fee -
					reserve_asset_deposisted.into()
			);
			// here target_account received reserve
			assert_eq!(
				<pallet_balances::Pallet<Runtime>>::free_balance(&target_account),
				existential_deposit + reserve_asset_deposisted.into()
			);
			assert_eq!(
				<pallet_balances::Pallet<Runtime>>::free_balance(
					&local_bridge_hub_multilocation_as_account_id
				),
				0.into()
			);
			assert_ne!(
				<pallet_balances::Pallet<Runtime>>::free_balance(&staking_pot_account_id),
				0.into()
			);

			// check NO asset trap occurred
			assert_eq!(
				false,
				<frame_system::Pallet<Runtime>>::events()
					.into_iter()
					.filter_map(|e| unwrap_pallet_xcm_event(e.event.encode()))
					.any(|e| matches!(e, pallet_xcm::Event::AssetsTrapped { .. }))
			);
		})
}
