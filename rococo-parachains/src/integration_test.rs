// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

use codec::Encode;
use futures::{
	future::{self, FutureExt},
	pin_mut, select,
};
use polkadot_primitives::v0::{Id as ParaId, Info, Scheduling};
use polkadot_runtime_common::registrar;
use polkadot_test_runtime_client::Sr25519Keyring;
use sc_client_api::execution_extensions::ExecutionStrategies;
use sc_informant::OutputFormat;
use sc_network::{config::TransportConfig, multiaddr};
use sc_service::{
	config::{
		DatabaseConfig, KeystoreConfig, MultiaddrWithPeerId, NetworkConfiguration,
		WasmExecutionMethod, PruningMode, OffchainWorkerConfig,
	},
	BasePath, Configuration, Error as ServiceError, Role, TaskExecutor,
};
use std::{sync::Arc, time::Duration};
use substrate_test_client::BlockchainEventsExt;
use substrate_test_runtime_client::AccountKeyring::*;
use tokio::{spawn, time::delay_for as sleep};
use sc_chain_spec::ChainSpec;

static INTEGRATION_TEST_ALLOWED_TIME: Option<&str> = option_env!("INTEGRATION_TEST_ALLOWED_TIME");

#[tokio::test]
#[ignore]
async fn integration_test() {
	sc_cli::init_logger("network=warn,cumulus-network=trace,validation=debug");
	let task_executor: TaskExecutor = (|fut, _| spawn(fut).map(|_| ())).into();

	//let polkadot_spec = polkadot_service::chain_spec::rococo_staging_testnet_config().unwrap();
	let polkadot_spec = polkadot_service::chain_spec::rococo_local_testnet_config().unwrap();

	// start alice
	/*
	let mut alice =
		polkadot_test_service::run_test_node(task_executor.clone(), Alice, || {
			//polkadot_test_runtime::ExpectedBlockTime::set(&10000);
		}, vec![]);
	*/
	let mut alice_config = polkadot_test_service::node_config(
		|| {},
		task_executor.clone(),
		Alice,
		vec![],
	);
	alice_config.chain_spec = Box::new(polkadot_spec.clone());
	alice_config.pruning = PruningMode::ArchiveAll;
	println!("{:?}", alice_config);
	let multiaddr = alice_config.network.listen_addresses[0].clone();
	let authority_discovery_disabled = false;
	let grandpa_pause = None;
	let (task_manager, client, handles, network, rpc_handlers) = polkadot_service::new_full::<rococo_runtime::RuntimeApi, polkadot_service::RococoExecutor>(
		alice_config,
		None,
		None,
		authority_discovery_disabled,
		6000,
		grandpa_pause,
		true,
	).unwrap();
	let peer_id = network.local_peer_id().clone();
	let addr = MultiaddrWithPeerId { multiaddr, peer_id };
	let mut alice = polkadot_test_service::PolkadotTestNode {
		task_manager,
		client,
		handles,
		addr,
		rpc_handlers,
	};

	// start bob
	/*
	let mut bob = polkadot_test_service::run_test_node(
		task_executor.clone(),
		Bob,
		|| {
			//polkadot_test_runtime::ExpectedBlockTime::set(&10000);
		},
		vec![alice.addr.clone()],
	);
	*/
	let mut bob_config = polkadot_test_service::node_config(
		|| {},
		task_executor.clone(),
		Bob,
		vec![alice.addr.clone()],
	);
	bob_config.chain_spec = Box::new(polkadot_spec.clone());
	bob_config.pruning = PruningMode::ArchiveAll;
	let multiaddr = bob_config.network.listen_addresses[0].clone();
	let authority_discovery_disabled = false;
	let grandpa_pause = None;
	let (task_manager, client, handles, network, rpc_handlers) = polkadot_service::new_full::<rococo_runtime::RuntimeApi, polkadot_service::RococoExecutor>(
		bob_config,
		None,
		None,
		authority_discovery_disabled,
		6000,
		grandpa_pause,
		true,
	).unwrap();
	let peer_id = network.local_peer_id().clone();
	let addr = MultiaddrWithPeerId { multiaddr, peer_id };
	let mut bob = polkadot_test_service::PolkadotTestNode {
		task_manager,
		client,
		handles,
		addr,
		rpc_handlers,
	};

	let t1 = sleep(Duration::from_secs(
		INTEGRATION_TEST_ALLOWED_TIME
			.and_then(|x| x.parse().ok())
			.unwrap_or(600),
	))
	.fuse();

	let t2 = async {
		let para_id = ParaId::from(100);

		future::join(alice.wait_for_blocks(2), bob.wait_for_blocks(2)).await;

		// export genesis state
		let spec = Box::new(crate::chain_spec::get_chain_spec(para_id));
		/*
		let spec = Box::new(crate::chain_spec::ChainSpec::from_json_bytes(
			&include_bytes!("./integration_test.json")[..],
		).unwrap());
		*/
		let genesis_state = crate::command::generate_genesis_state(&(spec.clone() as Box<_>))
			.unwrap()
			.encode();

		// create and sign transaction
		let function = rococo_runtime::Call::Sudo(pallet_sudo::Call::sudo(Box::new(
			rococo_runtime::Call::Registrar(registrar::Call::register_para(
				para_id,
				Info {
					scheduling: Scheduling::Always,
				},
				parachain_runtime::WASM_BINARY
					.expect("You need to build the WASM binary to run this test!")
					.to_vec()
					.into(),
				genesis_state.into(),
			)),
		)));

		use sp_blockchain::HeaderBackend;
		use sp_runtime::SaturatedConversion;
		use sp_runtime::generic;
		use polkadot_runtime_common::BlockHashCount;
		let caller = Alice;
		let current_block_hash = alice.client.info().best_hash;
		let current_block = alice.client.info().best_number.saturated_into();
		let genesis_block = alice.client.hash(0).unwrap().unwrap();
		let nonce = 0;
		let period = BlockHashCount::get()
			.checked_next_power_of_two()
			.map(|c| c / 2)
			.unwrap_or(2) as u64;
		let tip = 0;
		let extra: rococo_runtime::SignedExtra = (
			//rococo_runtime::RestrictFunctionality,
			frame_system::CheckSpecVersion::<rococo_runtime::Runtime>::new(),
			frame_system::CheckTxVersion::<rococo_runtime::Runtime>::new(),
			frame_system::CheckGenesis::<rococo_runtime::Runtime>::new(),
			frame_system::CheckEra::<rococo_runtime::Runtime>::from(generic::Era::mortal(period, current_block)),
			frame_system::CheckNonce::<rococo_runtime::Runtime>::from(nonce),
			frame_system::CheckWeight::<rococo_runtime::Runtime>::new(),
			pallet_transaction_payment::ChargeTransactionPayment::<rococo_runtime::Runtime>::from(tip),
			registrar::LimitParathreadCommits::<rococo_runtime::Runtime>::new(),
			polkadot_runtime_common::parachains::ValidateDoubleVoteReports::<rococo_runtime::Runtime>::new(),
		);
		let raw_payload = rococo_runtime::SignedPayload::from_raw(
			function.clone(),
			extra.clone(),
			(
				rococo_runtime::VERSION.spec_version,
				rococo_runtime::VERSION.transaction_version,
				genesis_block,
				current_block_hash,
				(),
				(),
				(),
				(),
				(),
			),
		);
		let signature = raw_payload.using_encoded(|e| caller.sign(e));
		let extrinsic = rococo_runtime::UncheckedExtrinsic::new_signed(
			function.clone(),
			caller.public().into(),
			polkadot_primitives::v0::Signature::Sr25519(signature.clone()),
			extra.clone(),
		);

		// register parachain
		use substrate_test_client::RpcHandlersExt;
		let _ = alice.rpc_handlers.send_transaction(extrinsic.into()).await.unwrap();

		// run cumulus charlie
		let key = Arc::new(sp_core::Pair::generate().0);
		let mut polkadot_config = polkadot_test_service::node_config(
			|| {},
			task_executor.clone(),
			Charlie,
			vec![alice.addr.clone(), bob.addr.clone()],
		);
		polkadot_config.chain_spec = Box::new(polkadot_spec.clone());
		polkadot_config.role = Role::Full;
		polkadot_config.execution_strategies.importing = sc_client_api::ExecutionStrategy::NativeElseWasm;
		polkadot_config.dev_key_seed = None;
		println!("{:?}", polkadot_config);
		let parachain_config =
			parachain_config(task_executor.clone(), Charlie, vec![], spec).unwrap();
		println!("{:?}", parachain_config);
		let (_service, charlie_client) =
			crate::service::start_node(parachain_config, key, polkadot_config, para_id, true, false)
				.unwrap();
		charlie_client.wait_for_blocks(4).await;

		alice.task_manager.terminate();
		bob.task_manager.terminate();
	}
	.fuse();

	pin_mut!(t1, t2);

	select! {
		_ = t1 => {
			panic!("the test took too long, maybe no parachain blocks have been produced");
		},
		_ = t2 => {},
	}
}

pub fn parachain_config(
	task_executor: TaskExecutor,
	key: Sr25519Keyring,
	boot_nodes: Vec<MultiaddrWithPeerId>,
	spec: Box<dyn ChainSpec>,
) -> Result<Configuration, ServiceError> {
	let base_path = BasePath::new_temp_dir()?;
	let root = base_path.path().to_path_buf();
	let role = Role::Authority {
		sentry_nodes: Vec::new(),
	};
	let key_seed = key.to_seed();

	let mut network_config = NetworkConfiguration::new(
		format!("Cumulus Test Node for: {}", key_seed),
		"network/test/0.1",
		Default::default(),
		None,
	);
	let informant_output_format = OutputFormat {
		enable_color: false,
		prefix: format!("[{}] ", key_seed),
	};

	network_config.boot_nodes = boot_nodes;

	network_config.allow_non_globals_in_dht = false;

	network_config
		.listen_addresses
		.push(multiaddr::Protocol::Memory(rand::random()).into());

	network_config.transport = TransportConfig::MemoryOnly;

	Ok(Configuration {
		impl_name: "cumulus-test-node".to_string(),
		impl_version: "0.1".to_string(),
		role,
		task_executor,
		transaction_pool: Default::default(),
		network: network_config,
		keystore: KeystoreConfig::Path {
			path: root.join("key"),
			password: None,
		},
		database: DatabaseConfig::RocksDb {
			path: root.join("db"),
			cache_size: 128,
		},
		state_cache_size: 67108864,
		state_cache_child_ratio: None,
		pruning: PruningMode::ArchiveAll,
		chain_spec: spec,
		wasm_method: WasmExecutionMethod::default(),
		// NOTE: we enforce the use of the native runtime to make the errors more debuggable
		execution_strategies: ExecutionStrategies {
			syncing: sc_client_api::ExecutionStrategy::NativeElseWasm,
			importing: sc_client_api::ExecutionStrategy::AlwaysWasm,
			block_construction: sc_client_api::ExecutionStrategy::AlwaysWasm,
			offchain_worker: sc_client_api::ExecutionStrategy::NativeWhenPossible,
			other: sc_client_api::ExecutionStrategy::NativeWhenPossible,
		},
		//execution_strategies: Default::default(),
		rpc_http: None,
		rpc_ws: None,
		rpc_ipc: None,
		rpc_ws_max_connections: None,
		rpc_cors: None,
		rpc_methods: Default::default(),
		prometheus_config: None,
		telemetry_endpoints: None,
		telemetry_external_transport: None,
		default_heap_pages: None,
		offchain_worker: OffchainWorkerConfig { enabled: true, indexing_enabled: false },
		force_authoring: false,
		disable_grandpa: false,
		//dev_key_seed: Some(key_seed),
		dev_key_seed: None,
		tracing_targets: None,
		tracing_receiver: Default::default(),
		max_runtime_instances: 8,
		announce_block: true,
		base_path: Some(base_path),
		informant_output_format,
	})
}
