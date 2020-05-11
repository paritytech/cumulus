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

// TODO
// This test needs --release to work
//#![cfg(not(debug_assertions))]
#![allow(unused_imports)]

use assert_cmd::cargo::cargo_bin;
use std::{
	convert::TryInto, process::{Child, Command, Stdio}, thread, time::Duration, fs, path::PathBuf,
	env, net, io,
};
use tempfile::tempdir;
use jsonrpc_client_transports::transports::ws;
use jsonrpc_client_transports::RpcChannel;
use url::Url;
use polkadot_primitives::parachain::{Info, Scheduling};
use polkadot_primitives::Hash as PHash;
use polkadot_runtime_common::{registrar, parachains, BlockHashCount};
use polkadot_runtime::{Runtime, OnlyStakingAndClaims, SignedExtra, SignedPayload, System};
use polkadot_service::chain_spec::{polkadot_local_testnet_config, PolkadotChainSpec};
use polkadot_service::ChainSpec;
use codec::Encode;
use substrate_test_runtime_client::AccountKeyring::Alice;
use sp_runtime::generic;
use sp_arithmetic::traits::SaturatedConversion;
use sp_api::runtime_decl_for_Core::Core;
use sp_state_machine::BasicExternalities;
use sp_runtime::traits::Verify;
use sp_runtime::traits::SignedExtension;

static POLKADOT_ARGS: &[&str] = &["polkadot", "--chain=res/polkadot_chainspec.json"];

jsonrpsee::rpc_api! {
	Author {
		#[rpc(method = "author_submitExtrinsic", positional_params)]
		fn submit_extrinsic(extrinsic: String) -> String;
	}

	Chain {
		#[rpc(method = "chain_getFinalizedHead")]
		fn finalized_head() -> PHash;

		#[rpc(method = "chain_getHeader", positional_params)]
		fn header(hash: PHash) -> Option<polkadot_runtime::Header>;

		#[rpc(method = "chain_getBlockHash", positional_params)]
		fn block_hash(hash: Option<u64>) -> Option<PHash>;
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

struct ProcessCleanUp<'a>(&'a mut Child);

impl<'a> Drop for ProcessCleanUp<'a> {
	fn drop(&mut self) {
		#[cfg(unix)]
		{
			use nix::sys::signal::{kill, Signal::SIGTERM};
			use nix::unistd::Pid;

			kill(Pid::from_raw(self.0.id().try_into().unwrap()), SIGTERM).unwrap();

			let mut tries = 30;

			let success = loop {
				tries -= 1;

				match self.0.try_wait() {
					Ok(Some(_)) => break true,
					Ok(None) if tries == 0 => break false,
					Ok(None) => thread::sleep(Duration::from_secs(1)),
					Err(err) => {
						eprintln!("could not wait for child process to finish: {}", err);
						break false;
					},
				}
			};

			if !success {
				let _ = self.0.kill();
			}
		}

		#[cfg(not(unix))]
		let _ = self.0.kill();

		let _ = self.0.wait();
	}
}

fn wait_for_tcp<A: net::ToSocketAddrs>(address: A) -> io::Result<()> {
	let mut tries = 10;

	loop {
		tries -= 1;

		match net::TcpStream::connect(&address) {
			Ok(_) => break Ok(()),
			Err(err) if tries == 0 => break Err(err),
			_ => thread::sleep(Duration::from_secs(1)),
		}
	}
}

#[test]
//#[tokio::test]
fn integration_test() {
	// starts Alice
	let polkadot_alice_dir = tempdir().unwrap();
	let mut polkadot_alice = Command::new(cargo_bin("cumulus-test-parachain-collator"))
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.args(POLKADOT_ARGS)
		.arg("--base-path")
		.arg(polkadot_alice_dir.path())
		.arg("--alice")
		.arg("--unsafe-rpc-expose")
		.spawn()
		.unwrap();
	let polkadot_alice_clean_up = ProcessCleanUp(&mut polkadot_alice);
	wait_for_tcp("127.0.0.1:9933").unwrap();

	// starts Bob
	let polkadot_bob_dir = tempdir().unwrap();
	let mut polkadot_bob = Command::new(cargo_bin("cumulus-test-parachain-collator"))
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.args(POLKADOT_ARGS)
		.arg("--base-path")
		.arg(polkadot_bob_dir.path())
		.arg("--bob")
		.spawn()
		.unwrap();
	let polkadot_bob_clean_up = ProcessCleanUp(&mut polkadot_bob);

	// export genesis state
	let cmd = Command::new(cargo_bin("cumulus-test-parachain-collator"))
		.arg("export-genesis-state")
		.output()
		.unwrap();
	assert!(cmd.status.success());
	let output = &cmd.stdout[2..];
	let genesis_state = hex::decode(&output[2..output.len() - 1]).unwrap();
	thread::sleep_ms(10000);

	// connect RPC client
	let transport_client =
		jsonrpsee::transport::http::HttpTransportClient::new("http://127.0.0.1:9933");
	let mut client = jsonrpsee::raw::RawClient::new(transport_client);

	// get the current block
	/*
	let current_block = System::block_number()
		.saturated_into::<u64>()
		.saturating_sub(1);
	*/
	/*
	let finalized_head = async_std::task::block_on(async {
		Chain::finalized_head(&mut client).await.unwrap()
	});
	*/
	let finalized_head = async_std::task::block_on(async {
		Chain::block_hash(&mut client, None).await.unwrap()
	}).unwrap();
	let current_block = async_std::task::block_on(async {
		Chain::header(&mut client, finalized_head).await.unwrap()
	}).unwrap().number.saturated_into::<u64>();

	let genesis_block = async_std::task::block_on(async {
		Chain::block_hash(&mut client, 0).await.unwrap()
	}).unwrap();

	// create and sign transaction
	let wasm = fs::read(target_dir()
		.join("wbuild/cumulus-test-parachain-runtime/cumulus_test_parachain_runtime.compact.wasm")).unwrap();
	let call = registrar::Call::<Runtime>::register_para(
		100.into(),
		Info {
			scheduling: Scheduling::Always,
		},
		wasm.into(),
		genesis_state.into(),
	);
	let nonce = 0;
	let period = BlockHashCount::get()
		.checked_next_power_of_two()
		.map(|c| c / 2)
		.unwrap_or(2) as u64;
	let tip = 0;
	let extra: SignedExtra = (
		OnlyStakingAndClaims,
		frame_system::CheckVersion::<Runtime>::new(),
		frame_system::CheckGenesis::<Runtime>::new(),
		frame_system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
		frame_system::CheckNonce::<Runtime>::from(nonce),
		frame_system::CheckWeight::<Runtime>::new(),
		pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
		registrar::LimitParathreadCommits::<Runtime>::new(),
		parachains::ValidateDoubleVoteReports::<Runtime>::new(),
	);
	//let genesis_config = polkadot_local_testnet_config();
	/*
	let genesis_config = PolkadotChainSpec::from_json_bytes(&include_bytes!("../res/polkadot_chainspec.json")[..]).unwrap();
	let storage = genesis_config.as_storage_builder().build_storage().unwrap();
	let mut basic_ext = BasicExternalities::new(storage);
	let raw_payload = basic_ext.execute_with(|| {
		panic!("{:?} {:?}", extra.additional_signed(), finalized_head);
		SignedPayload::new(call.clone().into(), extra.clone())
	}).unwrap();
	*/
	let raw_payload = SignedPayload::from_raw(call.clone().into(), extra.clone(), (
		(),
		Runtime::version().spec_version,
		genesis_block,
		finalized_head,
		(),
		(),
		(),
		(),
		(),
	));
	//panic!("{:?} {:?} {:?}", Runtime::version().spec_version, genesis_block, finalized_head);
	let signature = raw_payload.using_encoded(|e| Alice.sign(e));
	//assert!(raw_payload.using_encoded(|payload| signature.verify(payload, &Alice.public())));

	// register parachain
	let ex = polkadot_runtime::UncheckedExtrinsic::new_signed(
		call.into(),
		Alice.into(),
		sp_runtime::MultiSignature::Sr25519(signature),
		extra,
	);
	/*
	async_std::task::block_on(async {
		let uri = "http://localhost:9933";

		http::connect(uri)
			.and_then(|client: AuthorClient<Hash, Hash>| {
				remove_all_extrinsics(client)
			})
			.map_err(|e| {
				println!("Error: {:?}", e);
			})
			.await
	});
	*/
	//let client = ws::connect(&Url::parse("ws://127.0.0.1:9944").unwrap()).await;
	let v = async_std::task::block_on(async {
		Author::submit_extrinsic(&mut client, format!("0x{}", hex::encode(ex.encode()))).await.unwrap()
	});
	/*
	assert!(Command::new("/tmp/b/node_modules/.bin/polkadot-js-api")
		.args(&["--ws", "ws://127.0.0.1:9944", "--sudo", "--seed", "//Alice", "tx.registrar.registerPara", "100", "{\"scheduling\":\"Always\"}", "@/home/cecile/repos/cumulus/target/release/wbuild/cumulus-test-parachain-runtime/cumulus_test_parachain_runtime.compact.wasm"])
		.arg(&String::from_utf8(genesis_state).unwrap())
		.status()
		.unwrap()
		.success());
	*/

	panic!("{:?}", v);
}
