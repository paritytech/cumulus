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

//! Pallet for stuff specific to parachains' usage of XCM. Right now that's just the origin
//! used by parachains when receiving `Transact` messages from other parachains or the Relay chain
//! which must be natively represented.

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{prelude::*, convert::TryFrom};
use cumulus_primitives_core::relay_chain::BlockNumber as RelayBlockNumber;
use cumulus_primitives_core::DmpMessageHandler;
use codec::{Encode, Decode};
use sp_runtime::RuntimeDebug;
use xcm::{VersionedXcm, v0::{Xcm, Junction, Outcome, ExecuteXcm, Error as XcmError}};
use frame_support::{traits::EnsureOrigin, dispatch::Weight, weights::{PostDispatchInfo, constants::WEIGHT_PER_MILLIS}};
pub use pallet::*;

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct ConfigData {
	/// The maximum amount of weight any individual message may consume. Messages above this weight
	/// go into the overweight queue and may only be serviced explicitly by the
	/// `ExecuteOverweightOrigin`.
	max_individual: Weight,
}

impl Default for ConfigData {
	fn default() -> Self {
		Self {
			max_individual: 10 * WEIGHT_PER_MILLIS,	// 10 ms of execution time maximum by default
		}
	}
}

/// Information concerning our message pages.
#[derive(Copy, Clone, Eq, PartialEq, Default, Encode, Decode, RuntimeDebug)]
pub struct PageIndexData {
	/// The lowest used page index.
	begin_used: PageCounter,
	/// The lowest unused page index.
	end_used: PageCounter,
	/// The number of overweight messages ever recorded (and thus the lowest free index).
	overweight_count: OverweightIndex,
}

/// Simple type used to identify messages for the purpose of reporting events. Secure if and only
/// if the message content is unique.
pub type MessageId = [u8; 32];

/// Index used to identify overweight messages.
pub type OverweightIndex = u64;

/// Index used to identify normal pages.
pub type PageCounter = u32;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// The module configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type XcmExecutor: ExecuteXcm<Self::Call>;

		/// Origin which is allowed to execute overweight messages.
		type ExecuteOverweightOrigin: EnsureOrigin<Self::Origin>;
	}

	/// The configuration.
	#[pallet::storage]
	pub(super) type Configuration<T> = StorageValue<_, ConfigData, ValueQuery>;

	/// The page index.
	#[pallet::storage]
	pub(super) type PageIndex<T> = StorageValue<_, PageIndexData, ValueQuery>;

	/// The queue pages.
	#[pallet::storage]
	pub(super) type Pages<T> = StorageMap<
		_,
		Blake2_128Concat,
		PageCounter,
		Vec<(RelayBlockNumber, Vec<u8>)>,
		ValueQuery,
	>;

	/// The overweight messages.
	#[pallet::storage]
	pub(super) type Overweight<T> = StorageMap<
		_,
		Blake2_128Concat,
		OverweightIndex,
		(RelayBlockNumber, Vec<u8>),
		OptionQuery,
	>;

	#[pallet::error]
	pub enum Error<T> {
		/// The message index given is unknown.
		Unknown,
		/// The amount of weight given is possibly not enough for executing the message.
		OverLimit,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_idle(_now: T::BlockNumber, max_weight: Weight) -> Weight {
			// on_idle processes additional messages with any remaining block weight.
			Self::service_queue(max_weight)
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Service a single overweight message.
		///
		/// - `origin`: Must pass `ExecuteOverweightOrigin`.
		/// - `index`: The index of the overweight message to service.
		/// - `weight_limit`: The amount of weight that message execution may take.
		///
		/// Errors:
		/// - `Unknown`: Message of `index` is unknown.
		/// - `OverLimit`: Message execution may used greater than `weight_limit`.
		///
		/// Events:
		/// - `OverweightServiced`: On success.
		#[pallet::weight(1_000_000 + weight_limit)]
		fn service_overweight(
			origin: OriginFor<T>,
			index: OverweightIndex,
			weight_limit: Weight,
		) -> DispatchResultWithPostInfo {
			T::ExecuteOverweightOrigin::ensure_origin(origin)?;

			let (sent_at, data) = Overweight::<T>::get(index).ok_or(Error::<T>::Unknown)?;
			let used = Self::try_service_message(weight_limit, sent_at, &data[..])
				.map_err(|_| Error::<T>::OverLimit)?;
			Overweight::<T>::remove(index);
			Self::deposit_event(Event::OverweightServiced(index, used));
			Ok(PostDispatchInfo { actual_weight: Some(1_000_000 + used), pays_fee: Pays::Yes })
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	#[pallet::metadata(T::BlockNumber = "BlockNumber")]
	pub enum Event<T: Config> {
		/// Downward message is invalid XCM.
		/// \[ id \]
		InvalidFormat(MessageId),
		/// Downward message is unsupported version of XCM.
		/// \[ id \]
		UnsupportedVersion(MessageId),
		/// Downward message executed with the given outcome.
		/// \[ id, outcome \]
		ExecutedDownward(MessageId, Outcome),
		/// \[ id, remaining, required \]
		WeightExhausted(MessageId, Weight, Weight),
		/// \[ id, index, required \]
		OverweightEnqueued(MessageId, OverweightIndex, Weight),
		/// \[ index, used \]
		OverweightServiced(OverweightIndex, Weight),
	}

	impl<T: Config> Pallet<T> {
		/// Service the message queue up to some given weight `limit`.
		///
		/// Returns the weight consumed by executing messages in the queue.
		fn service_queue(limit: Weight) -> Weight {
			PageIndex::<T>::mutate(|page_index| Self::do_service_queue(limit, page_index))
		}

		/// Exactly equivalent to `service_queue` but expects a mutable `page_index` to be passed
		/// in and any changes stored.
		fn do_service_queue(limit: Weight, page_index: &mut PageIndexData) -> Weight {
			let mut used = 0;
			while page_index.begin_used < page_index.end_used {
				let page = Pages::<T>::take(page_index.begin_used);
				for (i, &(sent_at, ref data)) in page.iter().enumerate() {
					match Self::try_service_message(limit - used, sent_at, &data[..]) {
						Ok(w) => used += w,
						Err(..) => {
							// Too much weight needed - put the remaining messages back and bail
							Pages::<T>::insert(page_index.begin_used, &page[i..]);
							return used;
						}
					}
				}
				page_index.begin_used += 1;
			}
			if page_index.begin_used == page_index.end_used {
				// Reset if there's no pages left.
				page_index.begin_used = 0;
				page_index.end_used = 0;
			}
			used
		}

		/// Attempt to service an individual message. Will return `Ok` with the execution weight
		/// consumed unless the message was found to need more weight than `limit`.
		///
		/// NOTE: This will return `Ok` in the case of an error decoding, weighing or executing
		/// the message. This is why it's called message "servicing" rather than "execution".
		pub(crate) fn try_service_message(
			limit: Weight,
			_sent_at: RelayBlockNumber,
			data: &[u8],
		) -> Result<Weight, (MessageId, Weight)> {
			let id = sp_io::hashing::blake2_256(&data[..]);
			let maybe_msg = VersionedXcm::<T::Call>::decode(&mut &data[..])
				.map(Xcm::<T::Call>::try_from);
			match maybe_msg {
				Err(_) => {
					Self::deposit_event(Event::InvalidFormat(id));
					Ok(0)
				},
				Ok(Err(())) => {
					Self::deposit_event(Event::UnsupportedVersion(id));
					Ok(0)
				},
				Ok(Ok(x)) => {
					let outcome = T::XcmExecutor::execute_xcm(Junction::Parent.into(), x, limit);
					match outcome {
						Outcome::Error(XcmError::WeightLimitReached(required)) => Err((id, required)),
						outcome => {
							let weight_used = outcome.weight_used();
							Self::deposit_event(Event::ExecutedDownward(id, outcome));
							Ok(weight_used)
						}
					}
				}
			}
		}
	}

	/// For an incoming downward message, this just adapts an XCM executor and executes DMP messages
	/// immediately up until some `MaxWeight` at which point it errors. Their origin is asserted to be
	/// the `Parent` location.
	impl<T: Config> DmpMessageHandler for Pallet<T> {
		fn handle_dmp_messages(
			iter: impl Iterator<Item=(RelayBlockNumber, Vec<u8>)>,
			limit: Weight,
		) -> Weight {
			let mut page_index = PageIndex::<T>::get();
			let config = Configuration::<T>::get();

			// First try to use `max_weight` to service the current queue.
			let mut used = Self::do_service_queue(limit, &mut page_index);

			// Then if the queue is empty, use the weight remaining to service the incoming messages
			// and once we run out of weight, place them in the queue.
			let item_count = iter.size_hint().0;
			let mut maybe_enqueue_page = if page_index.end_used > page_index.begin_used {
				// queue is already non-empty - start a fresh page.
				Some(Vec::with_capacity(item_count))
			} else {
				None
			};

			for (i, (sent_at, data)) in iter.enumerate() {
				if maybe_enqueue_page.is_none() {
					// We're not currently enqueuing - try to execute inline.
					let remaining = limit.saturating_sub(used);
					match Self::try_service_message(remaining, sent_at, &data[..]) {
						Ok(consumed) => used += consumed,
						Err((id, required)) =>
							// Too much weight required right now.
							if required > config.max_individual {
								// overweight - add to overweight queue and continue with
								// message execution.
								let index = page_index.overweight_count;
								Overweight::<T>::insert(index, (sent_at, data));
								Self::deposit_event(Event::OverweightEnqueued(id, index, required));
								page_index.overweight_count += 1;
								// Not needed for control flow, but only to ensure that the compiler
								// understands that we won't attempt to re-use `data` later.
								continue;
							} else {
								// not overweight. stop executing inline and enqueue normally
								// from here on.
								let item_count_left = item_count.saturating_sub(i);
								maybe_enqueue_page = Some(Vec::with_capacity(item_count_left));
								Self::deposit_event(Event::WeightExhausted(id, remaining, required));
							}
					}
				}
				// Cannot be an `else` here since the `maybe_enqueue_page` may have changed.
				if let Some(ref mut enqueue_page) = maybe_enqueue_page {
					enqueue_page.push((sent_at, data));
				}
			}

			// Deposit the enqueued page if any and save the index.
			if let Some(enqueue_page) = maybe_enqueue_page {
				Pages::<T>::insert(page_index.end_used, enqueue_page);
				page_index.end_used += 1;
			}
			PageIndex::<T>::put(page_index);

			used
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use codec::Encode;
	use cumulus_primitives_core::{
		AbridgedHrmpChannel, InboundDownwardMessage, InboundHrmpMessage, PersistedValidationData,
		relay_chain::BlockNumber as RelayBlockNumber, ParaId,
	};
	use frame_support::{
		assert_ok,
		dispatch::UnfilteredDispatchable,
		parameter_types,
		traits::{OnFinalize, OnInitialize},
	};
	use frame_system::{InitKind, RawOrigin};
	use cumulus_primitives_core::relay_chain::v1::HrmpChannelId;
	use sp_core::H256;
	use sp_runtime::{testing::Header, traits::{IdentityLookup, BlakeTwo256}};
	use sp_version::RuntimeVersion;
	use std::cell::RefCell;
	use xcm::opaque::v0::MultiLocation;

	use crate as dmp_queue;

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
	type Block = frame_system::mocking::MockBlock<Test>;

	frame_support::construct_runtime!(
		pub enum Test where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
			DMPQueue: dmp_queue::{Pallet, Call, Storage, Event<T>},
		}
	);

	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub Version: RuntimeVersion = RuntimeVersion {
			spec_name: sp_version::create_runtime_str!("test"),
			impl_name: sp_version::create_runtime_str!("system-test"),
			authoring_version: 1,
			spec_version: 1,
			impl_version: 1,
			apis: sp_version::create_apis_vec!([]),
			transaction_version: 1,
		};
		pub const ParachainId: ParaId = ParaId::new(200);
		pub const ReservedXcmpWeight: Weight = 0;
		pub const ReservedDmpWeight: Weight = 0;
	}

	type AccountId = u64;

	impl frame_system::Config for Test {
		type Origin = Origin;
		type Call = Call;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = AccountId;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = Event;
		type BlockHashCount = BlockHashCount;
		type BlockLength = ();
		type BlockWeights = ();
		type Version = Version;
		type PalletInfo = PalletInfo;
		type AccountData = ();
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type DbWeight = ();
		type BaseCallFilter = ();
		type SystemWeightInfo = ();
		type SS58Prefix = ();
		type OnSetCode = ();
	}

	pub struct MockExec;
	impl ExecuteXcm<Call> for MockExec {
		type Call = Call;
		fn execute_xcm(_origin: MultiLocation, _message: Xcm<Call>, weight_limit: Weight) -> Outcome {
			if weight_limit < 100 {
				Outcome::Error(XcmError::WeightLimitReached(101))
			} else if weight_limit < 200 {
				Outcome::Incomplete(weight_limit / 2, XcmError::Barrier)
			} else {
				Outcome::Complete(weight_limit / 2)
			}
		}
	}

	impl Config for Test {
		type Event = Event;
		type XcmExecutor = MockExec;
		type ExecuteOverweightOrigin = frame_system::EnsureRoot<AccountId>;
	}

	pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
		frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[test]
	fn try_service_message() {
		new_test_ext().execute_with(|| {
			let limit = 1_000;
			let sent_at = 0;
			let data = VersionedXcm::<Call>::V0(Xcm::<Call>::WithdrawAsset { assets: Vec::new(), effects: Vec::new() });
			let mut garbage = vec![5; 4];
			garbage.append(&mut data.encode());
			// incorrectly encoded messages
			assert_eq!(DMPQueue::try_service_message(
				limit,
				sent_at,
				&garbage,
			), Ok(0));

			let encoded = data.encode();
			assert_eq!(DMPQueue::try_service_message(
				limit,
				sent_at,
				&encoded,
			), Ok(limit / 2));

			let low_limit = 50;
			let id = sp_io::hashing::blake2_256(&encoded[..]);
			assert_eq!(DMPQueue::try_service_message(
				low_limit,
				sent_at,
				&encoded,
			), Err((id, 101)));

			let medium_limit = 160;
			assert_eq!(DMPQueue::try_service_message(
				medium_limit,
				sent_at,
				&encoded,
			), Ok(medium_limit / 2));
		});
	}
}
