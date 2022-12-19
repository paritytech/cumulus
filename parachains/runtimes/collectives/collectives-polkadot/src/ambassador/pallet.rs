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

#[frame_support::pallet]
pub mod pallet {
	use super::super::types::Cid;

	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(PhantomData<T>);

	/// The module configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// A member of the Ambassador Program.
		type MemberOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Ambassador plurality voice.
		type AmbassadorOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The maximum number of announcements.
		#[pallet::constant]
		type MaxAnnouncementsCount: Get<u32>;
	}

	#[derive(PartialEq, Eq, Clone, MaxEncodedLen, Encode, Decode, TypeInfo, RuntimeDebug)]
	#[pallet::origin]
	pub enum Origin {
		/// Candidates plurality voice given via referendum.
		Candidate,
		/// Ambassador plurality voice given via referendum.
		Ambassador,
		/// SeniorAmbassador plurality voice given via referendum.
		SeniorAmbassador,
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The announcement is not found.
		MissingAnnouncement,
		/// Number of announcements exceeds `MaxAnnouncementsCount`.
		TooManyAnnouncements,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new charter has been set.
		NewCharterSet { cid: Cid },
		/// A new announcement has been proposed.
		AnnouncementAnnounced { cid: Cid },
		/// An on-chain announcement has been removed.
		AnnouncementRemoved { cid: Cid },
	}

	/// The Ambassador Program's charter.
	#[pallet::storage]
	#[pallet::getter(fn charter)]
	pub type Charter<T: Config> = StorageValue<_, Cid, OptionQuery>;

	/// The Ambassador Program's announcements.
	#[pallet::storage]
	#[pallet::getter(fn announcements)]
	pub type Announcements<T: Config> =
		StorageValue<_, BoundedVec<Cid, T::MaxAnnouncementsCount>, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set the Ambassador Program's  charter.
		#[pallet::call_index(0)]
		#[pallet::weight(0)] // TODO
		pub fn set_charter(origin: OriginFor<T>, cid: Cid) -> DispatchResult {
			T::AmbassadorOrigin::ensure_origin(origin)?;

			Charter::<T>::put(&cid);

			Self::deposit_event(Event::<T>::NewCharterSet { cid });
			Ok(())
		}

		/// Publish an announcement.
		#[pallet::call_index(1)]
		#[pallet::weight(0)] // TODO
		pub fn announce(origin: OriginFor<T>, cid: Cid) -> DispatchResult {
			T::MemberOrigin::ensure_origin(origin)?;

			let mut announcements = <Announcements<T>>::get();
			announcements
				.try_push(cid.clone())
				.map_err(|_| Error::<T>::TooManyAnnouncements)?;
			<Announcements<T>>::put(announcements);

			Self::deposit_event(Event::<T>::AnnouncementAnnounced { cid });
			Ok(())
		}

		/// Remove an announcement.
		#[pallet::call_index(2)]
		#[pallet::weight(0)] // TODO
		pub fn remove_announcement(origin: OriginFor<T>, cid: Cid) -> DispatchResult {
			T::MemberOrigin::ensure_origin(origin)?;

			let mut announcements = <Announcements<T>>::get();
			let pos =
				announcements.binary_search(&cid).ok().ok_or(Error::<T>::MissingAnnouncement)?;
			announcements.remove(pos);
			<Announcements<T>>::put(announcements);

			Self::deposit_event(Event::<T>::AnnouncementRemoved { cid });
			Ok(())
		}
	}

	pub struct Ambassador;
	// todo P2 check impl
	impl<O: Into<Result<Origin, O>> + From<Origin>> EnsureOrigin<O> for Ambassador {
		type Success = ();
		fn try_origin(o: O) -> Result<Self::Success, O> {
			o.into().and_then(|o| match o {
				Origin::Ambassador => Ok(()),
				r => Err(O::from(r)),
			})
		}
	}

	pub struct SeniorAmbassador;
	impl<O: Into<Result<Origin, O>> + From<Origin>> EnsureOrigin<O> for SeniorAmbassador {
		type Success = ();
		fn try_origin(o: O) -> Result<Self::Success, O> {
			o.into().and_then(|o| match o {
				Origin::SeniorAmbassador => Ok(()),
				r => Err(O::from(r)),
			})
		}
	}
}
