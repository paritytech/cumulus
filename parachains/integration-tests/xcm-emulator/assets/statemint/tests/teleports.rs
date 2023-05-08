// use crate::PolkadotMockNet;
use statemint_it::*;
use frame_support::{
	assert_ok,
	instances::Instance1,
	pallet_prelude::Hooks,
	sp_io, sp_tracing,
	traits::{fungibles::Inspect, GenesisBuild},
};

use codec::Encode;
use polkadot_runtime_parachains::configuration::HostConfiguration;
use sp_core::parameter_types;
use statemint_runtime::constants::currency::DOLLARS;
use xcm::prelude::*;
use xcm_emulator::{TestExt, Parachain as Para, RelayChain as Relay};

use xcm_executor::traits::Convert;
use xcm::latest::Error;
use xcm_emulator::{cumulus_pallet_dmp_queue, bx};
use parachains_common::Balance;
use polkadot_core_primitives::InboundDownwardMessage;
use polkadot_parachain::primitives::DmpMessageHandler;
use sp_weights::Weight;
use xcm::{
	latest::{Ancestor, MultiLocation},
	prelude::{AccountId32, Here, Parachain},
	v3::Outcome,
	VersionedMultiAssets,
};

// use integration_tests_common::{AccountId, PolkadotMockNet, Polkadot, Statemint, PenpalPolkadot, constants::accounts::{ALICE, BOB}};

pub const INITIAL_BALANCE: u128 = 1000 * DOLLARS;

pub type RelayChainPalletXcm = pallet_xcm::Pallet<polkadot_runtime::Runtime>;
pub type StatemintPalletXcm = pallet_xcm::Pallet<statemint_runtime::Runtime>;
pub type PenpalPolkadotPalletXcm = pallet_xcm::Pallet<penpal_runtime::Runtime>;


#[test]
// NOTE: This needs to be run before every other test to ensure that chains can communicate with one
// another.
fn force_xcm_version() {
	let xcm_version = 3;
	PolkadotRelay::execute_with(|| {
		use polkadot_runtime::{RuntimeEvent, System};

		let statemint_location: MultiLocation = (Ancestor(0), Parachain(1000)).into();
		let penpal_location: MultiLocation = (Ancestor(0), Parachain(2000)).into();

		// Check that we can force xcm version for Statemint and PenpalPolkadot from Polkadot.
		for location in [statemint_location, penpal_location] {
			assert_ok!(RelayChainPalletXcm::force_xcm_version(
				polkadot_runtime::RuntimeOrigin::root(),
				Box::new(location),
				xcm_version,
			));
			assert!(System::events().iter().any(|r| matches!(
				r.event,
				RuntimeEvent::XcmPallet(pallet_xcm::Event::SupportedVersionChanged {
					0: loc,
					1: ver,
				}) if loc == location && ver == xcm_version
			)));
		}
	});

	// PenpalPolkadot forces Polkadot xcm version.
	PenpalPolkadot::execute_with(|| {
		use penpal_runtime::{RuntimeEvent, System};

		let location: MultiLocation = (Parent).into();

		assert_ok!(PenpalPolkadotPalletXcm::force_xcm_version(
			penpal_runtime::RuntimeOrigin::root(),
			Box::new(location),
			xcm_version,
		));

		assert!(System::events().iter().any(|r| matches!(
			r.event,
			RuntimeEvent::PolkadotXcm(pallet_xcm::Event::SupportedVersionChanged {
				0: loc,
				1: ver,
			}) if loc == location && ver == xcm_version
		)));
	});
}

// Direct message passing tests.

// use super::*;


#[test]
fn teleport_native_assets_relay_to_assets_para() {
	force_xcm_version();
	// const LALA: AccountId = Polkadot::account_id_of(ALICE);
	let amount = 1000_000_000;
	let assets: VersionedMultiAssets = (Here, amount).into();
	let relay_sender_balance_before = PolkadotRelay::account_data_of(PolkadotSender::get()).free;
	let para_receiver_balance_before = Statemint::account_data_of(StatemintReceiver::get()).free;

	let mut messages: Vec<InboundDownwardMessage> = Vec::new();

	PolkadotRelay::execute_with(|| {
		use polkadot_runtime::{RuntimeEvent, RuntimeOrigin, System};

		assert_ok!(RelayChainPalletXcm::limited_teleport_assets(
			RuntimeOrigin::signed(PolkadotSender::get()),
			bx!(
				(Ancestor(0), Parachain(Statemint::para_id().into())).into()
			),
			bx!(
				AccountId32 { network: None, id: Statemint::account_id_of(BOB).into()}.into())
				,
			bx!(assets),
			0,
			WeightLimit::Unlimited,
		));
		assert!(System::events().iter().any(|r| matches!(
			r.event,
			RuntimeEvent::XcmPallet(pallet_xcm::Event::Attempted(Outcome::Complete { .. }))
		)));
	});

	Statemint::execute_with(|| {
		use statemint_runtime::{Runtime, RuntimeEvent, System};
		assert!(System::events().iter().any(|r| matches!(
			&r.event,
			RuntimeEvent::Balances(pallet_balances::Event::Deposit { who, .. })
			if *who == Statemint::account_id_of(BOB).into()
		)));
	});

	// let (relay_balance_after, ap_balance_after) = get_balances();
	let relay_sender_balance_after = PolkadotRelay::account_data_of(PolkadotSender::get()).free;
	let para_sender_balance_after = Statemint::account_data_of(StatemintReceiver::get()).free;

	assert_eq!(relay_sender_balance_before - amount, relay_sender_balance_after);
	assert!(para_sender_balance_after > para_receiver_balance_before);
}

// #[test]
// fn transact_sudo_relay_to_assets_para_works() {
// 	force_xcm_version();

// 	Polkadot::execute_with(|| {
// 		use polkadot_runtime::{RuntimeEvent, RuntimeOrigin, System};

// 		let call = statemint_runtime::RuntimeCall::Assets(pallet_assets::Call::<
// 			statemint_runtime::Runtime,
// 			Instance1,
// 		>::force_create {
// 			id: 1.into(),
// 			is_sufficient: true,
// 			min_balance: 1000,
// 			owner: ALICE.into(),
// 		});
// 		let xcm = Xcm(vec![
// 			UnpaidExecution { weight_limit: WeightLimit::Unlimited, check_origin: None },
// 			Transact {
// 				require_weight_at_most: Weight::from_parts(1000000000, 200000),
// 				origin_kind: OriginKind::Superuser,
// 				call: call.encode().into(),
// 			},
// 		]);
// 		assert_ok!(RelayChainPalletXcm::send(
// 			RuntimeOrigin::root(),
// 			Box::new(AP_DEST.into()),
// 			Box::new(VersionedXcm::from(xcm)),
// 		));
// 		assert!(System::events().iter().any(|r| matches!(
// 			r.event,
// 			RuntimeEvent::XcmPallet(pallet_xcm::Event::Sent { .. })
// 		)));
// 	});

// 	Statemint::execute_with(|| {
// 		assert!(statemint_runtime::Assets::asset_exists(1));
// 	});
// }

// #[test]
// fn reserved_transfer_native_relay_to_assets_para_fails() {
// 	force_xcm_version();
// 	let (relay_balance, ap_balance) = get_balances();
// 	let amount = 1000_000_000;
// 	let assets: VersionedMultiAssets = (Here, amount).into();

// 	Polkadot::execute_with(|| {
// 		use polkadot_runtime::{RuntimeEvent, RuntimeOrigin, System};

// 		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
// 			RuntimeOrigin::signed(ALICE.into()),
// 			Box::new(AP_DEST.into()),
// 			Box::new(get_benf().into()),
// 			Box::new(assets),
// 			0,
// 			WeightLimit::Unlimited,
// 		));

// 		assert!(System::events().iter().any(|r| matches!(
// 			r.event,
// 			RuntimeEvent::XcmPallet(pallet_xcm::Event::Attempted(Outcome::Complete { .. }))
// 		)));
// 	});

// 	Statemint::execute_with(|| {
// 		use statemint_runtime::{RuntimeEvent, System};

// 		assert!(System::events().iter().any(|r| matches!(
// 			r.event,
// 			RuntimeEvent::DmpQueue(cumulus_pallet_dmp_queue::Event::ExecutedDownward {
// 				outcome: Outcome::Incomplete(_, Error::UntrustedReserveLocation),
// 				..
// 			})
// 		)));
// 	});

// 	let (relay_balance_after, ap_balance_after) = get_balances();
// 	assert_eq!(relay_balance - amount, relay_balance_after);
// 	assert_eq!(ap_balance_after, ap_balance);
// }
