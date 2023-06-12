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

//! Primitives for matching asset `MultiLocation`.

use xcm::prelude::*;

/// Trait for matching asset location
pub trait MatchAssetLocation {
	fn matches(&self, location: &MultiLocation) -> bool;
}

/// Simple asset location filter
#[derive(Debug)]
pub enum AssetFilter {
	ByMultiLocation(MultiLocationFilter),
}

impl MatchAssetLocation for AssetFilter {
	fn matches(&self, asset_location: &MultiLocation) -> bool {
		match self {
			AssetFilter::ByMultiLocation(by_location) => by_location.matches(asset_location),
		}
	}
}

#[derive(Debug, Default)]
pub struct MultiLocationFilter {
	/// Requested location equals to `MultiLocation`
	pub equals_any: sp_std::vec::Vec<MultiLocation>,
	/// Requested location starts with `MultiLocation`
	pub starts_with_any: sp_std::vec::Vec<MultiLocation>,
}

impl MultiLocationFilter {
	pub fn add_equals(mut self, filter: MultiLocation) -> Self {
		self.equals_any.push(filter);
		self
	}
	pub fn add_starts_with(mut self, filter: MultiLocation) -> Self {
		self.starts_with_any.push(filter);
		self
	}
}

impl MatchAssetLocation for MultiLocationFilter {
	fn matches(&self, location: &MultiLocation) -> bool {
		for filter in &self.equals_any {
			if location.eq(filter) {
				return true
			}
		}
		for filter in &self.starts_with_any {
			if location.starts_with(filter) {
				return true
			}
		}
		false
	}
}
