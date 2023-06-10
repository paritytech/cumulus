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

use crate as bridge_transfer;
use frame_support::traits::{AsEnsureOriginWithArg, ConstU32, Contains, ContainsPair};

use crate::{
	features::{
		AllowedUniversalAliasesOf, IsAllowedReserveBasedAssetTransferForConcreteAsset,
		IsTrustedBridgedReserveForConcreteAsset,
	},
	pallet::{AllowedReserveLocations, AllowedUniversalAliases},
	types::{
		filter::{AssetFilter, MultiLocationFilter},
		BridgeConfig, LatestVersionedMultiLocation,
	},
	AllowedExporters, Config,
};
use frame_support::{
	assert_noop, assert_ok, dispatch::DispatchError, parameter_types, sp_io, sp_tracing, BoundedVec,
};
use frame_system::EnsureRoot;
use sp_runtime::{
	testing::{Header, H256},
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32, ModuleError,
};
use sp_version::RuntimeVersion;
use xcm::prelude::*;
use xcm_builder::ExporterFor;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

frame_support::construct_runtime!(
	pub enum TestRuntime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		BridgeTransfer: bridge_transfer::{Pallet, Call, Event<T>} = 52,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub Version: RuntimeVersion = RuntimeVersion {
		spec_name: sp_version::create_runtime_str!("test"),
		impl_name: sp_version::create_runtime_str!("system-test"),
		authoring_version: 1,
		spec_version: 1,
		impl_version: 1,
		apis: sp_version::create_apis_vec!([]),
		transaction_version: 1,
		state_version: 1,
	};
}

pub type AccountId = AccountId32;

impl frame_system::Config for TestRuntime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type BlockLength = ();
	type BlockWeights = ();
	type Version = Version;
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = frame_support::traits::Everything;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const BridgedNetwork: NetworkId = NetworkId::ByGenesis([4; 32]);
	pub const RelayNetwork: NetworkId = NetworkId::ByGenesis([9; 32]);
	pub UniversalLocation: InteriorMultiLocation = X2(GlobalConsensus(RelayNetwork::get()), Parachain(1000));
	// Relay chain currency/balance location (e.g. KsmLocation, DotLocation, ..)
	pub const RelayLocation: MultiLocation = MultiLocation::parent();
}

parameter_types! {
	pub const AssetsLimit: u8 = 1;
}

impl Config for TestRuntime {
	type RuntimeEvent = RuntimeEvent;
	type UniversalLocation = UniversalLocation;
	type WeightInfo = ();
	type AdminOrigin = EnsureRoot<AccountId>;
	type AllowReserveAssetTransferOrigin = AsEnsureOriginWithArg<EnsureRoot<AccountId>>;
	type UniversalAliasesLimit = ConstU32<2>;
	type ReserveLocationsLimit = ConstU32<2>;
	type AssetsPerReserveLocationLimit = ConstU32<2>;
	type TargetLocationsPerExporterLimit = ConstU32<2>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
	sp_tracing::try_init_simple();
	let t = frame_system::GenesisConfig::default().build_storage::<TestRuntime>().unwrap();

	// with 0 block_number events dont work
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		frame_system::Pallet::<TestRuntime>::set_block_number(1u32.into());
	});

	ext
}

fn account(account: u8) -> AccountId32 {
	AccountId32::new([account; 32])
}

#[test]
fn allowed_exporters_management_works() {
	let bridged_network = BridgedNetwork::get();
	let bridge_location = MultiLocation::new(1, X1(Parachain(1013)));
	let dummy_xcm = Xcm(vec![]);
	let dummy_remote_interior_multilocation = X1(Parachain(1234));

	new_test_ext().execute_with(|| {
		assert_eq!(AllowedExporters::<TestRuntime>::iter().count(), 0);

		// should fail - just root is allowed
		assert_noop!(
			BridgeTransfer::add_exporter_config(
				RuntimeOrigin::signed(account(1)),
				bridged_network,
				Box::new(bridge_location.clone().into_versioned()),
				None
			),
			DispatchError::BadOrigin
		);

		// should fail - we expect local bridge
		assert_noop!(
			BridgeTransfer::add_exporter_config(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(MultiLocation::new(2, X1(Parachain(1234))).into_versioned()),
				None
			),
			DispatchError::Module(ModuleError {
				index: 52,
				error: [0, 0, 0, 0],
				message: Some("InvalidConfiguration")
			})
		);
		assert_eq!(AllowedExporters::<TestRuntime>::iter().count(), 0);
		assert_eq!(
			BridgeTransfer::exporter_for(
				&bridged_network,
				&dummy_remote_interior_multilocation,
				&dummy_xcm
			),
			None
		);

		// add with root
		assert_ok!(BridgeTransfer::add_exporter_config(
			RuntimeOrigin::root(),
			bridged_network,
			Box::new(bridge_location.clone().into_versioned()),
			None
		));
		assert_eq!(AllowedExporters::<TestRuntime>::iter().count(), 1);
		assert_eq!(
			AllowedExporters::<TestRuntime>::get(bridged_network),
			Some(BridgeConfig::new(VersionedMultiLocation::from(bridge_location), None))
		);
		assert_eq!(AllowedExporters::<TestRuntime>::get(&RelayNetwork::get()), None);
		assert_eq!(
			BridgeTransfer::exporter_for(
				&bridged_network,
				&dummy_remote_interior_multilocation,
				&dummy_xcm
			),
			Some((bridge_location.clone(), None))
		);
		assert_eq!(
			BridgeTransfer::exporter_for(
				&RelayNetwork::get(),
				&dummy_remote_interior_multilocation,
				&dummy_xcm
			),
			None
		);

		// update fee
		// remove
		assert_ok!(BridgeTransfer::update_exporter_config(
			RuntimeOrigin::root(),
			bridged_network,
			Some(VersionedMultiAsset::V3((Parent, 200u128).into()).into()),
		));
		assert_eq!(AllowedExporters::<TestRuntime>::iter().count(), 1);
		assert_eq!(
			AllowedExporters::<TestRuntime>::get(bridged_network),
			Some(BridgeConfig::new(
				VersionedMultiLocation::from(bridge_location),
				Some((Parent, 200u128).into())
			))
		);
		assert_eq!(
			BridgeTransfer::exporter_for(
				&bridged_network,
				&dummy_remote_interior_multilocation,
				&dummy_xcm
			),
			Some((bridge_location, Some((Parent, 200u128).into())))
		);

		// remove
		assert_ok!(BridgeTransfer::remove_exporter_config(RuntimeOrigin::root(), bridged_network,));
		assert_eq!(AllowedExporters::<TestRuntime>::get(bridged_network), None);
		assert_eq!(AllowedExporters::<TestRuntime>::iter().count(), 0);
	})
}

#[test]
fn allowed_universal_aliases_management_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(AllowedUniversalAliases::<TestRuntime>::iter().count(), 0);

		let location1 = MultiLocation::new(1, X1(Parachain(1014)));
		let junction1 = GlobalConsensus(ByGenesis([1; 32]));
		let junction2 = GlobalConsensus(ByGenesis([2; 32]));

		// should fail - just root is allowed
		assert_noop!(
			BridgeTransfer::add_universal_alias(
				RuntimeOrigin::signed(account(1)),
				Box::new(VersionedMultiLocation::V3(location1.clone())),
				junction1.clone(),
			),
			DispatchError::BadOrigin
		);
		assert_eq!(AllowedUniversalAliases::<TestRuntime>::iter().count(), 0);
		assert!(!AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction1)));
		assert!(!AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction2)));

		// add ok
		assert_ok!(BridgeTransfer::add_universal_alias(
			RuntimeOrigin::root(),
			Box::new(VersionedMultiLocation::V3(location1.clone())),
			junction1.clone(),
		));
		assert_ok!(BridgeTransfer::add_universal_alias(
			RuntimeOrigin::root(),
			Box::new(VersionedMultiLocation::V3(location1.clone())),
			junction2.clone(),
		));
		assert!(AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction1)));
		assert!(AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction2)));
		assert_eq!(AllowedUniversalAliases::<TestRuntime>::iter().count(), 1);

		// remove ok
		assert_ok!(BridgeTransfer::remove_universal_alias(
			RuntimeOrigin::root(),
			Box::new(VersionedMultiLocation::V3(location1.clone())),
			vec![junction1.clone()],
		));
		assert!(!AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction1)));
		assert!(AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction2)));
		assert_eq!(AllowedUniversalAliases::<TestRuntime>::iter().count(), 1);

		assert_ok!(BridgeTransfer::remove_universal_alias(
			RuntimeOrigin::root(),
			Box::new(VersionedMultiLocation::V3(location1.clone())),
			vec![junction2.clone()],
		));
		assert!(!AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction1)));
		assert!(!AllowedUniversalAliasesOf::<TestRuntime>::contains(&(location1, junction2)));
		assert_eq!(AllowedUniversalAliases::<TestRuntime>::iter().count(), 0);
	})
}

#[test]
fn allowed_reserve_locations_management_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(0, AllowedReserveLocations::<TestRuntime>::iter_values().count());

		let location1 = MultiLocation::new(1, X1(Parachain(1014)));
		let location1_as_latest = VersionedMultiLocation::from(location1.clone());
		let location1_as_latest_as_key =
			LatestVersionedMultiLocation::try_from(&location1_as_latest).expect("ok");
		let location2 =
			MultiLocation::new(2, X2(GlobalConsensus(ByGenesis([1; 32])), Parachain(1014)));
		let location2_as_latest = VersionedMultiLocation::from(location2.clone());
		let location2_as_key =
			LatestVersionedMultiLocation::try_from(&location2_as_latest).expect("ok");

		let asset_location = MultiLocation::parent();
		let asset: MultiAsset = (asset_location, 200u128).into();

		let asset_filter_for_asset_by_multilocation = MultiLocationFilter {
			equals_any: BoundedVec::truncate_from(vec![asset_location.into_versioned()]),
			starts_with_any: Default::default(),
		};
		let asset_filter_for_asset =
			AssetFilter::ByMultiLocation(asset_filter_for_asset_by_multilocation.clone());
		let asset_filter_for_other_by_multilocation = MultiLocationFilter {
			equals_any: BoundedVec::truncate_from(vec![
				MultiLocation::new(3, Here).into_versioned()
			]),
			starts_with_any: Default::default(),
		};
		let asset_filter_for_other =
			AssetFilter::ByMultiLocation(asset_filter_for_other_by_multilocation.clone());

		// should fail - just root is allowed
		assert_noop!(
			BridgeTransfer::add_reserve_location(
				RuntimeOrigin::signed(account(1)),
				Box::new(location1_as_latest.clone()),
				asset_filter_for_asset.clone(),
			),
			DispatchError::BadOrigin
		);
		assert!(AllowedReserveLocations::<TestRuntime>::get(&location1_as_latest_as_key).is_none());
		assert!(AllowedReserveLocations::<TestRuntime>::get(&location2_as_key).is_none());
		assert!(!IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
			&asset, &location1
		));
		assert!(!IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
			&asset, &location2
		));

		// add ok
		assert_ok!(BridgeTransfer::add_reserve_location(
			RuntimeOrigin::root(),
			Box::new(VersionedMultiLocation::V3(location1.clone())),
			asset_filter_for_asset.clone()
		));
		assert_ok!(BridgeTransfer::add_reserve_location(
			RuntimeOrigin::root(),
			Box::new(VersionedMultiLocation::V3(location2.clone())),
			asset_filter_for_other.clone()
		));
		assert_eq!(2, AllowedReserveLocations::<TestRuntime>::iter_values().count());
		assert!(IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
			&asset, &location1
		));
		assert!(!IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
			&asset, &location2
		));

		assert_ok!(BridgeTransfer::add_reserve_location(
			RuntimeOrigin::root(),
			Box::new(location2_as_latest.clone()),
			asset_filter_for_asset.clone()
		));
		assert!(IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
			&asset, &location2
		));

		// test remove
		assert_noop!(
			BridgeTransfer::remove_reserve_location(
				RuntimeOrigin::root(),
				Box::new(location1_as_latest.clone()),
				Some(asset_filter_for_other_by_multilocation.clone())
			),
			DispatchError::Module(ModuleError {
				index: 52,
				error: [0, 0, 0, 0],
				message: Some("UnavailableConfiguration")
			})
		);

		assert_ok!(BridgeTransfer::remove_reserve_location(
			RuntimeOrigin::root(),
			Box::new(location1_as_latest.clone()),
			Some(asset_filter_for_asset_by_multilocation.clone())
		));
		assert_eq!(1, AllowedReserveLocations::<TestRuntime>::iter_values().count());
		assert!(!IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
			&asset, &location1
		));
		assert!(IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
			&asset, &location2
		));

		assert_ok!(BridgeTransfer::remove_reserve_location(
			RuntimeOrigin::root(),
			Box::new(location2_as_latest.clone()),
			Some(asset_filter_for_other_by_multilocation.clone())
		));
		assert!(!IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
			&asset, &location1
		));
		assert!(IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
			&asset, &location2
		));

		assert_ok!(BridgeTransfer::remove_reserve_location(
			RuntimeOrigin::root(),
			Box::new(location2_as_latest),
			Some(asset_filter_for_asset_by_multilocation)
		));
		assert!(!IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
			&asset, &location1
		));
		assert!(!IsTrustedBridgedReserveForConcreteAsset::<TestRuntime>::contains(
			&asset, &location2
		));
	})
}

#[test]
fn allowed_bridged_target_location_management_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(0, AllowedExporters::<TestRuntime>::iter_values().count());

		let bridged_network = BridgedNetwork::get();
		let bridge_location: MultiLocation = (Parent, Parachain(1013)).into();
		let target_location: MultiLocation =
			MultiLocation::new(2, X2(GlobalConsensus(bridged_network), Parachain(1000)));

		// should fail - we need BridgeConfig first
		assert_noop!(
			BridgeTransfer::allow_reserve_asset_transfer_for(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(target_location.clone().into_versioned()),
				AssetFilter::All,
			),
			DispatchError::Module(ModuleError {
				index: 52,
				error: [2, 0, 0, 0],
				message: Some("UnavailableConfiguration")
			})
		);

		// add bridge config
		assert_ok!(BridgeTransfer::add_exporter_config(
			RuntimeOrigin::root(),
			bridged_network,
			Box::new(bridge_location.into_versioned()),
			None,
		));

		// should fail - we need also target_location first
		assert_noop!(
			BridgeTransfer::allow_reserve_asset_transfer_for(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(target_location.clone().into_versioned()),
				AssetFilter::All,
			),
			DispatchError::Module(ModuleError {
				index: 52,
				error: [1, 0, 0, 0],
				message: Some("InvalidBridgeConfiguration")
			})
		);

		// insert allowed target location
		assert_ok!(BridgeTransfer::update_bridged_target_location(
			RuntimeOrigin::root(),
			bridged_network,
			Box::new(target_location.clone().into_versioned()),
			None,
		));

		let asset1_location = MultiLocation::new(2, X2(Parachain(1235), Parachain(5678)));
		let asset1 = MultiAsset::from((asset1_location, 1000));
		assert!(!IsAllowedReserveBasedAssetTransferForConcreteAsset::<TestRuntime>::contains(
			&asset1,
			&target_location
		));

		// now should pass - add one start_with pattern
		assert_ok!(BridgeTransfer::allow_reserve_asset_transfer_for(
			RuntimeOrigin::root(),
			bridged_network,
			Box::new(target_location.clone().into_versioned()),
			AssetFilter::ByMultiLocation(MultiLocationFilter {
				equals_any: Default::default(),
				starts_with_any: BoundedVec::truncate_from(vec![MultiLocation::new(
					2,
					X1(Parachain(2223))
				)
				.into_versioned()]),
			}),
		));

		// not allowed yet
		assert!(!IsAllowedReserveBasedAssetTransferForConcreteAsset::<TestRuntime>::contains(
			&asset1,
			&target_location
		));

		// now should pass - add another start_with pattern
		assert_ok!(BridgeTransfer::allow_reserve_asset_transfer_for(
			RuntimeOrigin::root(),
			bridged_network,
			Box::new(target_location.clone().into_versioned()),
			AssetFilter::ByMultiLocation(MultiLocationFilter {
				equals_any: Default::default(),
				starts_with_any: BoundedVec::truncate_from(vec![MultiLocation::new(
					2,
					X1(Parachain(1235))
				)
				.into_versioned()]),
			}),
		));

		// ok
		assert!(IsAllowedReserveBasedAssetTransferForConcreteAsset::<TestRuntime>::contains(
			&asset1,
			&target_location
		));
	})
}
