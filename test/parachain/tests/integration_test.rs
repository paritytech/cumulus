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

// TODO: this is necessary for the jsonrpsee macro used
#![allow(unused_variables, dead_code)]

use assert_cmd::cargo::cargo_bin;
use codec::Encode;
use polkadot_primitives::parachain::{Info, Scheduling};
use polkadot_primitives::Hash as PHash;
use polkadot_runtime::{Header, OnlyStakingAndClaims, Runtime, SignedExtra, SignedPayload};
use polkadot_runtime_common::{parachains, registrar, BlockHashCount};
use regex::Regex;
use sp_arithmetic::traits::SaturatedConversion;
use sp_runtime::generic;
use sp_version::RuntimeVersion;
use std::io::Read;
use std::{
	convert::TryInto,
	env, fs, io, net,
	path::PathBuf,
	process::{Child, Command, Stdio},
	thread,
	time::Duration,
};
use substrate_test_runtime_client::AccountKeyring::Alice;
use tempfile::tempdir;

static POLKADOT_ARGS: &[&str] = &["polkadot", "--chain=res/polkadot_chainspec.json"];

jsonrpsee::rpc_api! {
	Author {
		#[rpc(method = "author_submitExtrinsic", positional_params)]
		fn submit_extrinsic(extrinsic: String) -> PHash;
	}

	Chain {
		#[rpc(method = "chain_getFinalizedHead")]
		fn current_block_hash() -> PHash;

		#[rpc(method = "chain_getHeader", positional_params)]
		fn header(hash: PHash) -> Option<Header>;

		#[rpc(method = "chain_getBlockHash", positional_params)]
		fn block_hash(hash: Option<u64>) -> Option<PHash>;
	}

	State {
		#[rpc(method = "state_getRuntimeVersion")]
		fn runtime_version() -> RuntimeVersion;
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

struct ChildHelper<'a> {
	name: String,
	child: &'a mut Child,
	stdout: String,
	stderr: String,
}

impl<'a> Drop for ChildHelper<'a> {
	fn drop(&mut self) {
		let name = self.name.clone();

		self.terminate();
		eprintln!(
			"process '{}' stdout:\n{}\n",
			name,
			self.read_stdout_to_end().unwrap_or_default()
		);
		eprintln!(
			"process '{}' stderr:\n{}\n",
			name,
			self.read_stderr_to_end().unwrap_or_default()
		);
	}
}

impl<'a> ChildHelper<'a> {
	fn new(name: &str, child: &'a mut Child) -> ChildHelper<'a> {
		ChildHelper {
			name: name.to_string(),
			child,
			stdout: Default::default(),
			stderr: Default::default(),
		}
	}

	fn read_stdout_to_end(&mut self) -> io::Result<&str> {
		let mut output = String::new();

		self.child
			.stdout
			.as_mut()
			.ok_or_else(|| io::Error::new(io::ErrorKind::Other, "stdout not captured"))?
			.read_to_string(&mut output)?;
		self.stdout.push_str(output.as_str());

		Ok(&self.stdout)
	}

	fn read_stderr_to_end(&mut self) -> io::Result<&str> {
		let mut output = String::new();

		self.child
			.stderr
			.as_mut()
			.ok_or_else(|| io::Error::new(io::ErrorKind::Other, "stderr not captured"))?
			.read_to_string(&mut output)?;
		self.stderr.push_str(output.as_str());

		Ok(&self.stderr)
	}

	fn read_stderr(&mut self, size: usize) -> io::Result<&str> {
		let mut buffer = vec![0; size];
		let size = self
			.child
			.stderr
			.as_mut()
			.ok_or_else(|| io::Error::new(io::ErrorKind::Other, "stderr not captured"))?
			.read(&mut buffer)?;

		self.stderr
			.push_str(&String::from_utf8_lossy(&buffer[..size]));

		Ok(&self.stderr)
	}

	fn terminate(&mut self) {
		match self.child.try_wait() {
			Ok(Some(_)) => return,
			Ok(None) => {}
			Err(err) => {
				eprintln!("could not wait for child process to finish: {}", err);
				let _ = self.child.kill();
				let _ = self.child.wait();
				return;
			}
		}

		#[cfg(unix)]
		{
			use nix::sys::signal::{kill, Signal::SIGTERM};
			use nix::unistd::Pid;

			kill(Pid::from_raw(self.child.id().try_into().unwrap()), SIGTERM).unwrap();

			let mut tries = 30;

			let success = loop {
				tries -= 1;

				match self.child.try_wait() {
					Ok(Some(_)) => break true,
					Ok(None) if tries == 0 => break false,
					Ok(None) => thread::sleep(Duration::from_secs(1)),
					Err(err) => {
						eprintln!("could not wait for child process to finish: {}", err);
						break false;
					}
				}
			};

			if !success {
				let _ = self.child.kill();
			}
		}

		#[cfg(not(unix))]
		let _ = self.child.kill();

		let _ = self.child.wait();
	}
}

fn tcp_port_is_open<A: net::ToSocketAddrs>(address: A) -> bool {
	match net::TcpStream::connect(&address) {
		Ok(_) => true,
		Err(_) => false,
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
#[ignore]
fn integration_test() {
	assert!(
		!tcp_port_is_open("127.0.0.1:9933"),
		"tcp port is already open 127.0.0.1:9933, this test cannot be ran",
	);

	// start alice
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
	let mut polkadot_alice_child = ChildHelper::new("alice", &mut polkadot_alice);
	wait_for_tcp("127.0.0.1:9933").unwrap();
	let polkadot_alice_id = find_local_node_identity(&mut polkadot_alice_child);

	// start bob
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
	let mut polkadot_bob_child = ChildHelper::new("bob", &mut polkadot_bob);
	let polkadot_bob_id = find_local_node_identity(&mut polkadot_bob_child);

	// wait a bit for some relay chains blocks to be generated
	thread::sleep(Duration::from_secs(10));

	// export genesis state
	let cmd = Command::new(cargo_bin("cumulus-test-parachain-collator"))
		.arg("export-genesis-state")
		.output()
		.unwrap();
	assert!(cmd.status.success());
	let output = &cmd.stdout;
	let genesis_state = hex::decode(&output[2..output.len() - 1]).unwrap();

	// connect RPC client
	let transport_client =
		jsonrpsee::transport::http::HttpTransportClient::new("http://127.0.0.1:9933");
	let mut client = jsonrpsee::raw::RawClient::new(transport_client);

	// retrieve runtime version
	let runtime_version =
		async_std::task::block_on(async { State::runtime_version(&mut client).await.unwrap() });

	// get the current block
	let current_block_hash =
		async_std::task::block_on(async { Chain::block_hash(&mut client, None).await.unwrap() })
			.unwrap();
	let current_block = async_std::task::block_on(async {
		Chain::header(&mut client, current_block_hash)
			.await
			.unwrap()
	})
	.unwrap()
	.number
	.saturated_into::<u64>();

	let genesis_block =
		async_std::task::block_on(async { Chain::block_hash(&mut client, 0).await.unwrap() })
			.unwrap();

	// create and sign transaction
	let wasm =
		fs::read(target_dir().join(
			"wbuild/cumulus-test-parachain-runtime/cumulus_test_parachain_runtime.compact.wasm",
		))
		.unwrap();
	let call = pallet_sudo::Call::sudo(Box::new(
		registrar::Call::<Runtime>::register_para(
			100.into(),
			Info {
				scheduling: Scheduling::Always,
			},
			wasm.into(),
			genesis_state.into(),
		)
		.into(),
	));
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
	let raw_payload = SignedPayload::from_raw(
		call.clone().into(),
		extra.clone(),
		(
			(),
			runtime_version.spec_version,
			genesis_block,
			current_block_hash,
			(),
			(),
			(),
			(),
			(),
		),
	);
	let signature = raw_payload.using_encoded(|e| Alice.sign(e));

	// register parachain
	let ex = polkadot_runtime::UncheckedExtrinsic::new_signed(
		call.into(),
		Alice.into(),
		sp_runtime::MultiSignature::Sr25519(signature),
		extra,
	);
	let _register_block_hash = async_std::task::block_on(async {
		Author::submit_extrinsic(&mut client, format!("0x{}", hex::encode(ex.encode()))).await
	})
	.unwrap();

	// run cumulus
	let cumulus_dir = tempdir().unwrap();
	let mut cumulus = Command::new(cargo_bin("cumulus-test-parachain-collator"))
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.arg("--base-path")
		.arg(cumulus_dir.path())
		.arg("--")
		.arg(format!(
			"--bootnodes=/ip4/127.0.0.1/tcp/30333/p2p/{}",
			polkadot_alice_id
		))
		.arg(format!(
			"--bootnodes=/ip4/127.0.0.1/tcp/50666/p2p/{}",
			polkadot_bob_id
		))
		.spawn()
		.unwrap();
	let mut cumulus_child = ChildHelper::new("cumulus", &mut cumulus);

	// wait for blocks to be generated
	thread::sleep(Duration::from_secs(60));

	// check output
	cumulus_child.terminate();
	assert!(
		cumulus_child
			.read_stderr_to_end()
			.unwrap()
			.contains("best: #2"),
		"no parachain blocks seems to have been produced",
	);
}

fn find_local_node_identity(instance: &mut ChildHelper) -> String {
	let regex = Regex::new(r"Local node identity is: (.+)\n").unwrap();

	loop {
		let s = instance.read_stderr(200).unwrap();

		if let Some(captures) = regex.captures(s) {
			break captures.get(1).unwrap().as_str().to_string();
		} else if s.len() > 2000 {
			panic!("could not find node identity");
		}
	}
}
