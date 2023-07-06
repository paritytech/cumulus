// pub use crate::{
// 	paste
// };

#[macro_export]
macro_rules! test_parachain_is_trusted_teleporter {
	( $sender_para:ty, ($assets:expr, $amount:expr), vec![$( $receiver_para:ty ),+] ) => {
		$crate::paste::paste! {
			// Origin
			// let amount = KUSAMA_ED * 10;
			let para_sender_balance_before =
				<$sender_para>::account_data_of([<$sender_para Sender>]::get()).free;
			let origin = <$sender_para as $crate::Parachain>::RuntimeOrigin::signed([<$sender_para Sender>]::get());

			let para_receiver_balance_before =
				BridgeHubKusama::account_data_of(BridgeHubKusamaReceiver::get()).free;
			let para_destination: VersionedMultiLocation =
				<$sender_para>::sibling_location_of(BridgeHubKusama::para_id()).into();
			let beneficiary: VersionedMultiLocation =
				AccountId32 { network: None, id: BridgeHubKusamaReceiver::get().into() }.into();
			// let native_assets: VersionedMultiAssets = (Parent, amount).into();
			let fee_asset_item = 0;
			let weight_limit = WeightLimit::Unlimited;

			// Send XCM message from Origin Parachain
			<$sender_para>::execute_with(|| {
				assert_ok!(<$sender_para as [<$sender_para Pallet>]>::PolkadotXcm::limited_teleport_assets(
					origin,
					bx!(para_destination),
					bx!(beneficiary),
					bx!($assets),
					fee_asset_item,
					weight_limit,
				));

				type RuntimeEvent = <$sender_para as $crate::Parachain>::RuntimeEvent;

				assert_expected_events!(
					$sender_para,
					vec![
						RuntimeEvent::PolkadotXcm(
							pallet_xcm::Event::Attempted { outcome: Outcome::Complete { .. } }
						) => {},
					]
				);
			});

			// Receive XCM message in Assets Parachain
			BridgeHubKusama::execute_with(|| {
				type RuntimeEvent = <BridgeHubKusama as Para>::RuntimeEvent;

				assert_expected_events!(
					BridgeHubKusama,
					vec![
						RuntimeEvent::Balances(pallet_balances::Event::Deposit { who, .. }) => {
							who: *who == BridgeHubKusamaReceiver::get().into(),
						},
					]
				);
			});

			// Check if balances are updated accordingly in Relay Chain and Assets Parachain
			let para_sender_balance_after =
				<$sender_para>::account_data_of([<$sender_para Sender>]::get()).free;
			let para_receiver_balance_after =
				BridgeHubKusama::account_data_of(BridgeHubKusamaReceiver::get()).free;

			assert_eq!(para_sender_balance_before - $amount, para_sender_balance_after);
			assert!(para_receiver_balance_after > para_receiver_balance_before);
		}

	};
}

// pub use test_system_para_is_trusted_teleporter;
