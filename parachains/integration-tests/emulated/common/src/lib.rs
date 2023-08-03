pub use lazy_static;
pub mod constants;
pub mod impls;

pub use codec::Encode;
pub use constants::{
	accounts::{ALICE, BOB},
	asset_hub_kusama, asset_hub_polkadot, asset_hub_westend, bridge_hub_kusama,
	bridge_hub_polkadot, bridge_hub_rococo, collectives, kusama, penpal, polkadot, rococo, westend,
	PROOF_SIZE_THRESHOLD, REF_TIME_THRESHOLD,
};
use frame_support::{assert_ok, parameter_types, sp_tracing};
pub use impls::{RococoWococoMessageHandler, WococoRococoMessageHandler};
pub use parachains_common::{AccountId, Balance};
use polkadot_parachain::primitives::HrmpChannelId;
pub use polkadot_runtime_parachains::inclusion::{AggregateMessageOrigin, UmpQueueId};
pub use sp_core::{sr25519, storage::Storage, Get};
use xcm_emulator::{
	assert_expected_events, decl_test_bridges, decl_test_networks, decl_test_parachains,
	decl_test_relay_chains, decl_test_sender_receiver_accounts_parameter_types,
	helpers::weight_within_threshold, BridgeMessageHandler, Chain, DefaultMessageProcessor, ParaId,
	Parachain, RelayChain, TestExt,
};

pub use xcm::{
	prelude::{
		AccountId32, All, BuyExecution, DepositAsset, MultiAsset, MultiAssets, MultiLocation,
		OriginKind, Outcome, RefundSurplus, Transact, UnpaidExecution, VersionedXcm, Weight,
		WeightLimit, WithdrawAsset, Xcm, X1,
	},
	v3::Error,
	DoubleEncoded,
};

decl_test_relay_chains! {
	#[api_version(5)]
	pub struct Polkadot {
		genesis = polkadot::genesis(),
		on_init = (),
		runtime = polkadot_runtime,
		core = {
			MessageProcessor: DefaultMessageProcessor<Polkadot>,
			SovereignAccountOf: polkadot_runtime::xcm_config::SovereignAccountOf,
		},
		pallets = {
			XcmPallet: polkadot_runtime::XcmPallet,
			Balances: polkadot_runtime::Balances,
			Hrmp: polkadot_runtime::Hrmp,
		}
	},
	#[api_version(5)]
	pub struct Kusama {
		genesis = kusama::genesis(),
		on_init = (),
		runtime = kusama_runtime,
		core = {
			MessageProcessor: DefaultMessageProcessor<Kusama>,
			SovereignAccountOf: kusama_runtime::xcm_config::SovereignAccountOf,
		},
		pallets = {
			XcmPallet: kusama_runtime::XcmPallet,
			Balances: kusama_runtime::Balances,
			Hrmp: kusama_runtime::Hrmp,
		}
	},
	#[api_version(5)]
	pub struct Westend {
		genesis = westend::genesis(),
		on_init = (),
		runtime = westend_runtime,
		core = {
			MessageProcessor: DefaultMessageProcessor<Westend>,
			SovereignAccountOf: westend_runtime::xcm_config::LocationConverter, //TODO: rename to SovereignAccountOf,
		},
		pallets = {
			XcmPallet: westend_runtime::XcmPallet,
			Sudo: westend_runtime::Sudo,
			Balances: westend_runtime::Balances,
		}
	},
	#[api_version(5)]
	pub struct Rococo {
		genesis = rococo::genesis(),
		on_init = (),
		runtime = rococo_runtime,
		core = {
			MessageProcessor: DefaultMessageProcessor<Rococo>,
			SovereignAccountOf: rococo_runtime::xcm_config::LocationConverter, //TODO: rename to SovereignAccountOf,
		},
		pallets = {
			XcmPallet: rococo_runtime::XcmPallet,
			Sudo: rococo_runtime::Sudo,
		}
	},
	#[api_version(5)]
	pub struct Wococo {
		genesis = rococo::genesis(),
		on_init = (),
		runtime = rococo_runtime,
		core = {
			MessageProcessor: DefaultMessageProcessor<Wococo>,
			SovereignAccountOf: rococo_runtime::xcm_config::LocationConverter, //TODO: rename to SovereignAccountOf,
		},
		pallets = {
			XcmPallet: rococo_runtime::XcmPallet,
			Sudo: rococo_runtime::Sudo,
		}
	}
}

decl_test_parachains! {
	// Polkadot Parachains
	pub struct AssetHubPolkadot {
		genesis = asset_hub_polkadot::genesis(),
		on_init = (),
		runtime = asset_hub_polkadot_runtime,
		core = {
			XcmpMessageHandler: asset_hub_polkadot_runtime::XcmpQueue,
			DmpMessageHandler: asset_hub_polkadot_runtime::DmpQueue,
			LocationToAccountId: asset_hub_polkadot_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: asset_hub_polkadot_runtime::ParachainInfo,
		},
		pallets = {
			PolkadotXcm: asset_hub_polkadot_runtime::PolkadotXcm,
			Assets: asset_hub_polkadot_runtime::Assets,
		}
	},
	pub struct Collectives {
		genesis = collectives::genesis(),
		on_init = (),
		runtime = collectives_polkadot_runtime,
		core = {
			XcmpMessageHandler: collectives_polkadot_runtime::XcmpQueue,
			DmpMessageHandler: collectives_polkadot_runtime::DmpQueue,
			LocationToAccountId: collectives_polkadot_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: collectives_polkadot_runtime::ParachainInfo,
		},
		pallets = {
			PolkadotXcm: collectives_polkadot_runtime::PolkadotXcm,
		}
	},
	pub struct BridgeHubPolkadot {
		genesis = bridge_hub_polkadot::genesis(),
		on_init = (),
		runtime = bridge_hub_polkadot_runtime,
		core = {
			XcmpMessageHandler: bridge_hub_polkadot_runtime::XcmpQueue,
			DmpMessageHandler: bridge_hub_polkadot_runtime::DmpQueue,
			LocationToAccountId: bridge_hub_polkadot_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: bridge_hub_polkadot_runtime::ParachainInfo,
		},
		pallets = {
			PolkadotXcm: bridge_hub_polkadot_runtime::PolkadotXcm,
		}
	},
	pub struct PenpalPolkadotA {
		genesis = penpal::genesis(penpal::PARA_ID_A),
		on_init = (),
		runtime = penpal_runtime,
		core = {
			XcmpMessageHandler: penpal_runtime::XcmpQueue,
			DmpMessageHandler: penpal_runtime::DmpQueue,
			LocationToAccountId: penpal_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: penpal_runtime::ParachainInfo,
		},
		pallets = {
			PolkadotXcm: penpal_runtime::PolkadotXcm,
			Assets: penpal_runtime::Assets,
		}
	},
	pub struct PenpalPolkadotB {
		genesis = penpal::genesis(penpal::PARA_ID_B),
		on_init = (),
		runtime = penpal_runtime,
		core = {
			XcmpMessageHandler: penpal_runtime::XcmpQueue,
			DmpMessageHandler: penpal_runtime::DmpQueue,
			LocationToAccountId: penpal_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: penpal_runtime::ParachainInfo,
		},
		pallets = {
			PolkadotXcm: penpal_runtime::PolkadotXcm,
			Assets: penpal_runtime::Assets,
		}
	},
	// Kusama Parachains
	pub struct AssetHubKusama {
		genesis = asset_hub_kusama::genesis(),
		on_init = (),
		runtime = asset_hub_kusama_runtime,
		core = {
			XcmpMessageHandler: asset_hub_kusama_runtime::XcmpQueue,
			DmpMessageHandler: asset_hub_kusama_runtime::DmpQueue,
			LocationToAccountId: asset_hub_kusama_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: asset_hub_kusama_runtime::ParachainInfo,
		},
		pallets = {
			PolkadotXcm: asset_hub_kusama_runtime::PolkadotXcm,
			Assets: asset_hub_kusama_runtime::Assets,
			ForeignAssets: asset_hub_kusama_runtime::Assets,
		}
	},
	pub struct BridgeHubKusama {
		genesis = bridge_hub_kusama::genesis(),
		on_init = (),
		runtime = bridge_hub_kusama_runtime,
		core = {
			XcmpMessageHandler: bridge_hub_kusama_runtime::XcmpQueue,
			DmpMessageHandler: bridge_hub_kusama_runtime::DmpQueue,
			LocationToAccountId: bridge_hub_kusama_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: bridge_hub_kusama_runtime::ParachainInfo,
		},
		pallets = {
			PolkadotXcm: bridge_hub_kusama_runtime::PolkadotXcm,
		}
	},
	pub struct PenpalKusamaA {
		genesis = penpal::genesis(penpal::PARA_ID_A),
		on_init = (),
		runtime = penpal_runtime,
		core = {
			XcmpMessageHandler: penpal_runtime::XcmpQueue,
			DmpMessageHandler: penpal_runtime::DmpQueue,
			LocationToAccountId: penpal_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: penpal_runtime::ParachainInfo,
		},
		pallets = {
			PolkadotXcm: penpal_runtime::PolkadotXcm,
			Assets: penpal_runtime::Assets,
		}
	},
	pub struct PenpalKusamaB {
		genesis = penpal::genesis(penpal::PARA_ID_B),
		on_init = (),
		runtime = penpal_runtime,
		core = {
			XcmpMessageHandler: penpal_runtime::XcmpQueue,
			DmpMessageHandler: penpal_runtime::DmpQueue,
			LocationToAccountId: penpal_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: penpal_runtime::ParachainInfo,
		},
		pallets = {
			PolkadotXcm: penpal_runtime::PolkadotXcm,
			Assets: penpal_runtime::Assets,
		}
	},
	// Westend Parachains
	pub struct AssetHubWestend {
		genesis = asset_hub_westend::genesis(),
		on_init = (),
		runtime = asset_hub_westend_runtime,
		core = {
			XcmpMessageHandler: asset_hub_westend_runtime::XcmpQueue,
			DmpMessageHandler: asset_hub_westend_runtime::DmpQueue,
			LocationToAccountId: asset_hub_westend_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: asset_hub_westend_runtime::ParachainInfo,
		},
		pallets = {
			PolkadotXcm: asset_hub_westend_runtime::PolkadotXcm,
			Balances: asset_hub_westend_runtime::Balances,
			Assets: asset_hub_westend_runtime::Assets,
			ForeignAssets: asset_hub_westend_runtime::ForeignAssets,
			PoolAssets: asset_hub_westend_runtime::PoolAssets,
			AssetConversion: asset_hub_westend_runtime::AssetConversion,
		}
	},
	pub struct PenpalWestendA {
		genesis = penpal::genesis(penpal::PARA_ID_A),
		on_init = (),
		runtime = penpal_runtime,
		core = {
			XcmpMessageHandler: penpal_runtime::XcmpQueue,
			DmpMessageHandler: penpal_runtime::DmpQueue,
			LocationToAccountId: penpal_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: penpal_runtime::ParachainInfo,
		},
		pallets = {
			PolkadotXcm: penpal_runtime::PolkadotXcm,
			Assets: penpal_runtime::Assets,
		}
	},
	// Rococo Parachains
	pub struct BridgeHubRococo {
		genesis = bridge_hub_rococo::genesis(),
		on_init = (),
		runtime = bridge_hub_rococo_runtime,
		core = {
			XcmpMessageHandler: bridge_hub_rococo_runtime::XcmpQueue,
			DmpMessageHandler: bridge_hub_rococo_runtime::DmpQueue,
			LocationToAccountId: bridge_hub_rococo_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: bridge_hub_rococo_runtime::ParachainInfo,
		},
		pallets = {
			PolkadotXcm: bridge_hub_rococo_runtime::PolkadotXcm,
		}
	},
	pub struct AssetHubRococo {
		genesis = asset_hub_polkadot::genesis(),
		on_init = (),
		runtime = asset_hub_polkadot_runtime,
		core = {
			XcmpMessageHandler: asset_hub_polkadot_runtime::XcmpQueue,
			DmpMessageHandler: asset_hub_polkadot_runtime::DmpQueue,
			LocationToAccountId: asset_hub_polkadot_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: asset_hub_polkadot_runtime::ParachainInfo,
		},
		pallets = {
			PolkadotXcm: asset_hub_polkadot_runtime::PolkadotXcm,
			Assets: asset_hub_polkadot_runtime::Assets,
		}
	},
	// Wococo Parachains
	pub struct BridgeHubWococo {
		genesis = bridge_hub_rococo::genesis(),
		on_init = (),
		runtime = bridge_hub_rococo_runtime,
		core = {
			XcmpMessageHandler: bridge_hub_rococo_runtime::XcmpQueue,
			DmpMessageHandler: bridge_hub_rococo_runtime::DmpQueue,
			LocationToAccountId: bridge_hub_rococo_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: bridge_hub_rococo_runtime::ParachainInfo,
		},
		pallets = {
			PolkadotXcm: bridge_hub_rococo_runtime::PolkadotXcm,
		}
	},
	pub struct AssetHubWococo {
		genesis = asset_hub_polkadot::genesis(),
		on_init = (),
		runtime = asset_hub_polkadot_runtime,
		core = {
			XcmpMessageHandler: asset_hub_polkadot_runtime::XcmpQueue,
			DmpMessageHandler: asset_hub_polkadot_runtime::DmpQueue,
			LocationToAccountId: asset_hub_polkadot_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: asset_hub_polkadot_runtime::ParachainInfo,
		},
		pallets = {
			PolkadotXcm: asset_hub_polkadot_runtime::PolkadotXcm,
			Assets: asset_hub_polkadot_runtime::Assets,
		}
	},
	pub struct PenpalRococoA {
		genesis = penpal::genesis(penpal::PARA_ID_A),
		on_init = (),
		runtime = penpal_runtime,
		core = {
			XcmpMessageHandler: penpal_runtime::XcmpQueue,
			DmpMessageHandler: penpal_runtime::DmpQueue,
			LocationToAccountId: penpal_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: penpal_runtime::ParachainInfo,
		},
		pallets = {
			PolkadotXcm: penpal_runtime::PolkadotXcm,
			Assets: penpal_runtime::Assets,
		}
	}
}

decl_test_networks! {
	pub struct PolkadotMockNet {
		relay_chain = Polkadot,
		parachains = vec![
			AssetHubPolkadot,
			Collectives,
			BridgeHubPolkadot,
			PenpalPolkadotA,
			PenpalPolkadotB,
		],
		// TODO: uncomment when https://github.com/paritytech/cumulus/pull/2528 is merged
		// bridge = PolkadotKusamaMockBridge
		bridge = ()
	},
	pub struct KusamaMockNet {
		relay_chain = Kusama,
		parachains = vec![
			AssetHubKusama,
			PenpalKusamaA,
			BridgeHubKusama,
			PenpalKusamaB,
		],
		// TODO: uncomment when https://github.com/paritytech/cumulus/pull/2528 is merged
		// bridge = KusamaPolkadotMockBridge
		bridge = ()
	},
	pub struct WestendMockNet {
		relay_chain = Westend,
		parachains = vec![
			AssetHubWestend,
			PenpalWestendA,
		],
		bridge = ()
	},
	pub struct RococoMockNet {
		relay_chain = Rococo,
		parachains = vec![
			AssetHubRococo,
			BridgeHubRococo,
			PenpalRococoA,
		],
		bridge = RococoWococoMockBridge
	},
	pub struct WococoMockNet {
		relay_chain = Wococo,
		parachains = vec![
			AssetHubWococo,
			BridgeHubWococo,
		],
		bridge = WococoRococoMockBridge
	}
}

decl_test_bridges! {
	pub struct RococoWococoMockBridge {
		source = BridgeHubRococo,
		target = BridgeHubWococo,
		handler = RococoWococoMessageHandler
	},
	pub struct WococoRococoMockBridge {
		source = BridgeHubWococo,
		target = BridgeHubRococo,
		handler = WococoRococoMessageHandler
	}
	// TODO: uncomment when https://github.com/paritytech/cumulus/pull/2528 is merged
	// pub struct PolkadotKusamaMockBridge {
	// 	source = BridgeHubPolkadot,
	// 	target = BridgeHubKusama,
	//  handler = PolkadotKusamaMessageHandler
	// },
	// pub struct KusamaPolkadotMockBridge {
	// 	source = BridgeHubKusama,
	// 	target = BridgeHubPolkadot,
	// 	handler = KusamaPolkadotMessageHandler
	// }
}

decl_test_sender_receiver_accounts_parameter_types! {
	// Relays
	Polkadot { sender: ALICE, receiver: BOB },
	Kusama { sender: ALICE, receiver: BOB },
	Westend { sender: ALICE, receiver: BOB },
	Rococo { sender: ALICE, receiver: BOB },
	Wococo { sender: ALICE, receiver: BOB },
	// Asset Hubs
	AssetHubPolkadot { sender: ALICE, receiver: BOB },
	AssetHubKusama { sender: ALICE, receiver: BOB },
	AssetHubWestend { sender: ALICE, receiver: BOB },
	AssetHubRococo { sender: ALICE, receiver: BOB },
	AssetHubWococo { sender: ALICE, receiver: BOB },
	// Collectives
	Collectives { sender: ALICE, receiver: BOB },
	// Bridged Hubs
	BridgeHubPolkadot { sender: ALICE, receiver: BOB },
	BridgeHubKusama { sender: ALICE, receiver: BOB },
	BridgeHubRococo { sender: ALICE, receiver: BOB },
	BridgeHubWococo { sender: ALICE, receiver: BOB },
	// Penpals
	PenpalPolkadotA { sender: ALICE, receiver: BOB },
	PenpalPolkadotB { sender: ALICE, receiver: BOB },
	PenpalKusamaA { sender: ALICE, receiver: BOB },
	PenpalKusamaB { sender: ALICE, receiver: BOB },
	PenpalWestendA { sender: ALICE, receiver: BOB },
	PenpalRococoA { sender: ALICE, receiver: BOB }
}

pub mod events {
	pub mod polkadot {
		use crate::*;
		type RuntimeEvent = <Polkadot as Chain>::RuntimeEvent;

		// Dispatchable is completely executed and XCM sent
		pub fn xcm_pallet_attempted_complete(expected_weight: Option<Weight>) {
			assert_expected_events!(
				Polkadot,
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
		pub fn xcm_pallet_attempted_incomplete(
			expected_weight: Option<Weight>,
			expected_error: Option<Error>,
		) {
			assert_expected_events!(
				Polkadot,
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
				Polkadot,
				vec![
					RuntimeEvent::XcmPallet(pallet_xcm::Event::Sent { .. }) => {},
				]
			);
		}

		// XCM from System Parachain is succesfully received and proccessed
		pub fn ump_queue_processed(
			expected_success: bool,
			expected_id: Option<ParaId>,
			expected_weight: Option<Weight>,
		) {
			assert_expected_events!(
				Polkadot,
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
	pub mod kusama {
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
		pub fn xcm_pallet_attempted_incomplete(
			expected_weight: Option<Weight>,
			expected_error: Option<Error>,
		) {
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
			expected_weight: Option<Weight>,
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

	pub mod westend {
		use crate::*;
		type RuntimeEvent = <Westend as Chain>::RuntimeEvent;

		// Dispatchable is completely executed and XCM sent
		pub fn xcm_pallet_attempted_complete(expected_weight: Option<Weight>) {
			assert_expected_events!(
				Westend,
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
		pub fn xcm_pallet_attempted_incomplete(
			expected_weight: Option<Weight>,
			expected_error: Option<Error>,
		) {
			assert_expected_events!(
				Westend,
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
				Westend,
				vec![
					RuntimeEvent::XcmPallet(pallet_xcm::Event::Sent { .. }) => {},
				]
			);
		}

		// XCM from System Parachain is succesfully received and proccessed
		pub fn ump_queue_processed(
			expected_success: bool,
			expected_id: Option<ParaId>,
			expected_weight: Option<Weight>,
		) {
			assert_expected_events!(
				Westend,
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

	pub mod rococo {
		use crate::*;
		type RuntimeEvent = <Rococo as Chain>::RuntimeEvent;

		// Dispatchable is completely executed and XCM sent
		pub fn xcm_pallet_attempted_complete(expected_weight: Option<Weight>) {
			assert_expected_events!(
				Rococo,
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
		pub fn xcm_pallet_attempted_incomplete(
			expected_weight: Option<Weight>,
			expected_error: Option<Error>,
		) {
			assert_expected_events!(
				Rococo,
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
				Rococo,
				vec![
					RuntimeEvent::XcmPallet(pallet_xcm::Event::Sent { .. }) => {},
				]
			);
		}

		// XCM from System Parachain is succesfully received and proccessed
		pub fn ump_queue_processed(
			expected_success: bool,
			expected_id: Option<ParaId>,
			expected_weight: Option<Weight>,
		) {
			assert_expected_events!(
				Rococo,
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
}

pub fn xcm_paid_execution(
	call: DoubleEncoded<()>,
	origin_kind: OriginKind,
	native_asset: MultiAsset,
	beneficiary: AccountId,
) -> VersionedXcm<()> {
	let weight_limit = WeightLimit::Unlimited;
	let require_weight_at_most = Weight::from_parts(1000000000, 200000);
	let native_assets: MultiAssets = native_asset.clone().into();

	VersionedXcm::from(Xcm(vec![
		WithdrawAsset(native_assets),
		BuyExecution { fees: native_asset, weight_limit },
		Transact { require_weight_at_most, origin_kind, call },
		RefundSurplus,
		DepositAsset {
			assets: All.into(),
			beneficiary: MultiLocation {
				parents: 0,
				interior: X1(AccountId32 { network: None, id: beneficiary.into() }),
			},
		},
	]))
}

pub fn xcm_unpaid_execution(call: DoubleEncoded<()>, origin_kind: OriginKind) -> VersionedXcm<()> {
	let weight_limit = WeightLimit::Unlimited;
	let require_weight_at_most = Weight::from_parts(1000000000, 200000);
	let check_origin = None;

	VersionedXcm::from(Xcm(vec![
		UnpaidExecution { weight_limit, check_origin },
		Transact { require_weight_at_most, origin_kind, call },
	]))
}

impl Kusama {
	pub fn fund_para_sovereign(amount: Balance, para_id: ParaId) -> sp_runtime::AccountId32 {
		let sovereign_account = Self::sovereign_account_id_of_child_para(para_id);
		Self::execute_with(|| {
			assert_ok!(<Self as KusamaPallet>::Balances::force_set_balance(
				<Self as Chain>::RuntimeOrigin::root(),
				sp_runtime::MultiAddress::Id(sovereign_account.clone()),
				amount,
			));
		});
		sovereign_account
	}

	pub fn init_open_channel_call(
		recipient_para_id: ParaId,
		max_capacity: u32,
		max_message_size: u32,
	) -> DoubleEncoded<()> {
		<Self as Chain>::RuntimeCall::Hrmp(polkadot_runtime_parachains::hrmp::Call::<
			<Self as Chain>::Runtime,
		>::hrmp_init_open_channel {
			recipient: recipient_para_id,
			proposed_max_capacity: max_capacity,
			proposed_max_message_size: max_message_size,
		})
		.encode()
		.into()
	}

	pub fn accept_open_channel_call(sender_para_id: ParaId) -> DoubleEncoded<()> {
		<Self as Chain>::RuntimeCall::Hrmp(polkadot_runtime_parachains::hrmp::Call::<
			<Self as Chain>::Runtime,
		>::hrmp_accept_open_channel {
			sender: sender_para_id,
		})
		.encode()
		.into()
	}

	pub fn force_process_hrmp_open(sender: ParaId, recipient: ParaId) {
		Self::execute_with(|| {
			let relay_root_origin = <Self as Chain>::RuntimeOrigin::root();

			// Force process HRMP open channel requests without waiting for the next session
			assert_ok!(<Self as KusamaPallet>::Hrmp::force_process_hrmp_open(relay_root_origin, 0));

			let channel_id = HrmpChannelId { sender, recipient };

			let hrmp_channel_exist = polkadot_runtime_parachains::hrmp::HrmpChannels::<
				<Self as Chain>::Runtime,
			>::contains_key(&channel_id);

			// Check the HRMP channel has been successfully registrered
			assert!(hrmp_channel_exist)
		});
	}
}

impl Polkadot {
	pub fn fund_para_sovereign(amount: Balance, para_id: ParaId) -> sp_runtime::AccountId32 {
		let sovereign_account = Self::sovereign_account_id_of_child_para(para_id);
		Self::execute_with(|| {
			assert_ok!(<Self as PolkadotPallet>::Balances::force_set_balance(
				<Self as Chain>::RuntimeOrigin::root(),
				sp_runtime::MultiAddress::Id(sovereign_account.clone()),
				amount,
			));
		});
		sovereign_account
	}

	pub fn init_open_channel_call(
		recipient_para_id: ParaId,
		max_capacity: u32,
		max_message_size: u32,
	) -> DoubleEncoded<()> {
		<Self as Chain>::RuntimeCall::Hrmp(polkadot_runtime_parachains::hrmp::Call::<
			<Self as Chain>::Runtime,
		>::hrmp_init_open_channel {
			recipient: recipient_para_id,
			proposed_max_capacity: max_capacity,
			proposed_max_message_size: max_message_size,
		})
		.encode()
		.into()
	}

	pub fn accept_open_channel_call(sender_para_id: ParaId) -> DoubleEncoded<()> {
		<Self as Chain>::RuntimeCall::Hrmp(polkadot_runtime_parachains::hrmp::Call::<
			<Self as Chain>::Runtime,
		>::hrmp_accept_open_channel {
			sender: sender_para_id,
		})
		.encode()
		.into()
	}

	pub fn force_process_hrmp_open(sender: ParaId, recipient: ParaId) {
		Self::execute_with(|| {
			let relay_root_origin = <Self as Chain>::RuntimeOrigin::root();

			// Force process HRMP open channel requests without waiting for the next session
			assert_ok!(<Self as PolkadotPallet>::Hrmp::force_process_hrmp_open(
				relay_root_origin,
				0
			));

			let channel_id = HrmpChannelId { sender, recipient };

			let hrmp_channel_exist = polkadot_runtime_parachains::hrmp::HrmpChannels::<
				<Self as Chain>::Runtime,
			>::contains_key(&channel_id);

			// Check the HRMP channel has been successfully registrered
			assert!(hrmp_channel_exist)
		});
	}
}
