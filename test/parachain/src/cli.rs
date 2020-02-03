// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Cumulus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

use std::path::PathBuf;

use sc_cli;
use polkadot_cli::Cli as PolkadotCli;
use structopt::StructOpt;

/// Sub-commands supported by the collator.
#[derive(Debug, StructOpt, Clone)]
enum Subcommand {
	#[structopt(flatten)]
	Base(sc_cli::Subcommand),
	/// Export the genesis state of the parachain.
	#[structopt(name = "export-genesis-state")]
	ExportGenesisState(ExportGenesisStateCommand),
}

/// Command for exporting the genesis state of the parachain
#[derive(Debug, StructOpt, Clone)]
struct ExportGenesisStateCommand {
	/// Output file name or stdout if unspecified.
	#[structopt(parse(from_os_str))]
	pub output: Option<PathBuf>,
}

#[derive(Debug, StructOpt, Clone)]
struct Cli {
	#[structopt(subcommand)]
	subcommand: Option<Subcommand>,

	#[structopt(flatten)]
	run: sc_cli::RunCmd,

	/// Relaychain arguments
	#[structopt(raw = true)]
	relaychain_args: Vec<String>,
}
