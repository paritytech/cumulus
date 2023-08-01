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

pub use bp_messages::LaneId;
pub use codec::Encode;
pub use frame_support::{
	assert_err, assert_ok,
	instances::Instance1,
	pallet_prelude::Weight,
	sp_runtime::{AccountId32, DispatchError, DispatchResult, MultiAddress},
	traits::{fungibles::Inspect, OriginTrait},
};
pub use integration_tests_common::{
	constants::{
		accounts::{ALICE, BOB},
		asset_hub_kusama::ED as ASSET_HUB_ROCOCO_ED,
		kusama::ED as ROCOCO_ED,
		PROOF_SIZE_THRESHOLD, REF_TIME_THRESHOLD, XCM_V3,
	},
	lazy_static::lazy_static,
	xcm_paid_execution, xcm_unpaid_execution, AssetHubRococo, AssetHubRococoPallet,
	AssetHubRococoReceiver, AssetHubRococoSender, AssetHubWococo, AssetHubWococoPallet,
	AssetHubWococoReceiver, AssetHubWococoSender, BridgeHubRococo, BridgeHubRococoPallet,
	BridgeHubRococoReceiver, BridgeHubRococoSender, BridgeHubWococo, BridgeHubWococoPallet,
	BridgeHubWococoReceiver, BridgeHubWococoSender, Collectives, CollectivesPallet,
	CollectivesReceiver, CollectivesSender, PenpalRococoA, PenpalRococoAPallet,
	PenpalRococoAReceiver, PenpalRococoASender, Rococo, RococoMockNet, RococoPallet,
	RococoReceiver, RococoSender, Wococo, WococoMockNet, WococoPallet, WococoReceiver,
	WococoSender,
};
pub use parachains_common::{AccountId, Balance};
pub use polkadot_core_primitives::InboundDownwardMessage;
pub use polkadot_parachain::primitives::{HrmpChannelId, Id};
pub use polkadot_runtime_parachains::inclusion::{AggregateMessageOrigin, UmpQueueId};
pub use xcm::{
	prelude::*,
	v3::{
		Error,
		NetworkId::{Rococo as RococoId, Wococo as WococoId},
	},
	DoubleEncoded,
};
pub use xcm_emulator::{
	assert_expected_events, bx, cumulus_pallet_dmp_queue, helpers::weight_within_threshold,
	AccountId32Junction, Chain, ParaId, Parachain as Para, RelayChain as Relay, Test, TestArgs,
	TestContext, TestExt, TestExternalities,
};

pub const ASSET_ID: u32 = 1;
pub const ASSET_MIN_BALANCE: u128 = 1000;
pub const ASSETS_PALLET_ID: u8 = 50;

pub type RelayToSystemParaTest = Test<Rococo, AssetHubRococo>;
pub type SystemParaToRelayTest = Test<AssetHubRococo, Rococo>;
pub type SystemParaToParaTest = Test<AssetHubRococo, PenpalRococoA>;

pub fn relay_test_args(amount: Balance) -> TestArgs {
	TestArgs {
		dest: Rococo::child_location_of(AssetHubRococo::para_id()),
		beneficiary: AccountId32Junction {
			network: None,
			id: AssetHubRococoReceiver::get().into(),
		}
		.into(),
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
		beneficiary: AccountId32Junction { network: None, id: beneficiary_id.into() }.into(),
		amount,
		assets,
		asset_id,
		fee_asset_item: 0,
		weight_limit: WeightLimit::Unlimited,
	}
}

pub mod events {
	pub mod relay_chain {
		pub use integration_tests_common::events::rococo::{
			ump_queue_processed, xcm_pallet_attempted_complete, xcm_pallet_attempted_incomplete,
			xcm_pallet_sent,
		};
	}

	pub mod parachain {
		use crate::*;
		type RuntimeEvent = <AssetHubRococo as Chain>::RuntimeEvent;

		// Dispatchable is completely executed and XCM sent
		pub fn xcm_pallet_attempted_complete(expected_weight: Option<Weight>) {
			assert_expected_events!(
				AssetHubRococo,
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
		pub fn xcm_pallet_attempted_incomplete(
			expected_weight: Option<Weight>,
			expected_error: Option<Error>,
		) {
			assert_expected_events!(
				AssetHubRococo,
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
				AssetHubRococo,
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
				AssetHubRococo,
				vec![
					RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
				]
			);
		}

		// XCM message is sent to Relay Chain
		pub fn parachain_system_ump_sent() {
			assert_expected_events!(
				AssetHubRococo,
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
				AssetHubRococo,
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
		pub fn dmp_queue_incomplete(
			expected_weight: Option<Weight>,
			expected_error: Option<Error>,
		) {
			assert_expected_events!(
				AssetHubRococo,
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
				AssetHubRococo,
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
	min_balance: Balance,
) -> DoubleEncoded<()> {
	<AssetHubRococo as Chain>::RuntimeCall::Assets(pallet_assets::Call::<
		<AssetHubRococo as Chain>::Runtime,
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
	min_balance: Balance,
) -> VersionedXcm<()> {
	let call = force_create_call(asset_id, owner, is_sufficient, min_balance);
	xcm_unpaid_execution(call, origin_kind)
}

pub fn mint_asset(
	signed_origin: <AssetHubRococo as Chain>::RuntimeOrigin,
	id: u32,
	beneficiary: AccountId,
	amount_to_mint: u128,
) {
	AssetHubRococo::execute_with(|| {
		assert_ok!(<AssetHubRococo as AssetHubRococoPallet>::Assets::mint(
			signed_origin,
			id.into(),
			beneficiary.clone().into(),
			amount_to_mint
		));

		type RuntimeEvent = <AssetHubRococo as Chain>::RuntimeEvent;

		assert_expected_events!(
			AssetHubRococo,
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
	let root_origin = <Rococo as Chain>::RuntimeOrigin::root();
	let destination = Rococo::child_location_of(AssetHubRococo::para_id());
	let xcm = force_create_asset_xcm(
		OriginKind::Superuser,
		id,
		asset_owner.clone(),
		is_sufficient,
		min_balance,
	);

	Rococo::execute_with(|| {
		assert_ok!(<Rococo as RococoPallet>::XcmPallet::send(
			root_origin,
			bx!(destination.into()),
			bx!(xcm),
		));

		events::relay_chain::xcm_pallet_sent();
	});

	AssetHubRococo::execute_with(|| {
		type RuntimeEvent = <AssetHubRococo as Chain>::RuntimeEvent;

		events::parachain::dmp_queue_complete(Some(Weight::from_parts(1_019_445_000, 200_000)));

		assert_expected_events!(
			AssetHubRococo,
			vec![
				// Asset has been created
				RuntimeEvent::Assets(pallet_assets::Event::ForceCreated { asset_id, owner }) => {
					asset_id: *asset_id == id,
					owner: *owner == asset_owner.clone(),
				},
			]
		);

		assert!(<AssetHubRococo as AssetHubRococoPallet>::Assets::asset_exists(id.into()));
	});

	let signed_origin = <AssetHubRococo as Chain>::RuntimeOrigin::signed(asset_owner.clone());

	// Mint asset for System Parachain's sender
	mint_asset(signed_origin, id, asset_owner, amount_to_mint);
}

#[cfg(test)]
mod tests;
