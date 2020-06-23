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

use assert_cmd::cargo::cargo_bin;
use async_std::{net, task::sleep};
use codec::Encode;
use futures::{future::FutureExt, join, pin_mut, select};
use polkadot_primitives::parachain::{Info, Scheduling};
use polkadot_primitives::Hash as PHash;
use polkadot_runtime::{Header, Runtime, SignedExtra, SignedPayload, IsCallable};
use polkadot_test_runtime::VERSION;
use polkadot_runtime_common::{parachains, registrar, BlockHashCount, claims, TransactionCallFilter};
use serde_json::Value;
use sp_arithmetic::traits::SaturatedConversion;
use sp_runtime::{generic, OpaqueExtrinsic};
use sp_version::RuntimeVersion;
use std::{
	collections::HashSet,
	env, fs,
	io::Read,
	path::PathBuf,
	pin::Pin,
	process::{Child, Command, Stdio},
	sync::Arc,
	time::Duration,
};
use substrate_test_runtime_client::AccountKeyring::*;
use tempfile::tempdir;
use sc_service::{
	AbstractService, Configuration, Error as ServiceError,
	config::{
		MultiaddrWithPeerId, NetworkConfiguration, DatabaseConfig, KeystoreConfig,
		WasmExecutionMethod,
	}, TaskType,
	BasePath, Role,
};
use polkadot_test_runtime_client::{sp_consensus::BlockOrigin, Sr25519Keyring};
use sp_blockchain::HeaderBackend;
use sc_block_builder::BlockBuilderProvider;
use substrate_test_client::ClientBlockImportExt;
use polkadot_service::ChainSpec;
use sp_state_machine::BasicExternalities;
use sc_network::{multiaddr, config::TransportConfig};

static POLKADOT_ARGS: &[&str] = &["polkadot", "--chain=res/polkadot_chainspec.json"];
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

#[async_std::test]
#[ignore]
async fn integration_test() {
	sc_cli::init_logger("runtime=debug");

	let t1 = sleep(Duration::from_secs(
		INTEGRATION_TEST_ALLOWED_TIME
			.and_then(|x| x.parse().ok())
			.unwrap_or(600),
	))
	.fuse();
	let t2 = async {
		let task_executor = Arc::new(
			move |fut: Pin<Box<dyn futures::Future<Output = ()> + Send>>, _| {
				async_std::task::spawn(fut.unit_error());
			},
		);

		// start alice
		let alice = polkadot_test_service::run_test_node(
			task_executor.clone(),
			Alice,
			|| {},
			vec![],
		).unwrap();

		// start bob
		let bob = polkadot_test_service::run_test_node(
			task_executor.clone(),
			Bob,
			|| {},
			vec![alice.multiaddr_with_peer_id.clone()],
		).unwrap();

		// export genesis state
		let cmd = Command::new(cargo_bin("cumulus-test-parachain-collator"))
			.arg("export-genesis-state")
			.output()
			.unwrap();
		assert!(cmd.status.success());
		let output = &cmd.stdout;
		let genesis_state = hex::decode(&output[2..output.len() - 1]).unwrap();

		// get the current block
		let current_block_hash = alice.client.info().best_hash;
		let current_block = alice.client.info().best_number as u64; // TODO: not sure why the type is different here
		let genesis_block = alice.client.hash(0).unwrap().unwrap();

		// create and sign transaction
		let wasm = fs::read(target_dir().join(
			"wbuild/cumulus-test-parachain-runtime/cumulus_test_parachain_runtime.compact.wasm",
		))
		.unwrap();

		// register parachain
		let mut builder = alice.client.new_block(Default::default()).unwrap();

		for extrinsic in polkadot_test_runtime_client::needed_extrinsics(vec![], current_block as u64) {
			builder.push(OpaqueExtrinsic(extrinsic.encode())).unwrap()
		}

		builder.push(
			OpaqueExtrinsic(polkadot_test_runtime::UncheckedExtrinsic {
				function: polkadot_test_runtime::Call::Registrar(registrar::Call::register_para(
					100.into(),
					Info {
						scheduling: Scheduling::Always,
					},
					wasm.into(),
					genesis_state.into(),
				)),
				signature: None,
			}.encode()),
		).unwrap();

		let block = builder.build().unwrap().block;
		//alice.client.import(BlockOrigin::Own, block).unwrap(); // TODO

		// run cumulus charlie
		let charlie_temp_dir = tempdir().unwrap();
		let key = Arc::new(sp_core::Pair::from_seed(&[10; 32]));
		let polkadot_config = polkadot_test_service::node_config(
			|| {},
			task_executor.clone(),
			Charlie,
			charlie_temp_dir,
			vec![
				alice.multiaddr_with_peer_id.clone(),
				bob.multiaddr_with_peer_id.clone(),
			],
		).unwrap();
		let parachain_config = parachain_config(
			|| {},
			task_executor.clone(),
			Charlie,
			vec![],
		).unwrap();
		let service = cumulus_test_parachain_collator::run_collator(parachain_config, key, polkadot_config).unwrap();
		let _base_path = service.base_path();
		service.fuse().await;

		// connect rpc client to cumulus
		/*
		wait_for_blocks(4, &mut client_cumulus_charlie).await;
		*/
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
	storage_update_func: impl Fn(),
	task_executor: Arc<
		dyn Fn(Pin<Box<dyn futures::Future<Output = ()> + Send>>, TaskType) + Send + Sync,
	>,
	key: Sr25519Keyring,
	boot_nodes: Vec<MultiaddrWithPeerId>,
) -> Result<Configuration, ServiceError> {
	let base_path = BasePath::new_temp_dir()?;
	let root = base_path.path().to_path_buf();
	let role = Role::Authority {
		sentry_nodes: Vec::new(),
	};
	let key_seed = key.to_seed();
	let mut spec = cumulus_test_parachain_collator::get_chain_spec();
	let mut storage = spec.as_storage_builder().build_storage()?;

	BasicExternalities::execute_with_storage(&mut storage, storage_update_func);
	spec.set_storage(storage);

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

	Ok(Configuration {
		impl_name: "cumulus-test-node",
		impl_version: "0.1",
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
		execution_strategies: Default::default(),
		rpc_http: None,
		rpc_ws: None,
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
	})
}
