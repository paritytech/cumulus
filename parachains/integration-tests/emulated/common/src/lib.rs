pub mod constants;

pub use constants::{
	accounts::{ALICE, BOB},
	bridge_hub_kusama, bridge_hub_polkadot, collectives, kusama, penpal, polkadot, statemine,
	statemint, westend, westmint,
};
use frame_support::{parameter_types, sp_io, sp_tracing};
pub use parachains_common::{AccountId, AuraId, Balance, BlockNumber, StatemintAuraId};
pub use sp_core::{sr25519, storage::Storage, Get};
use xcm::prelude::*;
use xcm_emulator::{
	decl_test_networks, decl_test_parachains, decl_test_relay_chains, Parachain, RelayChain,
	TestExt,
};
use xcm_executor::traits::Convert;

decl_test_relay_chains! {
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
	#[api_version(4)]
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
	#[api_version(4)]
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
	pub struct Westmint {
		genesis = westmint::genesis(),
		on_init = (),
		runtime = {
			Runtime: westmint_runtime::Runtime,
			RuntimeOrigin: westmint_runtime::RuntimeOrigin,
			RuntimeCall: westmint_runtime::RuntimeCall,
			RuntimeEvent: westmint_runtime::RuntimeEvent,
			XcmpMessageHandler: westmint_runtime::XcmpQueue,
			DmpMessageHandler: westmint_runtime::DmpQueue,
			LocationToAccountId: westmint_runtime::xcm_config::LocationToAccountId,
			System: westmint_runtime::System,
			Balances: westmint_runtime::Balances,
			ParachainSystem: westmint_runtime::ParachainSystem,
			ParachainInfo: westmint_runtime::ParachainInfo,
		},
		pallets_extra = {
			PolkadotXcm: westmint_runtime::PolkadotXcm,
			Assets: westmint_runtime::Assets,
			ForeignAssets: westmint_runtime::ForeignAssets,
		}
	},
	// Polkadot
	pub struct Statemint {
		genesis = statemint::genesis(),
		on_init = (),
		runtime = {
			Runtime: statemint_runtime::Runtime,
			RuntimeOrigin: statemint_runtime::RuntimeOrigin,
			RuntimeCall: statemint_runtime::RuntimeCall,
			RuntimeEvent: statemint_runtime::RuntimeEvent,
			XcmpMessageHandler: statemint_runtime::XcmpQueue,
			DmpMessageHandler: statemint_runtime::DmpQueue,
			LocationToAccountId: statemint_runtime::xcm_config::LocationToAccountId,
			System: statemint_runtime::System,
			Balances: statemint_runtime::Balances,
			ParachainSystem: statemint_runtime::ParachainSystem,
			ParachainInfo: statemint_runtime::ParachainInfo,
		},
		pallets_extra = {
			PolkadotXcm: statemint_runtime::PolkadotXcm,
			Assets: statemint_runtime::Assets,
		}
	},
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
	pub struct Statemine {
		genesis = statemine::genesis(),
		on_init = (),
		runtime = {
			Runtime: statemine_runtime::Runtime,
			RuntimeOrigin: statemine_runtime::RuntimeOrigin,
			RuntimeCall: statemine_runtime::RuntimeCall,
			RuntimeEvent: statemine_runtime::RuntimeEvent,
			XcmpMessageHandler: statemine_runtime::XcmpQueue,
			DmpMessageHandler: statemine_runtime::DmpQueue,
			LocationToAccountId: statemine_runtime::xcm_config::LocationToAccountId,
			System: statemine_runtime::System,
			Balances: statemine_runtime::Balances,
			ParachainSystem: statemine_runtime::ParachainSystem,
			ParachainInfo: statemine_runtime::ParachainInfo,
		},
		pallets_extra = {
			PolkadotXcm: statemine_runtime::PolkadotXcm,
			Assets: statemine_runtime::Assets,
			ForeignAssets: statemine_runtime::ForeignAssets,
		}
	},
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
	pub struct PolkadotMockNet {
		relay_chain = Polkadot,
		parachains = vec![
			Statemint,
			PenpalPolkadot,
			Collectives,
			BHPolkadot,
		],
	},
	pub struct KusamaMockNet {
		relay_chain = Kusama,
		parachains = vec![
			Statemine,
			PenpalKusama,
			BHKusama,
		],
	},
	pub struct WestendMockNet {
		relay_chain = Westend,
		parachains = vec![
			Westmint,
			PenpalWestend,
		],
	}
}

parameter_types! {
	// Polkadot
	pub PolkadotSender: AccountId = Polkadot::account_id_of(ALICE);
	pub PolkadotReceiver: AccountId = Polkadot::account_id_of(BOB);
	// Kusama
	pub KusamaSender: AccountId = Kusama::account_id_of(ALICE);
	pub KusamaReceiver: AccountId = Kusama::account_id_of(BOB);
	// Westend
	pub WestendSender: AccountId = Westend::account_id_of(ALICE);
	pub WestendReceiver: AccountId = Westend::account_id_of(BOB);
	// Westmint
	pub WestmintSender: AccountId = Westmint::account_id_of(ALICE);
	pub WestmintReceiver: AccountId = Westmint::account_id_of(BOB);
	// Statemint
	pub StatemintSender: AccountId = Statemint::account_id_of(ALICE);
	pub StatemintReceiver: AccountId = Statemint::account_id_of(BOB);
	// Statemine
	pub StatemineSender: AccountId = Statemine::account_id_of(ALICE);
	pub StatemineReceiver: AccountId = Statemine::account_id_of(BOB);
	// Penpal Polkadot
	pub PenpalPolkadotSender: AccountId = PenpalPolkadot::account_id_of(ALICE);
	pub PenpalPolkadotReceiver: AccountId = PenpalPolkadot::account_id_of(BOB);
	// Penpal Kusama
	pub PenpalKusamaSender: AccountId = PenpalKusama::account_id_of(ALICE);
	pub PenpalKusamaReceiver: AccountId = PenpalKusama::account_id_of(BOB);
	// Penpal Westend
	pub PenpalWestendSender: AccountId = PenpalWestend::account_id_of(ALICE);
	pub PenpalWestendReceiver: AccountId = PenpalWestend::account_id_of(BOB);
	// Collectives
	pub CollectivesSender: AccountId = Collectives::account_id_of(ALICE);
	pub CollectivesReceiver: AccountId = Collectives::account_id_of(BOB);
	// Bridge Hub Polkadot
	pub BHPolkadotSender: AccountId = BHPolkadot::account_id_of(ALICE);
	pub BHPolkadotReceiver: AccountId = BHPolkadot::account_id_of(BOB);
	// Bridge Hub Kusama
	pub BHKusamaSender: AccountId = BHKusama::account_id_of(ALICE);
	pub BHKusamaReceiver: AccountId = BHKusama::account_id_of(BOB);
}
