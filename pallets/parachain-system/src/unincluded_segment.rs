// Copyright 2023 Parity Technologies (UK) Ltd.
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

//! Primitives used for tracking message queues constraints in an unincluded block segment
//! of the parachain.
//!
//! Unincluded segment describes a chain of latest included block descendants, which are not yet
//! sent to relay chain.

use super::relay_state_snapshot::MessagingStateSnapshot;
use codec::{Decode, Encode};
use cumulus_primitives_core::{relay_chain, ParaId};
use scale_info::TypeInfo;
use sp_std::{collections::btree_map::BTreeMap, marker::PhantomData};

/// Constraints on outbound HRMP channel.
#[derive(Clone)]
pub struct HrmpOutboundLimits {
	/// The maximum bytes that can be written to the channel.
	pub bytes_remaining: u32,
	/// The maximum messages that can be written to the channel.
	pub messages_remaining: u32,
}

/// Limits on outbound message bandwidth.
#[derive(Clone)]
pub struct OutboundBandwidthLimits {
	/// The amount of UMP messages remaining.
	pub ump_messages_remaining: u32,
	/// The amount of UMP bytes remaining.
	pub ump_bytes_remaining: u32,
	/// The limitations of all registered outbound HRMP channels.
	pub hrmp_outgoing: BTreeMap<ParaId, HrmpOutboundLimits>,
}

impl OutboundBandwidthLimits {
	/// Creates new limits from the messaging state and upward message queue maximums fetched
	/// from the host configuration.
	///
	/// These will be the total bandwidth limits across the entire unincluded segment.
	pub fn from_relay_chain_state(
		messaging_state: &MessagingStateSnapshot,
		max_upward_queue_count: u32,
		max_upward_queue_size: u32,
	) -> Self {
		let (ump_messages_in_relay, ump_bytes_in_relay) =
			messaging_state.relay_dispatch_queue_size;

		let (ump_messages_remaining, ump_bytes_remaining) = (
			max_upward_queue_count.saturating_sub(ump_messages_in_relay),
			max_upward_queue_size.saturating_sub(ump_bytes_in_relay),
		);

		let hrmp_outgoing = messaging_state
			.egress_channels
			.iter()
			.map(|(id, channel)| {
				(
					*id,
					HrmpOutboundLimits {
						bytes_remaining: channel.max_total_size.saturating_sub(channel.total_size),
						messages_remaining: channel.max_capacity.saturating_sub(channel.msg_count),
					},
				)
			})
			.collect();

		Self { ump_messages_remaining, ump_bytes_remaining, hrmp_outgoing }
	}

	/// Compute the remaining bandwidth when accounting for the used amounts provided.
	pub fn subtract(&mut self, used: &UsedBandwidth) {
		self.ump_messages_remaining = self.ump_messages_remaining.saturating_sub(used.ump_msg_count);
		self.ump_bytes_remaining = self.ump_bytes_remaining.saturating_sub(used.ump_total_bytes);
		for (para_id, channel_limits) in self.hrmp_outgoing.iter_mut() {
			if let Some(update) = used.hrmp_outgoing.get(para_id) {
				channel_limits.bytes_remaining = channel_limits.bytes_remaining.saturating_sub(update.total_bytes);
				channel_limits.messages_remaining = channel_limits.messages_remaining.saturating_sub(update.msg_count);
			}
		}
	}
}

/// The error type for updating bandwidth used by a segment.
#[derive(Debug)]
pub enum BandwidthUpdateError {
	/// Too many messages submitted to HRMP channel.
	HrmpMessagesOverflow {
		/// Parachain id of the recipient.
		recipient: ParaId,
		/// The amount of remaining messages in the capacity of the channel.
		messages_remaining: u32,
		/// The amount of messages submitted to the channel.
		messages_submitted: u32,
	},
	/// Too many bytes submitted to HRMP channel.
	HrmpBytesOverflow {
		/// Parachain id of the recipient.
		recipient: ParaId,
		/// The amount of remaining bytes in the capacity of the channel.
		bytes_remaining: u32,
		/// The amount of bytes submitted to the channel.
		bytes_submitted: u32,
	},
	/// Too many messages submitted to UMP queue.
	UmpMessagesOverflow {
		/// The amount of remaining messages in the capacity of UMP.
		messages_remaining: u32,
		/// The amount of messages submitted to UMP.
		messages_submitted: u32,
	},
	/// Too many bytes submitted to UMP.
	UmpBytesOverflow {
		/// The amount of remaining bytes in the capacity of UMP.
		bytes_remaining: u32,
		/// The amount of bytes submitted to UMP.
		bytes_submitted: u32,
	},
	/// Invalid HRMP watermark.
	InvalidHrmpWatermark {
		/// HRMP watermark submitted by the candidate.
		submitted: relay_chain::BlockNumber,
		/// Latest tracked HRMP watermark.
		latest: relay_chain::BlockNumber,
	},
}

/// The number of messages and size in bytes submitted to HRMP channel.
#[derive(Debug, Default, Copy, Clone, Encode, Decode, TypeInfo)]
pub struct HrmpChannelUpdate {
	/// The amount of messages submitted to the channel.
	pub msg_count: u32,
	/// The amount of bytes submitted to the channel.
	pub total_bytes: u32,
}

impl HrmpChannelUpdate {
	/// Returns `true` if the update is empty, `false` otherwise.
	fn is_empty(&self) -> bool {
		self.msg_count == 0 && self.total_bytes == 0
	}

	/// Tries to append another update, respecting given bandwidth limits.
	fn append(
		&self,
		other: &Self,
		recipient: ParaId,
		limits: &OutboundBandwidthLimits,
	) -> Result<Self, BandwidthUpdateError> {
		let limits = limits
			.hrmp_outgoing
			.get(&recipient)
			.expect("limit for declared hrmp channel must be present; qed");

		let mut new = *self;

		new.msg_count = new.msg_count.saturating_add(other.msg_count);
		if new.msg_count > limits.messages_remaining {
			return Err(BandwidthUpdateError::HrmpMessagesOverflow {
				recipient,
				messages_remaining: limits.messages_remaining,
				messages_submitted: new.msg_count,
			})
		}
		new.total_bytes = new.total_bytes.saturating_add(other.total_bytes);
		if new.total_bytes > limits.bytes_remaining {
			return Err(BandwidthUpdateError::HrmpBytesOverflow {
				recipient,
				bytes_remaining: limits.bytes_remaining,
				bytes_submitted: new.total_bytes,
			})
		}

		Ok(new)
	}

	/// Subtracts previously added channel update.
	fn subtract(&mut self, other: &Self) {
		self.msg_count -= other.msg_count;
		self.total_bytes -= other.total_bytes;
	}
}

/// Bandwidth used by a parachain block(s).
///
/// This struct can be created with pub items, however, it should
/// never hit the storage directly to avoid bypassing limitations checks.
#[derive(Default, Clone, Encode, Decode, TypeInfo)]
pub struct UsedBandwidth {
	/// The amount of UMP messages sent.
	pub ump_msg_count: u32,
	/// The amount of UMP bytes sent.
	pub ump_total_bytes: u32,
	/// Outbound HRMP channels updates.
	pub hrmp_outgoing: BTreeMap<ParaId, HrmpChannelUpdate>,
}

impl UsedBandwidth {
	/// Tries to append another update, respecting given bandwidth limits.
	fn append(
		&self,
		other: &Self,
		limits: &OutboundBandwidthLimits,
	) -> Result<Self, BandwidthUpdateError> {
		let mut new = self.clone();

		new.ump_msg_count = new.ump_msg_count.saturating_add(other.ump_msg_count);
		if new.ump_msg_count > limits.ump_messages_remaining {
			return Err(BandwidthUpdateError::UmpMessagesOverflow {
				messages_remaining: limits.ump_messages_remaining,
				messages_submitted: new.ump_msg_count,
			})
		}
		new.ump_total_bytes = new.ump_total_bytes.saturating_add(other.ump_total_bytes);
		if new.ump_total_bytes > limits.ump_bytes_remaining {
			return Err(BandwidthUpdateError::UmpBytesOverflow {
				bytes_remaining: limits.ump_bytes_remaining,
				bytes_submitted: new.ump_total_bytes,
			})
		}

		for (id, channel) in other.hrmp_outgoing.iter() {
			let current = new.hrmp_outgoing.entry(*id).or_default();
			*current = current.append(channel, *id, limits)?;
		}

		Ok(new)
	}

	/// Subtracts previously added bandwidth update.
	fn subtract(&mut self, other: &Self) {
		self.ump_msg_count -= other.ump_msg_count;
		self.ump_total_bytes -= other.ump_total_bytes;

		for (id, channel) in other.hrmp_outgoing.iter() {
			let entry = self
				.hrmp_outgoing
				.get_mut(id)
				.expect("entry's been inserted earlier with `append`; qed");
			entry.subtract(channel);
		}

		self.hrmp_outgoing.retain(|_, channel| !channel.is_empty());
	}
}

/// Ancestor of the block being currently executed, not yet included
/// into the relay chain.
#[derive(Encode, Decode, TypeInfo)]
pub struct Ancestor<H> {
	/// Bandwidth used by this block.
	used_bandwidth: UsedBandwidth,
	/// Output head data hash of this block. This may be optional in case the head data has not
	/// yet been posted on chain, but should be updated during initialization of the next block.
	para_head_hash: Option<H>,
}

impl<H> Ancestor<H> {
	/// Creates new ancestor without validating the bandwidth used.
	pub fn new_unchecked(used_bandwidth: UsedBandwidth) -> Self {
		Self { used_bandwidth, para_head_hash: None }
	}

	/// Returns [`UsedBandwidth`] of this block.
	pub fn used_bandwidth(&self) -> &UsedBandwidth {
		&self.used_bandwidth
	}

	/// Returns hashed [output head data](`relay_chain::HeadData`) of this block.
	pub fn para_head_hash(&self) -> Option<&H> {
		self.para_head_hash.as_ref()
	}

	/// Set para head hash of this block.
	pub fn replace_para_head_hash(&mut self, para_head_hash: H) {
		self.para_head_hash.replace(para_head_hash);
	}
}

/// Struct that keeps track of bandwidth used by the unincluded part of the chain
/// along with the latest HRMP watermark.
#[derive(Default, Encode, Decode, TypeInfo)]
pub struct SegmentTracker<H> {
	/// Bandwidth used by the segment.
	used_bandwidth: UsedBandwidth,
	/// The mark which specifies the block number up to which all inbound HRMP messages are processed.
	hrmp_watermark: Option<relay_chain::BlockNumber>,
	/// `H` is the type of para head hash.
	phantom_data: PhantomData<H>,
}

impl<H> SegmentTracker<H> {
	/// Tries to append another block to the tracker, respecting given bandwidth limits.
	/// In practice, the bandwidth limits supplied should be the total allowed within the
	/// block.
	pub fn append(
		&mut self,
		block: &Ancestor<H>,
		hrmp_watermark: relay_chain::BlockNumber,
		limits: &OutboundBandwidthLimits,
	) -> Result<(), BandwidthUpdateError> {
		if let Some(watermark) = self.hrmp_watermark.as_ref() {
			if &hrmp_watermark <= watermark {
				return Err(BandwidthUpdateError::InvalidHrmpWatermark {
					submitted: hrmp_watermark,
					latest: *watermark,
				})
			}
		}

		self.used_bandwidth = self.used_bandwidth.append(block.used_bandwidth(), limits)?;
		self.hrmp_watermark.replace(hrmp_watermark);

		Ok(())
	}

	/// Removes previously added block from the tracker.
	pub fn subtract(&mut self, block: &Ancestor<H>) {
		self.used_bandwidth.subtract(block.used_bandwidth());
		// Watermark doesn't need to be updated since the is always dropped
		// from the tail of the segment.
	}

	/// Return a reference to the used bandwidth across the entire segment.
	pub fn used_bandwidth(&self) -> &UsedBandwidth {
		&self.used_bandwidth
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use assert_matches::assert_matches;

	#[test]
	fn outbound_limits_constructed_correctly() {
		let para_a = ParaId::from(0);
		let para_a_channel = relay_chain::AbridgedHrmpChannel {
			max_message_size: 15,

			// Msg count capacity left is 2.
			msg_count: 5,
			max_capacity: 7,

			// Bytes capacity left is 10.
			total_size: 50,
			max_total_size: 60,
			mqc_head: None,
		};

		let para_b = ParaId::from(1);
		let para_b_channel = relay_chain::AbridgedHrmpChannel {
			max_message_size: 15,

			// Msg count capacity left is 10.
			msg_count: 40,
			max_capacity: 50,

			// Bytes capacity left is 0.
			total_size: 500,
			max_total_size: 500,
			mqc_head: None,
		};
		let messaging_state = MessagingStateSnapshot {
			dmq_mqc_head: relay_chain::Hash::zero(),
			// (msg_count, bytes)
			relay_dispatch_queue_size: (10, 100),
			ingress_channels: Vec::new(),

			egress_channels: vec![(para_a, para_a_channel), (para_b, para_b_channel)],
		};

		let max_upward_queue_count = 11;
		let max_upward_queue_size = 150;
		let limits = OutboundBandwidthLimits::from_relay_chain_state(
			&messaging_state, max_upward_queue_count, max_upward_queue_size
		);

		// UMP.
		assert_eq!(limits.ump_messages_remaining, 1);
		assert_eq!(limits.ump_bytes_remaining, 50);

		// HRMP.
		let para_a_limits = limits.hrmp_outgoing.get(&para_a).expect("channel must be present");
		let para_b_limits = limits.hrmp_outgoing.get(&para_b).expect("channel must be present");
		assert_eq!(
			para_a_limits.bytes_remaining, 10
		);
		assert_eq!(
			para_a_limits.messages_remaining, 2
		);
		assert_eq!(
			para_b_limits.bytes_remaining, 0
		);
		assert_eq!(
			para_b_limits.messages_remaining, 10
		);
	}

	#[test]
	fn hrmp_msg_count_limits() {
		let para_0 = ParaId::from(0);
		let para_0_limits = HrmpOutboundLimits { bytes_remaining: u32::MAX, messages_remaining: 5 };

		let para_1 = ParaId::from(1);
		let para_1_limits = HrmpOutboundLimits { bytes_remaining: u32::MAX, messages_remaining: 3 };
		let hrmp_outgoing = [(para_0, para_0_limits), (para_1, para_1_limits)].into();
		let limits = OutboundBandwidthLimits {
			ump_messages_remaining: 0,
			ump_bytes_remaining: 0,
			hrmp_outgoing,
		};

		let mut hrmp_update = HrmpChannelUpdate::default();
		assert!(hrmp_update.is_empty());

		for _ in 0..5 {
			hrmp_update = hrmp_update
				.append(&HrmpChannelUpdate { msg_count: 1, total_bytes: 10 }, para_0, &limits)
				.expect("update is withing the limits");
		}
		assert_matches!(
			hrmp_update.append(
				&HrmpChannelUpdate { msg_count: 1, total_bytes: 10 },
				para_0,
				&limits,
			),
			Err(BandwidthUpdateError::HrmpMessagesOverflow {
				recipient,
				messages_remaining,
				messages_submitted,
			}) if recipient == para_0 && messages_remaining == 5 && messages_submitted == 6
		);

		let mut hrmp_update = HrmpChannelUpdate::default();
		hrmp_update = hrmp_update
			.append(&HrmpChannelUpdate { msg_count: 2, total_bytes: 10 }, para_1, &limits)
			.expect("update is withing the limits");
		assert_matches!(
			hrmp_update.append(
				&HrmpChannelUpdate { msg_count: 3, total_bytes: 10 },
				para_1,
				&limits,
			),
			Err(BandwidthUpdateError::HrmpMessagesOverflow {
				recipient,
				messages_remaining,
				messages_submitted,
			}) if recipient == para_1 && messages_remaining == 3 && messages_submitted == 5
		);
	}

	#[test]
	fn hrmp_bytes_limits() {
		let para_0 = ParaId::from(0);
		let para_0_limits =
			HrmpOutboundLimits { bytes_remaining: 25, messages_remaining: u32::MAX };

		let hrmp_outgoing = [(para_0, para_0_limits)].into();
		let limits = OutboundBandwidthLimits {
			ump_messages_remaining: 0,
			ump_bytes_remaining: 0,
			hrmp_outgoing,
		};

		let mut hrmp_update = HrmpChannelUpdate::default();
		assert!(hrmp_update.is_empty());

		for _ in 0..5 {
			hrmp_update = hrmp_update
				.append(&HrmpChannelUpdate { msg_count: 1, total_bytes: 4 }, para_0, &limits)
				.expect("update is withing the limits");
		}
		assert_matches!(
			hrmp_update.append(
				&HrmpChannelUpdate { msg_count: 1, total_bytes: 6 },
				para_0,
				&limits,
			),
			Err(BandwidthUpdateError::HrmpBytesOverflow {
				recipient,
				bytes_remaining,
				bytes_submitted,
			}) if recipient == para_0 && bytes_remaining == 25 && bytes_submitted == 26
		);
	}

	#[test]
	fn hrmp_limits_with_segment() {
		let create_used_hrmp =
			|hrmp_outgoing| UsedBandwidth { ump_msg_count: 0, ump_total_bytes: 0, hrmp_outgoing };

		let para_0 = ParaId::from(0);
		let para_0_limits = HrmpOutboundLimits { bytes_remaining: 30, messages_remaining: 10 };

		let para_1 = ParaId::from(1);
		let para_1_limits = HrmpOutboundLimits { bytes_remaining: 20, messages_remaining: 3 };
		let hrmp_outgoing = [(para_0, para_0_limits), (para_1, para_1_limits)].into();
		let limits = OutboundBandwidthLimits {
			ump_messages_remaining: 0,
			ump_bytes_remaining: 0,
			hrmp_outgoing,
		};

		let mut segment = SegmentTracker::default();

		let para_0_update = HrmpChannelUpdate { msg_count: 1, total_bytes: 6 };
		let ancestor_0 = Ancestor {
			used_bandwidth: create_used_hrmp([(para_0, para_0_update)].into()),
			para_head_hash: None::<relay_chain::Hash>,
		};
		segment.append(&ancestor_0, 0, &limits).expect("update is withing the limits");

		for watermark in 1..5 {
			let ancestor = Ancestor {
				used_bandwidth: create_used_hrmp([(para_0, para_0_update)].into()),
				para_head_hash: None::<relay_chain::Hash>,
			};
			segment
				.append(&ancestor, watermark, &limits)
				.expect("update is withing the limits");
		}

		let para_0_update = HrmpChannelUpdate { msg_count: 1, total_bytes: 1 };
		let ancestor_5 = Ancestor {
			used_bandwidth: create_used_hrmp([(para_0, para_0_update)].into()),
			para_head_hash: None::<relay_chain::Hash>,
		};
		assert_matches!(
			segment.append(&ancestor_5, 5, &limits),
			Err(BandwidthUpdateError::HrmpBytesOverflow {
				recipient,
				bytes_remaining,
				bytes_submitted,
			}) if recipient == para_0 && bytes_remaining == 30 && bytes_submitted == 31
		);
		// Remove the first ancestor from the segment to make space.
		segment.subtract(&ancestor_0);
		segment.append(&ancestor_5, 5, &limits).expect("update is withing the limits");

		let para_1_update = HrmpChannelUpdate { msg_count: 3, total_bytes: 10 };
		let ancestor = Ancestor {
			used_bandwidth: create_used_hrmp([(para_1, para_1_update)].into()),
			para_head_hash: None::<relay_chain::Hash>,
		};
		segment.append(&ancestor, 6, &limits).expect("update is withing the limits");

		assert_matches!(
			segment.append(&ancestor, 7, &limits),
			Err(BandwidthUpdateError::HrmpMessagesOverflow {
				recipient,
				messages_remaining,
				messages_submitted,
			}) if recipient == para_1 && messages_remaining == 3 && messages_submitted == 6
		);
	}

	#[test]
	fn ump_limits_with_segment() {
		let create_used_ump = |(ump_msg_count, ump_total_bytes)| UsedBandwidth {
			ump_msg_count,
			ump_total_bytes,
			hrmp_outgoing: BTreeMap::default(),
		};

		let limits = OutboundBandwidthLimits {
			ump_messages_remaining: 5,
			ump_bytes_remaining: 50,
			hrmp_outgoing: BTreeMap::default(),
		};

		let mut segment = SegmentTracker::default();

		let ancestor_0 = Ancestor {
			used_bandwidth: create_used_ump((1, 10)),
			para_head_hash: None::<relay_chain::Hash>,
		};
		segment.append(&ancestor_0, 0, &limits).expect("update is withing the limits");

		for watermark in 1..4 {
			let ancestor = Ancestor {
				used_bandwidth: create_used_ump((1, 10)),
				para_head_hash: None::<relay_chain::Hash>,
			};
			segment
				.append(&ancestor, watermark, &limits)
				.expect("update is withing the limits");
		}

		let ancestor_4 = Ancestor {
			used_bandwidth: create_used_ump((1, 30)),
			para_head_hash: None::<relay_chain::Hash>,
		};
		assert_matches!(
			segment.append(&ancestor_4, 4, &limits),
			Err(BandwidthUpdateError::UmpBytesOverflow {
				bytes_remaining,
				bytes_submitted,
			}) if bytes_remaining == 50 && bytes_submitted == 70
		);

		let ancestor = Ancestor {
			used_bandwidth: create_used_ump((1, 5)),
			para_head_hash: None::<relay_chain::Hash>,
		};
		segment.append(&ancestor, 4, &limits).expect("update is withing the limits");
		assert_matches!(
			segment.append(&ancestor, 5, &limits),
			Err(BandwidthUpdateError::UmpMessagesOverflow {
				messages_remaining,
				messages_submitted,
			}) if messages_remaining == 5 && messages_submitted == 6
		);
	}

	#[test]
	fn segment_hrmp_watermark() {
		let mut segment = SegmentTracker::default();

		let ancestor = Ancestor {
			used_bandwidth: UsedBandwidth::default(),
			para_head_hash: None::<relay_chain::Hash>,
		};
		let limits = OutboundBandwidthLimits {
			ump_messages_remaining: 0,
			ump_bytes_remaining: 0,
			hrmp_outgoing: BTreeMap::default(),
		};

		segment
			.append(&ancestor, 0, &limits)
			.expect("nothing to compare the watermark with in default segment");
		assert_matches!(
			segment.append(&ancestor, 0, &limits),
			Err(BandwidthUpdateError::InvalidHrmpWatermark {
				submitted,
				latest,
			}) if submitted == 0 && latest == 0
		);

		for watermark in 1..5 {
			segment.append(&ancestor, watermark, &limits).expect("hrmp watermark is valid");
		}
		for watermark in 0..5 {
			assert_matches!(
				segment.append(&ancestor, watermark, &limits),
				Err(BandwidthUpdateError::InvalidHrmpWatermark {
					submitted,
					latest,
				}) if submitted == watermark && latest == 4
			);
		}
	}

	#[test]
	fn segment_drops_empty_hrmp_channels() {
		let create_used_hrmp =
			|hrmp_outgoing| UsedBandwidth { ump_msg_count: 0, ump_total_bytes: 0, hrmp_outgoing };

		let para_0 = ParaId::from(0);
		let para_0_limits =
			HrmpOutboundLimits { bytes_remaining: u32::MAX, messages_remaining: u32::MAX };

		let para_1 = ParaId::from(1);
		let para_1_limits =
			HrmpOutboundLimits { bytes_remaining: u32::MAX, messages_remaining: u32::MAX };
		let hrmp_outgoing = [(para_0, para_0_limits), (para_1, para_1_limits)].into();
		let limits = OutboundBandwidthLimits {
			ump_messages_remaining: 0,
			ump_bytes_remaining: 0,
			hrmp_outgoing,
		};

		let mut segment = SegmentTracker::default();

		let para_0_update = HrmpChannelUpdate { msg_count: 1, total_bytes: 1 };
		let ancestor_0 = Ancestor {
			used_bandwidth: create_used_hrmp([(para_0, para_0_update)].into()),
			para_head_hash: None::<relay_chain::Hash>,
		};
		segment.append(&ancestor_0, 0, &limits).expect("update is withing the limits");
		let para_1_update = HrmpChannelUpdate { msg_count: 3, total_bytes: 10 };
		let ancestor_1 = Ancestor {
			used_bandwidth: create_used_hrmp([(para_1, para_1_update)].into()),
			para_head_hash: None::<relay_chain::Hash>,
		};
		segment.append(&ancestor_1, 1, &limits).expect("update is withing the limits");

		assert_eq!(segment.used_bandwidth.hrmp_outgoing.len(), 2);

		segment.subtract(&ancestor_0);
		assert_eq!(segment.used_bandwidth.hrmp_outgoing.len(), 1);

		segment.subtract(&ancestor_1);
		assert_eq!(segment.used_bandwidth.hrmp_outgoing.len(), 0);
	}
}
