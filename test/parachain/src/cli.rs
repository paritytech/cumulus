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

use crate::chain_spec;

use parachain_runtime::Block;

pub use sc_cli::{error::{self, Result}, VersionInfo};
use sc_client::genesis;
use sc_service::{Configuration, Roles as ServiceRoles};
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::{
	traits::{Block as BlockT, Hash as HashT, Header as HeaderT},
	BuildStorage,
};
use polkadot_service::ChainSpec as ChainSpecPolkadot;
use polkadot_cli::Cli as PolkadotCli;

use futures::{channel::oneshot, future::Map, FutureExt};

use codec::Encode;

use log::info;

use std::{cell::RefCell, path::PathBuf, sync::Arc};

use structopt::StructOpt;

/// Sub-commands supported by the collator.
#[derive(Debug, StructOpt, Clone)]
enum SubCommands {
	/// Export the genesis state of the parachain.
	#[structopt(name = "export-genesis-state")]
	ExportGenesisState(ExportGenesisStateCommand),
}

impl sc_cli::GetSharedParams for SubCommands {
	fn shared_params(&self) -> Option<&sc_cli::SharedParams> {
		None
	}
}

/// Command for exporting the genesis state of the parachain
#[derive(Debug, StructOpt, Clone)]
struct ExportGenesisStateCommand {
	/// Output file name or stdout if unspecified.
	#[structopt(parse(from_os_str))]
	pub output: Option<PathBuf>,
}
