// Unix only since it uses signals.
#![cfg(unix)]

use assert_cmd::cargo::cargo_bin;
use nix::{
	sys::signal::{kill, Signal::SIGINT},
	unistd::Pid,
};
use std::{path::Path, process::Command, thread, time::Duration};
use tempfile::tempdir;

pub mod common;

/// The runtimes that this command supports.
static RUNTIMES: [&'static str; 3] = ["westmint", "statemine", "statemint"];

/// `benchmark block` works for all dev runtimes using the wasm executor.
#[test]
#[ignore]
fn benchmark_block_works() {
	for runtime in RUNTIMES {
		let tmp_dir = tempdir().expect("could not create a temp dir");
		let base_path = tmp_dir.path();
		let runtime = format!("{}-dev", runtime);

		// Build a chain with at least one block.
		build_chain(&runtime, base_path);
		// Benchmark the that one block.
		benchmark_block(&runtime, base_path, 1);
	}
}

/// Builds a chain with at least one block for the given runtime and base path.
fn build_chain(runtime: &str, base_path: &Path) {
	let mut cmd = Command::new(cargo_bin("polkadot-collator"))
		.args(["--chain", runtime, "--force-authoring", "--alice"])
		.arg("-d")
		.arg(base_path)
		.spawn()
		.unwrap();

	// Let it produce some blocks.
	thread::sleep(Duration::from_secs(30));
	assert!(cmd.try_wait().unwrap().is_none(), "the process should still be running");

	// Stop the process
	kill(Pid::from_raw(cmd.id().try_into().unwrap()), SIGINT).unwrap();
	assert!(common::wait_for(&mut cmd, 30).map(|x| x.success()).unwrap_or_default());
}

/// Benchmarks the given block with the wasm executor.
fn benchmark_block(runtime: &str, base_path: &Path, block: u32) {
	// Invoke `benchmark block` with all options to make sure that they are valid.
	let status = Command::new(cargo_bin("polkadot-collator"))
		.args(["benchmark", "block", "--chain", runtime])
		.arg("-d")
		.arg(base_path)
		.args(["--pruning", "archive"])
		.args(["--from", &block.to_string(), "--to", &block.to_string()])
		.args(["--repeat", "1"])
		.args(["--execution", "wasm", "--wasm-execution", "compiled"])
		.status()
		.unwrap();

	assert!(status.success());
}
