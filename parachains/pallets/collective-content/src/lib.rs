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

//! Managed Collective Content Pallet
//!
//! The pallet provides the functionality to store different types of content. This would typically
//! be used by an on-chain collective, such as the Polkadot Alliance or Ambassador Program.
//!
//! The pallet stores content as a [OpaqueCid], which should correspond to some off-chain hosting service,
//! such as IPFS, and contain any type of data. Each type of content has its own origin from which
//! it can be managed. The origins are configurable in the runtime. Storing content does not require
//! a deposit, as it is expected to be managed by a trusted collective.
//!
//! Content types:
//!
//! - Collective [charter](pallet::Charter): A single document (`OpaqueCid`) managed by
//!   [CharterOrigin](pallet::Config::CharterOrigin).
//! - Collective [announcements](pallet::Announcements): A list of announcements managed by
//!   [AnnouncementOrigin](pallet::Config::AnnouncementOrigin).

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
use sp_std::prelude::*;

/// IPFS compatible CID.
// worst case 2 bytes base and codec, 2 bytes hash type and size, 64 bytes hash digest.
pub type OpaqueCid = BoundedVec<u8, ConstU32<68>>;

/// The block number type of [frame_system::Config].
pub type BlockNumberFor<T> = <T as frame_system::Config>::BlockNumber;

/// [DispatchTime] of [frame_system::Config].
pub type DispatchTimeFor<T> = DispatchTime<BlockNumberFor<T>>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
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
		///
		/// NOTE: Benchmarks need to be re-run if this changes.
		#[pallet::constant]
		type MaxAnnouncementsCount: Get<u32>;

		/// Weight information needed for the pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// The announcement is not found.
		MissingAnnouncement,
		/// Number of announcements exceeds `MaxAnnouncementsCount`.
		TooManyAnnouncements,
		/// Cannot expire in the past.
		InvalidExpiration,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// A new charter has been set.
		NewCharterSet { cid: OpaqueCid },
		/// A new announcement has been made.
		AnnouncementAnnounced { cid: OpaqueCid, maybe_expire_at: Option<T::BlockNumber> },
		/// An on-chain announcement has been removed.
		AnnouncementRemoved { cid: OpaqueCid },
	}

	/// The collective charter.
	#[pallet::storage]
	#[pallet::getter(fn charter)]
	pub type Charter<T: Config<I>, I: 'static = ()> = StorageValue<_, OpaqueCid, OptionQuery>;

	/// The collective announcements.
	#[pallet::storage]
	#[pallet::getter(fn announcements)]
	pub type Announcements<T: Config<I>, I: 'static = ()> = StorageValue<
		_,
		BoundedVec<(OpaqueCid, Option<T::BlockNumber>), T::MaxAnnouncementsCount>,
		ValueQuery,
	>;

	/// The closest expiration block number of an announcement.
	#[pallet::storage]
	pub type NextAnnouncementExpireAt<T: Config<I>, I: 'static = ()> =
		StorageValue<_, T::BlockNumber, OptionQuery>;

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Set the collective  charter.
		///
		/// Parameters:
		/// - `origin`: Must be the [Config::CharterOrigin].
		/// - `cid`: [CID](super::OpaqueCid) of the IPFS document of the collective charter.
		///
		/// Weight: `O(1)`.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::set_charter())]
		pub fn set_charter(origin: OriginFor<T>, cid: OpaqueCid) -> DispatchResult {
			T::CharterOrigin::ensure_origin(origin)?;

			Charter::<T, I>::put(&cid);

			Self::deposit_event(Event::<T, I>::NewCharterSet { cid });
			Ok(())
		}

		/// Publish an announcement.
		///
		/// Parameters:
		/// - `origin`: Must be the [Config::CharterOrigin].
		/// - `cid`: [CID](super::OpaqueCid) of the IPFS document to announce.
		/// - `maybe_expire`: Expiration block of the announcement.
		///
		/// Weight: `O(1)`.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::announce(maybe_expire.map_or(0, |_| 1)))]
		pub fn announce(
			origin: OriginFor<T>,
			cid: OpaqueCid,
			maybe_expire: Option<DispatchTimeFor<T>>,
		) -> DispatchResult {
			T::AnnouncementOrigin::ensure_origin(origin)?;

			let now = frame_system::Pallet::<T>::block_number();
			let maybe_expire_at = maybe_expire.map(|e| e.evaluate(now));
			ensure!(maybe_expire_at.map_or(true, |e| e > now), Error::<T, I>::InvalidExpiration);

			let mut announcements = <Announcements<T, I>>::get();
			announcements
				.try_push((cid.clone(), maybe_expire_at.clone()))
				.map_err(|_| Error::<T, I>::TooManyAnnouncements)?;
			<Announcements<T, I>>::put(announcements);

			if let Some(expire_at) = maybe_expire_at {
				if NextAnnouncementExpireAt::<T, I>::get().map_or(true, |n| n > expire_at) {
					NextAnnouncementExpireAt::<T, I>::put(expire_at);
				}
			}

			Self::deposit_event(Event::<T, I>::AnnouncementAnnounced { cid, maybe_expire_at });
			Ok(())
		}

		/// Remove an announcement.
		///
		/// Parameters:
		/// - `origin`: Must be the [Config::CharterOrigin].
		/// - `cid`: [CID](super::OpaqueCid) of the IPFS document to remove.
		///
		/// Weight: `O(1)`, less of the [Config::MaxAnnouncementsCount] is lower.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::remove_announcement())]
		pub fn remove_announcement(origin: OriginFor<T>, cid: OpaqueCid) -> DispatchResult {
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

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Clean up expired announcements.
		pub fn cleanup_announcements(now: T::BlockNumber) {
			if NextAnnouncementExpireAt::<T, I>::get().map_or(true, |next| next > now) {
				// no expired announcements expected.
				return
			}
			// find expired announcements.
			let announcements = <Announcements<T, I>>::get();
			let (live, expired): (Vec<(_, _)>, Vec<(_, _)>) =
				announcements.clone().into_iter().partition(|(_, maybe_expire_at)| {
					maybe_expire_at.map_or(true, |expire_at| expire_at > now)
				});
			if expired.is_empty() {
				// no expired announcements.
				return
			}
			// convert into bounded vec.
			let live: BoundedVec<(_, _), T::MaxAnnouncementsCount> = live.try_into().map_or_else(
				|err| {
					// should never happen.
					debug_assert!(false, "failed to convert into bounded vec: {:?}", err);
					announcements
				},
				|live| live,
			);
			// determine the next announcement expire at.
			NextAnnouncementExpireAt::<T, I>::set(live.clone().into_iter().fold(
				None,
				|next, (_, maybe_expire_at)| match (next, maybe_expire_at) {
					(Some(next), Some(expire_at)) if next > expire_at => Some(expire_at),
					(None, Some(expire_at)) => Some(expire_at),
					_ => next,
				},
			));
			// save new list of announcements.
			<Announcements<T, I>>::put(live);
			// publish events for removed announcements.
			expired.into_iter().for_each(|(cid, _)| {
				Self::deposit_event(Event::<T, I>::AnnouncementRemoved { cid })
			});
		}
	}

	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<T::BlockNumber> for Pallet<T, I> {
		/// Clean up expired announcements if there is enough `remaining_weight` weight left.
		fn on_idle(now: T::BlockNumber, remaining_weight: Weight) -> Weight {
			let weight = T::WeightInfo::cleanup_announcements();
			if remaining_weight.any_lt(weight) {
				return Weight::from_ref_time(0)
			}
			Self::cleanup_announcements(now);
			weight
		}
	}
}
