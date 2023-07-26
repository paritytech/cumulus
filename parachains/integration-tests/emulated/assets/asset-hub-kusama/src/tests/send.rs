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

use crate::*;

const ASSET_ID: u32 = 1;

fn get_relay_xcm_message(asset_owner: &AccountId, origin_kind: OriginKind) -> VersionedXcm<()> {
	let call = <AssetHubKusama as Chain>::RuntimeCall::Assets(pallet_assets::Call::<
		<AssetHubKusama as Chain>::Runtime,
		Instance1,
	>::force_create {
		id: ASSET_ID.into(),
		is_sufficient: true,
		min_balance: 1000,
		owner: MultiAddress::Id(asset_owner.clone()),
	})
	.encode()
	.into();

	let weight_limit = WeightLimit::Unlimited;
	let require_weight_at_most = Weight::from_parts(1000000000, 200000);
	let check_origin = None;

	VersionedXcm::from(Xcm(vec![
		UnpaidExecution { weight_limit, check_origin },
		Transact { require_weight_at_most, origin_kind, call },
	]))
}

fn get_system_xcm_message(origin_kind: OriginKind) -> VersionedXcm<()> {
	let call = <Kusama as Chain>::RuntimeCall::System(frame_system::Call::<
		<Kusama as Chain>::Runtime
	>::remark { remark: vec![0, 1, 2, 3] })
	.encode()
	.into();

	let weight_limit = WeightLimit::Unlimited;
	let require_weight_at_most = Weight::from_parts(1000000000, 200000);
	let check_origin = None;

	VersionedXcm::from(Xcm(vec![
		UnpaidExecution { weight_limit, check_origin },
		Transact { require_weight_at_most, origin_kind, call },
	]))
}


/// Relay Chain should be able to execute `Transact` instructions in System Parachain
/// when `OriginKind::Superuser` and signer is `sudo`
#[test]
fn send_transact_sudo_from_relay_to_system_para() {
	// Init tests variables
	let root_origin = <Kusama as Chain>::RuntimeOrigin::root();
	let system_para_destination: VersionedMultiLocation =
		Kusama::child_location_of(AssetHubKusama::para_id()).into();
	let asset_owner: AccountId = AssetHubKusamaSender::get().into();
	let xcm = get_relay_xcm_message(&asset_owner, OriginKind::Superuser);

	// Send XCM message from Relay Chain
	Kusama::execute_with(|| {
		assert_ok!(<Kusama as KusamaPallet>::XcmPallet::send(
			root_origin,
			bx!(system_para_destination),
			bx!(xcm),
		));

		type RuntimeEvent = <Kusama as Chain>::RuntimeEvent;

		assert_expected_events!(
			Kusama,
			vec![
				RuntimeEvent::XcmPallet(pallet_xcm::Event::Sent { .. }) => {},
			]
		);
	});

	// Receive XCM message in Assets Parachain
	AssetHubKusama::execute_with(|| {

		type RuntimeEvent = <AssetHubKusama as Chain>::RuntimeEvent;

		assert_expected_events!(
			AssetHubKusama,
			vec![
				RuntimeEvent::Assets(pallet_assets::Event::ForceCreated { asset_id, owner }) => {
					asset_id: *asset_id == ASSET_ID,
					owner: *owner == asset_owner,
				},
				RuntimeEvent::DmpQueue(cumulus_pallet_dmp_queue::Event::ExecutedDownward {
					outcome: Outcome::Complete(weight), ..
				}) => {
					weight: weight_within_threshold(
						(REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD),
						Weight::from_parts(1_019_445_000, 200_000),
						*weight
					),
				},
			]
		);

		assert!(<AssetHubKusama as AssetHubKusamaPallet>::Assets::asset_exists(ASSET_ID));
	});
}

/// Relay Chain shouldn't be able to execute `Transact` instructions in System Parachain
/// when `OriginKind::Native`
#[test]
fn send_transact_native_from_relay_to_system_para() {
	// Init tests variables
	let signed_origin = <Kusama as Chain>::RuntimeOrigin::signed(KusamaSender::get().into());
	let system_para_destination: VersionedMultiLocation =
		Kusama::child_location_of(AssetHubKusama::para_id()).into();
	let asset_owner: AccountId = AssetHubKusamaSender::get().into();
	let xcm = get_relay_xcm_message(&asset_owner, OriginKind::Native);

	// Send XCM message from Relay Chain
	Kusama::execute_with(|| {
		assert_err!(
			<Kusama as KusamaPallet>::XcmPallet::send(
				signed_origin,
				bx!(system_para_destination),
				bx!(xcm)
			),
			DispatchError::BadOrigin
		);
	});
}

/// System Parachain shouldn't be able to execute `Transact` instructions in Relay Chain
/// when `OriginKind::Native`
#[test]
fn send_transact_native_from_system_para_to_relay() {
	// Init tests variables
	let signed_origin
		= <AssetHubKusama as Chain>::RuntimeOrigin::signed(AssetHubKusamaSender::get().into());
	let relay_para_destination: VersionedMultiLocation =
		AssetHubKusama::parent_location().into();
	let xcm = get_system_xcm_message(OriginKind::Native);

	// Send XCM message from Relay Chain
	AssetHubKusama::execute_with(|| {
		assert_err!(
			<AssetHubKusama as AssetHubKusamaPallet>::PolkadotXcm::send(
				signed_origin,
				bx!(relay_para_destination),
				bx!(xcm)
			),
			DispatchError::BadOrigin
		);
	});
}
