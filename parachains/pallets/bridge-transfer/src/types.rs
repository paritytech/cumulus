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

use xcm::prelude::*;

/// Pallet support two kinds of transfer.
#[cfg_attr(feature = "std", derive(Debug, PartialEq))]
pub enum AssetTransferKind {
	/// When we need to do a **reserve** on source chain to **reserve account** and then send it as `ReserveAssetDeposited`
	ReserveBased,
	/// When we need to do a opposite direction, withdraw/burn asset on source chain and send it as `Withdraw/Burn` on target chain from **reserve account**.
	WithdrawReserve,
	/// If not supported/permitted (e.g. missing configuration of trusted reserve location, ...).
	Unsupported,
}

/// Trait for resolving a transfer type for `asset` to `target_location`
pub trait ResolveAssetTransferKind {
	fn resolve(asset: &MultiAsset, target_location: &MultiLocation) -> AssetTransferKind;
}

impl ResolveAssetTransferKind for () {
	fn resolve(_asset: &MultiAsset, _target_location: &MultiLocation) -> AssetTransferKind {
		AssetTransferKind::Unsupported
	}
}
