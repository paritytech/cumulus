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

//! The Polkadot OpenGov.

use crate::{
	impls::RemoteFungible, xcm_config::DotLocation, AccountId, Balance, Balances, BlockNumber,
	Preimage, Referenda, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin, Scheduler, PolkadotXcm,
};
use frame_support::parameter_types;
use frame_system::EnsureRoot;
use polkadot_runtime_constants::DOLLARS;
use sp_core::ConstU32;
use xcm::latest::AssetId;

parameter_types! {
	pub const VoteLockingPeriod: BlockNumber = 10;
	pub const DotAssetId: AssetId = AssetId::Concrete(DotLocation::get());
}

impl pallet_conviction_voting::Config for Runtime {
	type WeightInfo = (); // todo weights
	type RuntimeEvent = RuntimeEvent;
	type Currency = RemoteFungible<Runtime, Balances, PolkadotXcm, DotAssetId>;
	type VoteLockingPeriod = VoteLockingPeriod;
	type MaxVotes = ConstU32<512>;
	type MaxTurnout = (); // frame_support::traits::tokens::currency::ActiveIssuanceOf<Balances, Self::AccountId>;
	type Polls = Referenda;
}

parameter_types! {
	pub const AlarmInterval: BlockNumber = 1;
	pub const SubmissionDeposit: Balance = 1;
	pub const UndecidingTimeout: BlockNumber = 14;
}

impl pallet_referenda::Config for Runtime {
	type WeightInfo = (); // todo
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type Scheduler = Scheduler;
	type Currency = Balances;
	type SubmitOrigin = frame_system::EnsureSigned<AccountId>;
	type CancelOrigin = EnsureRoot<AccountId>;
	type KillOrigin = EnsureRoot<AccountId>;
	type Slash = ();
	type Votes = pallet_conviction_voting::VotesOf<Runtime>;
	type Tally = pallet_conviction_voting::TallyOf<Runtime>;
	type SubmissionDeposit = SubmissionDeposit;
	type MaxQueued = ConstU32<100>;
	type UndecidingTimeout = UndecidingTimeout;
	type AlarmInterval = AlarmInterval;
	type Tracks = TracksInfo;
	type Preimages = Preimage;
}

const fn percent(x: i32) -> sp_arithmetic::FixedI64 {
	sp_arithmetic::FixedI64::from_rational(x as u128, 100)
}

use pallet_referenda::Curve;
const APP_ROOT: Curve = Curve::make_linear(1, 1, percent(0), percent(0));
const SUP_ROOT: Curve = Curve::make_linear(1, 1, percent(0), percent(0));

const TRACKS_DATA: [(u16, pallet_referenda::TrackInfo<Balance, BlockNumber>); 1] = [(
	0,
	pallet_referenda::TrackInfo {
		name: "root",
		max_deciding: 10,
		decision_deposit: 10 * DOLLARS,
		prepare_period: 2,
		decision_period: 1,
		confirm_period: 1,
		min_enactment_period: 1,
		min_approval: APP_ROOT,
		min_support: SUP_ROOT,
	},
)];

pub struct TracksInfo;
impl pallet_referenda::TracksInfo<Balance, BlockNumber> for TracksInfo {
	type Id = u16;
	type RuntimeOrigin = <RuntimeOrigin as frame_support::traits::OriginTrait>::PalletsOrigin;
	fn tracks() -> &'static [(Self::Id, pallet_referenda::TrackInfo<Balance, BlockNumber>)] {
		&TRACKS_DATA[..]
	}
	fn track_for(id: &Self::RuntimeOrigin) -> Result<Self::Id, ()> {
		if let Ok(system_origin) = frame_system::RawOrigin::try_from(id.clone()) {
			match system_origin {
				frame_system::RawOrigin::Root => Ok(0),
				_ => Err(()),
			}
		} else {
			Err(())
		}
	}
}
pallet_referenda::impl_tracksinfo_get!(TracksInfo, Balance, BlockNumber);
