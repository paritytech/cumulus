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
use cumulus_primitives_core::Junction::GeneralIndex;
use frame_system::EnsureNever;
pub use origins::{
	pallet_origins as pallet_fellowship_origins, Architects, Candidates, Fellows, Masters, Members,
};
use xcm_builder::{LocatableAssetId, PayOverXcm};

use self::origins::EnsureFellowship;
use crate::{
	constants, impls::ToParentTreasury, weights, AccountId, Balance, Balances, FellowshipReferenda,
	GovernanceLocation, PolkadotTreasuryAccount, Preimage, Runtime, RuntimeCall, RuntimeEvent,
	Scheduler, DAYS,
};
use frame_support::{
	parameter_types,
	traits::{EitherOf, EitherOfDiverse, MapSuccess, TryMapSuccess},
};
use pallet_xcm::{EnsureXcm, IsVoiceOfBody};
use polkadot_runtime_constants::{currency::UNITS, time::HOURS, xcm::body::FELLOWSHIP_ADMIN_INDEX};
use sp_core::{ConstU128, ConstU32};
use sp_runtime::traits::{
	AccountIdConversion, CheckedReduceBy, ConstU16, ConvertToValue, MorphWithUpperLimit, Replace,
};
use xcm::latest::BodyId;

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
	// Fellows can submit proposals.
	type SubmitOrigin =
		pallet_ranked_collective::EnsureMember<Runtime, FellowshipCollectiveInstance, 3>;
	type CancelOrigin = Architects;
	type KillOrigin = Masters;
	type Slash = ToParentTreasury<PolkadotTreasuryAccount, ReferendaPalletAccount, Runtime>;
	type Votes = pallet_ranked_collective::Votes;
	type Tally = pallet_ranked_collective::TallyOf<Runtime, FellowshipCollectiveInstance>;
	type SubmissionDeposit = ConstU128<0>;
	type MaxQueued = ConstU32<100>;
	type UndecidingTimeout = ConstU32<{ 7 * DAYS }>;
	type AlarmInterval = ConstU32<1>;
	type Tracks = tracks::TracksInfo;
	type Preimages = Preimage;
}

pub type FellowshipCollectiveInstance = pallet_ranked_collective::Instance1;

impl pallet_ranked_collective::Config<FellowshipCollectiveInstance> for Runtime {
	type WeightInfo = weights::pallet_ranked_collective::WeightInfo<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	// Promotions and the induction of new members are serviced by `FellowshipCore` pallet instance.
	type PromoteOrigin = EnsureNever<pallet_ranked_collective::Rank>;
	// Demotion is by any of:
	// - Root can demote arbitrarily.
	// - the FellowshipAdmin origin (i.e. token holder referendum);
	type DemoteOrigin = EitherOf<
		frame_system::EnsureRootWithSuccess<Self::AccountId, ConstU16<{ ranks::DAN_9 }>>,
		MapSuccess<
			EnsureXcm<IsVoiceOfBody<GovernanceLocation, FellowshipAdminBodyId>>,
			Replace<ConstU16<{ ranks::DAN_9 }>>,
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
	// - the FellowshipAdmin origin (i.e. token holder referendum);
	// - a vote among all Fellows.
	type ParamsOrigin = EitherOfDiverse<
		EnsureXcm<IsVoiceOfBody<GovernanceLocation, FellowshipAdminBodyId>>,
		Fellows,
	>;
	// Induction (creating a candidate) is by any of:
	// - Root;
	// - the FellowshipAdmin origin (i.e. token holder referendum);
	// - a single Fellow;
	// - a vote among all Members.
	type InductOrigin = EitherOfDiverse<
		EnsureXcm<IsVoiceOfBody<GovernanceLocation, FellowshipAdminBodyId>>,
		EitherOfDiverse<
			pallet_ranked_collective::EnsureMember<Runtime, FellowshipCollectiveInstance, 2>,
			Members,
		>,
	>;
	// Approval (rank-retention) of a Member's current rank is by any of:
	// - Root;
	// - the FellowshipAdmin origin (i.e. token holder referendum);
	// - a vote by the rank one above the current rank.
	type ApproveOrigin = EitherOf<
		MapSuccess<
			EnsureXcm<IsVoiceOfBody<GovernanceLocation, FellowshipAdminBodyId>>,
			Replace<ConstU16<{ ranks::DAN_9 }>>,
		>,
		TryMapSuccess<EnsureFellowship, CheckedReduceBy<ConstU16<1>>>,
	>;
	// Promotion is by any of:
	// - Root can promote arbitrarily.
	// - the FellowshipAdmin origin (i.e. token holder referendum);
	// - a vote by the rank two above the new rank for all promotions up to the Master rank.
	type PromoteOrigin = EitherOf<
		MapSuccess<
			EnsureXcm<IsVoiceOfBody<GovernanceLocation, FellowshipAdminBodyId>>,
			Replace<ConstU16<{ ranks::DAN_9 }>>,
		>,
		TryMapSuccess<
			EnsureFellowship,
			MorphWithUpperLimit<ConstU16<{ ranks::DAN_6 }>, CheckedReduceBy<ConstU16<2>>>,
		>,
	>;
	type EvidenceSize = ConstU32<65536>;
}

pub type FellowshipSalaryInstance = pallet_salary::Instance1;

use xcm::prelude::*;

parameter_types! {
	pub Statemint: MultiLocation = (Parent, Parachain(1000)).into();
	pub StatemintUsdtId: AssetId = GeneralIndex(1984).into();
	pub UsdtAsset: LocatableAssetId = LocatableAssetId {
		location: Statemint::get(),
		asset_id: StatemintUsdtId::get(),
	};
	// The interior location on Statemint for the paying account. This is the Fellowship Salary
	// pallet instance (which sits at index 64). This sovereign account will need funding.
	pub Interior: InteriorMultiLocation = PalletInstance(64).into();
}

// TODO https://github.com/paritytech/polkadot/pull/7321: Move into location_conversion.rs
/// Conversion implementation which converts from a `[u8; 32]`-based `AccountId` into a
/// `MultiLocation` consisting solely of a `AccountId32` junction with a fixed value for its
/// network (provided by `Network`) and the `AccountId`'s `[u8; 32]` datum for the `id`.
pub struct AliasesIntoAccountId32<Network, AccountId>(
	sp_std::marker::PhantomData<(Network, AccountId)>,
);
impl<
		'a,
		Network: sp_runtime::traits::Get<Option<NetworkId>>,
		AccountId: Clone + Into<[u8; 32]> + Clone,
	> sp_runtime::traits::Convert<&'a AccountId, MultiLocation>
	for AliasesIntoAccountId32<Network, AccountId>
{
	fn convert(who: &AccountId) -> MultiLocation {
		AccountId32 { network: Network::get(), id: who.clone().into() }.into()
	}
}

impl pallet_salary::Config<FellowshipSalaryInstance> for Runtime {
	type WeightInfo = weights::pallet_salary::WeightInfo<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	type Paymaster = PayOverXcm<
		Interior,
		crate::xcm_config::XcmRouter,
		crate::PolkadotXcm,
		ConstU32<{ 6 * HOURS }>,
		AccountId,
		(),
		ConvertToValue<UsdtAsset>,
		AliasesIntoAccountId32<(), AccountId>,
	>;
	type Members = pallet_ranked_collective::Pallet<Runtime, FellowshipCollectiveInstance>;

	#[cfg(not(feature = "runtime-benchmarks"))]
	type Salary = pallet_core_fellowship::Pallet<Runtime, FellowshipCoreInstance>;
	#[cfg(feature = "runtime-benchmarks")]
	type Salary = frame_support::traits::tokens::ConvertRank<
		crate::impls::benchmarks::RankToSalary<Balances>,
	>;
	// 15 days to register for a salary payment.
	type RegistrationPeriod = ConstU32<{ 15 * DAYS }>;
	// 15 days to claim the salary payment.
	type PayoutPeriod = ConstU32<{ 15 * DAYS }>;
	// Total monthly salary budget.
	type Budget = ConstU128<{ 100000 * UNITS }>;
}
