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

use codec::{Decode, Encode};
use cumulus_primitives_core::{relay_chain, ParaId};
use scale_info::TypeInfo;
use sp_std::collections::btree_map::BTreeMap;

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
	/// Number of remaining DMP messages.
	pub dmp_remaining_messages: u32,
}

/// The error type for updating bandwidth used by a segment.
pub enum LimitExceededError {
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
	UmpMessagesOverflow {
		messages_remaining: u32,
		messages_submitted: u32,
	},
	/// Too many messages submitted to UMP queue.
	UmpBytesOverflow {
		/// The amount of remaining messages in the capacity of UMP.
		bytes_remaining: u32,
		/// The amount of messages submitted to UMP.
		bytes_submitted: u32,
	},
	/// Too many messages processed from DMP.
	DmpMessagesUnderflow {
		/// The amount of messages waiting to be processed from DMP.
		messages_remaining: u32,
		/// The amount of DMP messages processed.
		messages_processed: u32,
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
	) -> Result<Self, LimitExceededError> {
		let limits = limits
			.hrmp_outgoing
			.get(&recipient)
			.expect("limit for declared hrmp channel must be present; qed");

		let mut new = *self;

		new.msg_count = new.msg_count.saturating_add(other.msg_count);
		if new.msg_count > limits.messages_remaining {
			return Err(LimitExceededError::HrmpMessagesOverflow {
				recipient,
				messages_remaining: limits.messages_remaining,
				messages_submitted: new.msg_count,
			})
		}
		new.total_bytes = new.total_bytes.saturating_add(other.total_bytes);
		if new.total_bytes > limits.bytes_remaining {
			return Err(LimitExceededError::HrmpBytesOverflow {
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
	/// The amount of DMP messages processed.
	pub dmp_processed_count: u32,
}

impl UsedBandwidth {
	/// Tries to append another update, respecting given bandwidth limits.
	fn append(
		&self,
		other: &Self,
		limits: &TotalBandwidthLimits,
	) -> Result<Self, LimitExceededError> {
		let mut new = self.clone();

		new.ump_msg_count = new.ump_msg_count.saturating_add(other.ump_msg_count);
		if new.ump_msg_count > limits.ump_messages_remaining {
			return Err(LimitExceededError::UmpMessagesOverflow {
				messages_remaining: limits.ump_messages_remaining,
				messages_submitted: new.ump_msg_count,
			})
		}
		new.ump_total_bytes = new.ump_total_bytes.saturating_add(other.ump_total_bytes);
		if new.ump_total_bytes > limits.ump_bytes_remaining {
			return Err(LimitExceededError::UmpBytesOverflow {
				bytes_remaining: limits.ump_bytes_remaining,
				bytes_submitted: new.ump_total_bytes,
			})
		}
		new.dmp_processed_count = new.dmp_processed_count.saturating_add(other.dmp_processed_count);
		if new.dmp_processed_count > limits.dmp_remaining_messages {
			return Err(LimitExceededError::DmpMessagesUnderflow {
				messages_remaining: limits.dmp_remaining_messages,
				messages_processed: new.dmp_processed_count,
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
		self.dmp_processed_count -= other.dmp_processed_count;

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
pub struct Ancestor {
	/// Bandwidth used by this block.
	used_bandwidth: UsedBandwidth,
	/// Output head data of this block.
	para_head: relay_chain::HeadData,
}

impl Ancestor {
	/// Returns [`UsedBandwidth`] of this block.
	pub fn used_bandwidth(&self) -> &UsedBandwidth {
		&self.used_bandwidth
	}

	/// Returns [output head data](`relay_chain::HeadData`) of this block.
	pub fn para_head(&self) -> &relay_chain::HeadData {
		&self.para_head
	}
}

/// Struct that keeps track of bandwidth used by the unincluded part of the chain
/// along with the latest HRMP watermark.
#[derive(Encode, Decode, TypeInfo)]
pub struct SegmentTracker {
	/// Bandwidth used by the segment.
	used_bandwidth: UsedBandwidth,
	/// The mark which specifies the block number up to which all inbound HRMP messages are processed.
	hrmp_watermark: relay_chain::BlockNumber,
}

impl SegmentTracker {
	/// Tries to append another block to the tracker, respecting given bandwidth limits.
	pub fn append(
		&mut self,
		block: &Ancestor,
		hrmp_watermark: relay_chain::BlockNumber,
		limits: &TotalBandwidthLimits,
	) -> Result<(), LimitExceededError> {
		self.used_bandwidth = self.used_bandwidth.append(block.used_bandwidth(), limits)?;
		self.hrmp_watermark = hrmp_watermark;

		Ok(())
	}

	/// Removes previously added block from the tracker.
	pub fn subtract(&mut self, block: &Ancestor) {
		self.used_bandwidth.subtract(block.used_bandwidth());
	}
}
