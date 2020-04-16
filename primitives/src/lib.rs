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

//! Cumulus related primitive types and traits.

/// Inherents identifiers and types related to
pub mod inherents {
	use sp_inherents::InherentIdentifier;

	/// Inherent identifier for downward messages.
	pub const DOWNWARD_MESSAGES_IDENTIFIER: InherentIdentifier = *b"cumdownm";

	/// The type of the inherent downward messages.
	pub type DownwardMessagesType = Vec<()>;
}
