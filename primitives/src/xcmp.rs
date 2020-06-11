// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

//! XMCP related primitives

use polkadot_primitives::parachain::Id as ParaId;

/// A raw XCMP message that is being send between two Parachain's.
pub struct RawXCMPMessage {
	/// Parachain sending the message.
	pub from: ParaId,
	/// Parachain receiving the message.
	pub to: ParaId,
	/// SCALE encoded message.
	pub data: Vec<u8>,
}

/// Something that can handle XCMP messages.
pub trait XCMPMessageHandler<Message: codec::Decode> {
	/// Handle a XCMP message.
	fn handle_xcmp_message(msg: &Message);
}

/// Something that can send XCMP messages.
pub trait XCMPMessageSender<Message: codec::Encode> {
	/// Send a XCMP message.
	fn send_xcmp_message(msg: &Message) -> Result<(), ()>;
}
