// Copyright Parity Technologies (UK) Ltd.
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

pub use codec::Encode;
pub use frame_support::{
	assert_err, assert_ok, instances::Instance1, pallet_prelude::Weight,
	traits::{fungibles::Inspect, OriginTrait},
	sp_runtime::{AccountId32, DispatchError, DispatchResult, MultiAddress}
};
pub use parachains_common::{AccountId, Balance};
pub use polkadot_core_primitives::InboundDownwardMessage;
pub use xcm::{
	prelude::*,
	v3::{Error, NetworkId::Kusama as KusamaId},
	DoubleEncoded,
};
pub use polkadot_parachain::primitives::{HrmpChannelId, Id};
pub use polkadot_runtime_parachains::inclusion::{AggregateMessageOrigin, UmpQueueId};
pub use xcm_emulator::{
	assert_expected_events, bx, cumulus_pallet_dmp_queue, helpers::weight_within_threshold, Chain,
	Parachain as Para, RelayChain as Relay, TestExt, TestExternalities,
	Test, TestContext, AccountId32Junction, TestArgs, ParaId
};
pub use integration_tests_common::{
	constants::{
		accounts::{ALICE, BOB},
		kusama::ED as KUSAMA_ED,
		asset_hub_kusama::ED as ASSET_HUB_KUSAMA_ED,
		PROOF_SIZE_THRESHOLD, REF_TIME_THRESHOLD, XCM_V3,
	},
	lazy_static::lazy_static,
	AssetHubKusama, AssetHubKusamaPallet, AssetHubKusamaReceiver, AssetHubKusamaSender,
	BridgeHubKusama, BridgeHubKusamaPallet, BridgeHubKusamaReceiver, BridgeHubKusamaSender,
	BridgeHubPolkadot, BridgeHubPolkadotPallet, BridgeHubPolkadotReceiver, BridgeHubPolkadotSender,
	Collectives, CollectivesPallet, CollectivesReceiver, CollectivesSender, Kusama, KusamaMockNet,
	KusamaPallet, KusamaReceiver, KusamaSender, PenpalKusamaA, PenpalKusamaAReceiver,
	PenpalKusamaASender, PenpalPolkadotA, PenpalPolkadotAReceiver, PenpalPolkadotASender, Polkadot,
	PolkadotMockNet, PolkadotPallet, PolkadotReceiver, PolkadotSender, PenpalKusamaAPallet,
	PenpalKusamaB, PenpalKusamaBReceiver, PenpalKusamaBSender, PenpalKusamaBPallet
};

pub type RelayToSystemParaTest = Test<Kusama, AssetHubKusama>;
pub type SystemParaToRelayTest = Test<AssetHubKusama, Kusama>;
pub type SystemParaToParaTest = Test<AssetHubKusama, PenpalKusamaA>;

pub fn get_relay_test_args(
	amount: Balance
) -> TestArgs {
	TestArgs {
		dest: Kusama::child_location_of(AssetHubKusama::para_id()),
		beneficiary: AccountId32Junction {
			network: None,
			id: AssetHubKusamaReceiver::get().into()
		}.into(),
		amount,
		assets: (Here, amount).into(),
		asset_id: None,
		fee_asset_item: 0,
		weight_limit: WeightLimit::Unlimited,
	}
}

pub fn get_system_para_test_args(
	dest: MultiLocation,
	beneficiary_id: AccountId32,
	amount: Balance,
	assets: MultiAssets,
	asset_id: Option<u32>,
) -> TestArgs {
	TestArgs {
		dest,
		beneficiary: AccountId32Junction {
			network: None,
			id: beneficiary_id.into()
		}.into(),
		amount,
		assets,
		asset_id,
		fee_asset_item: 0,
		weight_limit: WeightLimit::Unlimited,
	}
}

pub fn force_create_asset_xcm(
	origin_kind: OriginKind,
	asset_id: u32,
	owner: AccountId,
	is_sufficient: bool,
	min_balance: Balance
// ) -> VersionedXcm<<Kusama as Chain>::RuntimeCall> {
) -> VersionedXcm<()> {
	let call =
		<AssetHubKusama as Chain>::RuntimeCall::Assets(pallet_assets::Call::<
			<AssetHubKusama as Chain>::Runtime,
			Instance1,
		>::force_create {
			id: asset_id.into(),
			owner: owner.into(),
			is_sufficient,
			min_balance,
		})
		.encode()
		.into();

	let weight_limit = WeightLimit::Unlimited;
	let require_weight_at_most = Weight::from_parts(1000000000, 200000);
	let check_origin = None;

	VersionedXcm::from(Xcm(vec![
		UnpaidExecution {
			weight_limit,
			check_origin,
		},
		Transact {
			require_weight_at_most,
			origin_kind,
			call
		},
	]))
}

pub fn mint_asset(
	signed_origin: <AssetHubKusama as Chain>::RuntimeOrigin,
	id: u32,
	beneficiary: AccountId,
	amount_to_mint: u128
) {
	AssetHubKusama::execute_with(|| {
		assert_ok!(
			<AssetHubKusama as AssetHubKusamaPallet>::Assets::mint(
				signed_origin,
				id.into(),
				beneficiary.clone().into(),
				amount_to_mint
			)
		);

		type RuntimeEvent = <AssetHubKusama as Chain>::RuntimeEvent;

		assert_expected_events!(
			AssetHubKusama,
			vec![
				RuntimeEvent::Assets(pallet_assets::Event::Issued { asset_id, owner, amount }) => {
					asset_id: *asset_id == id,
					owner: *owner == beneficiary.clone().into(),
					amount: *amount == amount_to_mint,
				},
			]
		);
	});
}

pub fn force_create_and_mint_asset(
	id: u32,
	min_balance: u128,
	is_sufficient: bool,
	asset_owner: AccountId
) {
	// Init values for Relay Chain
	let root_origin = <Kusama as Chain>::RuntimeOrigin::root();
	let destination = Kusama::child_location_of(AssetHubKusama::para_id());
	let xcm = force_create_asset_xcm(
		OriginKind::Superuser,
		id,
		asset_owner.clone(),
		is_sufficient,
		min_balance
	);

	Kusama::execute_with(|| {
		assert_ok!(
			<Kusama as KusamaPallet>::XcmPallet::send(
				root_origin,
				bx!(destination.into()),
				bx!(xcm),
			)
		);

		type RuntimeEvent = <Kusama as Chain>::RuntimeEvent;

		assert_expected_events!(
			Kusama,
			vec![
				RuntimeEvent::XcmPallet(
					pallet_xcm::Event::Sent { .. }
				) => {},
			]
		);
	});

	AssetHubKusama::execute_with(|| {
		type RuntimeEvent = <AssetHubKusama as Chain>::RuntimeEvent;

		assert_expected_events!(
			AssetHubKusama,
			vec![
				// XCM message is succesfully executed in destination
				RuntimeEvent::DmpQueue(cumulus_pallet_dmp_queue::Event::ExecutedDownward {
					outcome: Outcome::Complete(weight), ..
				}) => {
					weight: weight_within_threshold(
						(REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD),
						Weight::from_parts(1_019_445_000, 200_000),
						*weight
					),
				},
				// Asset has been created
				RuntimeEvent::Assets(pallet_assets::Event::ForceCreated { asset_id, owner }) => {
					asset_id: *asset_id == id,
					owner: *owner == asset_owner.clone(),
				},
			]
		);

		assert!(<AssetHubKusama as AssetHubKusamaPallet>::Assets::asset_exists(id.into()));
	});

	let signed_origin = <AssetHubKusama as Chain>::RuntimeOrigin::signed(
		asset_owner.clone()
	);

	// Mint asset for System Parachain's sender
	mint_asset(
		signed_origin,
		id,
		asset_owner,
		min_balance * 10000,
	);
}

#[cfg(test)]
mod tests;
