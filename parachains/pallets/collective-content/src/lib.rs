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

//! Collective content.
//! TODO docs

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

pub use pallet::*;
pub use weights::WeightInfo;

use frame_support::BoundedVec;
use sp_core::ConstU32;

/// IPFS compatible CID.
// worst case 2 bytes base and codec, 2 bytes hash type and size, 64 bytes hash digest.
pub type Cid = BoundedVec<u8, ConstU32<68>>;

#[frame_support::pallet]
pub mod pallet {
	use super::{Cid, WeightInfo};
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

		/// The origin to control the collective announcements.
		type AnnouncementOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The origin to control the collective charter.
		type CharterOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The maximum number of announcements.
		#[pallet::constant]
		type MaxAnnouncementsCount: Get<u32>;

		/// Weights information needed for the pallet.
		type WeightInfo: WeightInfo;
	}

	/// Errors encountered by the pallet (not a full list).
	#[pallet::error]
	pub enum Error<T> {
		/// The announcement is not found.
		MissingAnnouncement,
		/// Number of announcements exceeds `MaxAnnouncementsCount`.
		TooManyAnnouncements,
	}

	/// Events emitted by the pallet.
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

	/// The collective charter.
	#[pallet::storage]
	#[pallet::getter(fn charter)]
	pub type Charter<T: Config> = StorageValue<_, Cid, OptionQuery>;

	/// The collective announcements.
	#[pallet::storage]
	#[pallet::getter(fn announcements)]
	pub type Announcements<T: Config> =
		StorageValue<_, BoundedVec<Cid, T::MaxAnnouncementsCount>, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set the collective  charter.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::set_charter())]
		pub fn set_charter(origin: OriginFor<T>, cid: Cid) -> DispatchResult {
			T::CharterOrigin::ensure_origin(origin)?;

			Charter::<T>::put(&cid);

			Self::deposit_event(Event::<T>::NewCharterSet { cid });
			Ok(())
		}

		/// Publish an announcement.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::announce())]
		pub fn announce(origin: OriginFor<T>, cid: Cid) -> DispatchResult {
			T::AnnouncementOrigin::ensure_origin(origin)?;

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
		#[pallet::weight(T::WeightInfo::remove_announcement())]
		pub fn remove_announcement(origin: OriginFor<T>, cid: Cid) -> DispatchResult {
			T::AnnouncementOrigin::ensure_origin(origin)?;

			let mut announcements = <Announcements<T>>::get();
			let pos =
				announcements.binary_search(&cid).ok().ok_or(Error::<T>::MissingAnnouncement)?;
			announcements.remove(pos);
			<Announcements<T>>::put(announcements);

			Self::deposit_event(Event::<T>::AnnouncementRemoved { cid });
			Ok(())
		}
	}
}
