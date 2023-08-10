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

//! The Ambassador Program's origins.

#[frame_support::pallet]
pub mod pallet_origins {
	use crate::ambassador::ranks;
	use frame_support::pallet_prelude::*;
	use pallet_ranked_collective::Rank;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// The pallet configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[derive(PartialEq, Eq, Clone, MaxEncodedLen, Encode, Decode, TypeInfo, RuntimeDebug)]
	#[pallet::origin]
	pub enum Origin {
		/// Plurality voice of the [ranks::AMBASSADOR_TIER_1] members or above given via referendum.
		AmbassadorTier1,
		/// Plurality voice of the [ranks::AMBASSADOR_TIER_2] members or above given via referendum.
		AmbassadorTier2,
		/// Plurality voice of the [ranks::SENIOR_AMBASSADOR_TIER_3] members or above given via referendum.
		SeniorAmbassadorTier3,
		/// Plurality voice of the [ranks::SENIOR_AMBASSADOR_TIER_4] members or above given via referendum.
		SeniorAmbassadorTier4,
		/// Plurality voice of the [ranks::HEAD_AMBASSADOR_TIER_5] members or above given via referendum.
		HeadAmbassadorTier5,
		/// Plurality voice of the [ranks::HEAD_AMBASSADOR_TIER_6] members or above given via referendum.
		HeadAmbassadorTier6,
		/// Plurality voice of the [ranks::HEAD_AMBASSADOR_TIER_7] members or above given via referendum.
		HeadAmbassadorTier7,
		/// Plurality voice of the [ranks::MASTER_AMBASSADOR_TIER_8] members or above given via referendum.
		MasterAmbassadorTier8,
		/// Plurality voice of the [ranks::MASTER_AMBASSADOR_TIER_9] members or above given via referendum.
		MasterAmbassadorTier9,
	}

	impl Origin {
		/// Returns the rank that the origin `self` speaks for, or `None` if it doesn't speak for any.
		pub fn as_voice(&self) -> Option<Rank> {
			Some(match &self {
				Origin::AmbassadorTier1 => ranks::AMBASSADOR_TIER_1,
				Origin::AmbassadorTier2 => ranks::AMBASSADOR_TIER_2,
				Origin::SeniorAmbassadorTier3 => ranks::SENIOR_AMBASSADOR_TIER_3,
				Origin::SeniorAmbassadorTier4 => ranks::SENIOR_AMBASSADOR_TIER_4,
				Origin::HeadAmbassadorTier5 => ranks::HEAD_AMBASSADOR_TIER_5,
				Origin::HeadAmbassadorTier6 => ranks::HEAD_AMBASSADOR_TIER_6,
				Origin::HeadAmbassadorTier7 => ranks::HEAD_AMBASSADOR_TIER_7,
				Origin::MasterAmbassadorTier8 => ranks::MASTER_AMBASSADOR_TIER_8,
				Origin::MasterAmbassadorTier9 => ranks::MASTER_AMBASSADOR_TIER_9,
			})
		}
	}

	/// Implementation of the [EnsureOrigin] trait for the [Origin::HeadAmbassadorTier5] origin.
	pub struct EnsureHeadAmbassadorVoice;
	impl<O: Into<Result<Origin, O>> + From<Origin>> EnsureOrigin<O> for EnsureHeadAmbassadorVoice {
		type Success = ();
		fn try_origin(o: O) -> Result<Self::Success, O> {
			o.into().and_then(|o| match o {
				Origin::HeadAmbassadorTier5 => Ok(()),
				r => Err(O::from(r)),
			})
		}

		#[cfg(feature = "runtime-benchmarks")]
		fn try_successful_origin() -> Result<O, ()> {
			Ok(O::from(Origin::HeadAmbassadorTier5))
		}
	}

	/// Implementation of the [EnsureOrigin] trait for the plurality voice [Origin]s
	/// from a given rank `R` with the success result of the corresponding [Rank].
	pub struct EnsureAmbassadorVoiceFrom<R>(PhantomData<R>);
	impl<R: Get<Rank>, O: Into<Result<Origin, O>> + From<Origin>> EnsureOrigin<O>
		for EnsureAmbassadorVoiceFrom<R>
	{
		type Success = Rank;
		fn try_origin(o: O) -> Result<Self::Success, O> {
			o.into().and_then(|o| match Origin::as_voice(&o) {
				Some(r) if r >= R::get() => Ok(r),
				_ => Err(O::from(o)),
			})
		}

		#[cfg(feature = "runtime-benchmarks")]
		fn try_successful_origin() -> Result<O, ()> {
			ranks::MASTER_AMBASSADOR_TIER_9
				.ge(&R::get())
				.then(O::from(Origin::MasterAmbassadorTier9))
				.ok_or(Err(()))
		}
	}

	/// Implementation of the [EnsureOrigin] trait for the plurality voice [Origin]s with the
	/// success result of the corresponding [Rank].
	pub struct EnsureAmbassadorVoice;
	impl<O: Into<Result<Origin, O>> + From<Origin>> EnsureOrigin<O> for EnsureAmbassadorVoice {
		type Success = Rank;
		fn try_origin(o: O) -> Result<Self::Success, O> {
			o.into().and_then(|o| Origin::as_voice(&o).ok_or(O::from(o)))
		}

		#[cfg(feature = "runtime-benchmarks")]
		fn try_successful_origin() -> Result<O, ()> {
			Ok(O::from(Origin::MasterAmbassadorTier9))
		}
	}
}
