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

//! TODO: this will go away from Westmint for better reusability
//!
//! All conversions stuff to support bridging scenario (asset transfer, asset creation, ...)

use codec::{Decode, Encode};
use frame_support::{sp_runtime::RuntimeDebug, traits::Contains};
use scale_info::TypeInfo;
use sp_core::{Get, TypeId};
use sp_runtime::traits::AccountIdConversion;
use xcm::latest::{prelude::*, Junction, MultiLocation};
use xcm_executor::traits::Convert;

/// [`MultiLocation`] is supposed to be local bridge-hub and [`Junction]` is foreign [`GlobalConsensus`]
pub type BridgedUniversalAliases = sp_std::vec::Vec<(MultiLocation, Junction)>;

/// Checks if [`MultiLocation`] is from different global consensus and if it is allowed
pub struct IsTrustedGlobalConsensus<UniversalAliases>(
	sp_std::marker::PhantomData<UniversalAliases>,
);
impl<UniversalAliases: Get<BridgedUniversalAliases>> Contains<MultiLocation>
	for IsTrustedGlobalConsensus<UniversalAliases>
{
	fn contains(location: &MultiLocation) -> bool {
		match location {
			MultiLocation { parents, interior } if *parents > 1 =>
				match interior.global_consensus() {
					Ok(location_global_consensus) =>
						Self::contains(&GlobalConsensus(location_global_consensus)),
					Err(_) => false,
				},
			_ => false,
		}
	}
}
impl<UniversalAliases: Get<BridgedUniversalAliases>> Contains<Junction>
	for IsTrustedGlobalConsensus<UniversalAliases>
{
	fn contains(junction: &Junction) -> bool {
		UniversalAliases::get()
			.iter()
			.find(
				|(_, known_global_consensus)| {
					if junction.eq(known_global_consensus) {
						true
					} else {
						false
					}
				},
			)
			.is_some()
	}
}

/// Converts global consensus to local account
pub struct GlobalConsensusConvertsVia<
	TrustedGlobalConsensus,
	GlobalConsensusAccountIdConversion,
	AccountId,
>(
	sp_std::marker::PhantomData<(
		TrustedGlobalConsensus,
		GlobalConsensusAccountIdConversion,
		AccountId,
	)>,
);
impl<
		TrustedGlobalConsensus: Contains<Junction>,
		GlobalConsensusAccountIdConversion: From<(u8, NetworkId)> + Into<(u8, NetworkId)> + AccountIdConversion<AccountId>,
		AccountId: Clone,
	> Convert<MultiLocation, AccountId>
	for GlobalConsensusConvertsVia<
		TrustedGlobalConsensus,
		GlobalConsensusAccountIdConversion,
		AccountId,
	>
{
	fn convert_ref(location: impl sp_std::borrow::Borrow<MultiLocation>) -> Result<AccountId, ()> {
		match location.borrow() {
			MultiLocation { parents, interior: X1(GlobalConsensus(network_id)) } =>
				Ok(GlobalConsensusAccountIdConversion::from((*parents, *network_id))
					.into_account_truncating()),
			_ => Err(()),
		}
	}

	fn reverse_ref(who: impl sp_std::borrow::Borrow<AccountId>) -> Result<MultiLocation, ()> {
		if let Some(conv) = GlobalConsensusAccountIdConversion::try_from_account(who.borrow()) {
			let (parents, network_id): (u8, NetworkId) = conv.into();
			Ok(MultiLocation { parents, interior: X1(GlobalConsensus(network_id)) })
		} else {
			Err(())
		}
	}
}

#[derive(Clone, Copy, Encode, Decode, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, TypeInfo)]
pub struct GlobalConsensusAsAccountId(pub u8, pub NetworkId);

impl TypeId for GlobalConsensusAsAccountId {
	const TYPE_ID: [u8; 4] = *b"glcs";
}

impl From<(u8, NetworkId)> for GlobalConsensusAsAccountId {
	fn from((parents, network_id): (u8, NetworkId)) -> Self {
		GlobalConsensusAsAccountId(parents, network_id)
	}
}

impl From<GlobalConsensusAsAccountId> for (u8, NetworkId) {
	fn from(value: GlobalConsensusAsAccountId) -> Self {
		(value.0, value.1)
	}
}
