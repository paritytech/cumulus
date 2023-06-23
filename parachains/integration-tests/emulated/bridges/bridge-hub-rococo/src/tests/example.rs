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

use crate::*;

#[test]
fn example() {
	Rococo::execute_with(|| {
		// Init tests variables
		let origin = <Rococo as Relay>::RuntimeOrigin::signed(RococoSender::get());
		// assert_ok!(<Rococo as RococoPallet>::XcmPallet::send(
		// 	origin,
		// 	bx!(assets_para_destination),
		// 	bx!(beneficiary),
		// 	bx!(native_assets),
		// 	fee_asset_item,
		// 	weight_limit,
		// ));
	});

	BridgeHubWococo::execute_with(|| {
		// panic!("{:?}", <BridgeHubKusama as Para>::BridgeMessages::module_owner());
	});

	AssetHubRococo::execute_with(|| {
		// panic!("{:?}", <BridgeHubKusama as Para>::BridgeMessages::module_owner());
	});

	AssetHubWococo::execute_with(|| {
		// panic!("{:?}", <BridgeHubKusama as Para>::BridgeMessages::module_owner());
	});
}
