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
use async_std::{net, task::{sleep, spawn}};
//use tokio::{time::delay_for as sleep, spawn};
use codec::Encode;
use futures::{future::{Future, FutureExt}, join, pin_mut, select, stream::StreamExt};
use polkadot_primitives::parachain::{Info, Scheduling, Id as ParaId};
use polkadot_primitives::Hash as PHash;
use polkadot_test_runtime::{VERSION, Header};
use polkadot_runtime_common::{parachains, registrar, BlockHashCount, claims};
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
	Configuration, Error as ServiceError,
	config::{
		MultiaddrWithPeerId, NetworkConfiguration, DatabaseConfig, KeystoreConfig,
		WasmExecutionMethod,
	}, TaskType,
	BasePath, Role, RpcSession, TaskExecutor,
};
use polkadot_test_runtime_client::{sp_consensus::BlockOrigin, Sr25519Keyring};
use sp_blockchain::HeaderBackend;
use sc_block_builder::BlockBuilderProvider;
use substrate_test_client::ClientBlockImportExt;
use polkadot_service::ChainSpec;
use sp_state_machine::BasicExternalities;
use sc_network::{multiaddr, config::TransportConfig};
use polkadot_test_runtime::{SignedExtra, Runtime, RestrictFunctionality, SignedPayload};
use sc_client_api::{execution_extensions::ExecutionStrategies, BlockchainEvents};
use sp_core::{Decode};
use sp_consensus::{BlockImport, BlockImportParams, ForkChoiceStrategy};
use sp_runtime::traits::Block as BlockT;
use futures::future;
use sp_runtime::traits::Checkable;
use sp_runtime::traits::Verify;
use sp_transaction_pool::TransactionPool;
use log::error;

static POLKADOT_ARGS: &[&str] = &["polkadot", "--chain=res/polkadot_chainspec.json"];
static INTEGRATION_TEST_ALLOWED_TIME: Option<&str> = option_env!("INTEGRATION_TEST_ALLOWED_TIME");

jsonrpsee::rpc_api! {
	Author {
		#[rpc(method = "author_submitExtrinsic", positional_params)]
		fn submit_extrinsic(extrinsic: String) -> PHash;
	}
}

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

//#[tokio::test]
#[async_std::test]
#[ignore]
async fn integration_test() {
	let task_executor: TaskExecutor = (|fut, _| {
		spawn(fut);
	}).into();

	//sc_cli::init_logger("runtime=debug,babe=trace");
	sc_cli::init_logger("", None);

	let t1 = sleep(Duration::from_secs(
		INTEGRATION_TEST_ALLOWED_TIME
			.and_then(|x| x.parse().ok())
			.unwrap_or(600),
	))
	.fuse();
	let t2 = async {
		// start alice
		let alice = polkadot_test_service::run_test_node(
			task_executor.clone(),
			Alice,
			|| {},
			vec![],
		);

		// start bob
		let bob = polkadot_test_service::run_test_node(
			task_executor.clone(),
			Bob,
			|| {},
			vec![alice.addr.clone()],
		);

		future::join(alice.wait_for_blocks(2_usize), bob.wait_for_blocks(2_usize)).await;
		//assert!(false);

		// export genesis state
		//let genesis_state = cumulus_test_parachain_collator::generate_genesis_state(); // TODO
		let cmd = Command::new(cargo_bin("cumulus-test-parachain-collator"))
			.arg("export-genesis-state")
			.output()
			.unwrap();
		println!("{}", String::from_utf8_lossy(cmd.stdout.as_slice()));
		println!("{}", String::from_utf8_lossy(cmd.stderr.as_slice()));
		assert!(cmd.status.success());
		let output = &cmd.stdout;
		let genesis_state = hex::decode(&output[2..output.len() - 1]).unwrap();

		// get the current block
		let current_block_hash = alice.client.info().best_hash;
		let current_block = alice.client.info().best_number.saturated_into(); //.saturating_sub(1);
		let genesis_block = alice.client.hash(0).unwrap().unwrap();

		// create and sign transaction
		let wasm = fs::read(target_dir().join(
			"wbuild/cumulus-test-parachain-runtime/cumulus_test_parachain_runtime.compact.wasm",
		))
		.unwrap();
		let nonce = 0;
		let period = BlockHashCount::get()
			.checked_next_power_of_two()
			.map(|c| c / 2)
			.unwrap_or(2) as u64;
		let tip = 0;
		let extra: SignedExtra = (
			RestrictFunctionality,
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckTxVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
			frame_system::CheckNonce::<Runtime>::from(nonce),
			frame_system::CheckWeight::<Runtime>::new(),
			pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
			registrar::LimitParathreadCommits::<Runtime>::new(),
			parachains::ValidateDoubleVoteReports::<Runtime>::new(),
		);
		let function = polkadot_test_runtime::Call::Sudo(pallet_sudo::Call::sudo(Box::new(
			polkadot_test_runtime::Call::Registrar(registrar::Call::register_para(
				100.into(),
				Info {
					scheduling: Scheduling::Always,
				},
				wasm.into(),
				genesis_state.into(),
			)),
		)));
		let raw_payload = SignedPayload::from_raw(
			function.clone(),
			extra.clone(),
			((),
			VERSION.spec_version,
			VERSION.transaction_version,
			genesis_block,
			current_block_hash,
			(), (), (), (), ()),
		);
		let signature = raw_payload.using_encoded(|e| Alice.sign(e));

		// register parachain
		/*
		let mut builder = alice.client.new_block(Default::default()).unwrap();

		for extrinsic in polkadot_test_runtime_client::needed_extrinsics(vec![], current_block as u64) {
			builder.push(extrinsic.into()).unwrap()
		}

		// TODO: is the signature needed at all? both seem to work
		let extrinsic = polkadot_test_runtime::UncheckedExtrinsic::new_unsigned(
			function.clone(),
		);
		*/
		let ex = polkadot_test_runtime::UncheckedExtrinsic::new_signed(
			function.clone(),
			polkadot_test_runtime::Address::Id(Alice.public().into()),
			polkadot_primitives::Signature::Sr25519(signature.clone()),
			extra.clone(),
		);
		//extrinsic.check().unwrap();
		let signed = Alice.public();
		println!("{:?}", ((),
			VERSION.spec_version,
			VERSION.transaction_version,
			genesis_block,
			current_block_hash,
			(), (), (), (), ()));
		println!("{:?}", raw_payload.using_encoded(|payload| signature.verify(payload, &signed)));
		let (tx, rx) = futures01::sync::mpsc::channel(0);
		let mem = RpcSession::new(tx.into());
		let res = alice.call_function(function, Alice).await.unwrap();
		error!("############# {:?}", res.result);
		//error!("===== start");
		//sleep(Duration::from_secs(5)).await;
		//error!("===== end");
		//sleep(Duration::from_secs(600)).await;

		//alice.service.transaction_pool().submit_one((), (), extrinsic);
		/*
		builder.push(
			OpaqueExtrinsic::decode(&mut extrinsic.encode().as_slice()).unwrap(),
		).unwrap();

		let block = builder.build().unwrap().block;
		println!("######### {}", block.header.parent_hash);
		//alice.client.import(BlockOrigin::Own, block).unwrap(); // TODO
		*/

		/*
		let origin = sp_consensus::BlockOrigin::Own;
		let (header, extrinsics) = block.deconstruct();
		let mut import = BlockImportParams::new(origin, header);
		import.body = Some(extrinsics);
		import.fork_choice = Some(ForkChoiceStrategy::LongestChain);
		//println!("{}", *import.header.parent_hash());
		alice.client.import_block(import, std::collections::HashMap::new());
		//assert!(false);
		future::join(alice.wait_for_blocks(3), bob.wait_for_blocks(3)).await;
		assert!(false);
		//assert!(false);
		//let _telemetry = alice.service.telemetry();
		//alice.service.fuse().await;
		*/

		// run cumulus charlie
		let para_id = ParaId::from(100);
		let key = Arc::new(sp_core::Pair::from_seed(&[10; 32]));
		let mut polkadot_config = polkadot_test_service::node_config(
			|| {},
			task_executor.clone(),
			Charlie,
			vec![
				alice.addr.clone(),
				bob.addr.clone(),
			],
		);
		use std::net::{SocketAddr, Ipv4Addr};
		polkadot_config.rpc_http = Some(SocketAddr::new(
			Ipv4Addr::LOCALHOST.into(),
			27016,
		));
		polkadot_config.rpc_methods = sc_service::config::RpcMethods::Unsafe;
		let parachain_config = parachain_config(
			|| {},
			task_executor.clone(),
			Charlie,
			vec![],
			para_id,
		).unwrap();
		let service = cumulus_test_parachain_collator::run_collator(parachain_config, key, polkadot_config, para_id).unwrap();
		sleep(Duration::from_secs(3)).await;
		/*
		let transport_client_alice =
			jsonrpsee::transport::http::HttpTransportClient::new("http://127.0.0.1:27016");
		let mut client_alice = jsonrpsee::raw::RawClient::new(transport_client_alice);
		let _register_block_hash =
			Author::submit_extrinsic(&mut client_alice, format!("0x{}", hex::encode(ex.encode())))
				.await
				.unwrap();
		*/
		wait_for_blocks(service.client.clone(), 4).await;
		//service.fuse().await;

		// connect rpc client to cumulus
		/*
		wait_for_blocks(4, &mut client_cumulus_charlie).await;
		*/
		//sleep(Duration::from_secs(60)).await;
		//assert!(false);
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
	let mut spec = cumulus_test_parachain_collator::get_chain_spec(para_id);
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

	use std::net::{SocketAddr, Ipv4Addr};
	let rpc_ws = Some(SocketAddr::new(
		Ipv4Addr::LOCALHOST.into(),
		9944,
	));

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

fn wait_for_blocks<Block: BlockT>(client: Arc<impl BlockchainEvents<Block>>, count: usize) -> impl Future<Output = ()> {
	assert_ne!(count, 0, "'count' argument must be greater than 1");
	let client = client.clone();

	async move {
		let mut import_notification_stream = client.import_notification_stream();
		let mut blocks = HashSet::new();

		while let Some(notification) = import_notification_stream.next().await {
			blocks.insert(notification.hash);
			if blocks.len() == 3 {
				error!("################ {:?}", blocks);
				println!("{:?}", blocks);
				break;
			}
		}
	}
}
