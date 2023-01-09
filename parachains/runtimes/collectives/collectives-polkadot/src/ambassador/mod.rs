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
//! todo docs

mod pallet;
mod tracks;

use super::*;
use frame_support::traits::TryMapSuccess;
pub use pallet::pallet_ambassador;
use pallet::pallet_ambassador::{
	EnsureAmbassador, EnsureRankedAmbassador, EnsureSeniorAmbassador, Origin,
};
use sp_arithmetic::traits::CheckedSub;
use sp_runtime::{
	morph_types,
	traits::{ConstU16, TypedGet},
};

/// The Ambassador Program members ranks.
pub mod ranks {
	use pallet_ranked_collective::Rank;

	#[allow(dead_code)]
	pub const CANDIDATE: Rank = 0;
	pub const AMBASSADOR: Rank = 1;
	pub const SENIOR_AMBASSADOR: Rank = 2;
	pub const HEAD_AMBASSADOR: Rank = 3;
}

impl pallet_ambassador::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AmbassadorOrigin = EnsureAmbassador;
	type MemberOrigin = pallet_ranked_collective::EnsureMember<
		Runtime,
		AmbassadorCollectiveInstance,
		{ ranks::AMBASSADOR },
	>;
	type MaxAnnouncementsCount = ConstU32<100>;
}

parameter_types! {
	pub const AlarmInterval: BlockNumber = 1;
	pub const SubmissionDeposit: Balance = 0;
	pub const UndecidingTimeout: BlockNumber = 7 * DAYS;
}

pub type AmbassadorReferendaInstance = pallet_referenda::Instance1;

impl pallet_referenda::Config<AmbassadorReferendaInstance> for Runtime {
	type WeightInfo = (); // TODO use actual weights
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type Scheduler = Scheduler;
	type Currency = Balances;
	type SubmitOrigin = pallet_ranked_collective::EnsureMember<
		Runtime,
		AmbassadorCollectiveInstance,
		{ ranks::AMBASSADOR },
	>;
	type CancelOrigin = EitherOf<EnsureRoot<AccountId>, EnsureSeniorAmbassador>;
	type KillOrigin = EitherOf<EnsureRoot<AccountId>, EnsureSeniorAmbassador>;
	type Slash = ToParentTreasury<Runtime>;
	type Votes = pallet_ranked_collective::Votes;
	type Tally = pallet_ranked_collective::TallyOf<Runtime, AmbassadorCollectiveInstance>;
	type SubmissionDeposit = SubmissionDeposit;
	type MaxQueued = ConstU32<100>;
	type UndecidingTimeout = UndecidingTimeout;
	type AlarmInterval = AlarmInterval;
	type Tracks = tracks::TracksInfo;
	type Preimages = Preimage;
}

parameter_types! {
	pub const HeadAmbassadorRank: pallet_ranked_collective::Rank = ranks::HEAD_AMBASSADOR;
}

morph_types! {
	/// A `TryMorph` implementation to reduce a scalar by a particular amount, checking for
	/// underflow.
	pub type CheckedReduceBy<N: TypedGet>: TryMorph = |r: N::Type| -> Result<N::Type, ()> {
		r.checked_sub(&N::get()).ok_or(())
	} where N::Type: CheckedSub;
}

pub type AmbassadorCollectiveInstance = pallet_ranked_collective::Instance1;

impl pallet_ranked_collective::Config<AmbassadorCollectiveInstance> for Runtime {
	type WeightInfo = (); // TODO use actual weights
	type RuntimeEvent = RuntimeEvent;
	type PromoteOrigin = EitherOf<
		frame_system::EnsureRootWithSuccess<Self::AccountId, HeadAmbassadorRank>,
		TryMapSuccess<EnsureRankedAmbassador, CheckedReduceBy<ConstU16<1>>>,
	>;
	type DemoteOrigin = EitherOf<
		frame_system::EnsureRootWithSuccess<Self::AccountId, HeadAmbassadorRank>,
		TryMapSuccess<EnsureRankedAmbassador, CheckedReduceBy<ConstU16<1>>>,
	>;
	type Polls = AmbassadorReferenda;
	type MinRankOfClass = sp_runtime::traits::Identity;
	type VoteWeight = pallet_ranked_collective::Geometric;
}
