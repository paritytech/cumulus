// Copyright Parity Technologies (UK) Ltd.
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

//! Collectives Parachain integration tests based on xcm-emulator.

pub use frame_support::assert_ok;
pub use integration_tests_common::{
	constants::{accounts::ALICE, collectives::ED as COLLECTIVES_POLKADOT_ED},
	test_parachain_is_trusted_teleporter, AccountId, AssetHubPolkadot, AssetHubPolkadotPallet,
	AssetHubPolkadotReceiver, BridgeHubPolkadot, BridgeHubPolkadotReceiver, CollectivesPolkadot,
	CollectivesPolkadotPallet, CollectivesPolkadotSender, Polkadot, PolkadotMockNet,
};
pub use xcm::prelude::*;
pub use xcm_emulator::{assert_expected_events, bx, Parachain, TestExt};

#[cfg(test)]
mod tests;
