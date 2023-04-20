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
pub struct HrmpOutboundLimits {
	/// The maximum bytes that can be written to the channel.
	pub bytes_remaining: u32,
	/// The maximum messages that can be written to the channel.
	pub messages_remaining: u32,
}

/// Constraints imposed on the entire segment, i.e. based on the latest included parablock.
pub struct TotalBandwidthLimits {
	/// The amount of UMP messages remaining.
	pub ump_messages_remaining: u32,
	/// The amount of UMP bytes remaining.
	pub ump_bytes_remaining: u32,
	/// The limitations of all registered outbound HRMP channels.
	pub hrmp_outgoing: BTreeMap<ParaId, HrmpOutboundLimits>,
}

impl TotalBandwidthLimits {
	/// Creates new limits from the messaging state.
	pub fn new(messaging_state: &MessagingStateSnapshot) -> Self {
		let (ump_messages_remaining, ump_bytes_remaining) =
			messaging_state.relay_dispatch_queue_size;
		let hrmp_outgoing = messaging_state
			.egress_channels
			.iter()
			.map(|(id, channel)| {
				(
					*id,
					HrmpOutboundLimits {
						bytes_remaining: channel.max_total_size,
						messages_remaining: channel.max_capacity,
					},
				)
			})
			.collect();

		Self { ump_messages_remaining, ump_bytes_remaining, hrmp_outgoing }
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
#[derive(Default, Copy, Clone, Encode, Decode, TypeInfo)]
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
		limits: &TotalBandwidthLimits,
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
		limits: &TotalBandwidthLimits,
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
	pub fn append(
		&mut self,
		block: &Ancestor<H>,
		hrmp_watermark: relay_chain::BlockNumber,
		limits: &TotalBandwidthLimits,
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
}
