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

const MAX_CAPACITY: u32 = 8;
const MAX_MESSAGE_SIZE: u32 = 8192;

fn fund_para_sovereign(amount: Balance, sovereign_account_id: AccountId32) {
	Polkadot::execute_with(|| {
		assert_ok!(
			<Polkadot as PolkadotPallet>::Balances::force_set_balance(
				<Polkadot as Chain>::RuntimeOrigin::root(),
				MultiAddress::Id(sovereign_account_id),
				amount,
			)
		);
	});
}

fn init_open_channel_call(recipient_para_id: ParaId) -> DoubleEncoded<()> {
	<Polkadot as Chain>::RuntimeCall::Hrmp(polkadot_runtime_parachains::hrmp::Call::<
		<Polkadot as Chain>::Runtime
	>::hrmp_init_open_channel {
		recipient: recipient_para_id,
		proposed_max_capacity: MAX_CAPACITY,
		proposed_max_message_size: MAX_MESSAGE_SIZE
	})
	.encode()
	.into()
}

fn accept_open_channel_call(sender_para_id: ParaId) -> DoubleEncoded<()> {
	<Polkadot as Chain>::RuntimeCall::Hrmp(polkadot_runtime_parachains::hrmp::Call::<
		<Polkadot as Chain>::Runtime
	>::hrmp_accept_open_channel { sender: sender_para_id })
	.encode()
	.into()
}

fn force_process_hrmp_open(sender: Id, recipient: Id) {
	Polkadot::execute_with(|| {
		let relay_root_origin = <Polkadot as Chain>::RuntimeOrigin::root();

		// Force process HRMP open channel requests without waiting for the next session
		assert_ok!(
			<Polkadot as PolkadotPallet>::Hrmp::force_process_hrmp_open(
				relay_root_origin,
				0
			)
		);

		let channel_id = HrmpChannelId {
			sender,
			recipient
		};

		let hrmp_channel_exist
			= polkadot_runtime_parachains::hrmp::HrmpChannels::<<Polkadot as Chain>::Runtime>::contains_key(&channel_id);

		// Check the HRMP channel has been successfully registrered
		assert!(hrmp_channel_exist)
	});
}

/// Opening HRMP channels between Parachains should work
#[test]
fn open_hrmp_channel_between_paras_works() {
	// Parchain A init values
	let para_a_id = PenpalPolkadotA::para_id();
	let para_a_root_origin = <PenpalPolkadotA as Chain>::RuntimeOrigin::root();
	let para_a_sovereign_account = Polkadot::sovereign_account_id_of(
		Polkadot::child_location_of(para_a_id)
	);

	// Parachain B init values
	let para_b_id = PenpalPolkadotB::para_id();
	let para_b_root_origin = <PenpalPolkadotB as Chain>::RuntimeOrigin::root();
	let para_b_sovereign_account = Polkadot::sovereign_account_id_of(
		Polkadot::child_location_of(para_b_id)
	);

	let fee_amount = POLKADOT_ED * 1000;
	let fund_amount = POLKADOT_ED * 1000_000_000;

	// Fund Parachain's Sovereign accounts to be able to reserve the deposit
	fund_para_sovereign(fund_amount,  para_a_sovereign_account.clone());
	fund_para_sovereign(fund_amount, para_b_sovereign_account.clone());

	let relay_destination: VersionedMultiLocation = PenpalPolkadotA::parent_location().into();

	// ---- Init Open channel from Parachain to System Parachain
	let mut call = init_open_channel_call(para_b_id);
	let origin_kind = OriginKind::Native;
	let native_asset: MultiAsset = (Here, fee_amount).into();
	let beneficiary= Polkadot::sovereign_account_id_of(
		Polkadot::child_location_of(para_a_id)
	);

	let mut xcm = xcm_paid_execution(
		call,
		origin_kind,
		native_asset.clone(),
		beneficiary
	);

	PenpalPolkadotA::execute_with(|| {
		assert_ok!(
			<PenpalPolkadotA as PenpalPolkadotAPallet>::PolkadotXcm::send(
				para_a_root_origin,
				bx!(relay_destination.clone()),
				bx!(xcm),
		));

		events::parachain::xcm_pallet_sent();
	});

	Polkadot::execute_with(|| {
		type RuntimeEvent = <Polkadot as Chain>::RuntimeEvent;

		events::relay_chain::ump_queue_processed(
			true,
			Some(para_a_id),
			Some(Weight::from_parts(1_282_426_000, 207_186)),

		);

		assert_expected_events!(
			Polkadot,
			vec![
				// Parachain's Sovereign account balance is withdrawn to pay XCM fees
				RuntimeEvent::Balances(pallet_balances::Event::Withdraw { who, amount }) => {
					who: *who == para_a_sovereign_account.clone(),
					amount: *amount == fee_amount,
				},
				// Sender deposit is reserved for Parachain's Sovereign account
				RuntimeEvent::Balances(pallet_balances::Event::Reserved { who, .. }) =>{
					who: *who == para_a_sovereign_account,
				},
				// Open channel requested from Para A to Para B
				RuntimeEvent::Hrmp(
					polkadot_runtime_parachains::hrmp::Event::OpenChannelRequested(
						sender, recipient, max_capacity, max_message_size
					)
				) => {
					sender: *sender == para_a_id.into(),
					recipient: *recipient == para_b_id.into(),
					max_capacity: *max_capacity == MAX_CAPACITY,
					max_message_size: *max_message_size == MAX_MESSAGE_SIZE,
				},
			]
		);
	});

	// ---- Accept Open channel from Parachain to System Parachain
	call = accept_open_channel_call(para_a_id);
	let beneficiary= Polkadot::sovereign_account_id_of(
		Polkadot::child_location_of(para_b_id)
	);

	xcm = xcm_paid_execution(
		call,
		origin_kind,
		native_asset,
		beneficiary
	);

	PenpalPolkadotB::execute_with(|| {
		assert_ok!(
			<PenpalPolkadotB as PenpalPolkadotBPallet>::PolkadotXcm::send(
				para_b_root_origin,
				bx!(relay_destination),
				bx!(xcm),
		));

		events::parachain::xcm_pallet_sent();
	});

	Polkadot::execute_with(|| {
		type RuntimeEvent = <Polkadot as Chain>::RuntimeEvent;

		events::relay_chain::ump_queue_processed(
			true,
			Some(para_b_id),
			Some(Weight::from_parts(1_282_426_000, 207_186)),
		);

		assert_expected_events!(
			Polkadot,
			vec![
				// Parachain's Sovereign account balance is withdrawn to pay XCM fees
				RuntimeEvent::Balances(pallet_balances::Event::Withdraw { who, amount }) => {
					who: *who == para_b_sovereign_account.clone(),
					amount: *amount == fee_amount,
				},
				// Sender deposit is reserved for Parachain's Sovereign account
				RuntimeEvent::Balances(pallet_balances::Event::Reserved { who, .. }) =>{
					who: *who == para_b_sovereign_account,
				},
				// Open channel accepted for Para A to Para B
				RuntimeEvent::Hrmp(
					polkadot_runtime_parachains::hrmp::Event::OpenChannelAccepted(
						sender, recipient
					)
				) => {
					sender: *sender == para_a_id.into(),
					recipient: *recipient == para_b_id.into(),
				},
			]
		);
	});

	force_process_hrmp_open(para_a_id, para_b_id);
}

/// Opening HRMP channels between System Parachains and Parachains should work
#[test]
fn force_open_hrmp_channel_for_system_para_works() {
	// Relay Chain init values
	let relay_root_origin = <Polkadot as Chain>::RuntimeOrigin::root();

	// System Para init values
	let system_para_id = AssetHubPolkadot::para_id();
	let system_para_sovereign_account = Polkadot::sovereign_account_id_of(
		Polkadot::child_location_of(system_para_id)
	);

	// Parachain A init values
	let para_a_id = PenpalPolkadotA::para_id();
	let para_a_sovereign_account = Polkadot::sovereign_account_id_of(
		Polkadot::child_location_of(para_a_id)
	);

	let fund_amount = POLKADOT_ED * 1000_000_000;

	// Fund Parachain's Sovereign accounts to be able to reserve the deposit
	fund_para_sovereign(fund_amount,  system_para_sovereign_account.clone());
	fund_para_sovereign(fund_amount, para_a_sovereign_account.clone());

	Polkadot::execute_with(|| {
		assert_ok!(
			<Polkadot as PolkadotPallet>::Hrmp::force_open_hrmp_channel(
				relay_root_origin,
				system_para_id,
				para_a_id,
				MAX_CAPACITY,
				MAX_MESSAGE_SIZE
			)
		);

		type RuntimeEvent = <Polkadot as Chain>::RuntimeEvent;

		assert_expected_events!(
			Polkadot,
			vec![
				// Sender deposit is reserved for System Parachain's Sovereign account
				RuntimeEvent::Balances(pallet_balances::Event::Reserved { who, .. }) =>{
					who: *who == system_para_sovereign_account,
				},
				// Recipient deposit is reserved for Parachain's Sovereign account
				RuntimeEvent::Balances(pallet_balances::Event::Reserved { who, .. }) =>{
					who: *who == para_a_sovereign_account,
				},
				// HRMP channel forced opened
				RuntimeEvent::Hrmp(
					polkadot_runtime_parachains::hrmp::Event::HrmpChannelForceOpened(
						sender, recipient, max_capacity, max_message_size
					)
				) => {
					sender: *sender == system_para_id.into(),
					recipient: *recipient == para_a_id.into(),
					max_capacity: *max_capacity == MAX_CAPACITY,
					max_message_size: *max_message_size == MAX_MESSAGE_SIZE,
				},
			]
		);
	});

	force_process_hrmp_open(system_para_id, para_a_id);
}
