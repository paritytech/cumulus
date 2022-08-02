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

//! Pallet implementing a message queue for downward messages from the relay-chain.
//! Executes downward messages if there is enough weight available and schedules the rest for later
//! execution (by `on_idle` or another `handle_dmp_messages` call). Individual overweight messages
//! are scheduled into a separate queue that is only serviced by explicit extrinsic calls.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, DecodeLimit, Encode};
use cumulus_primitives_core::{
	relay_chain::BlockNumber as RelayBlockNumber, DmpMessageHandler, DmpMessageHandlerContext,
};
use frame_support::{
	dispatch::Weight, traits::EnsureOrigin, weights::constants::WEIGHT_PER_MILLIS,
};
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{BlakeTwo256, Hash},
	RuntimeDebug,
};
use sp_std::{convert::TryFrom, prelude::*};
use xcm::{latest::prelude::*, VersionedXcm, MAX_XCM_DECODE_DEPTH};

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ConfigData {
	/// The maximum amount of weight any individual message may consume. Messages above this weight
	/// go into the overweight queue and may only be serviced explicitly by the
	/// `ExecuteOverweightOrigin`.
	max_individual: Weight,
}

impl Default for ConfigData {
	fn default() -> Self {
		Self {
			max_individual: 10 * WEIGHT_PER_MILLIS, // 10 ms of execution time maximum by default
		}
	}
}

/// Information concerning our message pages.
#[derive(Copy, Clone, Eq, PartialEq, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
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
	#[pallet::without_storage_info]
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
	pub(super) type Pages<T> =
		StorageMap<_, Blake2_128Concat, PageCounter, Vec<(RelayBlockNumber, Vec<u8>)>, ValueQuery>;

	/// The overweight messages.
	#[pallet::storage]
	pub(super) type Overweight<T> =
		StorageMap<_, Blake2_128Concat, OverweightIndex, (RelayBlockNumber, Vec<u8>), OptionQuery>;

	#[pallet::error]
	pub enum Error<T> {
		/// The message index given is unknown.
		Unknown,
		/// The amount of weight given is possibly not enough for executing the message.
		OverLimit,
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
		/// - `OverLimit`: Message execution may use greater than `weight_limit`.
		///
		/// Events:
		/// - `OverweightServiced`: On success.
		#[pallet::weight(weight_limit.saturating_add(1_000_000))]
		pub fn service_overweight(
			origin: OriginFor<T>,
			index: OverweightIndex,
			weight_limit: Weight,
		) -> DispatchResultWithPostInfo {
			T::ExecuteOverweightOrigin::ensure_origin(origin)?;

			let (sent_at, data) = Overweight::<T>::get(index).ok_or(Error::<T>::Unknown)?;
			let weight_used = Self::try_service_message(weight_limit, sent_at, &data[..])
				.map_err(|_| Error::<T>::OverLimit)?;
			Overweight::<T>::remove(index);
			Self::deposit_event(Event::OverweightServiced { overweight_index: index, weight_used });
			Ok(Some(weight_used.saturating_add(1_000_000)).into())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Downward message is invalid XCM.
		InvalidFormat { message_id: MessageId },
		/// Downward message is unsupported version of XCM.
		UnsupportedVersion { message_id: MessageId },
		/// Downward message executed with the given outcome.
		ExecutedDownward { message_id: MessageId, outcome: Outcome },
		/// The weight limit for handling downward messages was reached.
		WeightExhausted { message_id: MessageId, remaining_weight: Weight, required_weight: Weight },
		/// Downward message is overweight and was placed in the overweight queue.
		OverweightEnqueued {
			message_id: MessageId,
			overweight_index: OverweightIndex,
			required_weight: Weight,
		},
		/// Downward message from the overweight queue was executed.
		OverweightServiced { overweight_index: OverweightIndex, weight_used: Weight },
	}

	impl<T: Config> Pallet<T> {
		/// Attempt to service an individual message. Will return `Ok` with the execution weight
		/// consumed unless the message was found to need more weight than `limit`.
		///
		/// NOTE: This will return `Ok` in the case of an error decoding, weighing or executing
		/// the message. This is why it's called message "servicing" rather than "execution".
		pub(crate) fn try_service_message(
			limit: Weight,
			_sent_at: RelayBlockNumber,
			mut data: &[u8],
		) -> Result<Weight, (MessageId, Weight)> {
			let message_id = sp_io::hashing::blake2_256(data);
			let maybe_msg = VersionedXcm::<T::Call>::decode_all_with_depth_limit(
				MAX_XCM_DECODE_DEPTH,
				&mut data,
			)
			.map(Xcm::<T::Call>::try_from);
			match maybe_msg {
				Err(_) => {
					Self::deposit_event(Event::InvalidFormat { message_id });
					Ok(0)
				},
				Ok(Err(())) => {
					Self::deposit_event(Event::UnsupportedVersion { message_id });
					Ok(0)
				},
				Ok(Ok(x)) => {
					let outcome = T::XcmExecutor::execute_xcm(Parent, x, limit);
					match outcome {
						Outcome::Error(XcmError::WeightLimitReached(required)) =>
							Err((message_id, required)),
						outcome => {
							let weight_used = outcome.weight_used();
							Self::deposit_event(Event::ExecutedDownward { message_id, outcome });
							Ok(weight_used)
						},
					}
				},
			}
		}
	}

	/// For an incoming downward message, this just adapts an XCM executor and executes DMP messages
	/// immediately up until some `MaxWeight` at which point it errors. Their origin is asserted to be
	/// the `Parent` location.
	impl<T: Config> DmpMessageHandler for Pallet<T> {
		fn handle_dmp_messages(
			// QQQ: This no longer looks appropriate, as we reconstruct the `InboundDownardMessage`. Should we change it
			// to `InboundDownardMessage`?
			iter: impl Iterator<Item = (RelayBlockNumber, Vec<u8>)>,
			context: &mut DmpMessageHandlerContext,
		) -> Weight {
			let mut page_index = PageIndex::<T>::get();
			let config = Configuration::<T>::get();
			let mut used = 0;

			for (_, (sent_at, data)) in iter.enumerate() {
				let remaining_weight = context.max_weight.saturating_sub(used);

				let new_head = context.mqc_head.extend(sent_at, BlakeTwo256::hash_of(&data));

				// Try to execute the message, abort if we run out of weight.
				match Self::try_service_message(remaining_weight, sent_at, &data[..]) {
					Ok(consumed) => {
						used += consumed;
						context.next_message_index += 1;
						// This head is valid for `next_message_index` - 1.
						context.mqc_head = new_head;
					},
					Err((message_id, required_weight)) =>
					// Executing this one inline is not possible.
					{
						// We are optimistic, even if this message doesn't fit we don't block here and
						// try to execute the next messages.
						if required_weight > config.max_individual {
							// Add this overweight message to the queue and continue with message execution.
							// The overweight queue is serviced by `ExecuteOverweightOrigin` call.
							let overweight_index = page_index.overweight_count;
							Overweight::<T>::insert(overweight_index, (sent_at, data));
							Self::deposit_event(Event::OverweightEnqueued {
								message_id,
								overweight_index,
								required_weight,
							});
							page_index.overweight_count += 1;

							context.mqc_head = new_head;
							context.next_message_index += 1;

							// Not needed for control flow, but only to ensure that the compiler
							// understands that we won't attempt to re-use `data` later.
							continue
						} else {
							Self::deposit_event(Event::WeightExhausted {
								message_id,
								remaining_weight,
								required_weight,
							});
							// Don't advance MQC head since we didn't process this message.
							break
						}
					},
				}
			}
			PageIndex::<T>::put(page_index);

			used
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate as dmp_queue;

	use codec::Encode;
	use cumulus_primitives_core::ParaId;
	use frame_support::{assert_noop, parameter_types};
	use sp_core::H256;
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentityLookup},
		DispatchError::BadOrigin,
	};
	use sp_version::RuntimeVersion;
	use std::cell::RefCell;
	use xcm::latest::{MultiLocation, OriginKind};

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
	type Block = frame_system::mocking::MockBlock<Test>;
	type Xcm = xcm::latest::Xcm<Call>;

	frame_support::construct_runtime!(
		pub enum Test where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
			DmpQueue: dmp_queue::{Pallet, Call, Storage, Event<T>},
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
			state_version: 1,
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
		type BaseCallFilter = frame_support::traits::Everything;
		type SystemWeightInfo = ();
		type SS58Prefix = ();
		type OnSetCode = ();
		type MaxConsumers = frame_support::traits::ConstU32<16>;
	}

	thread_local! {
		pub static TRACE: RefCell<Vec<(Xcm, Outcome)>> = RefCell::new(Vec::new());
	}
	pub fn take_trace() -> Vec<(Xcm, Outcome)> {
		TRACE.with(|q| {
			let q = &mut *q.borrow_mut();
			let r = q.clone();
			q.clear();
			r
		})
	}

	pub struct MockExec;
	impl ExecuteXcm<Call> for MockExec {
		fn execute_xcm_in_credit(
			_origin: impl Into<MultiLocation>,
			message: Xcm,
			weight_limit: Weight,
			_credit: Weight,
		) -> Outcome {
			let o = match (message.0.len(), &message.0.first()) {
				(1, Some(Transact { require_weight_at_most, .. })) => {
					if *require_weight_at_most <= weight_limit {
						Outcome::Complete(*require_weight_at_most)
					} else {
						Outcome::Error(XcmError::WeightLimitReached(*require_weight_at_most))
					}
				},
				// use 1000 to decide that it's not supported.
				_ => Outcome::Incomplete(1000.min(weight_limit), XcmError::Unimplemented),
			};
			TRACE.with(|q| q.borrow_mut().push((message, o.clone())));
			o
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

	fn handle_messages(incoming: &[Xcm], limit: Weight) -> Weight {
		let iter = incoming.iter().map(|m| (0, VersionedXcm::<Call>::from(m.clone()).encode()));
		let mut context = DmpMessageHandlerContext {
			max_weight: limit,
			next_message_index: sp_std::num::Wrapping(0),
			mqc_head: Hash::zero(),
		};
		DmpQueue::handle_dmp_messages(iter, &mut context)
	}

	fn msg(weight: Weight) -> Xcm {
		Xcm(vec![Transact {
			origin_type: OriginKind::Native,
			require_weight_at_most: weight,
			call: Vec::new().into(),
		}])
	}

	fn msg_complete(weight: Weight) -> (Xcm, Outcome) {
		(msg(weight), Outcome::Complete(weight))
	}

	fn msg_limit_reached(weight: Weight) -> (Xcm, Outcome) {
		(msg(weight), Outcome::Error(XcmError::WeightLimitReached(weight)))
	}

	fn pages_queued() -> PageCounter {
		PageIndex::<Test>::get().end_used - PageIndex::<Test>::get().begin_used
	}

	fn queue_is_empty() -> bool {
		pages_queued() == 0
	}

	fn overweights() -> Vec<OverweightIndex> {
		(0..PageIndex::<Test>::get().overweight_count)
			.filter(|i| Overweight::<Test>::contains_key(i))
			.collect::<Vec<_>>()
	}

	#[test]
	fn basic_setup_works() {
		new_test_ext().execute_with(|| {
			let weight_used = handle_messages(&[], 1000);
			assert_eq!(weight_used, 0);
			assert_eq!(take_trace(), Vec::new());
			assert!(queue_is_empty());
		});
	}

	#[test]
	fn service_inline_complete_works() {
		new_test_ext().execute_with(|| {
			let incoming = vec![msg(1000), msg(1001)];
			let weight_used = handle_messages(&incoming, 2500);
			assert_eq!(weight_used, 2001);
			assert_eq!(take_trace(), vec![msg_complete(1000), msg_complete(1001)]);
			assert!(queue_is_empty());
		});
	}

	#[test]
	fn service_inline() {
		new_test_ext().execute_with(|| {
			let incoming = vec![msg(1000), msg(1001), msg(1002)];
			let weight_used = handle_messages(&incoming, 1500);
			assert_eq!(weight_used, 1000);
			assert_eq!(pages_queued(), 0);
			assert_eq!(take_trace(), vec![msg_complete(1000), msg_limit_reached(1001),]);

			let weight_used = handle_messages(&[msg(1001), msg(1002)], 2500);
			assert_eq!(weight_used, 2003);
			assert_eq!(take_trace(), vec![msg_complete(1001), msg_complete(1002),]);
		});
	}

	#[test]
	fn overweight_should_not_block_queue() {
		new_test_ext().execute_with(|| {
			// Set the overweight threshold to 9999.
			Configuration::<Test>::put(ConfigData { max_individual: 9999 });

			let incoming = vec![msg(1000), msg(10001), msg(1002), msg(20000), msg(100), msg(500)];
			let weight_used = handle_messages(&incoming, 2500);
			assert_eq!(weight_used, 2102);
			assert!(queue_is_empty());
			assert_eq!(
				take_trace(),
				vec![
					msg_complete(1000),
					msg_limit_reached(10001),
					msg_complete(1002),
					msg_limit_reached(20000),
					msg_complete(100),
					msg_limit_reached(500)
				]
			);

			// We don't queue when messages that are below the threshold `max_individual` even when we run out of weight.
			assert!(queue_is_empty());

			// There must be two overweight messages > 9999.
			assert_eq!(overweights(), vec![0, 1]);
		});
	}

	#[test]
	fn overweights_should_be_manually_executable() {
		new_test_ext().execute_with(|| {
			// Set the overweight threshold to 9999.
			Configuration::<Test>::put(ConfigData { max_individual: 9999 });

			let incoming = vec![msg(10000)];
			let weight_used = handle_messages(&incoming, 2500);
			assert_eq!(weight_used, 0);
			assert_eq!(take_trace(), vec![msg_limit_reached(10000)]);
			assert_eq!(overweights(), vec![0]);

			assert_noop!(DmpQueue::service_overweight(Origin::signed(1), 0, 20000), BadOrigin);
			assert_noop!(
				DmpQueue::service_overweight(Origin::root(), 1, 20000),
				Error::<Test>::Unknown
			);
			assert_noop!(
				DmpQueue::service_overweight(Origin::root(), 0, 9999),
				Error::<Test>::OverLimit
			);
			assert_eq!(take_trace(), vec![msg_limit_reached(10000)]);

			let base_weight = super::Call::<Test>::service_overweight { index: 0, weight_limit: 0 }
				.get_dispatch_info()
				.weight;
			use frame_support::weights::GetDispatchInfo;
			let info = DmpQueue::service_overweight(Origin::root(), 0, 20000).unwrap();
			let actual_weight = info.actual_weight.unwrap();
			assert_eq!(actual_weight, base_weight + 10000);
			assert_eq!(take_trace(), vec![msg_complete(10000)]);
			assert!(overweights().is_empty());

			assert_noop!(
				DmpQueue::service_overweight(Origin::root(), 0, 20000),
				Error::<Test>::Unknown
			);
		});
	}
}
