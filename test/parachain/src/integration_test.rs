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
use polkadot_primitives::parachain::{Id as ParaId, Info, Scheduling};
use polkadot_runtime_common::registrar;
use polkadot_test_runtime_client::Sr25519Keyring;
use sc_client_api::execution_extensions::ExecutionStrategies;
use sc_network::{config::TransportConfig, multiaddr};
use sc_service::{
	config::{
		DatabaseConfig, KeystoreConfig, MultiaddrWithPeerId, NetworkConfiguration,
		WasmExecutionMethod,
	},
	BasePath, Configuration, Error as ServiceError, Role, TaskExecutor,
};
use std::{env, fs, path::PathBuf, sync::Arc, time::Duration};
use substrate_test_client::BlockchainEventsExt;
use substrate_test_runtime_client::AccountKeyring::*;
use tokio::{spawn, time::delay_for as sleep};

static INTEGRATION_TEST_ALLOWED_TIME: Option<&str> = option_env!("INTEGRATION_TEST_ALLOWED_TIME");

// Adapted from
// https://github.com/rust-lang/cargo/blob/485670b3983b52289a2f353d589c57fae2f60f82/tests/testsuite/support/mod.rs#L507
fn target_dir() -> PathBuf {
	env::current_exe()
		.ok()
		.map(|mut path| {
			path.pop();
			if path.ends_with("deps") {
				path.pop();
			}
			path
		})
		.unwrap()
}

#[tokio::test]
#[ignore]
async fn integration_test() {
	let task_executor: TaskExecutor = (|fut, _| {
		spawn(fut);
	})
	.into();

	// start alice
	let mut alice =
		polkadot_test_service::run_test_node(task_executor.clone(), Alice, || {}, vec![]);

	// start bob
	let mut bob = polkadot_test_service::run_test_node(
		task_executor.clone(),
		Bob,
		|| {},
		vec![alice.addr.clone()],
	);

	let t1 = sleep(Duration::from_secs(
		INTEGRATION_TEST_ALLOWED_TIME
			.and_then(|x| x.parse().ok())
			.unwrap_or(600),
	))
	.fuse();

	let t2 = async {
		let para_id = ParaId::from(100);

		future::join(alice.wait_for_blocks(2_usize), bob.wait_for_blocks(2_usize)).await;

		// export genesis state
		let genesis_state = crate::command::generate_genesis_state(para_id)
			.unwrap()
			.encode();

		// create and sign transaction
		let wasm = fs::read(target_dir().join(
			"wbuild/cumulus-test-parachain-runtime/cumulus_test_parachain_runtime.compact.wasm",
		))
		.unwrap();
		let function = polkadot_test_runtime::Call::Sudo(pallet_sudo::Call::sudo(Box::new(
			polkadot_test_runtime::Call::Registrar(registrar::Call::register_para(
				para_id,
				Info {
					scheduling: Scheduling::Always,
				},
				wasm.into(),
				genesis_state.into(),
			)),
		)));

		// register parachain
		let _ = alice.call_function(function, Alice).await.unwrap();

		// run cumulus charlie
		let key = Arc::new(sp_core::Pair::from_seed(&[10; 32]));
		let mut polkadot_config = polkadot_test_service::node_config(
			|| {},
			task_executor.clone(),
			Charlie,
			vec![alice.addr.clone(), bob.addr.clone()],
		);
		use std::net::{Ipv4Addr, SocketAddr};
		polkadot_config.rpc_http = Some(SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 27016));
		polkadot_config.rpc_methods = sc_service::config::RpcMethods::Unsafe;
		let parachain_config =
			parachain_config(task_executor.clone(), Charlie, vec![], para_id).unwrap();
		let service =
			crate::service::run_collator(parachain_config, key, polkadot_config, para_id).unwrap();
		sleep(Duration::from_secs(3)).await;
		service.client.wait_for_blocks(4).await;

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
	para_id: ParaId,
) -> Result<Configuration, ServiceError> {
	let base_path = BasePath::new_temp_dir()?;
	let root = base_path.path().to_path_buf();
	let role = Role::Authority {
		sentry_nodes: Vec::new(),
	};
	let key_seed = key.to_seed();
	let spec = crate::chain_spec::get_chain_spec(para_id);

	let mut network_config = NetworkConfiguration::new(
		format!("Cumulus Test Node for: {}", key_seed),
		"network/test/0.1",
		Default::default(),
		None,
	);

	network_config.boot_nodes = boot_nodes;

	network_config.allow_non_globals_in_dht = true;

	network_config
		.listen_addresses
		.push(multiaddr::Protocol::Memory(rand::random()).into());

	network_config.transport = TransportConfig::MemoryOnly;

	use std::net::{Ipv4Addr, SocketAddr};
	let rpc_ws = Some(SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9944));

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
		state_cache_size: 16777216,
		state_cache_child_ratio: None,
		pruning: Default::default(),
		chain_spec: Box::new(spec),
		wasm_method: WasmExecutionMethod::Interpreted,
		//execution_strategies: Default::default(),
		execution_strategies: ExecutionStrategies {
			syncing: sc_client_api::ExecutionStrategy::NativeWhenPossible,
			importing: sc_client_api::ExecutionStrategy::NativeWhenPossible,
			block_construction: sc_client_api::ExecutionStrategy::NativeWhenPossible,
			offchain_worker: sc_client_api::ExecutionStrategy::NativeWhenPossible,
			other: sc_client_api::ExecutionStrategy::NativeWhenPossible,
		},
		rpc_http: None,
		rpc_ws,
		rpc_ipc: None,
		rpc_ws_max_connections: None,
		rpc_cors: None,
		rpc_methods: Default::default(),
		prometheus_config: None,
		telemetry_endpoints: None,
		telemetry_external_transport: None,
		default_heap_pages: None,
		offchain_worker: Default::default(),
		force_authoring: false,
		disable_grandpa: false,
		dev_key_seed: Some(key_seed),
		tracing_targets: None,
		tracing_receiver: Default::default(),
		max_runtime_instances: 8,
		announce_block: true,
		base_path: Some(base_path),
		informant_output_format: Default::default(),
	})
}
