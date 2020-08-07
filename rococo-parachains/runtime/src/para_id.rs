// Copyright 2020 Parity Technologies (UK) Ltd.
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

//! Minimal Pallet that injects a ParachainId into Runtime storage from

use frame_support::{decl_module, decl_storage};

use cumulus_primitives::ParaId;

/// Configuration trait of this pallet.
pub trait Trait: frame_system::Trait {}


// This is basically a hack to make the parachain id easily configurable.
// Could also be done differently, but yeah..
// Maybe a runtime interface and host function is a better long-term solution?
decl_storage! {
	trait Store for Module<T: Trait> as ParachainUpgrade {}
	add_extra_genesis {
		config(parachain_id): ParaId;
		build(|config: &Self| {
			crate::ParachainId::set(&config.parachain_id);
		});
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin, system = frame_system {

	}
}
