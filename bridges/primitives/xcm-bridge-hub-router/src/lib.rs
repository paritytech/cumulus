// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of Parity Bridges Common.

// Parity Bridges Common is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Bridges Common is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Bridges Common.  If not, see <http://www.gnu.org/licenses/>.

//! Primitives of the `xcm-bridge-hub-router` pallet.

#![cfg_attr(not(feature = "std"), no_std)]

/// Local XCM channel that may report whether it is congested or not.
pub trait LocalXcmChannel {
	/// Returns true if the queue is currently congested.
	fn is_congested() -> bool;
}

impl LocalXcmChannel for () {
	fn is_congested() -> bool {
		false
	}
}
