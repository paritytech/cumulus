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
	Kusama::execute_with(|| {
		assert_ok!(
			<Kusama as KusamaPallet>::Balances::force_set_balance(
				<Kusama as Chain>::RuntimeOrigin::root(),
				MultiAddress::Id(sovereign_account_id),
				amount,
			)
		);
	});
}

fn get_init_open_channel_call(recipient_para_id: ParaId) -> DoubleEncoded<()> {
	<Kusama as Chain>::RuntimeCall::Hrmp(polkadot_runtime_parachains::hrmp::Call::<
		<Kusama as Chain>::Runtime
	>::hrmp_init_open_channel {
		recipient: recipient_para_id,
		proposed_max_capacity: MAX_CAPACITY,
		proposed_max_message_size: MAX_MESSAGE_SIZE
	})
	.encode()
	.into()
}

fn get_accept_open_channel_call(sender_para_id: ParaId) -> DoubleEncoded<()> {
	<Kusama as Chain>::RuntimeCall::Hrmp(polkadot_runtime_parachains::hrmp::Call::<
		<Kusama as Chain>::Runtime
	>::hrmp_accept_open_channel { sender: sender_para_id })
	.encode()
	.into()
}

fn get_xcm_message(call: DoubleEncoded<()>, fee_amount: Balance, para_id: ParaId) -> VersionedXcm<()> {
	let weight_limit = WeightLimit::Unlimited;
	let require_weight_at_most = Weight::from_parts(1000000000, 200000);
	let origin_kind = OriginKind::Native;
	let native_asset = MultiAsset {
		id: Concrete(MultiLocation { parents: 0, interior: Here }),
		fun: Fungible(fee_amount),
	};
	let native_assets: MultiAssets = native_asset.clone().into();
	let beneficiary= Kusama::child_location_of(para_id);

	VersionedXcm::from(Xcm(vec![
		WithdrawAsset(native_assets),
		BuyExecution { fees: native_asset , weight_limit },
		Transact { require_weight_at_most, origin_kind, call },
		RefundSurplus,
		DepositAsset {
			assets: All.into(),
			beneficiary
		},
	]))
}

#[test]
fn open_hrmp_channel_between_paras_works() {
	// Relay Chain init values
	let relay_root_origin = <Kusama as Chain>::RuntimeOrigin::root();

	// Parchain A init values
	let para_a_id = PenpalKusamaA::para_id();
	let para_a_root_origin = <PenpalKusamaA as Chain>::RuntimeOrigin::root();
	let para_a_sovereign_account = Kusama::sovereign_account_id_of(
		Kusama::child_location_of(para_a_id)
	);

	// Parachain B init values
	let para_b_id = PenpalKusamaB::para_id();
	let para_b_root_origin = <PenpalKusamaB as Chain>::RuntimeOrigin::root();
	let para_b_sovereign_account = Kusama::sovereign_account_id_of(
		Kusama::child_location_of(para_b_id)
	);

	let fee_amount = KUSAMA_ED * 1000;
	let fund_amount = KUSAMA_ED * 1000_000_000;

	// Fund Parachain's Sovereign accounts to be able to reserve the deposit
	fund_para_sovereign(fund_amount,  para_a_sovereign_account.clone());
	fund_para_sovereign(fund_amount, para_b_sovereign_account.clone());

	let relay_destination: VersionedMultiLocation = PenpalKusamaA::parent_location().into();

	// ---- Init Open channel from Parachain to System Parachain
	let mut call = get_init_open_channel_call(para_b_id);
	let mut xcm = get_xcm_message(
		call,
		fee_amount,
		para_a_id
	);

	PenpalKusamaA::execute_with(|| {
		assert_ok!(
			<PenpalKusamaA as PenpalKusamaAPallet>::PolkadotXcm::send(
				para_a_root_origin,
				bx!(relay_destination.clone()),
				bx!(xcm),
		));

		type RuntimeEvent = <PenpalKusamaA as Chain>::RuntimeEvent;

		assert_expected_events!(
			PenpalKusamaA,
			// Message is sent succesfully
			vec![
				RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
			]
		);
	});

	Kusama::execute_with(|| {
		type RuntimeEvent = <Kusama as Chain>::RuntimeEvent;

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
					id: *id == para_a_id,
					weight_used: weight_within_threshold(
						(REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD),
						Weight::from_parts(1_312_558_000, 200000),
						*weight_used
					),
					success: *success == true,
				},
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
						sender, receiver, max_capacity, max_message_size
					)
				) => {
					sender: *sender == para_a_id.into(),
					receiver: *receiver == para_b_id.into(),
					max_capacity: *max_capacity == MAX_CAPACITY,
					max_message_size: *max_message_size == MAX_MESSAGE_SIZE,
				},
			]
		);
	});

	// ---- Accept Open channel from Parachain to System Parachain
	call = get_accept_open_channel_call(para_a_id);
	xcm = get_xcm_message(
		call,
		fee_amount,
		para_b_id
	);

	PenpalKusamaB::execute_with(|| {
		assert_ok!(
			<PenpalKusamaB as PenpalKusamaBPallet>::PolkadotXcm::send(
				para_b_root_origin,
				bx!(relay_destination),
				bx!(xcm),
		));

		type RuntimeEvent = <PenpalKusamaB as Chain>::RuntimeEvent;

		assert_expected_events!(
			PenpalKusamaB,
			// Message is sent succesfully
			vec![
				RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
			]
		);
	});

	Kusama::execute_with(|| {
		type RuntimeEvent = <Kusama as Chain>::RuntimeEvent;

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
					id: *id == para_b_id,
					weight_used: weight_within_threshold(
						(REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD),
						Weight::from_parts(1_312_558_000, 200000),
						*weight_used
					),
					success: *success == true,
				},
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
						sender, receiver
					)
				) => {
					sender: *sender == para_a_id.into(),
					receiver: *receiver == para_b_id.into(),
				},
			]
		);
	});

	Kusama::execute_with(|| {
		// Force process HRMP open channel requests without waiting for the next session
		assert_ok!(
			<Kusama as KusamaPallet>::Hrmp::force_process_hrmp_open(relay_root_origin, 0)
		);

		let channel_id = polkadot_parachain::primitives::HrmpChannelId {
			sender: para_a_id,
			recipient: para_b_id
		};

		let hrmp_channel_exist
			= polkadot_runtime_parachains::hrmp::HrmpChannels::<<Kusama as Chain>::Runtime>::contains_key(&channel_id);

		// Check the HRMP channel has been successfully registrered
		assert!(hrmp_channel_exist)
	});
}
