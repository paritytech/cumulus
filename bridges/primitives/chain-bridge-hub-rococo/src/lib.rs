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

//! Module with configuration which reflects BridgeHubRococo runtime setup (AccountId, Headers,
//! Hashes...)

pub use bp_polkadot_core::*;
use bp_runtime::decl_bridge_finality_runtime_apis;
use frame_support::{parameter_types, sp_runtime::MultiAddress};

pub type BridgeHubRococo = PolkadotLike;
pub type WeightToFee = frame_support::weights::IdentityFee<Balance>;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

parameter_types! {
	pub const SS58Prefix: u16 = 42;
}

decl_bridge_finality_runtime_apis!(rococo);
