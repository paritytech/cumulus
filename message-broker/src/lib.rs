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

//! Cumulus message broker pallet.
//!
//! This pallet provides support for handling downward and upward messages.

use sp_inherents::{ProvideInherent, InherentData, InherentIdentifier, MakeFatalError};

/// Something that should be called when a downward message is received.
#[impl_trait_for_tuples::impl_for_tuples(30)]
pub trait DownwardMessageHandler {
	/// Handle the given downward message.
	fn handle_downward_message(msg: &());
}

/// Configuration trait of this pallet.
pub trait Trait: frame_system::Trait {
	/// The downward message handlers that will be informed when a message is received.
	type DownwardMessageHandlers: DownwardMessageHandler;
}

frame_support::decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

	}
}

impl<T: Trait> ProvideInherent for Module<T> {
	type Call = Call<T>;
	type Error = MakeFatalError<()>;
	const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

	fn create_inherent(data: &InherentData) -> Option<Self::Call> {
		let data: T::Moment = extract_inherent_data(data)
			.expect("Gets and decodes timestamp inherent data")
			.saturated_into();

		let next_time = cmp::max(data, Self::now() + T::MinimumPeriod::get());
		Some(Call::set(next_time.into()))
	}
}
