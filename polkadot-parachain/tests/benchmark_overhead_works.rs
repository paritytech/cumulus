use std::process::Command;

use assert_cmd::cargo::cargo_bin;
use tempfile::tempdir;

/// The runtimes that this command supports.
static RUNTIMES: [&str; 4] =
	["asset-hub-westend", "asset-hub-kusama", "asset-hub-polkadot", "collectives-polkadot"];

/// The `benchmark overhead` command works for the dev runtimes.
#[test]
#[ignore]
fn benchmark_overhead_works() {
	for runtime in RUNTIMES {
		let tmp_dir = tempdir().expect("could not create a temp dir");
		let base_path = tmp_dir.path();
		let runtime = format!("{}-dev", runtime);

		let status = Command::new(cargo_bin("polkadot-parachain"))
			.args(["benchmark", "overhead", "--chain", &runtime, "-d"])
			.arg(base_path)
			.arg("--weight-path")
			.arg(base_path)
			.args(["--warmup", "10", "--repeat", "10"])
			.args(["--add", "100", "--mul", "1.2", "--metric", "p75"])
			.args(["--max-ext-per-block", "10"])
			.status()
			.unwrap();
		assert!(status.success());

		// Weight files have been created.
		assert!(base_path.join("block_weights.rs").exists());
		assert!(base_path.join("extrinsic_weights.rs").exists());
	}
}
