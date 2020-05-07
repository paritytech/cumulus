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

// This test needs --release to work
//#![cfg(not(debug_assertions))]

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
use polkadot_runtime_common::registrar;
use codec::Encode;
use substrate_test_runtime_client::AccountKeyring::Alice;

static POLKADOT_ARGS: &[&str] = &["polkadot", "--chain=res/polkadot_chainspec.json"];

jsonrpsee::rpc_api! {
	Author {
		#[rpc(method = "author_submitExtrinsic")]
		fn submit_extrinsic(extrinsic: Vec<u8>) -> String;
	}
}

/*
jsonrpsee::rpc_api! {
	Health {
		/// Test
		fn system_name(foo: String, bar: i32) -> String;

		fn test_notif(foo: String, bar: i32);

		/// Test2
		#[rpc(method = "foo")]
		fn system_name2() -> String;
	}

	System {
		fn test_foo() -> String;
	}
}
*/

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

	// register parachain
	let wasm: Vec<u8> = vec![];
	let tx = registrar::Call::<polkadot_runtime::Runtime>::register_para(
		100.into(),
		Info {
			scheduling: Scheduling::Always,
		},
		wasm.into(),
		genesis_state.into(),
	);
	let signature = tx.using_encoded(|e| Alice.sign(e));
	let ex = polkadot_runtime::UncheckedExtrinsic {
		//signature: Some(((), sp_runtime::MultiSignature::Sr25519(signature), ())),
		signature: None,
		function: tx.into(),
	};
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
	let transport_client =
		jsonrpsee::transport::http::HttpTransportClient::new("http://127.0.0.1:9933");
	let mut client = jsonrpsee::raw::RawClient::new(transport_client);
	let v = async_std::task::block_on(async {
		Author::submit_extrinsic(&mut client, ex.encode()).await.unwrap()
	});
	/*
	assert!(Command::new("/tmp/b/node_modules/.bin/polkadot-js-api")
		.args(&["--ws", "ws://127.0.0.1:9944", "--sudo", "--seed", "//Alice", "tx.registrar.registerPara", "100", "{\"scheduling\":\"Always\"}", "@/home/cecile/repos/cumulus/target/release/wbuild/cumulus-test-parachain-runtime/cumulus_test_parachain_runtime.compact.wasm"])
		.arg(&String::from_utf8(genesis_state).unwrap())
		.status()
		.unwrap()
		.success());
	*/

	assert!(false);
}
