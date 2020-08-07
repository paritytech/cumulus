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
use std::{convert::TryInto, fs, process::Command, thread, time::Duration};

mod common;

#[test]
#[cfg(unix)]
fn interrupt_polkadot_mdns_issue_test() {
	use nix::{
		sys::signal::{
			kill,
			Signal::{self, SIGINT, SIGTERM},
		},
		unistd::Pid,
	};

	fn run_command_and_kill(signal: Signal) {
		let _ = fs::remove_dir_all("interrupt_polkadot_mdns_issue_test");
		let mut cmd = Command::new(cargo_bin("rococo-collator"))
			.args(&["-d", "interrupt_polkadot_mdns_issue_test", "--", "--dev"])
			.spawn()
			.unwrap();

		thread::sleep(Duration::from_secs(20));
		assert!(
			cmd.try_wait().unwrap().is_none(),
			"the process should still be running"
		);
		kill(Pid::from_raw(cmd.id().try_into().unwrap()), signal).unwrap();
		assert_eq!(
			common::wait_for(&mut cmd, 30).map(|x| x.success()),
			Some(true),
			"the process must exit gracefully after signal {}",
			signal,
		);
	}

	run_command_and_kill(SIGINT);
	run_command_and_kill(SIGTERM);
}
