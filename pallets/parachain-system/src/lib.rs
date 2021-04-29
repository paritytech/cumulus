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

//! cumulus-pallet-parachain-system is a base module for cumulus-based parachains.
//!
//! This module handles low-level details of being a parachain. It's responsibilities include:
//!
//! - ingestion of the parachain validation data
//! - ingestion of incoming downward and lateral messages and dispatching them
//! - coordinating upgrades with the relay-chain
//! - communication of parachain outputs, such as sent messages, signalling an upgrade, etc.
//!
//! Users must ensure that they register this pallet as an inherent provider.

use sp_std::{prelude::*, cmp, collections::btree_map::BTreeMap};
use sp_runtime::traits::{BlakeTwo256, Hash};
use sp_inherents::{InherentData, InherentIdentifier, ProvideInherent};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::{DispatchResult, DispatchResultWithPostInfo},
	ensure, storage,
	traits::Get,
	weights::{DispatchClass, Weight, PostDispatchInfo, Pays},
};
use frame_system::{ensure_none, ensure_root};
use polkadot_parachain::primitives::RelayChainBlockNumber;
use cumulus_primitives_core::{
	relay_chain,
	well_known_keys::{self, NEW_VALIDATION_CODE},
	AbridgedHostConfiguration, DownwardMessageHandler, XcmpMessageHandler,
	InboundDownwardMessage, InboundHrmpMessage, OnValidationData, OutboundHrmpMessage, ParaId,
	PersistedValidationData, UpwardMessage, UpwardMessageSender, MessageSendError,
	XcmpMessageSource, ChannelStatus, GetChannelInfo,
};
use cumulus_primitives_parachain_inherent::ParachainInherentData;
use relay_state_snapshot::MessagingStateSnapshot;
use sp_runtime::transaction_validity::{
	TransactionSource, TransactionValidity, InvalidTransaction, ValidTransaction,
	TransactionLongevity,
};
use sp_runtime::DispatchError;

mod relay_state_snapshot;
#[macro_use]
pub mod validate_block;

/// The pallet's configuration trait.
pub trait Config: frame_system::Config<OnSetCode = ParachainSetCode<Self>> {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

	/// Something which can be notified when the validation data is set.
	type OnValidationData: OnValidationData;

	/// Returns the parachain ID we are running with.
	type SelfParaId: Get<ParaId>;

	/// The downward message handlers that will be informed when a message is received.
	type DownwardMessageHandlers: DownwardMessageHandler;

	/// The place where outbound XCMP messages come from. This is queried in `finalize_block`.
	type OutboundXcmpMessageSource: XcmpMessageSource;

	/// The HRMP message handlers that will be informed when a message is received.
	///
	/// The messages are dispatched in the order they were relayed by the relay chain. If multiple
	/// messages were relayed at one block, these will be dispatched in ascending order of the
	/// sender's para ID.
	type XcmpMessageHandler: XcmpMessageHandler;

	/// The weight we reserve at the beginning of the block for processing XCMP messages.
	type ReservedXcmpWeight: Get<Weight>;
}

// This pallet's storage items.
decl_storage! {
	trait Store for Module<T: Config> as ParachainSystem {
		/// We need to store the new validation function for the span between
		/// setting it and applying it. If it has a
		/// value, then [`PendingValidationFunction`] must have a real value, and
		/// together will coordinate the block number where the upgrade will happen.
		PendingRelayChainBlockNumber: Option<RelayChainBlockNumber>;

		/// The new validation function we will upgrade to when the relay chain
		/// reaches [`PendingRelayChainBlockNumber`]. A real validation function must
		/// exist here as long as [`PendingRelayChainBlockNumber`] is set.
		PendingValidationFunction get(fn new_validation_function): Vec<u8>;

		/// The [`PersistedValidationData`] set for this block.
		ValidationData get(fn validation_data): Option<PersistedValidationData>;

		/// Were the validation data set to notify the relay chain?
		DidSetValidationCode: bool;

		/// The last relay parent block number at which we signalled the code upgrade.
		LastUpgrade: relay_chain::BlockNumber;

		/// The snapshot of some state related to messaging relevant to the current parachain as per
		/// the relay parent.
		///
		/// This field is meant to be updated each block with the validation data inherent. Therefore,
		/// before processing of the inherent, e.g. in `on_initialize` this data may be stale.
		///
		/// This data is also absent from the genesis.
		RelevantMessagingState get(fn relevant_messaging_state): Option<MessagingStateSnapshot>;
		/// The parachain host configuration that was obtained from the relay parent.
		///
		/// This field is meant to be updated each block with the validation data inherent. Therefore,
		/// before processing of the inherent, e.g. in `on_initialize` this data may be stale.
		///
		/// This data is also absent from the genesis.
		HostConfiguration get(fn host_configuration): Option<AbridgedHostConfiguration>;

		/// The last downward message queue chain head we have observed.
		///
		/// This value is loaded before and saved after processing inbound downward messages carried
		/// by the system inherent.
		LastDmqMqcHead: MessageQueueChain;
		/// The message queue chain heads we have observed per each channel incoming channel.
		///
		/// This value is loaded before and saved after processing inbound downward messages carried
		/// by the system inherent.
		LastHrmpMqcHeads: BTreeMap<ParaId, MessageQueueChain>;

		PendingUpwardMessages: Vec<UpwardMessage>;

		/// The number of HRMP messages we observed in `on_initialize` and thus used that number for
		/// announcing the weight of `on_initialize` and `on_finalize`.
		AnnouncedHrmpMessagesPerCandidate: u32;

		/// The weight we reserve at the beginning of the block for processing XCMP messages. This
		/// overrides the amount set in the Config trait.
		ReservedXcmpWeightOverride: Option<Weight>;

		/// The next authorized upgrade, if there is one.
		AuthorizedUpgrade: Option<T::Hash>;
	}
}

// The pallet's dispatchable functions.
decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		// Initializing events
		// this is needed only if you are using events in your pallet
		fn deposit_event() = default;

		/// Force an already scheduled validation function upgrade to happen on a particular block.
		///
		/// Note that coordinating this block for the upgrade has to happen independently on the relay
		/// chain and this parachain. Synchronizing the block for the upgrade is sensitive, and this
		/// bypasses all checks and and normal protocols. Very easy to brick your chain if done wrong.
		#[weight = (0, DispatchClass::Operational)]
		pub fn set_upgrade_block(origin, relay_chain_block: RelayChainBlockNumber) {
			ensure_root(origin)?;
			if let Some(_old_block) = PendingRelayChainBlockNumber::get() {
				PendingRelayChainBlockNumber::put(relay_chain_block);
			} else {
				return Err(Error::<T>::NotScheduled.into())
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
		#[weight = (0, DispatchClass::Mandatory)]
		// TODO: This weight should be corrected.
		pub fn set_validation_data(origin, data: ParachainInherentData) -> DispatchResultWithPostInfo {
			ensure_none(origin)?;
			assert!(
				!ValidationData::exists(),
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
			if let Some(apply_block) = PendingRelayChainBlockNumber::get() {
				if vfp.relay_parent_number >= apply_block {
					PendingRelayChainBlockNumber::kill();
					let validation_function = PendingValidationFunction::take();
					LastUpgrade::put(&apply_block);
					Self::put_parachain_code(&validation_function);
					Self::deposit_event(RawEvent::ValidationFunctionApplied(vfp.relay_parent_number));
				}
			}

			let (host_config, relevant_messaging_state) =
				match relay_state_snapshot::extract_from_proof(
					T::SelfParaId::get(),
					vfp.relay_parent_storage_root,
					relay_chain_state
				) {
					Ok(r) => r,
					Err(err) => {
						panic!("invalid relay chain merkle proof: {:?}", err);
					}
				};

			ValidationData::put(&vfp);
			RelevantMessagingState::put(relevant_messaging_state.clone());
			HostConfiguration::put(host_config);

			<T::OnValidationData as OnValidationData>::on_validation_data(&vfp);

			// TODO: This is more than zero, but will need benchmarking to figure out what.
			let mut total_weight = 0;
			total_weight += Self::process_inbound_downward_messages(
				relevant_messaging_state.dmq_mqc_head,
				downward_messages,
			);
			total_weight += Self::process_inbound_horizontal_messages(
				&relevant_messaging_state.ingress_channels,
				horizontal_messages,
			);

			Ok(PostDispatchInfo { actual_weight: Some(total_weight), pays_fee: Pays::No })
		}

		#[weight = (1_000, DispatchClass::Operational)]
		fn sudo_send_upward_message(origin, message: UpwardMessage) {
			ensure_root(origin)?;
			let _ = Self::send_upward_message(message);
		}

		#[weight = (1_000_000, DispatchClass::Operational)]
		fn authorize_upgrade(origin, code_hash: T::Hash) {
			ensure_root(origin)?;

			AuthorizedUpgrade::<T>::put(&code_hash);

			Self::deposit_event(RawEvent::UpgradeAuthorized(code_hash));
		}

		#[weight = 1_000_000]
		fn enact_authorized_upgrade(_origin, code: Vec<u8>) -> DispatchResultWithPostInfo {
			// No ensure origin on purpose. We validate by checking the code vs hash in storage.
			Self::validate_authorized_upgrade(&code[..])?;
			Self::set_code_impl(code)?;
			AuthorizedUpgrade::<T>::kill();
			Ok(Pays::No.into())
		}

		fn on_finalize() {
			DidSetValidationCode::kill();

			let host_config = match Self::host_configuration() {
				Some(ok) => ok,
				None => {
					debug_assert!(false, "host configuration is promised to set until `on_finalize`; qed");
					return
				}
			};
			let relevant_messaging_state = match Self::relevant_messaging_state() {
				Some(ok) => ok,
				None => {
					debug_assert!(false, "relevant messaging state is promised to be set until `on_finalize`; qed");
					return
				}
			};

			<Self as Store>::PendingUpwardMessages::mutate(|up| {
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

			let maximum_channels = host_config.hrmp_max_message_num_per_candidate
				.min(AnnouncedHrmpMessagesPerCandidate::take()) as usize;

			let outbound_messages = T::OutboundXcmpMessageSource::take_outbound_messages(
				maximum_channels,
			);

			// Note conversion to the `OutboundHrmpMessage` isn't needed since the data that
			// `take_outbound_messages` returns encodes equivalently.
			//
			// The following code is a smoke test to check that the `OutboundHrmpMessage` type
			// doesn't accidentally change (e.g. by having a field added to it). If the following
			// line breaks, then we'll need to revisit the assumption that the result of
			// `take_outbound_messages` can be placed into `HRMP_OUTBOUND_MESSAGES` directly without
			// a decode/encode round-trip.
			let _ = OutboundHrmpMessage { recipient: ParaId::from(0), data: vec![] };

			storage::unhashed::put(well_known_keys::HRMP_OUTBOUND_MESSAGES, &outbound_messages);
		}

		fn on_initialize(n: T::BlockNumber) -> Weight {
			// To prevent removing `NEW_VALIDATION_CODE` that was set by another `on_initialize` like
			// for example from scheduler, we only kill the storage entry if it was not yet updated
			// in the current block.
			if !DidSetValidationCode::get() {
				storage::unhashed::kill(NEW_VALIDATION_CODE);
			}

			// Remove the validation from the old block.
			ValidationData::kill();

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
			let hrmp_max_message_num_per_candidate =
				Self::host_configuration()
					.map(|cfg| cfg.hrmp_max_message_num_per_candidate)
					.unwrap_or(0);
			AnnouncedHrmpMessagesPerCandidate::put(hrmp_max_message_num_per_candidate);

			// NOTE that the actual weight consumed by `on_finalize` may turn out lower.
			weight += T::DbWeight::get().reads_writes(
				3 + hrmp_max_message_num_per_candidate as u64,
				4 + hrmp_max_message_num_per_candidate as u64,
			);

			weight
		}
	}
}

impl<T: Config> Module<T> {
	fn validate_authorized_upgrade(code: &[u8]) -> Result<T::Hash, DispatchError> {
		let required_hash = AuthorizedUpgrade::<T>::get()
			.ok_or(Error::<T>::NothingAuthorized)?;
		let actual_hash = T::Hashing::hash(&code[..]);
		ensure!(actual_hash == required_hash, Error::<T>::Unauthorized);
		Ok(actual_hash)
	}
}

impl<T: Config> sp_runtime::traits::ValidateUnsigned for Module<T> {
	type Call = Call<T>;

	fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
		if let Call::enact_authorized_upgrade(ref code) = call {
			if let Ok(hash) = Self::validate_authorized_upgrade(code) {
				return Ok(ValidTransaction {
					priority: 100,
					requires: vec![],
					provides: vec![hash.as_ref().to_vec()],
					longevity: TransactionLongevity::max_value(),
					propagate: true,
				})
			}
		}
		Err(InvalidTransaction::Call.into())
	}
}

impl<T: Config> GetChannelInfo for Module<T> {
	fn get_channel_status(id: ParaId) -> ChannelStatus {
		// Note, that we are using `relevant_messaging_state` which may be from the previous
		// block, in case this is called from `on_initialize`, i.e. before the inherent with fresh
		// data is submitted.
		//
		// That shouldn't be a problem though because this is anticipated and already can happen.
		// This is because sending implies that a message is buffered until there is space to send
		// a message in the candidate. After a while waiting in a buffer, it may be discovered that
		// the channel to which a message were addressed is now closed. Another possibility, is that
		// the maximum message size was decreased so that a message in the buffer doesn't fit. Should
		// any of that happen the sender should be notified about the message was discarded.
		//
		// Here it a similar case, with the difference that the realization that the channel is closed
		// came the same block.
		let channels = match Self::relevant_messaging_state() {
			None => {
				log::warn!("calling `get_channel_status` with no RelevantMessagingState?!");
				return ChannelStatus::Closed
			},
			Some(d) => d.egress_channels,
		};
		// ^^^ NOTE: This storage field should carry over from the previous block. So if it's None
		// then it must be that this is an edge-case where a message is attempted to be
		// sent at the first block. It should be safe to assume that there are no channels
		// opened at all so early. At least, relying on this assumption seems to be a better
		// tradeoff, compared to introducing an error variant that the clients should be
		// prepared to handle.
		let index = match channels.binary_search_by_key(&id, |item| item.0) {
			Err(_) => return ChannelStatus::Closed,
			Ok(i) => i,
		};
		let meta = &channels[index].1;
		if meta.msg_count + 1 > meta.max_capacity {
			// The channel is at its capacity. Skip it for now.
			return ChannelStatus::Full;
		}
		let max_size_now = meta.max_total_size - meta.total_size;
		let max_size_ever = meta.max_message_size;
		ChannelStatus::Ready(max_size_now as usize, max_size_ever as usize)
	}

	fn get_channel_max(id: ParaId) -> Option<usize> {
		let channels = Self::relevant_messaging_state()?.egress_channels;
		let index = channels.binary_search_by_key(&id, |item| item.0).ok()?;
		Some(channels[index].1.max_message_size as usize)
	}
}

impl<T: Config> Module<T> {
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
			assert_eq!(params.parent_head, validation_data.parent_head, "Parent head doesn't match");
			assert_eq!(
				params.relay_parent_number,
				validation_data.relay_parent_number,
				"Relay parent number doesn't match",
			);
			assert_eq!(
				params.relay_parent_storage_root,
				validation_data.relay_parent_storage_root,
				"Relay parent storage root doesn't match",
			);
		});
	}

	/// Process all inbound downward messages relayed by the collator.
	///
	/// Checks if the sequence of the messages is valid, dispatches them and communicates the number
	/// of processed messages to the collator via a storage update.
	///
	/// **Panics** if it turns out that after processing all messages the Message Queue Chain hash
	///            doesn't match the expected.
	fn process_inbound_downward_messages(
		expected_dmq_mqc_head: relay_chain::Hash,
		downward_messages: Vec<InboundDownwardMessage>,
	) -> Weight {
		let dm_count = downward_messages.len() as u32;
		let mut weight_used = 0;

		if dm_count != 0 {
			let mut processed_count = 0;

			Self::deposit_event(RawEvent::DownwardMessagesReceived(dm_count));

			// Reference fu to avoid the `move` capture.
			let weight_used_ref = &mut weight_used;
			let processed_count_ref = &mut processed_count;
			let result_mqc_head = LastDmqMqcHead::mutate(move |mqc| {
				for downward_message in downward_messages {
					mqc.extend_downward(&downward_message);
					*weight_used_ref += T::DownwardMessageHandlers::handle_downward_message(downward_message);
					*processed_count_ref += 1;
				}
				mqc.0
			});

			Self::deposit_event(RawEvent::DownwardMessagesProcessed(
				processed_count,
				weight_used,
				result_mqc_head.clone(),
				expected_dmq_mqc_head.clone(),
			));

			// After hashing each message in the message queue chain submitted by the collator, we should
			// arrive to the MQC head provided by the relay chain.
			//
			// A mismatch means that at least some of the submitted messages were altered, omitted or added
			// improperly.
			assert_eq!(result_mqc_head, expected_dmq_mqc_head);
		} else {
			assert_eq!(LastDmqMqcHead::get().0, expected_dmq_mqc_head);
		}

		// Store the processed_downward_messages here so that it will be accessible from
		// PVF's `validate_block` wrapper and collation pipeline.
		storage::unhashed::put(well_known_keys::PROCESSED_DOWNWARD_MESSAGES, &dm_count);

		weight_used
	}

	/// Process all inbound horizontal messages relayed by the collator.
	///
	/// This is similar to [`process_inbound_downward_messages`], but works on multiple inbound
	/// channels.
	///
	/// **Panics** if either any of horizontal messages submitted by the collator was sent from a
	///            para which has no open channel to this parachain or if after processing messages
	///            across all inbound channels MQCs were obtained which do not correspond to the
	///            ones found on the relay-chain.
	fn process_inbound_horizontal_messages(
		ingress_channels: &[(ParaId, cumulus_primitives_core::AbridgedHrmpChannel)],
		horizontal_messages: BTreeMap<ParaId, Vec<InboundHrmpMessage>>,
	) -> Weight {
		// First, check that all submitted messages are sent from channels that exist. The channel
		// exists if its MQC head is present in `vfp.hrmp_mqc_heads`.
		for sender in horizontal_messages.keys() {
			// A violation of the assertion below indicates that one of the messages submitted by
			// the collator was sent from a sender that doesn't have a channel opened to this parachain,
			// according to the relay-parent state.
			assert!(
				ingress_channels
					.binary_search_by_key(sender, |&(s, _)| s)
					.is_ok(),
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

		let last_mqc_heads = LastHrmpMqcHeads::get();
		let mut running_mqc_heads = BTreeMap::new();
		let mut hrmp_watermark = None;

		{
			for (sender, ref horizontal_message) in &horizontal_messages {
				if hrmp_watermark
					.map(|w| w < horizontal_message.sent_at)
					.unwrap_or(true)
				{
					hrmp_watermark = Some(horizontal_message.sent_at);
				}

				running_mqc_heads
					.entry(sender)
					.or_insert_with(|| last_mqc_heads.get(&sender).cloned().unwrap_or_default())
					.extend_hrmp(horizontal_message);
			}
		}
		let message_iter = horizontal_messages.iter()
			.map(|&(sender, ref message)| (sender, message.sent_at, &message.data[..]));

		let max_weight = ReservedXcmpWeightOverride::get().unwrap_or_else(T::ReservedXcmpWeight::get);
		let weight_used = T::XcmpMessageHandler::handle_xcmp_messages(message_iter, max_weight);

		// Check that the MQC heads for each channel provided by the relay chain match the MQC heads
		// we have after processing all incoming messages.
		//
		// Along the way we also carry over the relevant entries from the `last_mqc_heads` to
		// `running_mqc_heads`. Otherwise, in a block where no messages were sent in a channel
		// it won't get into next block's `last_mqc_heads` and thus will be all zeros, which
		// would corrupt the message queue chain.
		for &(ref sender, ref channel) in ingress_channels {
			let cur_head = running_mqc_heads
				.entry(sender)
				.or_insert_with(|| last_mqc_heads.get(&sender).cloned().unwrap_or_default())
				.head();
			let target_head = channel.mqc_head.unwrap_or_default();
			assert!(cur_head == target_head);
		}

		LastHrmpMqcHeads::put(running_mqc_heads);

		// If we processed at least one message, then advance watermark to that location.
		if let Some(hrmp_watermark) = hrmp_watermark {
			storage::unhashed::put(well_known_keys::HRMP_WATERMARK, &hrmp_watermark);
		}

		weight_used
	}

	/// Put a new validation function into a particular location where polkadot
	/// monitors for updates. Calling this function notifies polkadot that a new
	/// upgrade has been scheduled.
	fn notify_polkadot_of_pending_upgrade(code: &[u8]) {
		storage::unhashed::put_raw(NEW_VALIDATION_CODE, code);
		DidSetValidationCode::put(true);
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
		HostConfiguration::get().map(|cfg| cfg.max_code_size)
	}

	/// Returns if a PVF/runtime upgrade could be signalled at the current block, and if so
	/// when the new code will take the effect.
	fn code_upgrade_allowed(
		vfp: &PersistedValidationData,
		cfg: &AbridgedHostConfiguration,
	) -> Option<relay_chain::BlockNumber> {
		if PendingRelayChainBlockNumber::get().is_some() {
			// There is already upgrade scheduled. Upgrade is not allowed.
			return None;
		}

		let relay_blocks_since_last_upgrade =
			vfp.relay_parent_number.saturating_sub(LastUpgrade::get());

		if relay_blocks_since_last_upgrade <= cfg.validation_upgrade_frequency {
			// The cooldown after the last upgrade hasn't elapsed yet. Upgrade is not allowed.
			return None;
		}

		Some(vfp.relay_parent_number + cfg.validation_upgrade_delay)
	}

	/// The implementation of the runtime upgrade functionality for parachains.
	fn set_code_impl(validation_function: Vec<u8>) -> DispatchResult {
		ensure!(
			!PendingValidationFunction::exists(),
			Error::<T>::OverlappingUpgrades
		);
		let vfp = Self::validation_data().ok_or(Error::<T>::ValidationDataNotAvailable)?;
		let cfg = Self::host_configuration().ok_or(Error::<T>::HostConfigurationNotAvailable)?;
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
		PendingRelayChainBlockNumber::put(apply_block);
		PendingValidationFunction::put(validation_function);
		Self::deposit_event(RawEvent::ValidationFunctionStored(apply_block));

		Ok(())
	}
}

pub struct ParachainSetCode<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> frame_system::SetCode for ParachainSetCode<T> {
	fn set_code(code: Vec<u8>) -> DispatchResult {
		Module::<T>::set_code_impl(code)
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
struct MessageQueueChain(relay_chain::Hash);

impl MessageQueueChain {
	fn extend_hrmp(&mut self, horizontal_message: &InboundHrmpMessage) -> &mut Self {
		let prev_head = self.0;
		self.0 = BlakeTwo256::hash_of(&(
			prev_head,
			horizontal_message.sent_at,
			BlakeTwo256::hash_of(&horizontal_message.data),
		));
		self
	}

	fn extend_downward(&mut self, downward_message: &InboundDownwardMessage) -> &mut Self {
		let prev_head = self.0;
		self.0 = BlakeTwo256::hash_of(&(
			prev_head,
			downward_message.sent_at,
			BlakeTwo256::hash_of(&downward_message.msg),
		));
		self
	}

	fn head(&self) -> relay_chain::Hash {
		self.0
	}
}

impl<T: Config> Module<T> {
	pub fn send_upward_message(message: UpwardMessage) -> Result<u32, MessageSendError> {
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
					return Err(MessageSendError::TooBig);
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
		<Self as Store>::PendingUpwardMessages::append(message);
		Ok(0)
	}
}

impl<T: Config> UpwardMessageSender for Module<T> {
	fn send_upward_message(message: UpwardMessage) -> Result<u32, MessageSendError> {
		Self::send_upward_message(message)
	}
}

impl<T: Config> ProvideInherent for Module<T> {
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

	fn is_inherent(call: &Self::Call) -> bool {
		matches!(call, Call::set_validation_data(_))
	}
}

decl_event! {
	pub enum Event<T> where Hash = <T as frame_system::Config>::Hash {
		/// The validation function has been scheduled to apply as of the contained relay chain block number.
		ValidationFunctionStored(RelayChainBlockNumber),
		/// The validation function was applied as of the contained relay chain block number.
		ValidationFunctionApplied(RelayChainBlockNumber),
		/// An upgrade has been authorized.
		UpgradeAuthorized(Hash),
		/// Downward messages were processed using the given weight.
		/// \[ count, weight_used, result_mqc_head, expected_mqc_head \]
		DownwardMessagesProcessed(u32, Weight, relay_chain::Hash, relay_chain::Hash),
		/// Some downward messages have been received and will be processed.
		/// \[ count \]
		DownwardMessagesReceived(u32),
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
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
		/// No validation function upgrade is currently scheduled.
		NotScheduled,
		/// No code upgrade has been authorized.
		NothingAuthorized,
		/// The given code upgrade has not been authorized.
		Unauthorized,
	}
}

/// tests for this pallet
#[cfg(test)]
mod tests {
	use super::*;

	use codec::Encode;
	use cumulus_primitives_core::{
		AbridgedHrmpChannel, InboundDownwardMessage, InboundHrmpMessage, PersistedValidationData,
		relay_chain::BlockNumber as RelayBlockNumber,
	};
	use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;
	use frame_support::{
		assert_ok,
		dispatch::UnfilteredDispatchable,
		parameter_types,
		traits::{OnFinalize, OnInitialize},
	};
	use frame_system::{InitKind, RawOrigin};
	use hex_literal::hex;
	use relay_chain::v1::HrmpChannelId;
	use sp_core::H256;
	use sp_runtime::{testing::Header, traits::IdentityLookup};
	use sp_version::RuntimeVersion;
	use std::cell::RefCell;

	use crate as parachain_system;

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
	type Block = frame_system::mocking::MockBlock<Test>;

	frame_support::construct_runtime!(
		pub enum Test where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
			ParachainSystem: parachain_system::{Pallet, Call, Storage, Event<T>},
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
	}
	impl frame_system::Config for Test {
		type Origin = Origin;
		type Call = Call;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
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
		type OnSetCode = ParachainSetCode<Self>;
	}
	impl Config for Test {
		type Event = Event;
		type OnValidationData = ();
		type SelfParaId = ParachainId;
		type DownwardMessageHandlers = SaveIntoThreadLocal;
		type XcmpMessageHandler = SaveIntoThreadLocal;
		type OutboundXcmpMessageSource = FromThreadLocal;
		type ReservedXcmpWeight = ReservedXcmpWeight;
	}

	pub struct FromThreadLocal;
	pub struct SaveIntoThreadLocal;

	std::thread_local! {
		static HANDLED_DOWNWARD_MESSAGES: RefCell<Vec<InboundDownwardMessage>> = RefCell::new(Vec::new());
		static HANDLED_XCMP_MESSAGES: RefCell<Vec<(ParaId, relay_chain::BlockNumber, Vec<u8>)>> = RefCell::new(Vec::new());
		static SENT_MESSAGES: RefCell<Vec<(ParaId, Vec<u8>)>> = RefCell::new(Vec::new());
	}

	fn send_message(
		dest: ParaId,
		message: Vec<u8>,
	) {
		SENT_MESSAGES.with(|m| m.borrow_mut().push((dest, message)));
	}

	impl XcmpMessageSource for FromThreadLocal {
		fn take_outbound_messages(maximum_channels: usize) -> Vec<(ParaId, Vec<u8>)> {
			let mut ids = std::collections::BTreeSet::<ParaId>::new();
			let mut taken = 0;
			let mut result = Vec::new();
			SENT_MESSAGES.with(|ms| ms.borrow_mut()
				.retain(|m| {
					let status = <Module::<Test> as GetChannelInfo>::get_channel_status(m.0);
					let ready = matches!(status, ChannelStatus::Ready(..));
					if ready && !ids.contains(&m.0) && taken < maximum_channels {
						ids.insert(m.0);
						taken += 1;
						result.push(m.clone());
						false
					} else {
						true
					}
				})
			);
			result
		}
	}

	impl DownwardMessageHandler for SaveIntoThreadLocal {
		fn handle_downward_message(msg: InboundDownwardMessage) -> Weight {
			HANDLED_DOWNWARD_MESSAGES.with(|m| {
				m.borrow_mut().push(msg);
			});
			0
		}
	}

	impl XcmpMessageHandler for SaveIntoThreadLocal {
		fn handle_xcmp_messages<'a, I: Iterator<Item=(ParaId, RelayBlockNumber, &'a [u8])>>(
			iter: I,
			_max_weight: Weight,
		) -> Weight {
			HANDLED_XCMP_MESSAGES.with(|m| {
				for (sender, sent_at, message) in iter {
					m.borrow_mut().push((sender, sent_at, message.to_vec()));
				}
				0
			})
		}
	}

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> sp_io::TestExternalities {
		HANDLED_DOWNWARD_MESSAGES.with(|m| m.borrow_mut().clear());
		HANDLED_XCMP_MESSAGES.with(|m| m.borrow_mut().clear());

		frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap()
			.into()
	}

	struct CallInWasm(Vec<u8>);

	impl sp_core::traits::CallInWasm for CallInWasm {
		fn call_in_wasm(
			&self,
			_wasm_code: &[u8],
			_code_hash: Option<Vec<u8>>,
			_method: &str,
			_call_data: &[u8],
			_ext: &mut dyn sp_externalities::Externalities,
			_missing_host_functions: sp_core::traits::MissingHostFunctions,
		) -> Result<Vec<u8>, String> {
			Ok(self.0.clone())
		}
	}

	fn wasm_ext() -> sp_io::TestExternalities {
		let version = RuntimeVersion {
			spec_name: "test".into(),
			spec_version: 2,
			impl_version: 1,
			..Default::default()
		};
		let call_in_wasm = CallInWasm(version.encode());

		let mut ext = new_test_ext();
		ext.register_extension(sp_core::traits::CallInWasmExt::new(call_in_wasm));
		ext
	}

	struct BlockTest {
		n: <Test as frame_system::Config>::BlockNumber,
		within_block: Box<dyn Fn()>,
		after_block: Option<Box<dyn Fn()>>,
	}

	/// BlockTests exist to test blocks with some setup: we have to assume that
	/// `validate_block` will mutate and check storage in certain predictable
	/// ways, for example, and we want to always ensure that tests are executed
	/// in the context of some particular block number.
	#[derive(Default)]
	struct BlockTests {
		tests: Vec<BlockTest>,
		pending_upgrade: Option<RelayChainBlockNumber>,
		ran: bool,
		relay_sproof_builder_hook:
			Option<Box<dyn Fn(&BlockTests, RelayChainBlockNumber, &mut RelayStateSproofBuilder)>>,
		persisted_validation_data_hook:
			Option<Box<dyn Fn(&BlockTests, RelayChainBlockNumber, &mut PersistedValidationData)>>,
		inherent_data_hook:
			Option<Box<dyn Fn(&BlockTests, RelayChainBlockNumber, &mut ParachainInherentData)>>,
	}

	impl BlockTests {
		fn new() -> BlockTests {
			Default::default()
		}

		fn add_raw(mut self, test: BlockTest) -> Self {
			self.tests.push(test);
			self
		}

		fn add<F>(self, n: <Test as frame_system::Config>::BlockNumber, within_block: F) -> Self
		where
			F: 'static + Fn(),
		{
			self.add_raw(BlockTest {
				n,
				within_block: Box::new(within_block),
				after_block: None,
			})
		}

		fn add_with_post_test<F1, F2>(
			self,
			n: <Test as frame_system::Config>::BlockNumber,
			within_block: F1,
			after_block: F2,
		) -> Self
		where
			F1: 'static + Fn(),
			F2: 'static + Fn(),
		{
			self.add_raw(BlockTest {
				n,
				within_block: Box::new(within_block),
				after_block: Some(Box::new(after_block)),
			})
		}

		fn with_relay_sproof_builder<F>(mut self, f: F) -> Self
		where
			F: 'static + Fn(&BlockTests, RelayChainBlockNumber, &mut RelayStateSproofBuilder),
		{
			self.relay_sproof_builder_hook = Some(Box::new(f));
			self
		}

		#[allow(dead_code)] // might come in handy in future. If now is future and it still hasn't - feel free.
		fn with_validation_data<F>(mut self, f: F) -> Self
		where
			F: 'static + Fn(&BlockTests, RelayChainBlockNumber, &mut PersistedValidationData),
		{
			self.persisted_validation_data_hook = Some(Box::new(f));
			self
		}

		fn with_inherent_data<F>(mut self, f: F) -> Self
		where
			F: 'static + Fn(&BlockTests, RelayChainBlockNumber, &mut ParachainInherentData),
		{
			self.inherent_data_hook = Some(Box::new(f));
			self
		}

		fn run(&mut self) {
			self.ran = true;
			wasm_ext().execute_with(|| {
				for BlockTest {
					n,
					within_block,
					after_block,
				} in self.tests.iter()
				{
					// clear pending updates, as applicable
					if let Some(upgrade_block) = self.pending_upgrade {
						if n >= &upgrade_block.into() {
							self.pending_upgrade = None;
						}
					}

					// begin initialization
					System::initialize(
						&n,
						&Default::default(),
						&Default::default(),
						InitKind::Full,
					);

					// now mess with the storage the way validate_block does
					let mut sproof_builder = RelayStateSproofBuilder::default();
					if let Some(ref hook) = self.relay_sproof_builder_hook {
						hook(self, *n as RelayChainBlockNumber, &mut sproof_builder);
					}
					let (relay_parent_storage_root, relay_chain_state) =
						sproof_builder.into_state_root_and_proof();
					let mut vfp = PersistedValidationData {
						relay_parent_number: *n as RelayChainBlockNumber,
						relay_parent_storage_root,
						..Default::default()
					};
					if let Some(ref hook) = self.persisted_validation_data_hook {
						hook(self, *n as RelayChainBlockNumber, &mut vfp);
					}

					ValidationData::put(&vfp);
					storage::unhashed::kill(NEW_VALIDATION_CODE);

					// It is insufficient to push the validation function params
					// to storage; they must also be included in the inherent data.
					let inherent_data = {
						let mut inherent_data = InherentData::default();
						let mut system_inherent_data = ParachainInherentData {
							validation_data: vfp.clone(),
							relay_chain_state,
							downward_messages: Default::default(),
							horizontal_messages: Default::default(),
						};
						if let Some(ref hook) = self.inherent_data_hook {
							hook(self, *n as RelayChainBlockNumber, &mut system_inherent_data);
						}
						inherent_data
							.put_data(
								cumulus_primitives_parachain_inherent::INHERENT_IDENTIFIER,
								&system_inherent_data,
							)
							.expect("failed to put VFP inherent");
						inherent_data
					};

					// execute the block
					ParachainSystem::on_initialize(*n);
					ParachainSystem::create_inherent(&inherent_data)
						.expect("got an inherent")
						.dispatch_bypass_filter(RawOrigin::None.into())
						.expect("dispatch succeeded");
					within_block();
					ParachainSystem::on_finalize(*n);

					// did block execution set new validation code?
					if storage::unhashed::exists(NEW_VALIDATION_CODE) {
						if self.pending_upgrade.is_some() {
							panic!("attempted to set validation code while upgrade was pending");
						}
					}

					// clean up
					System::finalize();
					if let Some(after_block) = after_block {
						after_block();
					}
				}
			});
		}
	}

	impl Drop for BlockTests {
		fn drop(&mut self) {
			if !self.ran {
				self.run();
			}
		}
	}

	#[test]
	#[should_panic]
	fn block_tests_run_on_drop() {
		BlockTests::new().add(123, || {
			panic!("if this test passes, block tests run properly")
		});
	}

	#[test]
	fn events() {
		BlockTests::new()
			.with_relay_sproof_builder(|_, _, builder| {
				builder.host_config.validation_upgrade_delay = 1000;
			})
			.add_with_post_test(
				123,
				|| {
					assert_ok!(System::set_code(
						RawOrigin::Root.into(),
						Default::default()
					));
				},
				|| {
					let events = System::events();
					assert_eq!(
						events[0].event,
						Event::parachain_system(crate::RawEvent::ValidationFunctionStored(1123).into())
					);
				},
			)
			.add_with_post_test(
				1234,
				|| {},
				|| {
					let events = System::events();
					assert_eq!(
						events[0].event,
						Event::parachain_system(crate::RawEvent::ValidationFunctionApplied(1234).into())
					);
				},
			);
	}

	#[test]
	fn non_overlapping() {
		BlockTests::new()
			.with_relay_sproof_builder(|_, _, builder| {
				builder.host_config.validation_upgrade_delay = 1000;
			})
			.add(123, || {
				assert_ok!(System::set_code(
					RawOrigin::Root.into(),
					Default::default()
				));
			})
			.add(234, || {
				assert_eq!(
					System::set_code(RawOrigin::Root.into(), Default::default()),
					Err(Error::<Test>::OverlappingUpgrades.into()),
				)
			});
	}

	#[test]
	fn manipulates_storage() {
		BlockTests::new()
			.add(123, || {
				assert!(
					!PendingValidationFunction::exists(),
					"validation function must not exist yet"
				);
				assert_ok!(System::set_code(
					RawOrigin::Root.into(),
					Default::default()
				));
				assert!(
					PendingValidationFunction::exists(),
					"validation function must now exist"
				);
			})
			.add_with_post_test(
				1234,
				|| {},
				|| {
					assert!(
						!PendingValidationFunction::exists(),
						"validation function must have been unset"
					);
				},
			);
	}

	#[test]
	fn checks_size() {
		BlockTests::new()
			.with_relay_sproof_builder(|_, _, builder| {
				builder.host_config.max_code_size = 8;
			})
			.add(123, || {
				assert_eq!(
					System::set_code(RawOrigin::Root.into(), vec![0; 64]),
					Err(Error::<Test>::TooBig.into()),
				);
			});
	}

	#[test]
	fn send_upward_message_num_per_candidate() {
		BlockTests::new()
			.with_relay_sproof_builder(|_, _, sproof| {
				sproof.host_config.max_upward_message_num_per_candidate = 1;
				sproof.relay_dispatch_queue_size = None;
			})
			.add_with_post_test(
				1,
				|| {
					ParachainSystem::send_upward_message(b"Mr F was here".to_vec()).unwrap();
					ParachainSystem::send_upward_message(b"message 2".to_vec()).unwrap();
				},
				|| {
					let v: Option<Vec<Vec<u8>>> =
						storage::unhashed::get(well_known_keys::UPWARD_MESSAGES);
					assert_eq!(v, Some(vec![b"Mr F was here".to_vec()]),);
				},
			)
			.add_with_post_test(
				2,
				|| { /* do nothing within block */ },
				|| {
					let v: Option<Vec<Vec<u8>>> =
						storage::unhashed::get(well_known_keys::UPWARD_MESSAGES);
					assert_eq!(v, Some(vec![b"message 2".to_vec()]),);
				},
			);
	}

	#[test]
	fn send_upward_message_relay_bottleneck() {
		BlockTests::new()
			.with_relay_sproof_builder(|_, relay_block_num, sproof| {
				sproof.host_config.max_upward_message_num_per_candidate = 2;
				sproof.host_config.max_upward_queue_count = 5;

				match relay_block_num {
					1 => sproof.relay_dispatch_queue_size = Some((5, 0)),
					2 => sproof.relay_dispatch_queue_size = Some((4, 0)),
					_ => unreachable!(),
				}
			})
			.add_with_post_test(
				1,
				|| {
					ParachainSystem::send_upward_message(vec![0u8; 8]).unwrap();
				},
				|| {
					// The message won't be sent because there is already one message in queue.
					let v: Option<Vec<Vec<u8>>> =
						storage::unhashed::get(well_known_keys::UPWARD_MESSAGES);
					assert_eq!(v, Some(vec![]),);
				},
			)
			.add_with_post_test(
				2,
				|| { /* do nothing within block */ },
				|| {
					let v: Option<Vec<Vec<u8>>> =
						storage::unhashed::get(well_known_keys::UPWARD_MESSAGES);
					assert_eq!(v, Some(vec![vec![0u8; 8]]),);
				},
			);
	}

	#[test]
	fn send_hrmp_message_buffer_channel_close() {
		BlockTests::new()
			.with_relay_sproof_builder(|_, relay_block_num, sproof| {
				//
				// Base case setup
				//
				sproof.para_id = ParaId::from(200);
				sproof.hrmp_egress_channel_index = Some(vec![ParaId::from(300), ParaId::from(400)]);
				sproof.hrmp_channels.insert(
					HrmpChannelId {
						sender: ParaId::from(200),
						recipient: ParaId::from(300),
					},
					AbridgedHrmpChannel {
						max_capacity: 1,
						msg_count: 1, // <- 1/1 means the channel is full
						max_total_size: 1024,
						max_message_size: 8,
						total_size: 0,
						mqc_head: Default::default(),
					},
				);
				sproof.hrmp_channels.insert(
					HrmpChannelId {
						sender: ParaId::from(200),
						recipient: ParaId::from(400),
					},
					AbridgedHrmpChannel {
						max_capacity: 1,
						msg_count: 1,
						max_total_size: 1024,
						max_message_size: 8,
						total_size: 0,
						mqc_head: Default::default(),
					},
				);

				//
				// Adjustment according to block
				//
				match relay_block_num {
					1 => {}
					2 => {}
					3 => {
						// The channel 200->400 ceases to exist at the relay chain block 3
						sproof
							.hrmp_egress_channel_index
							.as_mut()
							.unwrap()
							.retain(|n| n != &ParaId::from(400));
						sproof.hrmp_channels.remove(&HrmpChannelId {
							sender: ParaId::from(200),
							recipient: ParaId::from(400),
						});

						// We also free up space for a message in the 200->300 channel.
						sproof
							.hrmp_channels
							.get_mut(&HrmpChannelId {
								sender: ParaId::from(200),
								recipient: ParaId::from(300),
							})
							.unwrap()
							.msg_count = 0;
					}
					_ => unreachable!(),
				}
			})
			.add_with_post_test(
				1,
				|| {
					send_message(
						ParaId::from(300),
						b"1".to_vec(),
					);
					send_message(
						ParaId::from(400),
						b"2".to_vec(),
					);
				},
				|| {},
			)
			.add_with_post_test(
				2,
				|| {},
				|| {
					// both channels are at capacity so we do not expect any messages.
					let v: Option<Vec<OutboundHrmpMessage>> =
						storage::unhashed::get(well_known_keys::HRMP_OUTBOUND_MESSAGES);
					assert_eq!(v, Some(vec![]));
				},
			)
			.add_with_post_test(
				3,
				|| {},
				|| {
					let v: Option<Vec<OutboundHrmpMessage>> =
						storage::unhashed::get(well_known_keys::HRMP_OUTBOUND_MESSAGES);
					assert_eq!(
						v,
						Some(vec![OutboundHrmpMessage {
							recipient: ParaId::from(300),
							data: b"1".to_vec(),
						}])
					);
				},
			);
	}

	#[test]
	fn message_queue_chain() {
		assert_eq!(MessageQueueChain::default().head(), H256::zero());

		// Note that the resulting hashes are the same for HRMP and DMP. That's because even though
		// the types are nominally different, they have the same structure and computation of the
		// new head doesn't differ.
		//
		// These cases are taken from https://github.com/paritytech/polkadot/pull/2351
		assert_eq!(
			MessageQueueChain::default()
				.extend_downward(&InboundDownwardMessage {
					sent_at: 2,
					msg: vec![1, 2, 3],
				})
				.extend_downward(&InboundDownwardMessage {
					sent_at: 3,
					msg: vec![4, 5, 6],
				})
				.head(),
			hex!["88dc00db8cc9d22aa62b87807705831f164387dfa49f80a8600ed1cbe1704b6b"].into(),
		);
		assert_eq!(
			MessageQueueChain::default()
				.extend_hrmp(&InboundHrmpMessage {
					sent_at: 2,
					data: vec![1, 2, 3],
				})
				.extend_hrmp(&InboundHrmpMessage {
					sent_at: 3,
					data: vec![4, 5, 6],
				})
				.head(),
			hex!["88dc00db8cc9d22aa62b87807705831f164387dfa49f80a8600ed1cbe1704b6b"].into(),
		);
	}

	#[test]
	fn receive_dmp() {
		lazy_static::lazy_static! {
			static ref MSG: InboundDownwardMessage = InboundDownwardMessage {
				sent_at: 1,
				msg: b"down".to_vec(),
			};
		}

		BlockTests::new()
			.with_relay_sproof_builder(|_, relay_block_num, sproof| match relay_block_num {
				1 => {
					sproof.dmq_mqc_head =
						Some(MessageQueueChain::default().extend_downward(&MSG).head());
				}
				_ => unreachable!(),
			})
			.with_inherent_data(|_, relay_block_num, data| match relay_block_num {
				1 => {
					data.downward_messages.push(MSG.clone());
				}
				_ => unreachable!(),
			})
			.add(1, || {
				HANDLED_DOWNWARD_MESSAGES.with(|m| {
					let mut m = m.borrow_mut();
					assert_eq!(&*m, &[MSG.clone()]);
					m.clear();
				});
			});
	}

	#[test]
	fn receive_hrmp() {
		lazy_static::lazy_static! {
			static ref MSG_1: InboundHrmpMessage = InboundHrmpMessage {
				sent_at: 1,
				data: b"1".to_vec(),
			};

			static ref MSG_2: InboundHrmpMessage = InboundHrmpMessage {
				sent_at: 1,
				data: b"2".to_vec(),
			};

			static ref MSG_3: InboundHrmpMessage = InboundHrmpMessage {
				sent_at: 2,
				data: b"3".to_vec(),
			};

			static ref MSG_4: InboundHrmpMessage = InboundHrmpMessage {
				sent_at: 2,
				data: b"4".to_vec(),
			};
		}

		BlockTests::new()
			.with_relay_sproof_builder(|_, relay_block_num, sproof| match relay_block_num {
				1 => {
					// 200 - doesn't exist yet
					// 300 - one new message
					sproof.upsert_inbound_channel(ParaId::from(300)).mqc_head =
						Some(MessageQueueChain::default().extend_hrmp(&MSG_1).head());
				}
				2 => {
					// 200 - two new messages
					// 300 - now present with one message.
					sproof.upsert_inbound_channel(ParaId::from(200)).mqc_head =
						Some(MessageQueueChain::default().extend_hrmp(&MSG_4).head());
					sproof.upsert_inbound_channel(ParaId::from(300)).mqc_head = Some(
						MessageQueueChain::default()
							.extend_hrmp(&MSG_1)
							.extend_hrmp(&MSG_2)
							.extend_hrmp(&MSG_3)
							.head(),
					);
				}
				3 => {
					// 200 - no new messages
					// 300 - is gone
					sproof.upsert_inbound_channel(ParaId::from(200)).mqc_head =
						Some(MessageQueueChain::default().extend_hrmp(&MSG_4).head());
				}
				_ => unreachable!(),
			})
			.with_inherent_data(|_, relay_block_num, data| match relay_block_num {
				1 => {
					data.horizontal_messages
						.insert(ParaId::from(300), vec![MSG_1.clone()]);
				}
				2 => {
					data.horizontal_messages.insert(
						ParaId::from(300),
						vec![
							// can't be sent at the block 1 actually. However, we cheat here
							// because we want to test the case where there are multiple messages
							// but the harness at the moment doesn't support block skipping.
							MSG_2.clone(),
							MSG_3.clone(),
						],
					);
					data.horizontal_messages
						.insert(ParaId::from(200), vec![MSG_4.clone()]);
				}
				3 => {}
				_ => unreachable!(),
			})
			.add(1, || {
				HANDLED_XCMP_MESSAGES.with(|m| {
					let mut m = m.borrow_mut();
					assert_eq!(&*m, &[(ParaId::from(300), 1, b"1".to_vec())]);
					m.clear();
				});
			})
			.add(2, || {
				HANDLED_XCMP_MESSAGES.with(|m| {
					let mut m = m.borrow_mut();
					assert_eq!(
						&*m,
						&[
							(ParaId::from(300), 1, b"2".to_vec()),
							(ParaId::from(200), 2, b"4".to_vec()),
							(ParaId::from(300), 2, b"3".to_vec()),
						]
					);
					m.clear();
				});
			})
			.add(3, || {});
	}

	#[test]
	fn receive_hrmp_empty_channel() {
		BlockTests::new()
			.with_relay_sproof_builder(|_, relay_block_num, sproof| match relay_block_num {
				1 => {
					// no channels
				}
				2 => {
					// one new channel
					sproof.upsert_inbound_channel(ParaId::from(300)).mqc_head =
						Some(MessageQueueChain::default().head());
				}
				_ => unreachable!(),
			})
			.add(1, || {})
			.add(2, || {});
	}

	#[test]
	fn receive_hrmp_after_pause() {
		lazy_static::lazy_static! {
			static ref MSG_1: InboundHrmpMessage = InboundHrmpMessage {
				sent_at: 1,
				data: b"mikhailinvanovich".to_vec(),
			};

			static ref MSG_2: InboundHrmpMessage = InboundHrmpMessage {
				sent_at: 3,
				data: b"1000000000".to_vec(),
			};
		}

		const ALICE: ParaId = ParaId::new(300);

		BlockTests::new()
			.with_relay_sproof_builder(|_, relay_block_num, sproof| match relay_block_num {
				1 => {
					sproof.upsert_inbound_channel(ALICE).mqc_head =
						Some(MessageQueueChain::default().extend_hrmp(&MSG_1).head());
				}
				2 => {
					// 300 - no new messages, mqc stayed the same.
					sproof.upsert_inbound_channel(ALICE).mqc_head =
						Some(MessageQueueChain::default().extend_hrmp(&MSG_1).head());
				}
				3 => {
					// 300 - new message.
					sproof.upsert_inbound_channel(ALICE).mqc_head = Some(
						MessageQueueChain::default()
							.extend_hrmp(&MSG_1)
							.extend_hrmp(&MSG_2)
							.head(),
					);
				}
				_ => unreachable!(),
			})
			.with_inherent_data(|_, relay_block_num, data| match relay_block_num {
				1 => {
					data.horizontal_messages.insert(ALICE, vec![MSG_1.clone()]);
				}
				2 => {
					// no new messages
				}
				3 => {
					data.horizontal_messages.insert(ALICE, vec![MSG_2.clone()]);
				}
				_ => unreachable!(),
			})
			.add(1, || {
				HANDLED_XCMP_MESSAGES.with(|m| {
					let mut m = m.borrow_mut();
					assert_eq!(&*m, &[(ALICE, 1, b"mikhailinvanovich".to_vec())]);
					m.clear();
				});
			})
			.add(2, || {})
			.add(3, || {
				HANDLED_XCMP_MESSAGES.with(|m| {
					let mut m = m.borrow_mut();
					assert_eq!(&*m, &[(ALICE, 3, b"1000000000".to_vec())]);
					m.clear();
				});
			});
	}
}
