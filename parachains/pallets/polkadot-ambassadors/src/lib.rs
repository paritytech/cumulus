// This file is part of Substrate.

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

//! # Polkadot Ambassador Program Pallet
//!
//! A simple extension of the Collectives pallet specific to the Polkadot Ambassador Program.
//!
//! ## Related Modules
//!
//! * [`Alliance`](https://github.com/paritytech/substrate/tree/master/frame/alliance)

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

mod types;

use frame_support::traits::InitializeMembers;

pub use pallet::*;
pub use types::*;

/// The log target of this pallet.
pub const LOG_TARGET: &str = "runtime::ambassador";

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Origin required for administering the program.
		type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Origin required for making announcements.
		type AnnouncementOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The maximum number of accounts that can be in the program.
		type MaxMembersCount: Get<u32>;

		/// What to do with initial members of the program.
		type InitializeMembers: InitializeMembers<Self::AccountId>;

		/// The limit of string storage.
		#[pallet::constant]
		type StringLimit: Get<u32>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The program has already been initialized and cannot be overwritten.
		ProgramAlreadyInitialized,
		/// At least one Head Ambassador must be present in initialization.
		MustProvideHeadAmbassador,
		/// The size of the ambassador set cannot exceed the configured limit.
		TooManyMembers,
		/// The account is not a member of the program.
		NotMember,
		/// The region ID given does not correspond to a region.
		NotRegion,
		/// The length of some input exceeded its maximum size.
		InputLimitExceeded,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new charter was set by the Ambassador Program.
		CharterSet,
		/// Some accounts have been initialized as members.
		MembersInitialized,
		/// A member's info was set by force.
		MemberInfoForceSet { member: T::AccountId },
	}

	/// The Ambassador Program's charter.
	#[pallet::storage]
	#[pallet::getter(fn charter)]
	pub type Charter<T: Config> = StorageValue<_, CharterLocation<T::StringLimit>, OptionQuery>;

	/// Maps member `AccountId` to their info.
	#[pallet::storage]
	#[pallet::getter(fn member_info_of)]
	pub type MemberInfoOf<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, MemberInfo<T::StringLimit>, OptionQuery>;

	/// Maps member type to members of each type.
	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type Members<T: Config> =
		StorageMap<_, Twox64Concat, Rank, BoundedVec<T::AccountId, T::MaxMembersCount>, ValueQuery>;

	/// The next region ID available.
	#[pallet::storage]
	#[pallet::getter(fn next_region_id)]
	pub type NextRegionId<T: Config> = StorageValue<_, RegionId, ValueQuery>;

	/// Maps `RegionId` to its info.
	#[pallet::storage]
	#[pallet::getter(fn region_info_of)]
	pub type RegionInfoOf<T: Config> =
		StorageMap<_, Twox64Concat, RegionId, Region<T::StringLimit, T::AccountId>, OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Root Calls

		/// Set the initial members of the Ambassador Program. Only the head ambassadors are
		/// initialized via Root. They are expected to handle applications to program entry.
		#[pallet::weight(1)]
		pub fn init_members(
			origin: OriginFor<T>,
			head_ambassadors: Vec<T::AccountId>,
		) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(!Self::is_initialized(), Error::<T>::ProgramAlreadyInitialized);
			ensure!(!head_ambassadors.is_empty(), Error::<T>::MustProvideHeadAmbassador);
			// todo
			Self::deposit_event(Event::MembersInitialized);
			Ok(())
		}

		/// Disband the Polkadot Ambassador Program. The origin for this call must be Root.
		#[pallet::weight(1)]
		pub fn disband(origin: OriginFor<T>) -> DispatchResult {
			ensure_root(origin)?;
			Ok(())
		}

		// Management

		/// Set the charter of the Ambassador Program.
		#[pallet::weight(1)]
		pub fn set_charter(
			origin: OriginFor<T>,
			charter: CharterLocation<T::StringLimit>,
		) -> DispatchResult {
			T::AdminOrigin::ensure_origin(origin)?;
			Charter::<T>::put(charter);
			Self::deposit_event(Event::CharterSet);
			Ok(())
		}

		/// Add a new region.
		#[pallet::weight(1)]
		pub fn add_new_region(
			origin: OriginFor<T>,
			name: Vec<u8>,
			lead: Option<T::AccountId>,
		) -> DispatchResult {
			T::AdminOrigin::ensure_origin(origin)?;
			if let Some(ref l) = lead {
				ensure!(Self::is_member(&l), Error::<T>::NotMember);
			}
			let next_region_id = Self::next_region_id();
			let bounded_name: BoundedVec<u8, T::StringLimit> =
				name.try_into().map_err(|_| Error::<T>::InputLimitExceeded)?;
			RegionInfoOf::<T>::insert(
				next_region_id,
				Region { name: bounded_name, lead },
			);
			NextRegionId::<T>::put(next_region_id.saturating_add(1 as RegionId));
			Ok(())
		}

		// todo: remove_region
		// Potentially messy as we would need to alter member info for members of that region?
		// Or add a counter like `RegionMembers: Map RegionId => u16` and ensure it is 0?

		/// Set some `Member` as the lead of a region.
		#[pallet::weight(1)]
		pub fn update_regional_info(
			origin: OriginFor<T>,
			region: RegionId,
			name: Vec<u8>,
			lead: T::AccountId,
		) -> DispatchResult {
			T::AdminOrigin::ensure_origin(origin)?;
			ensure!(Self::is_region(&region), Error::<T>::NotRegion);
			ensure!(Self::is_member(&lead), Error::<T>::NotMember);
			let bounded_name: BoundedVec<u8, T::StringLimit> =
				name.try_into().map_err(|_| Error::<T>::InputLimitExceeded)?;
			// todo
			Ok(())
		}

		/// Set the info for a member by the admin origin.
		#[pallet::weight(1)]
		pub fn force_set_member_info(
			origin: OriginFor<T>,
			member: T::AccountId,
			rank: Rank,
			contact: Contact<T::StringLimit>,
			region: RegionId,
			activity_tracks: Vec<ActivityTrack>,
			paid: bool,
		) -> DispatchResult {
			T::AdminOrigin::ensure_origin(origin)?;
			ensure!(Self::is_member(&member), Error::<T>::NotMember);
			// todo
			Self::deposit_event(Event::MemberInfoForceSet { member });
			Ok(())
		}

		// Announcements

		/// Make an announcement.
		#[pallet::weight(1)]
		pub fn make_announcement(
			origin: OriginFor<T>,
			track: ActivityTrack,
			link: Vec<u8>,
			expiration: T::BlockNumber,
		) -> DispatchResult {
			T::AnnouncementOrigin::ensure_origin(origin)?;
			Ok(())
		}

		/// Modify an existing announcement.
		#[pallet::weight(1)]
		pub fn modify_announcement(
			origin: OriginFor<T>,
			announcement_id: AnnouncementId,
			new_track: ActivityTrack,
			new_link: Vec<u8>,
		) -> DispatchResult {
			T::AnnouncementOrigin::ensure_origin(origin)?;
			Ok(())
		}

		/// Remove an announcement (presumably before scheduled removal).
		#[pallet::weight(1)]
		pub fn remove_announcement(
			origin: OriginFor<T>,
			announcement_id: AnnouncementId,
		) -> DispatchResult {
			T::AnnouncementOrigin::ensure_origin(origin)?;
			Ok(())
		}

		// Membership Management

		/// Apply and join as an Applicant and reserve a deposit.
		#[pallet::weight(1)]
		pub fn apply(origin: OriginFor<T>) -> DispatchResult {
			Ok(())
		}

		/// Voting member can nominate someone to join without a deposit.
		#[pallet::weight(1)]
		pub fn nominate(origin: OriginFor<T>, candidate: T::AccountId) -> DispatchResult {
			Ok(())
		}

		/// Bump a member up by a single rank. Privileged.
		#[pallet::weight(1)]
		pub fn promote(origin: OriginFor<T>, candidate: T::AccountId) -> DispatchResult {
			Ok(())
		}

		/// Remove a member from the ambassador program. Optionally slash their deposit. Privileged.
		#[pallet::weight(1)]
		pub fn remove_member(
			origin: OriginFor<T>,
			member: T::AccountId,
			slash_deposit: bool,
		) -> DispatchResult {
			T::AdminOrigin::ensure_origin(origin)?;
			Ok(())
		}

		/// Drop out of the ambassador program. Deposit is returned, but they would have to start over.
		#[pallet::weight(1)]
		pub fn leave_program(origin: OriginFor<T>) -> DispatchResult {
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Check if the program has been initialized.
	fn is_initialized() -> bool {
		Self::rank_has_member(Rank::HeadAmbassador) ||
			Self::rank_has_member(Rank::SeniorAmbassador) ||
			Self::rank_has_member(Rank::Ambassador) ||
			Self::rank_has_member(Rank::Applicant)
	}

	/// Check if a given rank has any members.
	fn rank_has_member(rank: Rank) -> bool {
		Members::<T>::decode_len(rank).unwrap_or_default() > 0
	}

	/// Check if a user is a member of the program.
	pub fn is_member(who: &T::AccountId) -> bool {
		Self::member_info_of(who).is_some()
	}

	/// Check if a region exists.
	fn is_region(id: &RegionId) -> bool {
		Self::region_info_of(id).is_some()
	}
}
