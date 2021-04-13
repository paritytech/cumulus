// Copyright 2020 Parity Technologies (UK) Ltd.
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

#![cfg_attr(not(feature = "std"), no_std)]

//! cumulus-pallet-parachain-system is a base pallet for cumulus-based parachains.
//!
//! This pallet handles low-level details of being a parachain. It's responsibilities include:
//!
//! - ingestion of the parachain validation data
//! - ingestion of incoming downward and lateral messages and dispatching them
//! - coordinating upgrades with the relay-chain
//! - communication of parachain outputs, such as sent messages, signalling an upgrade, etc.
//!
//! Users must ensure that they register this pallet as an inherent provider.

use cumulus_primitives_core::{
	relay_chain,
	well_known_keys::{self, NEW_VALIDATION_CODE},
	AbridgedHostConfiguration, DownwardMessageHandler, HrmpMessageHandler, HrmpMessageSender,
	InboundDownwardMessage, InboundHrmpMessage, OnValidationData, OutboundHrmpMessage, ParaId,
	PersistedValidationData, UpwardMessage, UpwardMessageSender,
};
use cumulus_primitives_parachain_inherent::ParachainInherentData;
use polkadot_parachain::primitives::RelayChainBlockNumber;
use relay_state_snapshot::MessagingStateSnapshot;
use sp_runtime::traits::{BlakeTwo256, Hash};
use sp_std::{cmp, collections::btree_map::BTreeMap, vec::Vec};

mod relay_state_snapshot;
#[cfg(test)]
mod tests;
#[macro_use]
pub mod validate_block;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	/// The pallet's configuration trait.
	pub trait Config: frame_system::Config<OnSetCode = ParachainSetCode<Self>> {
		/// The overarching event type.
		type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;

		/// Something which can be notified when the validation data is set.
		type OnValidationData: OnValidationData;

		/// Returns the parachain ID we are running with.
		type SelfParaId: Get<ParaId>;

		/// The downward message handlers that will be informed when a message is received.
		type DownwardMessageHandlers: DownwardMessageHandler;

		/// The HRMP message handlers that will be informed when a message is received.
		///
		/// The messages are dispatched in the order they were relayed by the relay chain. If multiple
		/// messages were relayed at one block, these will be dispatched in ascending order of the sender's para ID.
		type HrmpMessageHandlers: HrmpMessageHandler;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(_: T::BlockNumber) {
			<DidSetValidationCode<T>>::take();

			let host_config = Self::host_configuration()
				.expect("host configuration is promised to set until `on_finalize`; qed");
			let relevant_messaging_state = Self::relevant_messaging_state()
				.expect("relevant messaging state is promised to be set until `on_finalize`; qed");

			<PendingUpwardMessages<T>>::mutate(|up| {
				let (count, size) = relevant_messaging_state.relay_dispatch_queue_size;

				let available_capacity = cmp::min(
					host_config.max_upward_queue_count.saturating_sub(count),
					host_config.max_upward_message_num_per_candidate,
				);
				let available_size = host_config.max_upward_queue_size.saturating_sub(size);

				// Count the number of messages we can possibly fit in the given constraints, i.e.
				// available_capacity and available_size.
				let num = up
					.iter()
					.scan(
						(available_capacity as usize, available_size as usize),
						|state, msg| {
							let (cap_left, size_left) = *state;
							match (cap_left.checked_sub(1), size_left.checked_sub(msg.len())) {
								(Some(new_cap), Some(new_size)) => {
									*state = (new_cap, new_size);
									Some(())
								}
								_ => None,
							}
						},
					)
					.count();

				// TODO: #274 Return back messages that do not longer fit into the queue.

				storage::unhashed::put(well_known_keys::UPWARD_MESSAGES, &up[0..num]);
				*up = up.split_off(num);
			});

			// Sending HRMP messages is a little bit more involved. There are the following
			// constraints:
			//
			// - a channel should exist (and it can be closed while a message is buffered),
			// - at most one message can be sent in a channel,
			// - the sent out messages should be ordered by ascension of recipient para id.
			// - the capacity and total size of the channel is limited,
			// - the maximum size of a message is limited (and can potentially be changed),

			let mut non_empty_hrmp_channels = <NonEmptyHrmpChannels<T>>::get();
			// The number of messages we can send is limited by all of:
			// - the number of non empty channels
			// - the maximum number of messages per candidate according to the fresh config
			// - the maximum number of messages per candidate according to the stale config
			let outbound_hrmp_num = non_empty_hrmp_channels
				.len()
				.min(host_config.hrmp_max_message_num_per_candidate as usize)
				.min(<AnnouncedHrmpMessagesPerCandidate<T>>::take() as usize);

			let mut outbound_hrmp_messages = Vec::with_capacity(outbound_hrmp_num);
			let mut prune_empty = Vec::with_capacity(outbound_hrmp_num);

			for &recipient in non_empty_hrmp_channels.iter() {
				if outbound_hrmp_messages.len() == outbound_hrmp_num {
					// We have picked the required number of messages for the batch, no reason to
					// iterate further.
					//
					// We check this condition in the beginning of the loop so that we don't include
					// a message where the limit is 0.
					break;
				}

				let idx = match relevant_messaging_state
					.egress_channels
					.binary_search_by_key(&recipient, |(recipient, _)| *recipient)
				{
					Ok(m) => m,
					Err(_) => {
						// TODO: #274 This means that there is no such channel anymore. Means that we should
						// return back the messages from this channel.
						//
						// Until then pretend it became empty
						prune_empty.push(recipient);
						continue;
					}
				};

				let channel_meta = &relevant_messaging_state.egress_channels[idx].1;
				if channel_meta.msg_count + 1 > channel_meta.max_capacity {
					// The channel is at its capacity. Skip it for now.
					continue;
				}

				let mut pending = <OutboundHrmpMessages<T>>::get(&recipient);

				// This panics if `v` is empty. However, we are iterating only once over non-empty
				// channels, therefore it cannot panic.
				let message_payload = pending.remove(0);
				let became_empty = pending.is_empty();

				if channel_meta.total_size + message_payload.len() as u32
					> channel_meta.max_total_size
				{
					// Sending this message will make the channel total size overflow. Skip it for now.
					continue;
				}

				// If we reached here, then the channel has capacity to receive this message. However,
				// it doesn't mean that we are sending it just yet.
				if became_empty {
					<OutboundHrmpMessages<T>>::remove(&recipient);
					prune_empty.push(recipient);
				} else {
					<OutboundHrmpMessages<T>>::insert(&recipient, pending);
				}

				if message_payload.len() as u32 > channel_meta.max_message_size {
					// Apparently, the max message size was decreased since the message while the
					// message was buffered. While it's possible to make another iteration to fetch
					// the next message, we just keep going here to not complicate the logic too much.
					//
					// TODO: #274 Return back this message to sender.
					continue;
				}

				outbound_hrmp_messages.push(OutboundHrmpMessage {
					recipient,
					data: message_payload,
				});
			}

			// Sort the outbound messages by asceding recipient para id to satisfy the acceptance
			// criteria requirement.
			outbound_hrmp_messages.sort_by_key(|m| m.recipient);

			// Prune hrmp channels that became empty. Additionally, because it may so happen that we
			// only gave attention to some channels in `non_empty_hrmp_channels` it's important to
			// change the order. Otherwise, the next `on_finalize` we will again give attention
			// only to those channels that happen to be in the beginning, until they are emptied.
			// This leads to "starvation" of the channels near to the end.
			//
			// To mitigate this we shift all processed elements towards the end of the vector using
			// `rotate_left`. To get intution how it works see the examples in its rustdoc.
			non_empty_hrmp_channels.retain(|x| !prune_empty.contains(x));
			// `prune_empty.len()` is greater or equal to `outbound_hrmp_num` because the loop above
			// can only do `outbound_hrmp_num` iterations and `prune_empty` is appended to only inside
			// the loop body.
			non_empty_hrmp_channels.rotate_left(outbound_hrmp_num - prune_empty.len());

			<NonEmptyHrmpChannels<T>>::put(non_empty_hrmp_channels);
			storage::unhashed::put(
				well_known_keys::HRMP_OUTBOUND_MESSAGES,
				&outbound_hrmp_messages,
			);
		}

		fn on_initialize(_: T::BlockNumber) -> Weight {
			// To prevent removing `NEW_VALIDATION_CODE` that was set by another `on_initialize` like
			// for example from scheduler, we only kill the storage entry if it was not yet updated
			// in the current block.
			if !<DidSetValidationCode<T>>::get() {
				storage::unhashed::kill(NEW_VALIDATION_CODE);
			}

			// Remove the validation from the old block.
			<ValidationData<T>>::kill();

			let mut weight = T::DbWeight::get().writes(3);
			storage::unhashed::kill(well_known_keys::HRMP_WATERMARK);
			storage::unhashed::kill(well_known_keys::UPWARD_MESSAGES);
			storage::unhashed::kill(well_known_keys::HRMP_OUTBOUND_MESSAGES);

			// Here, in `on_initialize` we must report the weight for both `on_initialize` and
			// `on_finalize`.
			//
			// One complication here, is that the `host_configuration` is updated by an inherent and
			// those are processed after the block initialization phase. Therefore, we have to be
			// content only with the configuration as per the previous block. That means that
			// the configuration can be either stale (or be abscent altogether in case of the
			// beginning of the chain).
			//
			// In order to mitigate this, we do the following. At the time, we are only concerned
			// about `hrmp_max_message_num_per_candidate`. We reserve the amount of weight to process
			// the number of HRMP messages according to the potentially stale configuration. In
			// `on_finalize` we will process only the maximum between the announced number of messages
			// and the actual received in the fresh configuration.
			//
			// In the common case, they will be the same. In the case the actual value is smaller
			// than the announced, we would waste some of weight. In the case the actual value is
			// greater than the announced, we will miss opportunity to send a couple of messages.
			weight += T::DbWeight::get().reads_writes(1, 1);
			let hrmp_max_message_num_per_candidate = Self::host_configuration()
				.map(|cfg| cfg.hrmp_max_message_num_per_candidate)
				.unwrap_or(0);
			<AnnouncedHrmpMessagesPerCandidate<T>>::put(hrmp_max_message_num_per_candidate);

			// NOTE that the actual weight consumed by `on_finalize` may turn out lower.
			weight += T::DbWeight::get().reads_writes(
				3 + hrmp_max_message_num_per_candidate as u64,
				4 + hrmp_max_message_num_per_candidate as u64,
			);

			weight
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Force an already scheduled validation function upgrade to happen on a particular block.
		///
		/// Note that coordinating this block for the upgrade has to happen independently on the relay
		/// chain and this parachain. Synchronizing the block for the upgrade is sensitive, and this
		/// bypasses all checks and and normal protocols. Very easy to brick your chain if done wrong.
		#[pallet::weight((0, DispatchClass::Operational))]
		pub fn set_upgrade_block(
			origin: OriginFor<T>,
			relay_chain_block: RelayChainBlockNumber,
		) -> DispatchResult {
			ensure_root(origin)?;
			if <PendingRelayChainBlockNumber<T>>::get().is_some() {
				<PendingRelayChainBlockNumber<T>>::put(relay_chain_block);
				Ok(())
			} else {
				Err(Error::<T>::NotScheduled.into())
			}
		}

		/// Set the current validation data.
		///
		/// This should be invoked exactly once per block. It will panic at the finalization
		/// phase if the call was not invoked.
		///
		/// The dispatch origin for this call must be `Inherent`
		///
		/// As a side effect, this function upgrades the current validation function
		/// if the appropriate time has come.
		#[pallet::weight((0, DispatchClass::Mandatory))]
		pub fn set_validation_data(
			origin: OriginFor<T>,
			data: ParachainInherentData,
		) -> DispatchResult {
			ensure_none(origin)?;
			assert!(
				!<ValidationData<T>>::exists(),
				"ValidationData must be updated only once in a block",
			);

			let ParachainInherentData {
				validation_data: vfp,
				relay_chain_state,
				downward_messages,
				horizontal_messages,
			} = data;

			Self::validate_validation_data(&vfp);

			// initialization logic: we know that this runs exactly once every block,
			// which means we can put the initialization logic here to remove the
			// sequencing problem.
			if let Some(apply_block) = <PendingRelayChainBlockNumber<T>>::get() {
				if vfp.relay_parent_number >= apply_block {
					<PendingRelayChainBlockNumber<T>>::kill();
					let validation_function = <PendingValidationFunction<T>>::take();
					<LastUpgrade<T>>::put(&apply_block);
					Self::put_parachain_code(&validation_function);
					Self::deposit_event(Event::ValidationFunctionApplied(vfp.relay_parent_number));
				}
			}

			let (host_config, relevant_messaging_state) = relay_state_snapshot::extract_from_proof(
				T::SelfParaId::get(),
				vfp.relay_parent_storage_root,
				relay_chain_state,
			)
			.map_err(|err| {
				log::debug!("invalid relay chain merkle proof: {:?}", err);
				Error::<T>::InvalidRelayChainMerkleProof
			})?;

			<ValidationData<T>>::put(&vfp);
			<RelevantMessagingState<T>>::put(relevant_messaging_state.clone());
			<HostConfiguration<T>>::put(host_config);

			<T::OnValidationData as OnValidationData>::on_validation_data(&vfp);

			Self::process_inbound_downward_messages(
				relevant_messaging_state.dmq_mqc_head,
				downward_messages,
			)?;
			Self::process_inbound_horizontal_messages(
				&relevant_messaging_state.ingress_channels,
				horizontal_messages,
			)?;

			Ok(())
		}

		#[pallet::weight((1_000, DispatchClass::Operational))]
		fn sudo_send_upward_message(
			origin: OriginFor<T>,
			message: UpwardMessage,
		) -> DispatchResult {
			ensure_root(origin)?;
			let _ = Self::send_upward_message(message);
			Ok(())
		}

		#[pallet::weight((1_000, DispatchClass::Operational))]
		fn sudo_send_hrmp_message(
			origin: OriginFor<T>,
			message: OutboundHrmpMessage,
		) -> DispatchResult {
			ensure_root(origin)?;
			let _ = Self::send_hrmp_message(message);
			Ok(())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event {
		// The validation function has been scheduled to apply as of the contained relay chain block number.
		ValidationFunctionStored(RelayChainBlockNumber),
		// The validation function was applied as of the contained relay chain block number.
		ValidationFunctionApplied(RelayChainBlockNumber),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Attempt to upgrade validation function while existing upgrade pending
		OverlappingUpgrades,
		/// Polkadot currently prohibits this parachain from upgrading its validation function
		ProhibitedByPolkadot,
		/// The supplied validation function has compiled into a blob larger than Polkadot is willing to run
		TooBig,
		/// The inherent which supplies the validation data did not run this block
		ValidationDataNotAvailable,
		/// The inherent which supplies the host configuration did not run this block
		HostConfigurationNotAvailable,
		/// Invalid relay-chain storage merkle proof
		InvalidRelayChainMerkleProof,
		/// The messages submitted by the collator in the system inherent when hashed sequentially
		/// do not produce the hash that is produced by the relay-chain.
		///
		/// This means that at least some of the submitted messages were altered, omitted or added
		/// illegaly.
		DmpMqcMismatch,
		/// The collator submitted a message that is received from a sender that doesn't have a
		/// channel opened to this parachain, according to the relay-parent state.
		HrmpNoMqc,
		/// After processing all messages submitted by the collator and extending hash chains we
		/// haven't arrived to the MQCs that were produced by the relay-chain.
		///
		/// That means that one or more channels had at least some of the submitted messages altered,
		/// omitted or added illegaly.
		HrmpMqcMismatch,
		/// No validation function upgrade is currently scheduled.
		NotScheduled,
	}

	/// We need to store the new validation function for the span between
	/// setting it and applying it. If it has a
	/// value, then [`PendingValidationFunction`] must have a real value, and
	/// together will coordinate the block number where the upgrade will happen.
	#[pallet::storage]
	pub(super) type PendingRelayChainBlockNumber<T: Config> =
		StorageValue<_, RelayChainBlockNumber>;

	/// The new validation function we will upgrade to when the relay chain
	/// reaches [`PendingRelayChainBlockNumber`]. A real validation function must
	/// exist here as long as [`PendingRelayChainBlockNumber`] is set.
	#[pallet::storage]
	#[pallet::getter(fn new_validation_function)]
	pub(super) type PendingValidationFunction<T: Config> = StorageValue<_, Vec<u8>, ValueQuery>;

	/// The [`PersistedValidationData`] set for this block.
	#[pallet::storage]
	#[pallet::getter(fn validation_data)]
	pub(super) type ValidationData<T: Config> = StorageValue<_, PersistedValidationData>;

	/// Were the validation data set to notify the relay chain?
	#[pallet::storage]
	pub(super) type DidSetValidationCode<T: Config> = StorageValue<_, bool, ValueQuery>;

	/// The last relay parent block number at which we signalled the code upgrade.
	#[pallet::storage]
	pub(super) type LastUpgrade<T: Config> = StorageValue<_, relay_chain::BlockNumber, ValueQuery>;

	/// The snapshot of some state related to messaging relevant to the current parachain as per
	/// the relay parent.
	///
	/// This field is meant to be updated each block with the validation data inherent. Therefore,
	/// before processing of the inherent, e.g. in `on_initialize` this data may be stale.
	///
	/// This data is also absent from the genesis.
	#[pallet::storage]
	#[pallet::getter(fn relevant_messaging_state)]
	pub(super) type RelevantMessagingState<T: Config> = StorageValue<_, MessagingStateSnapshot>;

	/// The parachain host configuration that was obtained from the relay parent.
	///
	/// This field is meant to be updated each block with the validation data inherent. Therefore,
	/// before processing of the inherent, e.g. in `on_initialize` this data may be stale.
	///
	/// This data is also absent from the genesis.
	#[pallet::storage]
	#[pallet::getter(fn host_configuration)]
	pub(super) type HostConfiguration<T: Config> = StorageValue<_, AbridgedHostConfiguration>;

	/// The last downward message queue chain head we have observed.
	///
	/// This value is loaded before and saved after processing inbound downward messages carried
	/// by the system inherent.
	#[pallet::storage]
	pub(super) type LastDmqMqcHead<T: Config> = StorageValue<_, MessageQueueChain, ValueQuery>;

	/// The message queue chain heads we have observed per each channel incoming channel.
	///
	/// This value is loaded before and saved after processing inbound downward messages carried
	/// by the system inherent.
	#[pallet::storage]
	pub(super) type LastHrmpMqcHeads<T: Config> =
		StorageValue<_, BTreeMap<ParaId, MessageQueueChain>, ValueQuery>;

	#[pallet::storage]
	pub(super) type PendingUpwardMessages<T: Config> =
		StorageValue<_, Vec<UpwardMessage>, ValueQuery>;

	/// Essentially `OutboundHrmpMessage`s grouped by the recipients.
	#[pallet::storage]
	pub(super) type OutboundHrmpMessages<T: Config> =
		StorageMap<_, Twox64Concat, ParaId, Vec<Vec<u8>>, ValueQuery>;

	/// HRMP channels with the given recipients are awaiting to be processed. If a `ParaId` is
	/// present in this vector then `OutboundHrmpMessages` for it should be not empty.
	#[pallet::storage]
	pub(super) type NonEmptyHrmpChannels<T: Config> = StorageValue<_, Vec<ParaId>, ValueQuery>;

	/// The number of HRMP messages we observed in `on_initialize` and thus used that number for
	/// announcing the weight of `on_initialize` and `on_finialize`.
	#[pallet::storage]
	pub(super) type AnnouncedHrmpMessagesPerCandidate<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::inherent]
	impl<T: Config> ProvideInherent for Pallet<T> {
		type Call = Call<T>;
		type Error = sp_inherents::MakeFatalError<()>;
		const INHERENT_IDENTIFIER: InherentIdentifier =
			cumulus_primitives_parachain_inherent::INHERENT_IDENTIFIER;

		fn create_inherent(data: &InherentData) -> Option<Self::Call> {
			let data: ParachainInherentData = data
				.get_data(&Self::INHERENT_IDENTIFIER)
				.ok()
				.flatten()
				.expect("validation function params are always injected into inherent data; qed");

			Some(Call::set_validation_data(data))
		}
	}

	impl<T: Config> Pallet<T> {
		/// Validate the given [`PersistedValidationData`] against the
		/// [`ValidationParams`](polkadot_parachain::primitives::ValidationParams).
		///
		/// This check will only be executed when the block is currently being executed in the context
		/// of [`validate_block`]. If this is being executed in the context of block building or block
		/// import, this is a no-op.
		///
		/// # Panics
		fn validate_validation_data(validation_data: &PersistedValidationData) {
			validate_block::with_validation_params(|params| {
				assert_eq!(
					params.parent_head, validation_data.parent_head,
					"Parent head doesn't match"
				);
				assert_eq!(
					params.relay_parent_number, validation_data.relay_parent_number,
					"Relay parent number doesn't match",
				);
				assert_eq!(
					params.relay_parent_storage_root, validation_data.relay_parent_storage_root,
					"Relay parent stoarage root doesn't match",
				);
			});
		}

		/// Process all inbound downward messages relayed by the collator.
		///
		/// Checks if the sequence of the messages is valid, dispatches them and communicates the number
		/// of processed messages to the collator via a storage update.
		fn process_inbound_downward_messages(
			expected_dmq_mqc_head: relay_chain::Hash,
			downward_messages: Vec<InboundDownwardMessage>,
		) -> DispatchResult {
			let dm_count = downward_messages.len() as u32;

			let result_mqc_head = <LastDmqMqcHead<T>>::mutate(move |mqc| {
				for downward_message in downward_messages {
					mqc.extend_downward(&downward_message);
					T::DownwardMessageHandlers::handle_downward_message(downward_message);
				}
				mqc.0
			});

			// After hashing each message in the message queue chain submitted by the collator, we should
			// arrive to the MQC head provided by the relay chain.
			ensure!(
				result_mqc_head == expected_dmq_mqc_head,
				Error::<T>::DmpMqcMismatch
			);

			// Store the processed_downward_messages here so that it will be accessible from
			// PVF's `validate_block` wrapper and collation pipeline.
			storage::unhashed::put(well_known_keys::PROCESSED_DOWNWARD_MESSAGES, &dm_count);

			Ok(())
		}

		/// Process all inbound horizontal messages relayed by the collator.
		///
		/// This is similar to [`process_inbound_downward_messages`], but works on multiple inbound
		/// channels.
		fn process_inbound_horizontal_messages(
			ingress_channels: &[(ParaId, cumulus_primitives_core::AbridgedHrmpChannel)],
			horizontal_messages: BTreeMap<ParaId, Vec<InboundHrmpMessage>>,
		) -> DispatchResult {
			// First, check that all submitted messages are sent from channels that exist. The channel
			// exists if its MQC head is present in `vfp.hrmp_mqc_heads`.
			for sender in horizontal_messages.keys() {
				ensure!(
					ingress_channels
						.binary_search_by_key(sender, |&(s, _)| s)
						.is_ok(),
					Error::<T>::HrmpNoMqc,
				);
			}

			// Second, prepare horizontal messages for a more convenient processing:
			//
			// instead of a mapping from a para to a list of inbound HRMP messages, we will have a list
			// of tuples `(sender, message)` first ordered by `sent_at` (the relay chain block number
			// in which the message hit the relay-chain) and second ordered by para id ascending.
			//
			// The messages will be dispatched in this order.
			let mut horizontal_messages = horizontal_messages
				.into_iter()
				.flat_map(|(sender, channel_contents)| {
					channel_contents
						.into_iter()
						.map(move |message| (sender, message))
				})
				.collect::<Vec<_>>();
			horizontal_messages.sort_by(|a, b| {
				// first sort by sent-at and then by the para id
				match a.1.sent_at.cmp(&b.1.sent_at) {
					cmp::Ordering::Equal => a.0.cmp(&b.0),
					ord => ord,
				}
			});

			let last_mqc_heads = <LastHrmpMqcHeads<T>>::get();
			let mut running_mqc_heads = BTreeMap::new();
			let mut hrmp_watermark = None;

			for (sender, horizontal_message) in horizontal_messages {
				if hrmp_watermark
					.map(|w| w < horizontal_message.sent_at)
					.unwrap_or(true)
				{
					hrmp_watermark = Some(horizontal_message.sent_at);
				}

				running_mqc_heads
					.entry(sender)
					.or_insert_with(|| last_mqc_heads.get(&sender).cloned().unwrap_or_default())
					.extend_hrmp(&horizontal_message);

				T::HrmpMessageHandlers::handle_hrmp_message(sender, horizontal_message);
			}

			// Check that the MQC heads for each channel provided by the relay chain match the MQC heads
			// we have after processing all incoming messages.
			//
			// Along the way we also carry over the relevant entries from the `last_mqc_heads` to
			// `running_mqc_heads`. Otherwise, in a block where no messages were sent in a channel
			// it won't get into next block's `last_mqc_heads` and thus will be all zeros, which
			// would corrupt the message queue chain.
			for &(ref sender, ref channel) in ingress_channels {
				let cur_head = running_mqc_heads
					.entry(*sender)
					.or_insert_with(|| last_mqc_heads.get(&sender).cloned().unwrap_or_default())
					.head();
				let target_head = channel.mqc_head.unwrap_or_default();

				ensure!(cur_head == target_head, Error::<T>::HrmpMqcMismatch);
			}

			<LastHrmpMqcHeads<T>>::put(running_mqc_heads);

			// If we processed at least one message, then advance watermark to that location.
			if let Some(hrmp_watermark) = hrmp_watermark {
				storage::unhashed::put(well_known_keys::HRMP_WATERMARK, &hrmp_watermark);
			}

			Ok(())
		}

		/// Put a new validation function into a particular location where polkadot
		/// monitors for updates. Calling this function notifies polkadot that a new
		/// upgrade has been scheduled.
		fn notify_polkadot_of_pending_upgrade(code: &[u8]) {
			storage::unhashed::put_raw(NEW_VALIDATION_CODE, code);
			<DidSetValidationCode<T>>::put(true);
		}

		/// Put a new validation function into a particular location where this
		/// parachain will execute it on subsequent blocks.
		fn put_parachain_code(code: &[u8]) {
			storage::unhashed::put_raw(sp_core::storage::well_known_keys::CODE, code);
		}

		/// The maximum code size permitted, in bytes.
		///
		/// Returns `None` if the relay chain parachain host configuration hasn't been submitted yet.
		pub fn max_code_size() -> Option<u32> {
			<HostConfiguration<T>>::get().map(|cfg| cfg.max_code_size)
		}

		/// Returns if a PVF/runtime upgrade could be signalled at the current block, and if so
		/// when the new code will take the effect.
		fn code_upgrade_allowed(
			vfp: &PersistedValidationData,
			cfg: &AbridgedHostConfiguration,
		) -> Option<relay_chain::BlockNumber> {
			if <PendingRelayChainBlockNumber<T>>::get().is_some() {
				// There is already upgrade scheduled. Upgrade is not allowed.
				return None;
			}

			let relay_blocks_since_last_upgrade = vfp
				.relay_parent_number
				.saturating_sub(<LastUpgrade<T>>::get());

			if relay_blocks_since_last_upgrade <= cfg.validation_upgrade_frequency {
				// The cooldown after the last upgrade hasn't elapsed yet. Upgrade is not allowed.
				return None;
			}

			Some(vfp.relay_parent_number + cfg.validation_upgrade_delay)
		}

		/// The implementation of the runtime upgrade functionality for parachains.
		fn set_code_impl(validation_function: Vec<u8>) -> DispatchResult {
			ensure!(
				!<PendingValidationFunction<T>>::exists(),
				Error::<T>::OverlappingUpgrades
			);
			let vfp = Self::validation_data().ok_or(Error::<T>::ValidationDataNotAvailable)?;
			let cfg =
				Self::host_configuration().ok_or(Error::<T>::HostConfigurationNotAvailable)?;
			ensure!(
				validation_function.len() <= cfg.max_code_size as usize,
				Error::<T>::TooBig
			);
			let apply_block =
				Self::code_upgrade_allowed(&vfp, &cfg).ok_or(Error::<T>::ProhibitedByPolkadot)?;

			// When a code upgrade is scheduled, it has to be applied in two
			// places, synchronized: both polkadot and the individual parachain
			// have to upgrade on the same relay chain block.
			//
			// `notify_polkadot_of_pending_upgrade` notifies polkadot; the `PendingValidationFunction`
			// storage keeps track locally for the parachain upgrade, which will
			// be applied later.
			Self::notify_polkadot_of_pending_upgrade(&validation_function);
			<PendingRelayChainBlockNumber<T>>::put(apply_block);
			<PendingValidationFunction<T>>::put(validation_function);
			Self::deposit_event(Event::ValidationFunctionStored(apply_block));

			Ok(())
		}
	}

	pub struct ParachainSetCode<T>(sp_std::marker::PhantomData<T>);

	impl<T: Config> frame_system::SetCode for ParachainSetCode<T> {
		fn set_code(code: Vec<u8>) -> DispatchResult {
			Pallet::<T>::set_code_impl(code)
		}
	}

	/// This struct provides ability to extend a message queue chain (MQC) and compute a new head.
	///
	/// MQC is an instance of a [hash chain] applied to a message queue. Using a hash chain it's possible
	/// to represent a sequence of messages using only a single hash.
	///
	/// A head for an empty chain is agreed to be a zero hash.
	///
	/// [hash chain]: https://en.wikipedia.org/wiki/Hash_chain
	#[derive(Default, Clone, codec::Encode, codec::Decode)]
	pub(super) struct MessageQueueChain(relay_chain::Hash);

	impl MessageQueueChain {
		pub(super) fn extend_hrmp(&mut self, horizontal_message: &InboundHrmpMessage) -> &mut Self {
			let prev_head = self.0;
			self.0 = BlakeTwo256::hash_of(&(
				prev_head,
				horizontal_message.sent_at,
				BlakeTwo256::hash_of(&horizontal_message.data),
			));
			self
		}

		pub(super) fn extend_downward(
			&mut self,
			downward_message: &InboundDownwardMessage,
		) -> &mut Self {
			let prev_head = self.0;
			self.0 = BlakeTwo256::hash_of(&(
				prev_head,
				downward_message.sent_at,
				BlakeTwo256::hash_of(&downward_message.msg),
			));
			self
		}

		pub(super) fn head(&self) -> relay_chain::Hash {
			self.0
		}
	}

	/// An error that can be raised upon sending an upward message.
	#[derive(Debug, PartialEq)]
	pub enum SendUpErr {
		/// The message sent is too big.
		TooBig,
	}

	/// An error that can be raised upon sending a horizontal message.
	#[derive(Debug, PartialEq)]
	pub enum SendHorizontalErr {
		/// The message sent is too big.
		TooBig,
		/// There is no channel to the specified destination.
		NoChannel,
	}

	impl<T: Config> Pallet<T> {
		pub fn send_upward_message(message: UpwardMessage) -> Result<(), SendUpErr> {
			// Check if the message fits into the relay-chain constraints.
			//
			// Note, that we are using `host_configuration` here which may be from the previous
			// block, in case this is called from `on_initialize`, i.e. before the inherent with fresh
			// data is submitted.
			//
			// That shouldn't be a problem since this is a preliminary check and the actual check would
			// be performed just before submitting the message from the candidate, and it already can
			// happen that during the time the message is buffered for sending the relay-chain setting
			// may change so that the message is no longer valid.
			//
			// However, changing this setting is expected to be rare.
			match Self::host_configuration() {
				Some(cfg) => {
					if message.len() > cfg.max_upward_message_size as usize {
						return Err(SendUpErr::TooBig);
					}
				}
				None => {
					// This storage field should carry over from the previous block. So if it's None
					// then it must be that this is an edge-case where a message is attempted to be
					// sent at the first block.
					//
					// Let's pass this message through. I think it's not unreasonable to expect that the
					// message is not huge and it comes through, but if it doesn't it can be returned
					// back to the sender.
					//
					// Thus fall through here.
				}
			};
			<PendingUpwardMessages<T>>::append(message);
			Ok(())
		}

		pub fn send_hrmp_message(message: OutboundHrmpMessage) -> Result<(), SendHorizontalErr> {
			let OutboundHrmpMessage { recipient, data } = message;

			// First, check if the message is addressed into an opened channel.
			//
			// Note, that we are using `relevant_messaging_state` which may be from the previous
			// block, in case this is called from `on_initialize`, i.e. before the inherent with fresh
			// data is submitted.
			//
			// That shouldn't be a problem though because this is anticipated and already can happen.
			// This is because sending implies that a message is buffered until there is space to send
			// a message in the candidate. After a while waiting in a buffer, it may be discovered that
			// the channel to which a message were addressed is now closed. Another possibility, is that
			// the maximum message size was decreased so that a message in the bufer doesn't fit. Should
			// any of that happen the sender should be notified about the message was discarded.
			//
			// Here it a similar case, with the difference that the realization that the channel is closed
			// came the same block.
			let relevant_messaging_state = match Self::relevant_messaging_state() {
				Some(s) => s,
				None => {
					// This storage field should carry over from the previous block. So if it's None
					// then it must be that this is an edge-case where a message is attempted to be
					// sent at the first block. It should be safe to assume that there are no channels
					// opened at all so early. At least, relying on this assumption seems to be a better
					// tradeoff, compared to introducing an error variant that the clients should be
					// prepared to handle.
					return Err(SendHorizontalErr::NoChannel);
				}
			};
			let channel_meta = match relevant_messaging_state
				.egress_channels
				.binary_search_by_key(&recipient, |(recipient, _)| *recipient)
			{
				Ok(idx) => &relevant_messaging_state.egress_channels[idx].1,
				Err(_) => return Err(SendHorizontalErr::NoChannel),
			};
			if data.len() as u32 > channel_meta.max_message_size {
				return Err(SendHorizontalErr::TooBig);
			}

			// And then at last update the storage.
			<OutboundHrmpMessages<T>>::append(&recipient, data);
			<NonEmptyHrmpChannels<T>>::mutate(|v| {
				if !v.contains(&recipient) {
					v.push(recipient);
				}
			});

			Ok(())
		}
	}

	impl<T: Config> UpwardMessageSender for Pallet<T> {
		fn send_upward_message(message: UpwardMessage) -> Result<(), ()> {
			Self::send_upward_message(message).map_err(|_| ())
		}
	}

	impl<T: Config> HrmpMessageSender for Pallet<T> {
		fn send_hrmp_message(message: OutboundHrmpMessage) -> Result<(), ()> {
			Self::send_hrmp_message(message).map_err(|_| ())
		}
	}
}
