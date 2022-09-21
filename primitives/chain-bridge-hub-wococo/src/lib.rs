// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of Parity Bridges Common.

// Parity Bridges Common is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Bridges Common is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Bridges Common.  If not, see <http://www.gnu.org/licenses/>.

//! Module with configuration which reflects BridgeHubWococo runtime setup
//! (AccountId, Headers, Hashes...)
//!
//! but actually this is just reexported BridgeHubRococo stuff, because they are supposed to be
//! identical, at least uses the same parachain runtime

// Re-export only what is really needed
pub use bp_bridge_hub_rococo::{
	account_info_storage_key, AccountId, AccountPublic, Address, Balance, BlockNumber, Hash,
	Hashing, Header, Nonce, SS58Prefix, Signature, SignedBlock, SignedExtensions,
	UncheckedExtrinsic, WeightToFee, EXTRA_STORAGE_PROOF_SIZE,
};
use bp_runtime::decl_bridge_finality_runtime_apis;

pub type BridgeHubWococo = bp_bridge_hub_rococo::BridgeHubRococo;

decl_bridge_finality_runtime_apis!(wococo);
