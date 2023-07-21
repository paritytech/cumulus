pub use lazy_static;
pub mod constants;
pub mod impls;

pub use constants::{
	accounts::{ALICE, BOB},
	asset_hub_kusama, asset_hub_polkadot, asset_hub_westend, bridge_hub_kusama,
	bridge_hub_polkadot, bridge_hub_rococo, collectives, kusama, penpal, polkadot, rococo, westend,
};
pub use impls::{RococoWococoMessageHandler, WococoRococoMessageHandler};

use frame_support::{parameter_types, sp_tracing};
use parachains_common::{AccountId, Balance};
pub use sp_core::{sr25519, storage::Storage, Get};
use xcm_emulator::{
	decl_test_bridges, decl_test_networks, decl_test_parachains, decl_test_relay_chains,
	decl_test_sender_receiver_accounts_parameter_types, BridgeMessageHandler, Chain, Parachain,
	RelayChain, TestExt, DefaultMessageProcessor
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
	pub struct PenpalPolkadot {
		genesis = penpal::genesis(penpal::PARA_ID),
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
	pub struct PenpalKusama {
		genesis = penpal::genesis(penpal::PARA_ID),
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
			Assets: asset_hub_westend_runtime::Assets,
			ForeignAssets: asset_hub_westend_runtime::ForeignAssets,
			AssetConversion: asset_hub_westend_runtime::AssetConversion,
		}
	},
	pub struct PenpalWestend {
		genesis = penpal::genesis(penpal::PARA_ID),
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
	}
}

decl_test_networks! {
	pub struct PolkadotMockNet {
		relay_chain = Polkadot,
		parachains = vec![
			AssetHubPolkadot,
			PenpalPolkadot,
			Collectives,
			BridgeHubPolkadot,
		],
		// TODO: uncomment when https://github.com/paritytech/cumulus/pull/2528 is merged
		// bridge = PolkadotKusamaMockBridge
		bridge = ()
	},
	pub struct KusamaMockNet {
		relay_chain = Kusama,
		parachains = vec![
			AssetHubKusama,
			PenpalKusama,
			BridgeHubKusama,
		],
		// TODO: uncomment when https://github.com/paritytech/cumulus/pull/2528 is merged
		// bridge = KusamaPolkadotMockBridge
		bridge = ()
	},
	pub struct WestendMockNet {
		relay_chain = Westend,
		parachains = vec![
			AssetHubWestend,
			PenpalWestend,
		],
		bridge = ()
	},
	pub struct RococoMockNet {
		relay_chain = Rococo,
		parachains = vec![
			AssetHubRococo,
			BridgeHubRococo,
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
	PenpalPolkadot { sender: ALICE, receiver: BOB },
	PenpalKusama { sender: ALICE, receiver: BOB },
	PenpalWestend { sender: ALICE, receiver: BOB }
}
