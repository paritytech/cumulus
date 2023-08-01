// Copyright (C) 2022 Parity Technologies (UK) Ltd.
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

//! The Ambassador Program.
//!
//! The module defines the following on-chain functionality of the Ambassador Program:
//!
//! - Managed set of program members, where every member has a [rank](ranks) (via
//!   [pallet_ranked_collective]).
//! - Referendum functionality for the program members to propose, vote on, and execute proposals on
//!   behalf of the members of a certain [rank](Origin) (via [pallet_referenda]).
//! - Managed content (charter, announcements) (via [pallet_collective_content]).

pub mod origins;
mod tracks;

use super::*;
use frame_support::traits::{EitherOf, TryMapSuccess};
pub use origins::pallet_origins as pallet_ambassador_origins;
use origins::pallet_origins::{
	EnsureAmbassador, EnsureRankedAmbassador, EnsureSeniorAmbassador, Origin,
};
use sp_arithmetic::traits::CheckedSub;
use sp_runtime::{
	morph_types,
	traits::{ConstU16, TypedGet},
};

/// The Ambassador Program's member ranks.
pub mod ranks {
	use pallet_ranked_collective::Rank;

	#[allow(dead_code)]
	pub const CANDIDATE: Rank = 0;
	pub const AMBASSADOR: Rank = 1;
	pub const SENIOR_AMBASSADOR: Rank = 2;
	pub const HEAD_AMBASSADOR: Rank = 3;
}

pub type AmbassadorContentInstance = pallet_collective_content::Instance1;

impl pallet_collective_content::Config<AmbassadorContentInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CharterOrigin = EnsureAmbassador;
	// An announcement can be submitted by a Senior Ambassador member or the [Origin::Ambassador] plurality voice
	// taken via referendum on the [tracks::constants::AMBASSADOR] track.
	type AnnouncementOrigin = EitherOfDiverse<
		pallet_ranked_collective::EnsureMember<
			Runtime,
			AmbassadorCollectiveInstance,
			{ ranks::SENIOR_AMBASSADOR },
		>,
		EnsureAmbassador,
	>;
	type WeightInfo = weights::pallet_collective_content::WeightInfo<Runtime>;
}

impl pallet_ambassador_origins::Config for Runtime {}

parameter_types! {
	pub const AlarmInterval: BlockNumber = 1;
	pub const SubmissionDeposit: Balance = 0;
	pub const UndecidingTimeout: BlockNumber = 7 * DAYS;
	// The Ambassador Referenda pallet account, used as a temporarily place to deposit a slashed imbalance before teleport to the treasury.
	pub AmbassadorPalletAccount: AccountId = constants::account::AMBASSADOR_REFERENDA_PALLET_ID.into_account_truncating();
}

pub type AmbassadorReferendaInstance = pallet_referenda::Instance2;

impl pallet_referenda::Config<AmbassadorReferendaInstance> for Runtime {
	type WeightInfo = (); // TODO actual weights
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type Scheduler = Scheduler;
	type Currency = Balances;
	// A proposal can be submitted by a member of the Ambassador Program of [ranks::AMBASSADOR] rank or higher.
	type SubmitOrigin = pallet_ranked_collective::EnsureMember<
		Runtime,
		AmbassadorCollectiveInstance,
		{ ranks::AMBASSADOR },
	>;
	type CancelOrigin = EitherOf<EnsureRoot<AccountId>, EnsureSeniorAmbassador>;
	type KillOrigin = EitherOf<EnsureRoot<AccountId>, EnsureSeniorAmbassador>;
	type Slash = ToParentTreasury<PolkadotTreasuryAccount, AmbassadorPalletAccount, Runtime>;
	type Votes = pallet_ranked_collective::Votes;
	type Tally = pallet_ranked_collective::TallyOf<Runtime, AmbassadorCollectiveInstance>;
	type SubmissionDeposit = SubmissionDeposit;
	type MaxQueued = ConstU32<20>;
	type UndecidingTimeout = UndecidingTimeout;
	type AlarmInterval = AlarmInterval;
	type Tracks = tracks::TracksInfo;
	type Preimages = Preimage;
}

parameter_types! {
	// The benches requires max member rank to be at least 10. This rank is available only for root.
	pub const MaxAmbassadorRank: pallet_ranked_collective::Rank = ranks::HEAD_AMBASSADOR + 10;
}

morph_types! {
	/// A `TryMorph` implementation to reduce a scalar by a particular amount, checking for
	/// underflow.
	pub type CheckedReduceBy<N: TypedGet>: TryMorph = |r: N::Type| -> Result<N::Type, ()> {
		r.checked_sub(&N::get()).ok_or(())
	} where N::Type: CheckedSub;
}

pub type AmbassadorCollectiveInstance = pallet_ranked_collective::Instance2;

impl pallet_ranked_collective::Config<AmbassadorCollectiveInstance> for Runtime {
	type WeightInfo = (); // TODO actual weights
	type RuntimeEvent = RuntimeEvent;
	// Promotion is by any of:
	// - Root can promote arbitrarily.
	// - a vote by the rank above the new rank.
	type PromoteOrigin = EitherOf<
		frame_system::EnsureRootWithSuccess<Self::AccountId, MaxAmbassadorRank>,
		TryMapSuccess<EnsureRankedAmbassador, CheckedReduceBy<ConstU16<1>>>,
	>;
	// Demotion is by any of:
	// - Root can demote arbitrarily.
	// - a vote by the rank above the current rank.
	type DemoteOrigin = EitherOf<
		frame_system::EnsureRootWithSuccess<Self::AccountId, MaxAmbassadorRank>,
		TryMapSuccess<EnsureRankedAmbassador, CheckedReduceBy<ConstU16<1>>>,
	>;
	type Polls = AmbassadorReferenda;
	type MinRankOfClass = sp_runtime::traits::Identity;
	type VoteWeight = pallet_ranked_collective::Geometric;
}
