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

// TODO: join with primitives do we need this here or move to the primitives?

//! Types that are specific to the BridgeHubWococo runtime.

pub use bp_bridge_hub_wococo::SS58Prefix;

// We reuse everything from rococo runtime wrapper
pub const VERSION: sp_version::RuntimeVersion = relay_bridge_hub_rococo_client::runtime::VERSION;
pub type Call = relay_bridge_hub_rococo_client::runtime::Call;
pub type UncheckedExtrinsic = bp_bridge_hub_wococo::UncheckedExtrinsic<Call>;
pub type BridgeGrandpaRococoCall = relay_bridge_hub_rococo_client::runtime::BridgeGrandpaRococoCall;
pub type BridgeParachainCall = relay_bridge_hub_rococo_client::runtime::BridgeParachainCall;
pub type SystemCall = relay_bridge_hub_rococo_client::runtime::SystemCall;
