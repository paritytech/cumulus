// Copyright (C) 2023 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// TODO:check-parameter - is it worth to move it to the [`xcm-builder -> location_conversion.rs`]?

use codec::Encode;
use frame_support::sp_io::hashing::blake2_256;
use sp_std::{borrow::Borrow, marker::PhantomData};
use xcm::prelude::*;
use xcm_executor::traits::Convert;

/// Tries to convert **foreign** global consensus parachain to accountId.
///
/// **foreign** means `parents > 1`
///
/// (E.g.: can be used for sovereign account conversion)
pub struct GlobalConsensusParachainConvert<AccountId>(PhantomData<AccountId>);

impl<AccountId: From<[u8; 32]> + Clone> Convert<MultiLocation, AccountId>
	for GlobalConsensusParachainConvert<AccountId>
{
	fn convert_ref(location: impl Borrow<MultiLocation>) -> Result<AccountId, ()> {
		match location.borrow() {
			MultiLocation {
				parents,
				interior: X2(GlobalConsensus(network), Parachain(para_id)),
			} if parents > &1_u8 =>
				Ok(AccountId::from(GlobalConsensusParachainConvert::<AccountId>::from_params(
					network, para_id, *parents,
				))),
			_ => Err(()),
		}
	}

	fn reverse_ref(_: impl Borrow<AccountId>) -> Result<MultiLocation, ()> {
		// if this will be needed, we could implement some kind of guessing, if we have configuration for supported foreign networkId+paraId
		Err(())
	}
}

impl<AccountId> GlobalConsensusParachainConvert<AccountId> {
	fn from_params(network: &NetworkId, para_id: &u32, parents: u8) -> [u8; 32] {
		(network, para_id, parents).using_encoded(blake2_256)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn global_consensus_parachain_convert_works() {
		let test_data = vec![
			(
				MultiLocation::new(0, X2(GlobalConsensus(ByGenesis([0; 32])), Parachain(1000))),
				false,
			),
			(
				MultiLocation::new(1, X2(GlobalConsensus(ByGenesis([0; 32])), Parachain(1000))),
				false,
			),
			(
				MultiLocation::new(
					2,
					X3(
						GlobalConsensus(ByGenesis([0; 32])),
						Parachain(1000),
						AccountId32 { network: None, id: [1; 32].into() },
					),
				),
				false,
			),
			(MultiLocation::new(2, X1(GlobalConsensus(ByGenesis([0; 32])))), false),
			(MultiLocation::new(2, X2(GlobalConsensus(ByGenesis([0; 32])), Parachain(1000))), true),
			(MultiLocation::new(3, X2(GlobalConsensus(ByGenesis([0; 32])), Parachain(1000))), true),
			(MultiLocation::new(4, X2(GlobalConsensus(ByGenesis([0; 32])), Parachain(1000))), true),
			(
				MultiLocation::new(10, X2(GlobalConsensus(ByGenesis([0; 32])), Parachain(1000))),
				true,
			),
		];

		for (location, expected_result) in test_data {
			let result = GlobalConsensusParachainConvert::<[u8; 32]>::convert_ref(&location);
			match result {
				Ok(account) => {
					assert_eq!(
						true, expected_result,
						"expected_result: {}, but conversion passed: {:?}, location: {:?}",
						expected_result, account, location
					);
					match &location {
						MultiLocation { parents, interior: X2(GlobalConsensus(network), Parachain(para_id)) } =>
							assert_eq!(
								account,
								GlobalConsensusParachainConvert::<[u8; 32]>::from_params(network, para_id, *parents),
								"expected_result: {}, but conversion passed: {:?}, location: {:?}", expected_result, account, location
							),
						_ => assert_eq!(
							true,
							expected_result,
							"expected_result: {}, conversion passed: {:?}, but MultiLocation does not match expected pattern, location: {:?}", expected_result, account, location
						)
					}
				},
				Err(_) => {
					assert_eq!(
						false, expected_result,
						"expected_result: {} - but conversion failed, location: {:?}",
						expected_result, location
					);
				},
			}
		}

		// all success
		let res_2_1000 = GlobalConsensusParachainConvert::<[u8; 32]>::convert_ref(
			MultiLocation::new(2, X2(GlobalConsensus(ByGenesis([0; 32])), Parachain(1000))),
		)
		.expect("conversion is ok");
		let res_2_1001 = GlobalConsensusParachainConvert::<[u8; 32]>::convert_ref(
			MultiLocation::new(2, X2(GlobalConsensus(ByGenesis([0; 32])), Parachain(1001))),
		)
		.expect("conversion is ok");
		let res_3_1000 = GlobalConsensusParachainConvert::<[u8; 32]>::convert_ref(
			MultiLocation::new(3, X2(GlobalConsensus(ByGenesis([0; 32])), Parachain(1000))),
		)
		.expect("conversion is ok");
		let res_3_1001 = GlobalConsensusParachainConvert::<[u8; 32]>::convert_ref(
			MultiLocation::new(3, X2(GlobalConsensus(ByGenesis([0; 32])), Parachain(1001))),
		)
		.expect("conversion is ok");
		assert_ne!(res_2_1000, res_2_1001);
		assert_ne!(res_2_1000, res_3_1000);
		assert_ne!(res_2_1000, res_3_1001);
		assert_ne!(res_2_1001, res_3_1000);
		assert_ne!(res_2_1001, res_3_1001);
		assert_ne!(res_3_1000, res_3_1001);
	}
}
