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
	xcm_paid_execution, xcm_unpaid_execution,
	AssetHubKusama, AssetHubKusamaPallet, AssetHubKusamaReceiver, AssetHubKusamaSender,
	BridgeHubKusama, BridgeHubKusamaPallet, BridgeHubKusamaReceiver, BridgeHubKusamaSender,
	BridgeHubPolkadot, BridgeHubPolkadotPallet, BridgeHubPolkadotReceiver, BridgeHubPolkadotSender,
	Collectives, CollectivesPallet, CollectivesReceiver, CollectivesSender, Kusama, KusamaMockNet,
	KusamaPallet, KusamaReceiver, KusamaSender, PenpalKusamaA, PenpalKusamaAReceiver,
	PenpalKusamaASender, PenpalPolkadotA, PenpalPolkadotAReceiver, PenpalPolkadotASender, Polkadot,
	PolkadotMockNet, PolkadotPallet, PolkadotReceiver, PolkadotSender, PenpalKusamaAPallet,
	PenpalKusamaB, PenpalKusamaBReceiver, PenpalKusamaBSender, PenpalKusamaBPallet
};

pub const ASSET_ID: u32 = 1;
pub const ASSET_MIN_BALANCE: u128 = 1000;
pub const ASSETS_PALLET_ID: u8 = 50;

pub type RelayToSystemParaTest = Test<Kusama, AssetHubKusama>;
pub type SystemParaToRelayTest = Test<AssetHubKusama, Kusama>;
pub type SystemParaToParaTest = Test<AssetHubKusama, PenpalKusamaA>;

pub fn relay_test_args(
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

pub fn system_para_test_args(
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

pub mod events {
	pub mod relay_chain {
		use crate::*;
		type RuntimeEvent = <Kusama as Chain>::RuntimeEvent;

		// Dispatchable is completely executed and XCM sent
		pub fn xcm_pallet_attempted_complete(expected_weight: Option<Weight>) {
			assert_expected_events!(
				Kusama,
				vec![
					RuntimeEvent::XcmPallet(
						pallet_xcm::Event::Attempted { outcome: Outcome::Complete(weight) }
					) => {
						weight: weight_within_threshold(
							(REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD),
							expected_weight.unwrap_or(*weight),
							*weight
						),
					},
				]
			);
		}

		// Dispatchable is incompletely executed and XCM sent
		pub fn xcm_pallet_attempted_incomplete(expected_weight: Option<Weight>, expected_error: Option<Error>) {
			assert_expected_events!(
				Kusama,
				vec![
					// Dispatchable is properly executed and XCM message sent
					RuntimeEvent::XcmPallet(
						pallet_xcm::Event::Attempted { outcome: Outcome::Incomplete(weight, error) }
					) => {
						weight: weight_within_threshold(
							(REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD),
							expected_weight.unwrap_or(*weight),
							*weight
						),
						error: *error == expected_error.unwrap_or(*error),
					},
				]
			);
		}

		// XCM message is sent
		pub fn xcm_pallet_sent() {
			assert_expected_events!(
				Kusama,
				vec![
					RuntimeEvent::XcmPallet(pallet_xcm::Event::Sent { .. }) => {},
				]
			);
		}

		// XCM from System Parachain is succesfully received and proccessed
		pub fn ump_queue_processed(
			expected_success: bool,
			expected_id: Option<ParaId>,
			expected_weight: Option<Weight>
		) {
			assert_expected_events!(
				Kusama,
				vec![
					// XCM is succesfully received and proccessed
					RuntimeEvent::MessageQueue(pallet_message_queue::Event::Processed {
						origin: AggregateMessageOrigin::Ump(UmpQueueId::Para(id)),
						weight_used,
						success,
						..
					}) => {
						id: *id == expected_id.unwrap_or(*id),
						weight_used: weight_within_threshold(
							(REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD),
							expected_weight.unwrap_or(*weight_used),
							*weight_used
						),
						success: *success == expected_success,
					},
				]
			);
		}

	}

	pub mod parachain {
		use crate::*;
		type RuntimeEvent = <AssetHubKusama as Chain>::RuntimeEvent;

		// Dispatchable is completely executed and XCM sent
		pub fn xcm_pallet_attempted_complete(expected_weight: Option<Weight>) {
			assert_expected_events!(
				AssetHubKusama,
				vec![
					RuntimeEvent::PolkadotXcm(
						pallet_xcm::Event::Attempted { outcome: Outcome::Complete(weight) }
					) => {
						weight: weight_within_threshold(
							(REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD),
							expected_weight.unwrap_or(*weight),
							*weight
						),
					},
				]
			);
		}

		// Dispatchable is incompletely executed and XCM sent
		pub fn xcm_pallet_attempted_incomplete(expected_weight: Option<Weight>, expected_error: Option<Error>) {
			assert_expected_events!(
				AssetHubKusama,
				vec![
					// Dispatchable is properly executed and XCM message sent
					RuntimeEvent::PolkadotXcm(
						pallet_xcm::Event::Attempted { outcome: Outcome::Incomplete(weight, error) }
					) => {
						weight: weight_within_threshold(
							(REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD),
							expected_weight.unwrap_or(*weight),
							*weight
						),
						error: *error == expected_error.unwrap_or(*error),
					},
				]
			);
		}

		// Dispatchable throws and error when trying to be sent
		pub fn xcm_pallet_attempted_error(expected_error: Option<Error>) {
			assert_expected_events!(
				AssetHubKusama,
				vec![
					// Execution fails in the origin with `Barrier`
					RuntimeEvent::PolkadotXcm(
						pallet_xcm::Event::Attempted { outcome: Outcome::Error(error) }
					) => {
						error: *error == expected_error.unwrap_or(*error),
					},
				]
			);
		}

		// XCM message is sent
		pub fn xcm_pallet_sent() {
			assert_expected_events!(
				AssetHubKusama,
				vec![
					RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
				]
			);
		}

		// XCM message is sent to Relay Chain
		pub fn parachain_system_ump_sent() {
			assert_expected_events!(
				AssetHubKusama,
				vec![
					RuntimeEvent::ParachainSystem(
						cumulus_pallet_parachain_system::Event::UpwardMessageSent { .. }
					) => {},
				]
			);
		}

		// XCM from Relay Chain is completely executed
		pub fn dmp_queue_complete(expected_weight: Option<Weight>) {
			assert_expected_events!(
				AssetHubKusama,
				vec![
					RuntimeEvent::DmpQueue(cumulus_pallet_dmp_queue::Event::ExecutedDownward {
						outcome: Outcome::Complete(weight), ..
					}) => {
						weight: weight_within_threshold(
							(REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD),
							expected_weight.unwrap_or(*weight),
							*weight
						),
					},
				]
			);
		}

		// XCM from Relay Chain is incompletely executed
		pub fn dmp_queue_incomplete(expected_weight: Option<Weight>, expected_error: Option<Error>) {
			assert_expected_events!(
				AssetHubKusama,
				vec![
					RuntimeEvent::DmpQueue(cumulus_pallet_dmp_queue::Event::ExecutedDownward {
						outcome: Outcome::Incomplete(weight, error), ..
					}) => {
						weight: weight_within_threshold(
							(REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD),
							expected_weight.unwrap_or(*weight),
							*weight
						),
						error: *error == expected_error.unwrap_or(*error),
					},
				]
			);
		}

		// XCM from another Parachain is completely executed
		pub fn xcmp_queue_success(expected_weight: Option<Weight>) {
			assert_expected_events!(
				AssetHubKusama,
				vec![
					RuntimeEvent::XcmpQueue(
						cumulus_pallet_xcmp_queue::Event::Success { weight, .. }
					) => {
						weight: weight_within_threshold(
							(REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD),
							expected_weight.unwrap_or(*weight),
							*weight
						),
					},
				]
			);
		}
	}
}

pub fn force_create_call(
	asset_id: u32,
	owner: AccountId,
	is_sufficient: bool,
	min_balance: Balance
) -> DoubleEncoded<()> {
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
	.into()
}

pub fn force_create_asset_xcm(
	origin_kind: OriginKind,
	asset_id: u32,
	owner: AccountId,
	is_sufficient: bool,
	min_balance: Balance
) -> VersionedXcm<()> {
	let call = force_create_call(
		asset_id,
		owner,
		is_sufficient,
		min_balance
	);
	xcm_unpaid_execution(call, origin_kind)
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
	asset_owner: AccountId,
	amount_to_mint: u128,
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

		events::relay_chain::xcm_pallet_sent();
	});

	AssetHubKusama::execute_with(|| {
		type RuntimeEvent = <AssetHubKusama as Chain>::RuntimeEvent;

		events::parachain::dmp_queue_complete(
			Some(Weight::from_parts(1_019_445_000, 200_000))
		);

		assert_expected_events!(
			AssetHubKusama,
			vec![
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
		amount_to_mint,
	);
}

#[cfg(test)]
mod tests;
