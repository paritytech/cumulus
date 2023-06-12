// Copyright (C) 2023 Parity Technologies (UK) Ltd.
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

//! Primitives for bridge transfer pallet.

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use xcm::prelude::*;

mod asset_filter;
pub use asset_filter::*;

mod config;
pub use config::*;

/// Represents some `MultiLocation` with information if we need to pay fees or not.
#[derive(Clone, Debug, PartialEq)]
pub struct MaybePaidLocation {
	pub location: MultiLocation,
	pub maybe_fee: Option<MultiAsset>,
}

/// Represents ensured/verified reachable destination.
#[cfg_attr(feature = "std", derive(Debug, PartialEq))]
pub struct ReachableDestination {
	/// Bridge location
	pub bridge: MaybePaidLocation,
	/// Target location (e.g. remote parachain in different consensus)
	pub target: MaybePaidLocation,
	/// Destination on target (e.g. account on remote parachain in different consensus)
	pub target_destination: MultiLocation,
}

/// Ensures if `remote_destination` is reachable for requested `remote_network`.
pub trait EnsureReachableDestination {
	fn ensure_destination(
		remote_network: NetworkId,
		remote_destination: MultiLocation,
	) -> Result<ReachableDestination, ReachableDestinationError>;
}

/// Error type for [EnsureReachableDestination]
pub enum ReachableDestinationError {
	UnsupportedDestination,
	UnsupportedXcmVersion,
}
