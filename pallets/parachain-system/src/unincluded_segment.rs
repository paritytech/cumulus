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

pub struct BandwidthLimits {}

pub enum LimitExceededError {}

#[derive(Default, Copy, Clone, Encode, Decode, TypeInfo)]
pub struct HrmpChannelSize {
	pub msg_count: u32,
	pub total_bytes: u32,
}

impl HrmpChannelSize {
	pub fn is_empty(&self) -> bool {
		self.msg_count == 0 && self.total_bytes == 0
	}

	pub fn append(
		&self,
		other: &Self,
		limits: &BandwidthLimits,
	) -> Result<Self, LimitExceededError> {
		let mut new = *self;

		new.msg_count = new.msg_count.saturating_add(other.msg_count);
		new.total_bytes = new.total_bytes.saturating_add(other.total_bytes);

		Ok(new)
	}

	pub fn subtract(&mut self, other: &Self) {
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
	pub fn append(
		&self,
		other: &Self,
		limits: &BandwidthLimits,
	) -> Result<Self, LimitExceededError> {
		let mut new = self.clone();

		new.ump_msg_count = new.ump_msg_count.saturating_add(other.ump_msg_count);
		new.ump_total_bytes = new.ump_total_bytes.saturating_add(other.ump_total_bytes);

		for (id, channel) in other.hrmp_outgoing.iter() {
			new.hrmp_outgoing.entry(*id).or_default().append(channel, limits)?;
		}

		Ok(new)
	}

	pub fn subtract(&mut self, other: &Self) {
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
pub struct BlockTracker {
	used_bandwidth: UsedBandwidth,
	para_head: relay_chain::HeadData,
}

impl BlockTracker {
	pub fn used_bandwidth(&self) -> &UsedBandwidth {
		&self.used_bandwidth
	}

	pub fn para_head(&self) -> &relay_chain::HeadData {
		&self.para_head
	}
}

#[derive(Encode, Decode, TypeInfo)]
pub struct SegmentTracker {
	pub used_bandwidth: UsedBandwidth,
	pub hrmp_watermark: relay_chain::BlockNumber,
}
