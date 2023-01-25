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

//! Managed collective content.
//!
//! The pallet provides the functionality to store different types of the content.
//! The content presented as a [Cid] of the IPFS document which might contain any type of data.
//! Every type of the content has its own origin to be manage. The origins are configurable by clients.
//! Storing the content does not require a deposit, the content expected to be managed by a trusted collective.
//!
//! Content types:
//! - the collective [charter](pallet::Charter). A single document managed by [CharterOrigin](pallet::Config::CharterOrigin).
//! - the collective [announcements](pallet::Announcements). A list of announcements managed by [AnnouncementOrigin](pallet::Config::AnnouncementOrigin).

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

use frame_support::{traits::schedule::DispatchTime, BoundedVec};
use sp_core::ConstU32;

/// IPFS compatible CID.
// worst case 2 bytes base and codec, 2 bytes hash type and size, 64 bytes hash digest.
pub type Cid = BoundedVec<u8, ConstU32<68>>;

/// The block number type of [frame_system::Config].
pub type BlockNumberFor<T> = <T as frame_system::Config>::BlockNumber;

/// [DispatchTime] of [frame_system::Config].
pub type DispatchTimeFor<T> = DispatchTime<BlockNumberFor<T>>;

#[frame_support::pallet]
pub mod pallet {
	use super::{Cid, DispatchTimeFor, WeightInfo};
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	/// The module configuration trait.
	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

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
	pub enum Error<T, I = ()> {
		/// The announcement is not found.
		MissingAnnouncement,
		/// Number of announcements exceeds `MaxAnnouncementsCount`.
		TooManyAnnouncements,
		/// Cannot expire in the past.
		InvalidExpire,
	}

	/// Events emitted by the pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// A new charter has been set.
		NewCharterSet { cid: Cid },
		/// A new announcement has been made.
		AnnouncementAnnounced { cid: Cid, maybe_expire_at: Option<T::BlockNumber> },
		/// An on-chain announcement has been removed.
		AnnouncementRemoved { cid: Cid },
	}

	/// The collective charter.
	#[pallet::storage]
	#[pallet::getter(fn charter)]
	pub type Charter<T: Config<I>, I: 'static = ()> = StorageValue<_, Cid, OptionQuery>;

	/// The collective announcements.
	#[pallet::storage]
	#[pallet::getter(fn announcements)]
	pub type Announcements<T: Config<I>, I: 'static = ()> = StorageValue<
		_,
		BoundedVec<(Cid, Option<T::BlockNumber>), T::MaxAnnouncementsCount>,
		ValueQuery,
	>;

	/// The closest expiration block number of an announcement.
	#[pallet::storage]
	pub type NextAnnouncementExpire<T: Config<I>, I: 'static = ()> =
		StorageValue<_, T::BlockNumber, OptionQuery>;

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Set the collective  charter.
		///
		/// Parameters:
		/// - `origin`: Must be the [Config::CharterOrigin].
		/// - `cid`: [CID](super::Cid) of the IPFS document of the collective charter.
		///
		/// Weight: `O(1)`.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::set_charter())]
		pub fn set_charter(origin: OriginFor<T>, cid: Cid) -> DispatchResult {
			T::CharterOrigin::ensure_origin(origin)?;

			Charter::<T, I>::put(&cid);

			Self::deposit_event(Event::<T, I>::NewCharterSet { cid });
			Ok(())
		}

		/// Publish an announcement.
		///
		/// Parameters:
		/// - `origin`: Must be the [Config::CharterOrigin].
		/// - `cid`: [CID](super::Cid) of the IPFS document to announce.
		/// - `maybe_expire`: expiration block of the announcement.
		///
		/// Weight: `O(1)`.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::announce(maybe_expire.map_or(0, |_| 1)))]
		pub fn announce(
			origin: OriginFor<T>,
			cid: Cid,
			maybe_expire: Option<DispatchTimeFor<T>>,
		) -> DispatchResult {
			T::AnnouncementOrigin::ensure_origin(origin)?;

			let now = frame_system::Pallet::<T>::block_number();
			let maybe_expire_at = maybe_expire.map(|e| e.evaluate(now));
			ensure!(maybe_expire_at.map_or(true, |e| e > now), Error::<T, I>::InvalidExpire);

			let mut announcements = <Announcements<T, I>>::get();
			announcements
				.try_push((cid.clone(), maybe_expire_at.clone()))
				.map_err(|_| Error::<T, I>::TooManyAnnouncements)?;
			<Announcements<T, I>>::put(announcements);

			if let Some(expire_at) = maybe_expire_at {
				if NextAnnouncementExpire::<T, I>::get().map_or(true, |n| n > expire_at) {
					NextAnnouncementExpire::<T, I>::put(expire_at);
				}
			}

			Self::deposit_event(Event::<T, I>::AnnouncementAnnounced { cid, maybe_expire_at });
			Ok(())
		}

		/// Remove an announcement.
		///
		/// Parameters:
		/// - `origin`: Must be the [Config::CharterOrigin].
		/// - `cid`: [CID](super::Cid) of the IPFS document to remove.
		///
		/// Weight: `O(1)`, less of the [Config::MaxAnnouncementsCount] is lower.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::remove_announcement())]
		pub fn remove_announcement(origin: OriginFor<T>, cid: Cid) -> DispatchResult {
			T::AnnouncementOrigin::ensure_origin(origin)?;

			let mut announcements = <Announcements<T, I>>::get();
			let pos = announcements
				.binary_search_by_key(&cid, |(c, _)| c.clone())
				.ok()
				.ok_or(Error::<T, I>::MissingAnnouncement)?;
			announcements.remove(pos);
			<Announcements<T, I>>::put(announcements);

			Self::deposit_event(Event::<T, I>::AnnouncementRemoved { cid });
			Ok(())
		}
	}
}
