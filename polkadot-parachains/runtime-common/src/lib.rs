// Copyright (c) 2019 Alain Brenzikofer
// This file is part of Encointer
//
// Encointer is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Encointer is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Encointer.  If not, see <http://www.gnu.org/licenses/>.

//! Common definitions for runtimes. It contains both: definitions by encointer, and
//! definitions from the statemine runtime.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod deal_with_fees;
pub mod weights;

// copied from cumulus/parachains/runtimes/assets/statemine/constants
mod constants;
pub use constants::{currency, fee};
