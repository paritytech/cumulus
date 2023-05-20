// Copyright 2023 Parity Technologies (UK) Ltd.
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

//! The Polkadot Technical Fellowship.

pub(crate) mod migration;
mod origins;
mod tracks;
use frame_system::EnsureNever;
pub use origins::{
	pallet_origins as pallet_fellowship_origins, Fellows, FellowshipCandidates, FellowshipExperts,
	FellowshipMasters,
};

use crate::{
	constants, impls::ToParentTreasury, weights, AccountId, Balance, Balances, BlockNumber,
	FellowshipReferenda, GovernanceLocation, PolkadotTreasuryAccount, Preimage, Runtime,
	RuntimeCall, RuntimeEvent, Scheduler, DAYS,
};
use frame_support::{
	parameter_types,
	traits::{tokens::PayFromAccount, EitherOf, MapSuccess, TryMapSuccess},
};
use pallet_xcm::{EnsureXcm, IsVoiceOfBody};
use polkadot_runtime_constants::{currency::UNITS, xcm::body::FELLOWSHIP_ADMIN_INDEX};
use sp_arithmetic::traits::CheckedSub;
use sp_core::ConstU32;
use sp_runtime::{
	morph_types,
	traits::{AccountIdConversion, ConstU16, Replace, TypedGet},
};
use xcm::latest::BodyId;

use self::origins::EnsureFellowship;

/// The Fellowship members' ranks.
pub mod ranks {
	use pallet_ranked_collective::Rank;

	pub const CANDIDATES: Rank = 0;
	pub const DAN_1: Rank = 1;
	pub const DAN_2: Rank = 2;
	pub const DAN_3: Rank = 3; // aka Fellows.
	pub const DAN_4: Rank = 4;
	pub const DAN_5: Rank = 5; // aka Experts.
	pub const DAN_6: Rank = 6;
	pub const DAN_7: Rank = 7; // aka Masters.
	pub const DAN_8: Rank = 8;
	pub const DAN_9: Rank = 9;
}

parameter_types! {
	pub const AlarmInterval: BlockNumber = 1;
	pub const SubmissionDeposit: Balance = 0;
	pub const UndecidingTimeout: BlockNumber = 7 * DAYS;
	// Referenda pallet account, used to temporarily deposit slashed imbalance before teleporting.
	pub ReferendaPalletAccount: AccountId = constants::account::REFERENDA_PALLET_ID.into_account_truncating();
	pub const FellowshipAdminBodyId: BodyId = BodyId::Index(FELLOWSHIP_ADMIN_INDEX);
}

impl pallet_fellowship_origins::Config for Runtime {}

pub type FellowshipReferendaInstance = pallet_referenda::Instance1;

impl pallet_referenda::Config<FellowshipReferendaInstance> for Runtime {
	type WeightInfo = weights::pallet_referenda::WeightInfo<Runtime>;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type Scheduler = Scheduler;
	type Currency = Balances;
	type SubmitOrigin =
		pallet_ranked_collective::EnsureMember<Runtime, FellowshipCollectiveInstance, 1>;
	type CancelOrigin = FellowshipExperts;
	type KillOrigin = FellowshipMasters;
	type Slash = ToParentTreasury<PolkadotTreasuryAccount, ReferendaPalletAccount, Runtime>;
	type Votes = pallet_ranked_collective::Votes;
	type Tally = pallet_ranked_collective::TallyOf<Runtime, FellowshipCollectiveInstance>;
	type SubmissionDeposit = SubmissionDeposit;
	type MaxQueued = ConstU32<100>;
	type UndecidingTimeout = UndecidingTimeout;
	type AlarmInterval = AlarmInterval;
	type Tracks = tracks::TracksInfo;
	type Preimages = Preimage;
}

pub type FellowshipCollectiveInstance = pallet_ranked_collective::Instance1;

morph_types! {
	/// A `TryMorph` implementation to reduce a scalar by a particular amount, checking for
	/// underflow.
	pub type CheckedReduceBy<N: TypedGet>: TryMorph = |r: N::Type| -> Result<N::Type, ()> {
		r.checked_sub(&N::get()).ok_or(())
	} where N::Type: CheckedSub;
}

impl pallet_ranked_collective::Config<FellowshipCollectiveInstance> for Runtime {
	type WeightInfo = weights::pallet_ranked_collective::WeightInfo<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	// Promotions and the induction of new members are serviced by `FellowshipCore` pallet instance.
	type PromoteOrigin = EnsureNever<pallet_ranked_collective::Rank>;
	// Demotion is by any of:
	// - Root can demote arbitrarily.
	// - the FellowshipAdmin origin (i.e. token holder referendum);
	// - a vote by the rank two above the current rank.
	type DemoteOrigin = EitherOf<
		frame_system::EnsureRootWithSuccess<Self::AccountId, ConstU16<65535>>,
		EitherOf<
			MapSuccess<
				EnsureXcm<IsVoiceOfBody<GovernanceLocation, FellowshipAdminBodyId>>,
				Replace<ConstU16<9>>,
			>,
			TryMapSuccess<EnsureFellowship, CheckedReduceBy<ConstU16<2>>>,
		>,
	>;
	type Polls = FellowshipReferenda;
	type MinRankOfClass = sp_runtime::traits::Identity;
	type VoteWeight = pallet_ranked_collective::Geometric;
}

pub type FellowshipCoreInstance = pallet_core_fellowship::Instance1;

impl pallet_core_fellowship::Config<FellowshipCoreInstance> for Runtime {
	type WeightInfo = weights::pallet_core_fellowship::WeightInfo<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	type Members = pallet_ranked_collective::Pallet<Runtime, FellowshipCollectiveInstance>;
	type Balance = Balance;
	// Parameters are set by any of:
	// - Root;
	// - a vote of the rank 3 or above.
	type ParamsOrigin = Fellows;
	// Induction is by any of:
	// - Root;
	// - a member of the rank 1 or above.
	type InductOrigin = EnsureFellowship;
	// Approval of a member's current rank is by any of:
	// - Root;
	// - the FellowshipAdmin origin (i.e. token holder referendum);
	// - a vote by the rank above or equal to the current rank.
	type ApproveOrigin = EitherOf<
		MapSuccess<
			EnsureXcm<IsVoiceOfBody<GovernanceLocation, FellowshipAdminBodyId>>,
			Replace<ConstU16<9>>,
		>,
		EnsureFellowship,
	>;
	// Promotion is by any of:
	// - Root can promote arbitrarily.
	// - the FellowshipAdmin origin (i.e. token holder referendum);
	// - a vote by the rank *above* the new rank.
	type PromoteOrigin = EitherOf<
		frame_system::EnsureRootWithSuccess<Self::AccountId, ConstU16<65535>>,
		EitherOf<
			MapSuccess<
				EnsureXcm<IsVoiceOfBody<GovernanceLocation, FellowshipAdminBodyId>>,
				Replace<ConstU16<9>>,
			>,
			TryMapSuccess<EnsureFellowship, CheckedReduceBy<ConstU16<1>>>,
		>,
	>;
	type EvidenceSize = ConstU32<1024>;
}

parameter_types! {
	pub const RegistrationPeriod: BlockNumber = 75 * DAYS;
	pub const PayoutPeriod: BlockNumber = 15 * DAYS;
	/// A total budget of a single payout cycle.
	/// The cycle duration is a sum of `RegistrationPeriod` and `PayoutPeriod`.
	pub const Budget: Balance = 100000 * UNITS;
}

pub type FellowshipSalaryInstance = pallet_salary::Instance1;

impl pallet_salary::Config<FellowshipSalaryInstance> for Runtime {
	type WeightInfo = weights::pallet_salary::WeightInfo<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	type Paymaster = PayFromAccount<Balances, PolkadotTreasuryAccount>;
	type Members = pallet_ranked_collective::Pallet<Runtime, FellowshipCollectiveInstance>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type Salary = pallet_core_fellowship::Pallet<Runtime, FellowshipCoreInstance>;
	#[cfg(feature = "runtime-benchmarks")]
	type Salary = frame_support::traits::tokens::ConvertRank<
		crate::impls::benchmarks::RankToSalary<Balances>,
	>;
	type RegistrationPeriod = RegistrationPeriod;
	type PayoutPeriod = PayoutPeriod;
	type Budget = Budget;
}
