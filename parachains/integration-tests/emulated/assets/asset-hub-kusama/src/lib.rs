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

use asset_hub_kusama_runtime::RuntimeOrigin;
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
	Test, TestArgs, AccountId32Junction, DispatchArgs, ParaId
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

pub const ASSET_ID: u32 = 1;
pub const ASSET_MIN_BALANCE: u128 = 1;
pub const ASSETS_PALLET_ID: u8 = 50;

pub fn get_relay_dispatch_args(amount: Balance) -> DispatchArgs {
	DispatchArgs {
		dest: Kusama::child_location_of(AssetHubKusama::para_id()).into(),
		beneficiary: AccountId32Junction {
			network: None,
			id: AssetHubKusamaReceiver::get().into()
		}.into(),
		amount,
		assets: (Here, amount).into(),
		fee_asset_item: 0,
		weight_limit: WeightLimit::Unlimited,
	}
}

pub fn get_system_para_dispatch_args(
	dest: VersionedMultiLocation,
	beneficiary_id: AccountId32,
	amount: Balance,
	assets: VersionedMultiAssets,
) -> DispatchArgs {
	DispatchArgs {
		dest,
		beneficiary: AccountId32Junction {
			network: None,
			id: beneficiary_id.into()
		}.into(),
		amount,
		assets,
		fee_asset_item: 0,
		weight_limit: WeightLimit::Unlimited,
	}
}

fn relay_send_transact_assertion(_t: RelayToSystemParaTest) {
	type RuntimeEvent = <Kusama as Chain>::RuntimeEvent;

	assert_expected_events!(
		Kusama,
		vec![
			RuntimeEvent::XcmPallet(
				pallet_xcm::Event::Sent { .. }
			) => {},
		]
	);
}

fn system_para_dest_assertions_asset_created(t: RelayToSystemParaTest) {
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
				asset_id: *asset_id == ASSET_ID,
				owner: *owner == t.sender.account_id,
			},
		]
	);

	assert!(<AssetHubKusama as AssetHubKusamaPallet>::Assets::asset_exists(ASSET_ID));
}

fn relay_to_system_para_force_create_asset(
	t: RelayToSystemParaTest,
) -> DispatchResult {
	let xcm = force_create_asset_xcm(
		OriginKind::Superuser,
		ASSET_ID,
		t.receiver.account_id,
		true,
		ASSET_MIN_BALANCE
	);

	<Kusama as KusamaPallet>::XcmPallet::send(
			t.root_origin,
			bx!(t.args.dest),
			bx!(xcm),
	)
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
	asset_id: u32,
	beneficiary: AccountId,
	amount: u128
) {
	AssetHubKusama::execute_with(|| {
		assert_ok!(
			<AssetHubKusama as AssetHubKusamaPallet>::Assets::mint(
				signed_origin,
				asset_id.into(),
				beneficiary.into(),
				amount
			)
		);

		type RuntimeEvent = <AssetHubKusama as Chain>::RuntimeEvent;

		assert_expected_events!(
			AssetHubKusama,
			vec![
				RuntimeEvent::Assets(pallet_assets::Event::Issued { .. }) => {},
			]
		);
	});
}

pub fn force_create_and_mint_asset(

) {
	// Init values for Relay Chain
	let mut amount_to_send: Balance = ASSET_HUB_KUSAMA_ED * 1000;

	let test_relay_args = TestArgs {
		sender: KusamaSender::get(),
		receiver: AssetHubKusamaSender::get(),
		args: get_relay_dispatch_args(
			amount_to_send
		),
	};

	let mut relay_test = RelayToSystemParaTest::new(test_relay_args);

	// Force create sufficent asset from Relay Chain
	relay_test.set_assertion::<Kusama>(relay_send_transact_assertion);
	relay_test.set_assertion::<AssetHubKusama>(system_para_dest_assertions_asset_created);
	relay_test.set_dispatchable::<Kusama>(relay_to_system_para_force_create_asset);
	relay_test.assert();

	// // Init values for System Parachain
	// let destination = AssetHubKusama::sibling_location_of(
	// 	PenpalKusamaA::para_id()
	// ).into();
	// let beneficiary_id = PenpalKusamaAReceiver::get().into();
	// let amount_to_mint = ASSET_MIN_BALANCE * 10000;
	// amount_to_send = ASSET_MIN_BALANCE * 1000;
	// // let assets = (Parent, amount_to_send).into();
	// let assets: VersionedMultiAssets = (
	// 	X2(PalletInstance(ASSETS_PALLET_ID), GeneralIndex(ASSET_ID.into())),
	// 	amount_to_send
	// ).into();

	// let system_para_test_args = TestArgs {
	// 	sender: AssetHubKusamaSender::get(),
	// 	receiver: PenpalKusamaAReceiver::get(),
	// 	args: get_system_para_dispatch_args(
	// 		destination,
	// 		beneficiary_id,
	// 		amount_to_send,
	// 		assets
	// 	),
	// };

	// let mut system_para_test
	// 	= SystemParaToParaTest::new(system_para_test_args);

	// Mint asset for System Parachain's sender
	mint_asset(
		<AssetHubKusama as Chain>::RuntimeOrigin::signed(
			AssetHubKusamaSender::get()
		),
		ASSET_ID,
		AssetHubKusamaSender::get().into(),
		ASSET_MIN_BALANCE * 10000,
	);
}

#[cfg(test)]
mod tests;
