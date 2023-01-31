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
	use super::super::ranks;
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
		/// Plurality voice of the [ranks::CANDIDATE] members or above given via referendum.
		Candidate,
		/// Plurality voice of the [ranks::AMBASSADOR] members or above given via referendum.
		Ambassador,
		/// Plurality voice of the [ranks::SENIOR_AMBASSADOR] members or above given via referendum.
		SeniorAmbassador,
		/// Plurality voice of the [ranks::HEAD_AMBASSADOR] members or above given via referendum.
		HeadAmbassador,
	}

	/// Implementation of the [EnsureOrigin] trait for the [Origin::Ambassador] origin.
	pub struct EnsureAmbassador;
	impl<O: Into<Result<Origin, O>> + From<Origin>> EnsureOrigin<O> for EnsureAmbassador {
		type Success = ();
		fn try_origin(o: O) -> Result<Self::Success, O> {
			o.into().and_then(|o| match o {
				Origin::Ambassador => Ok(()),
				r => Err(O::from(r)),
			})
		}

		#[cfg(feature = "runtime-benchmarks")]
		fn try_successful_origin() -> Result<O, ()> {
			Ok(O::from(Origin::Ambassador))
		}
	}

	/// Implementation of the [EnsureOrigin] trait for the [Origin::SeniorAmbassador]
	/// and [Origin::HeadAmbassador] origins.
	pub struct EnsureSeniorAmbassador;
	impl<O: Into<Result<Origin, O>> + From<Origin>> EnsureOrigin<O> for EnsureSeniorAmbassador {
		type Success = ();
		fn try_origin(o: O) -> Result<Self::Success, O> {
			o.into().and_then(|o| match o {
				Origin::SeniorAmbassador => Ok(()),
				Origin::HeadAmbassador => Ok(()),
				r => Err(O::from(r)),
			})
		}

		#[cfg(feature = "runtime-benchmarks")]
		fn try_successful_origin() -> Result<O, ()> {
			Ok(O::from(Origin::HeadAmbassador))
		}
	}

	/// Implementation of the [EnsureOrigin] trait for the plurality voice [Origin]s with the
	/// success result of the corresponding [Rank]. Not implemented for [Origin::Candidate].
	pub struct EnsureRankedAmbassador;

	impl<O: Into<Result<Origin, O>> + From<Origin>> EnsureOrigin<O> for EnsureRankedAmbassador {
		type Success = Rank;
		fn try_origin(o: O) -> Result<Self::Success, O> {
			o.into().and_then(|o| match o {
				Origin::Ambassador => Ok(ranks::AMBASSADOR),
				Origin::SeniorAmbassador => Ok(ranks::SENIOR_AMBASSADOR),
				Origin::HeadAmbassador => Ok(ranks::HEAD_AMBASSADOR),
				r => Err(O::from(r)),
			})
		}

		#[cfg(feature = "runtime-benchmarks")]
		fn try_successful_origin() -> Result<O, ()> {
			Ok(O::from(Origin::HeadAmbassador))
		}
	}
}
