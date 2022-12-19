// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// Types and constants concerning the Ambassador Program.
use frame_support::BoundedVec;
use sp_core::ConstU32;

/// Referendum TrackId type.
pub type TrackId = u16;

/// Referendum tracks ids.
pub mod tracks {
	use super::TrackId;

	pub const CANDIDATE: TrackId = 0;
	pub const AMBASSADOR: TrackId = 1;
	pub const SENIOR_AMBASSADOR: TrackId = 2;
}

/// Members rank type.
pub type Rank = pallet_ranked_collective::Rank;

/// Members ranks.
pub mod ranks {
	use super::Rank;

	pub const CANDIDATE: Rank = 0;
	pub const AMBASSADOR: Rank = 1;
	pub const SENIOR_AMBASSADOR: Rank = 2;
}

/// IPFS compatible CID.
// worst case 2 bytes base and codec, 2 bytes hash type and size, 64 bytes hash digest.
pub type Cid = BoundedVec<u8, ConstU32<68>>;
