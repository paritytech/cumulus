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

use codec::{Decode, Encode};
use cumulus_primitives_core::{relay_chain, ParaId};
use scale_info::TypeInfo;
use sp_std::collections::btree_map::BTreeMap;

pub struct HrmpOutboundLimits {
	pub bytes_remaining: u32,
	pub messages_remaining: u32,
}

pub struct TotalBandwidthLimits {
	pub ump_messages_remaining: u32,
	pub ump_bytes_remaining: u32,
	pub hrmp_outgoing: BTreeMap<ParaId, HrmpOutboundLimits>,
}

pub enum LimitExceededError {
	HrmpMessagesOverflow { recipient: ParaId, messages_remaining: u32, messages_submitted: u32 },
	HrmpBytesOverflow { recipient: ParaId, bytes_remaining: u32, bytes_submitted: u32 },
	UmpMessagesOverflow { messages_remaining: u32, messages_submitted: u32 },
	UmpBytesOverflow { bytes_remaining: u32, bytes_submitted: u32 },
}

#[derive(Default, Copy, Clone, Encode, Decode, TypeInfo)]
pub struct HrmpChannelSize {
	pub msg_count: u32,
	pub total_bytes: u32,
}

impl HrmpChannelSize {
	fn is_empty(&self) -> bool {
		self.msg_count == 0 && self.total_bytes == 0
	}

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

	fn subtract(&mut self, other: &Self) {
		self.msg_count -= other.msg_count;
		self.total_bytes -= other.total_bytes;
	}
}

#[derive(Default, Clone, Encode, Decode, TypeInfo)]
pub struct UsedBandwidth {
	pub ump_msg_count: u32,
	pub ump_total_bytes: u32,
	pub hrmp_outgoing: BTreeMap<ParaId, HrmpChannelSize>,
}

impl UsedBandwidth {
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

		for (id, channel) in other.hrmp_outgoing.iter() {
			let current = new.hrmp_outgoing.entry(*id).or_default();
			*current = current.append(channel, *id, limits)?;
		}

		Ok(new)
	}

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

#[derive(Encode, Decode, TypeInfo)]
pub struct Ancestor {
	used_bandwidth: UsedBandwidth,
	para_head: relay_chain::HeadData,
}

impl Ancestor {
	pub fn used_bandwidth(&self) -> &UsedBandwidth {
		&self.used_bandwidth
	}

	pub fn para_head(&self) -> &relay_chain::HeadData {
		&self.para_head
	}
}

#[derive(Encode, Decode, TypeInfo)]
pub struct SegmentTracker {
	used_bandwidth: UsedBandwidth,
	hrmp_watermark: relay_chain::BlockNumber,
}

impl SegmentTracker {
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

	pub fn subtract(&mut self, block: &Ancestor) {
		self.used_bandwidth.subtract(block.used_bandwidth());
	}
}
