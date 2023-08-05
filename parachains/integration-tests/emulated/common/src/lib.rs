pub mod constants;

pub use constants::accounts::{ALICE, BOB};

#[cfg(all(feature = "asset-hub-runtime", feature = "kusama-runtime"))]
pub use constants::asset_hub_kusama;
#[cfg(all(feature = "asset-hub-runtime", feature = "polkadot-runtime"))]
pub use constants::asset_hub_polkadot;
#[cfg(all(feature = "asset-hub-runtime", feature = "westend-runtime"))]
pub use constants::asset_hub_westend;
#[cfg(feature = "collectives-runtime")]
pub use constants::collectives;
#[cfg(feature = "kusama-runtime")]
pub use constants::kusama;
#[cfg(feature = "penpal-runtime")]
pub use constants::penpal;
#[cfg(feature = "polkadot-runtime")]
pub use constants::polkadot;
#[cfg(feature = "westend-runtime")]
pub use constants::westend;
#[cfg(feature = "bridge-hub-runtime")]
pub use constants::{bridge_hub_kusama, bridge_hub_polkadot};

use frame_support::{parameter_types, sp_io, sp_tracing};
pub use parachains_common::{AccountId, AssetHubPolkadotAuraId, AuraId, Balance, BlockNumber};
pub use sp_core::{sr25519, storage::Storage, Get};
use xcm::prelude::*;
use xcm_emulator::{
	decl_test_networks, decl_test_parachains, decl_test_relay_chains, Parachain, RelayChain,
	TestExt,
};
use xcm_executor::traits::ConvertLocation;

decl_test_relay_chains! {
	#[cfg(feature = "westend-runtime")]
	#[api_version(5)]
	pub struct Westend {
		genesis = westend::genesis(),
		on_init = (),
		runtime = {
			Runtime: westend_runtime::Runtime,
			RuntimeOrigin: westend_runtime::RuntimeOrigin,
			RuntimeCall: westend_runtime::RuntimeCall,
			RuntimeEvent: westend_runtime::RuntimeEvent,
			MessageQueue: westend_runtime::MessageQueue,
			XcmConfig: westend_runtime::xcm_config::XcmConfig,
			SovereignAccountOf: westend_runtime::xcm_config::LocationConverter, //TODO: rename to SovereignAccountOf,
			System: westend_runtime::System,
			Balances: westend_runtime::Balances,
		},
		pallets_extra = {
			XcmPallet: westend_runtime::XcmPallet,
			Sudo: westend_runtime::Sudo,
		}
	},
	#[cfg(feature = "polkadot-runtime")]
	#[api_version(5)]
	pub struct Polkadot {
		genesis = polkadot::genesis(),
		on_init = (),
		runtime = {
			Runtime: polkadot_runtime::Runtime,
			RuntimeOrigin: polkadot_runtime::RuntimeOrigin,
			RuntimeCall: polkadot_runtime::RuntimeCall,
			RuntimeEvent: polkadot_runtime::RuntimeEvent,
			MessageQueue: polkadot_runtime::MessageQueue,
			XcmConfig: polkadot_runtime::xcm_config::XcmConfig,
			SovereignAccountOf: polkadot_runtime::xcm_config::SovereignAccountOf,
			System: polkadot_runtime::System,
			Balances: polkadot_runtime::Balances,
		},
		pallets_extra = {
			XcmPallet: polkadot_runtime::XcmPallet,
		}
	},
	#[cfg(feature = "kusama-runtime")]
	#[api_version(5)]
	pub struct Kusama {
		genesis = kusama::genesis(),
		on_init = (),
		runtime = {
			Runtime: kusama_runtime::Runtime,
			RuntimeOrigin: kusama_runtime::RuntimeOrigin,
			RuntimeCall: kusama_runtime::RuntimeCall,
			RuntimeEvent: kusama_runtime::RuntimeEvent,
			MessageQueue: kusama_runtime::MessageQueue,
			XcmConfig: kusama_runtime::xcm_config::XcmConfig,
			SovereignAccountOf: kusama_runtime::xcm_config::SovereignAccountOf,
			System: kusama_runtime::System,
			Balances: kusama_runtime::Balances,
		},
		pallets_extra = {
			XcmPallet: kusama_runtime::XcmPallet,
		}
	}
}

decl_test_parachains! {
	// Westend
	#[cfg(all(feature = "asset-hub-runtime", feature = "westend-runtime"))]
	pub struct AssetHubWestend {
		genesis = asset_hub_westend::genesis(),
		on_init = (),
		runtime = {
			Runtime: asset_hub_westend_runtime::Runtime,
			RuntimeOrigin: asset_hub_westend_runtime::RuntimeOrigin,
			RuntimeCall: asset_hub_westend_runtime::RuntimeCall,
			RuntimeEvent: asset_hub_westend_runtime::RuntimeEvent,
			XcmpMessageHandler: asset_hub_westend_runtime::XcmpQueue,
			DmpMessageHandler: asset_hub_westend_runtime::DmpQueue,
			LocationToAccountId: asset_hub_westend_runtime::xcm_config::LocationToAccountId,
			System: asset_hub_westend_runtime::System,
			Balances: asset_hub_westend_runtime::Balances,
			ParachainSystem: asset_hub_westend_runtime::ParachainSystem,
			ParachainInfo: asset_hub_westend_runtime::ParachainInfo,
		},
		pallets_extra = {
			PolkadotXcm: asset_hub_westend_runtime::PolkadotXcm,
			Assets: asset_hub_westend_runtime::Assets,
			ForeignAssets: asset_hub_westend_runtime::ForeignAssets,
			AssetConversion: asset_hub_westend_runtime::AssetConversion,
		}
	},
	// Polkadot
	#[cfg(all(feature = "asset-hub-runtime", feature = "polkadot-runtime"))]
	pub struct AssetHubPolkadot {
		genesis = asset_hub_polkadot::genesis(),
		on_init = (),
		runtime = {
			Runtime: asset_hub_polkadot_runtime::Runtime,
			RuntimeOrigin: asset_hub_polkadot_runtime::RuntimeOrigin,
			RuntimeCall: asset_hub_polkadot_runtime::RuntimeCall,
			RuntimeEvent: asset_hub_polkadot_runtime::RuntimeEvent,
			XcmpMessageHandler: asset_hub_polkadot_runtime::XcmpQueue,
			DmpMessageHandler: asset_hub_polkadot_runtime::DmpQueue,
			LocationToAccountId: asset_hub_polkadot_runtime::xcm_config::LocationToAccountId,
			System: asset_hub_polkadot_runtime::System,
			Balances: asset_hub_polkadot_runtime::Balances,
			ParachainSystem: asset_hub_polkadot_runtime::ParachainSystem,
			ParachainInfo: asset_hub_polkadot_runtime::ParachainInfo,
		},
		pallets_extra = {
			PolkadotXcm: asset_hub_polkadot_runtime::PolkadotXcm,
			Assets: asset_hub_polkadot_runtime::Assets,
		}
	},
	#[cfg(feature = "penpal-runtime")]
	pub struct PenpalPolkadot {
		genesis = penpal::genesis(penpal::PARA_ID),
		on_init = (),
		runtime = {
			Runtime: penpal_runtime::Runtime,
			RuntimeOrigin: penpal_runtime::RuntimeOrigin,
			RuntimeCall: penpal_runtime::RuntimeCall,
			RuntimeEvent: penpal_runtime::RuntimeEvent,
			XcmpMessageHandler: penpal_runtime::XcmpQueue,
			DmpMessageHandler: penpal_runtime::DmpQueue,
			LocationToAccountId: penpal_runtime::xcm_config::LocationToAccountId,
			System: penpal_runtime::System,
			Balances: penpal_runtime::Balances,
			ParachainSystem: penpal_runtime::ParachainSystem,
			ParachainInfo: penpal_runtime::ParachainInfo,
		},
		pallets_extra = {
			PolkadotXcm: penpal_runtime::PolkadotXcm,
			Assets: penpal_runtime::Assets,
		}
	},
	#[cfg(feature = "penpal-runtime")]
	pub struct PenpalWestend {
		genesis = penpal::genesis(penpal::PARA_ID),
		on_init = (),
		runtime = {
			Runtime: penpal_runtime::Runtime,
			RuntimeOrigin: penpal_runtime::RuntimeOrigin,
			RuntimeCall: penpal_runtime::RuntimeCall,
			RuntimeEvent: penpal_runtime::RuntimeEvent,
			XcmpMessageHandler: penpal_runtime::XcmpQueue,
			DmpMessageHandler: penpal_runtime::DmpQueue,
			LocationToAccountId: penpal_runtime::xcm_config::LocationToAccountId,
			System: penpal_runtime::System,
			Balances: penpal_runtime::Balances,
			ParachainSystem: penpal_runtime::ParachainSystem,
			ParachainInfo: penpal_runtime::ParachainInfo,
		},
		pallets_extra = {
			PolkadotXcm: penpal_runtime::PolkadotXcm,
			Assets: penpal_runtime::Assets,
		}
	},

	// Kusama
	#[cfg(all(feature = "asset-hub-runtime", feature = "kusama-runtime"))]
	pub struct AssetHubKusama {
		genesis = asset_hub_kusama::genesis(),
		on_init = (),
		runtime = {
			Runtime: asset_hub_kusama_runtime::Runtime,
			RuntimeOrigin: asset_hub_kusama_runtime::RuntimeOrigin,
			RuntimeCall: asset_hub_kusama_runtime::RuntimeCall,
			RuntimeEvent: asset_hub_kusama_runtime::RuntimeEvent,
			XcmpMessageHandler: asset_hub_kusama_runtime::XcmpQueue,
			DmpMessageHandler: asset_hub_kusama_runtime::DmpQueue,
			LocationToAccountId: asset_hub_kusama_runtime::xcm_config::LocationToAccountId,
			System: asset_hub_kusama_runtime::System,
			Balances: asset_hub_kusama_runtime::Balances,
			ParachainSystem: asset_hub_kusama_runtime::ParachainSystem,
			ParachainInfo: asset_hub_kusama_runtime::ParachainInfo,
		},
		pallets_extra = {
			PolkadotXcm: asset_hub_kusama_runtime::PolkadotXcm,
			Assets: asset_hub_kusama_runtime::Assets,
			ForeignAssets: asset_hub_kusama_runtime::Assets,
		}
	},
	#[cfg(feature = "penpal-runtime")]
	pub struct PenpalKusama {
		genesis = penpal::genesis(penpal::PARA_ID),
		on_init = (),
		runtime = {
			Runtime: penpal_runtime::Runtime,
			RuntimeOrigin: penpal_runtime::RuntimeOrigin,
			RuntimeCall: penpal_runtime::RuntimeCall,
			RuntimeEvent: penpal_runtime::RuntimeEvent,
			XcmpMessageHandler: penpal_runtime::XcmpQueue,
			DmpMessageHandler: penpal_runtime::DmpQueue,
			LocationToAccountId: penpal_runtime::xcm_config::LocationToAccountId,
			System: penpal_runtime::System,
			Balances: penpal_runtime::Balances,
			ParachainSystem: penpal_runtime::ParachainSystem,
			ParachainInfo: penpal_runtime::ParachainInfo,
		},
		pallets_extra = {
			PolkadotXcm: penpal_runtime::PolkadotXcm,
			Assets: penpal_runtime::Assets,
		}
	},
	#[cfg(all(feature = "collectives-runtime", feature = "polkadot-runtime"))]
	pub struct Collectives {
		genesis = collectives::genesis(),
		on_init = (),
		runtime = {
			Runtime: collectives_polkadot_runtime::Runtime,
			RuntimeOrigin: collectives_polkadot_runtime::RuntimeOrigin,
			RuntimeCall: collectives_polkadot_runtime::RuntimeCall,
			RuntimeEvent: collectives_polkadot_runtime::RuntimeEvent,
			XcmpMessageHandler: collectives_polkadot_runtime::XcmpQueue,
			DmpMessageHandler: collectives_polkadot_runtime::DmpQueue,
			LocationToAccountId: collectives_polkadot_runtime::xcm_config::LocationToAccountId,
			System: collectives_polkadot_runtime::System,
			Balances: collectives_polkadot_runtime::Balances,
			ParachainSystem: collectives_polkadot_runtime::ParachainSystem,
			ParachainInfo: collectives_polkadot_runtime::ParachainInfo,
		},
		pallets_extra = {
			PolkadotXcm: collectives_polkadot_runtime::PolkadotXcm,
		}
	},
	#[cfg(all(feature = "bridge-hub-runtime", feature = "kusama-runtime"))]
	pub struct BHKusama {
		genesis = bridge_hub_kusama::genesis(),
		on_init = (),
		runtime = {
			Runtime: bridge_hub_kusama_runtime::Runtime,
			RuntimeOrigin: bridge_hub_kusama_runtime::RuntimeOrigin,
			RuntimeCall: bridge_hub_kusama_runtime::RuntimeCall,
			RuntimeEvent: bridge_hub_kusama_runtime::RuntimeEvent,
			XcmpMessageHandler: bridge_hub_kusama_runtime::XcmpQueue,
			DmpMessageHandler: bridge_hub_kusama_runtime::DmpQueue,
			LocationToAccountId: bridge_hub_kusama_runtime::xcm_config::LocationToAccountId,
			System: bridge_hub_kusama_runtime::System,
			Balances: bridge_hub_kusama_runtime::Balances,
			ParachainSystem: bridge_hub_kusama_runtime::ParachainSystem,
			ParachainInfo:bridge_hub_kusama_runtime::ParachainInfo,
		},
		pallets_extra = {
			PolkadotXcm: bridge_hub_kusama_runtime::PolkadotXcm,
		}
	},
	#[cfg(all(feature = "bridge-hub-runtime", feature = "polkadot-runtime"))]
	pub struct BHPolkadot {
		genesis = bridge_hub_polkadot::genesis(),
		on_init = (),
		runtime = {
			Runtime: bridge_hub_polkadot_runtime::Runtime,
			RuntimeOrigin: bridge_hub_polkadot_runtime::RuntimeOrigin,
			RuntimeCall: bridge_hub_polkadot_runtime::RuntimeCall,
			RuntimeEvent: bridge_hub_polkadot_runtime::RuntimeEvent,
			XcmpMessageHandler: bridge_hub_polkadot_runtime::XcmpQueue,
			DmpMessageHandler: bridge_hub_polkadot_runtime::DmpQueue,
			LocationToAccountId: bridge_hub_polkadot_runtime::xcm_config::LocationToAccountId,
			System: bridge_hub_polkadot_runtime::System,
			Balances: bridge_hub_polkadot_runtime::Balances,
			ParachainSystem: bridge_hub_polkadot_runtime::ParachainSystem,
			ParachainInfo:bridge_hub_polkadot_runtime::ParachainInfo,
		},
		pallets_extra = {
			PolkadotXcm: bridge_hub_polkadot_runtime::PolkadotXcm,
		}
	}
}

decl_test_networks! {
	#[cfg(feature = "polkadot-runtime")]
	pub struct PolkadotMockNet {
		relay_chain = Polkadot,
		parachains = vec![
			#[cfg(feature = "asset-hub-runtime")]
			AssetHubPolkadot,
			#[cfg(feature = "penpal-runtime")]
			PenpalPolkadot,
			#[cfg(feature = "collectives-runtime")]
			Collectives,
			#[cfg(feature = "bridge-hub-runtime")]
			BHPolkadot,
		],
	},
	#[cfg(feature = "kusama-runtime")]
	pub struct KusamaMockNet {
		relay_chain = Kusama,
		parachains = vec![
			#[cfg(feature = "asset-hub-runtime")]
			AssetHubKusama,
			#[cfg(feature = "penpal-runtime")]
			PenpalKusama,
			#[cfg(feature = "bridge-hub-runtime")]
			BHKusama,
		],
	},
	#[cfg(feature = "westend-runtime")]
	pub struct WestendMockNet {
		relay_chain = Westend,
		parachains = vec![
			#[cfg(feature = "asset-hub-runtime")]
			AssetHubWestend,
			#[cfg(feature = "penpal-runtime")]
			PenpalWestend,
		],
	}
}

#[cfg(feature = "polkadot-runtime")]
parameter_types! {
	pub PolkadotSender: AccountId = Polkadot::account_id_of(ALICE);
	pub PolkadotReceiver: AccountId = Polkadot::account_id_of(BOB);
}

#[cfg(feature = "kusama-runtime")]
parameter_types! {
	pub KusamaSender: AccountId = Kusama::account_id_of(ALICE);
	pub KusamaReceiver: AccountId = Kusama::account_id_of(BOB);
}

#[cfg(feature = "westend-runtime")]
parameter_types! {
	pub WestendSender: AccountId = Westend::account_id_of(ALICE);
	pub WestendReceiver: AccountId = Westend::account_id_of(BOB);
}

#[cfg(feature = "asset-hun-westend-runtime")]
parameter_types! {
	pub AssetHubWestendSender: AccountId = AssetHubWestend::account_id_of(ALICE);
	pub AssetHubWestendReceiver: AccountId = AssetHubWestend::account_id_of(BOB);
}

#[cfg(feature = "asset-hub-polkadot-runtime")]
parameter_types! {
	pub AssetHubPolkadotSender: AccountId = AssetHubPolkadot::account_id_of(ALICE);
	pub AssetHubPolkadotReceiver: AccountId = AssetHubPolkadot::account_id_of(BOB);
}

#[cfg(feature = "asset-hub-kusama-runtime")]
parameter_types! {
	pub AssetHubKusamaSender: AccountId = AssetHubKusama::account_id_of(ALICE);
	pub AssetHubKusamaReceiver: AccountId = AssetHubKusama::account_id_of(BOB);
}

#[cfg(feature = "penpal-polkadot-runtime")]
parameter_types! {
	pub PenpalPolkadotSender: AccountId = PenpalPolkadot::account_id_of(ALICE);
	pub PenpalPolkadotReceiver: AccountId = PenpalPolkadot::account_id_of(BOB);
}

#[cfg(feature = "penpal-kusama-runtime")]
parameter_types! {
	pub PenpalKusamaSender: AccountId = PenpalKusama::account_id_of(ALICE);
	pub PenpalKusamaReceiver: AccountId = PenpalKusama::account_id_of(BOB);
}

#[cfg(feature = "penpal-runtime")]
parameter_types! {
	pub PenpalWestendSender: AccountId = PenpalWestend::account_id_of(ALICE);
	pub PenpalWestendReceiver: AccountId = PenpalWestend::account_id_of(BOB);
}

#[cfg(feature = "collectives-polkadot-runtime")]
parameter_types! {
	pub CollectivesSender: AccountId = Collectives::account_id_of(ALICE);
	pub CollectivesReceiver: AccountId = Collectives::account_id_of(BOB);
}

#[cfg(feature = "bridge-hub-runtime")]
parameter_types! {
	pub BHPolkadotSender: AccountId = BHPolkadot::account_id_of(ALICE);
	pub BHPolkadotReceiver: AccountId = BHPolkadot::account_id_of(BOB);
	pub BHKusamaSender: AccountId = BHKusama::account_id_of(ALICE);
	pub BHKusamaReceiver: AccountId = BHKusama::account_id_of(BOB);
}
