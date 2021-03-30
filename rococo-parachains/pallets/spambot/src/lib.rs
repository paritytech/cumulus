// Copyright 2020-2021 Parity Technologies (UK) Ltd.
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

//! Pallet to spam the XCM/UMP.

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_system::Config as SystemConfig;
use cumulus_primitives_core::ParaId;
use cumulus_pallet_xcm_handler::{Origin as CumulusOrigin, ensure_sibling_para};
use xcm::v0::{Xcm, SendXcm, OriginKind, MultiLocation, Junction};

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	/// The module configuration trait.
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Origin: From<<Self as SystemConfig>::Origin> + Into<Result<CumulusOrigin, <Self as Config>::Origin>>;

		/// The overarching call type; we assume sibling chains use the same type.
		type Call: From<Call<Self>> + Encode;

		type XcmSender: SendXcm;
	}

	#[pallet::storage]
	/// Details of an asset.
	pub(super) type Targets<T: Config> = StorageValue<
		_,
		Vec<ParaId>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// Details of an asset.
	pub(super) type PingCount<T: Config> = StorageValue<
		_,
		u32,
		ValueQuery,
	>;

	#[pallet::storage]
	/// Details of an asset.
	pub(super) type Pings<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u32,
		T::BlockNumber,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance", AssetIdOf<T> = "AssetId")]
	pub enum Event<T: Config> {
		PingSent(ParaId, u32),
		Pinged(ParaId, u32),
		PongSent(ParaId, u32),
		Ponged(ParaId, u32, T::BlockNumber),
		ErrorSendingPing(ParaId, u32),
		ErrorSendingPong(ParaId, u32),
		UnknownPong(ParaId, u32),
	}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_idle(
			n: T::BlockNumber,
			remaining_weight: Weight,
		) -> Weight {
			for para in Targets::<T>::get().into_iter() {
				let seq = PingCount::<T>::mutate(|seq| { *seq += 1; *seq });
				match T::XcmSender::send_xcm(
					MultiLocation::X2(Junction::Parent, Junction::Parachain { id: para.into() }),
					Xcm::Transact {
						origin_type: OriginKind::Native,
						call: <T as Config>::Call::from(Call::<T>::ping(seq)).encode(),
					},
				) {
					Ok(()) => {
						Pings::<T>::insert(seq, n);
						Self::deposit_event(Event::PingSent(para, seq));
					},
					Err(_) => {
						Self::deposit_event(Event::ErrorSendingPing(para, seq));
					}
				}
			}
			remaining_weight
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		fn start(origin: OriginFor<T>, para: ParaId) -> DispatchResult {
			ensure_root(origin)?;
			Targets::<T>::mutate(|t| t.push(para));
			Ok(())
		}

		#[pallet::weight(0)]
		fn start_many(origin: OriginFor<T>, para: ParaId, count: u32) -> DispatchResult {
			ensure_root(origin)?;
			for _ in 0..count {
				Targets::<T>::mutate(|t| t.push(para));
			}
			Ok(())
		}

		#[pallet::weight(0)]
		fn stop(origin: OriginFor<T>, para: ParaId) -> DispatchResult {
			ensure_root(origin)?;
			Targets::<T>::mutate(|t| if let Some(p) = t.iter().position(|p| p == &para) { t.swap_remove(p); });
			Ok(())
		}

		#[pallet::weight(0)]
		fn stop_all(origin: OriginFor<T>, maybe_para: Option<ParaId>) -> DispatchResult {
			ensure_root(origin)?;
			Targets::<T>::mutate(|t| t.retain(|&x| maybe_para.map_or(false, |para| x != para)));
			Ok(())
		}

		#[pallet::weight(0)]
		fn ping(origin: OriginFor<T>, seq: u32) -> DispatchResult {
			// Only accept pings from other chains.
			let para = ensure_sibling_para(<T as Config>::Origin::from(origin))?;

			Self::deposit_event(Event::Pinged(para, seq));
			match T::XcmSender::send_xcm(
				MultiLocation::X2(Junction::Parent, Junction::Parachain { id: para.into() }),
				Xcm::Transact {
					origin_type: OriginKind::Native,
					call: <T as Config>::Call::from(Call::<T>::pong(seq)).encode(),
				},
			) {
				Ok(()) => {
					Self::deposit_event(Event::PongSent(para, seq));
				},
				Err(_) => {
					Self::deposit_event(Event::ErrorSendingPong(para, seq));
				}
			}
			Ok(())
		}

		#[pallet::weight(0)]
		fn pong(origin: OriginFor<T>, seq: u32) -> DispatchResult {
			// Only accept pings from other chains.
			let para = ensure_sibling_para(<T as Config>::Origin::from(origin))?;

			if let Some(sent_at) = Pings::<T>::take(seq) {
				Self::deposit_event(Event::Ponged(para, seq, frame_system::Pallet::<T>::block_number() - sent_at));
			} else {
				// Pong received for a ping we apparently didn't send?!
				Self::deposit_event(Event::UnknownPong(para, seq));
			}
			Ok(())
		}
	}
}
