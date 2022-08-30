// Copyright 2022 Parity Technologies (UK) Ltd.
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

use sc_client_api::{blockchain::Backend as _, Backend, HeaderBackend as _};
use sp_runtime::traits::{Block as BlockT, NumberFor, UniqueSaturatedInto};
use std::{collections::HashMap, sync::Arc};

/// Value good enough to be used with parachains using the current backend implementation
/// that ships with Substrate. This value may change in the future.
pub const MAX_LEAVES_PER_LEVEL_SENSIBLE_DEFAULT: usize = 32;

/// Upper bound to the number of leaves allowed for each level of the blockchain.
///
/// If the limit is set and more leaves are detected on block import, then the older ones are
/// dropped to make space for the fresh blocks.
///
/// In environments where blocks confirmations from the relay chain may be "slow", then
/// setting an upper bound helps keeping the chain health by dropping old (presumably) stale
/// leaves and prevents discarding new blocks because we've reached the backend max value.
pub enum LevelLimit {
	/// Limit set to `MAX_LEAVES_PER_LEVEL_SENSIBLE_DEFAULT`.
	Default,
	/// No explicit limit, however a limit may be implicitly imposed by the backend implementation.
	None,
	/// Custom value.
	Some(usize),
}

// Support structure to constrain the number of leaves for each level.
pub struct LevelMonitor<Block: BlockT, BE> {
	// Max number of leaves for each level.
	pub level_limit: usize,
	// Monotonic counter used to keep track of block import age (used as a timestamp).
	pub import_counter: u64,
	// Map between leaves hashes and insertion timestamp.
	pub import_counters: HashMap<Block::Hash, u64>,
	// Blockhain levels cache.
	pub levels: HashMap<NumberFor<Block>, Vec<Block::Hash>>,
	// Backend reference to remove leaves on level saturation.
	pub backend: Arc<BE>,
}

// // Threshold after which we are going to cleanup our internal leaves cache by removing hashes that
// // doesn't belong to leaf blocks anymore.
// // This is a farly arbitrary value that can be changed in the future without breaking anything.
// const CLEANUP_THRESHOLD: usize = 64;

impl<Block: BlockT, BE> LevelMonitor<Block, BE> {
	pub fn new(level_limit: usize, backend: Arc<BE>) -> Self {
		LevelMonitor {
			level_limit,
			import_counter: 0,
			import_counters: HashMap::new(),
			levels: HashMap::new(),
			backend,
		}
	}
}

impl<Block, BE> LevelMonitor<Block, BE>
where
	Block: BlockT,
	BE: Backend<Block>,
{
	pub fn update(&mut self, number: NumberFor<Block>, hash: Block::Hash) {
		self.import_counters.insert(hash, self.import_counter);
		self.levels.entry(number).or_default().push(hash);
		self.import_counter += 1;
	}

	pub fn check(&mut self, number: NumberFor<Block>) {
		if self.import_counter == 0 {
			// First call detected, restore using backend.
			self.restore();
		}

		let level = self.levels.entry(number).or_default();
		if level.len() < self.level_limit {
			return
		}

		log::debug!(
			target: "parachain",
			"Detected leaves overflow at height {}, removing old blocks", number
		);

		let blockchain = self.backend.blockchain();
		let best = blockchain.info().best_hash;

		let mut leaves = blockchain.leaves().unwrap_or_default();
		// Sort leaves by import time.
		leaves
			.sort_unstable_by(|a, b| self.import_counters.get(a).cmp(&self.import_counters.get(b)));

		let remove_count = level.len() - self.level_limit + 1;

		let mut remove_one = || {
			let mut candidate_idx = 0;
			let mut candidate_routes = vec![];
			let mut candidate_fresher_leaf_idx = usize::MAX;

			for (blk_idx, blk_hash) in level.iter().enumerate() {
				let mut blk_leaves_routes = vec![];
				let mut blk_fresher_leaf_idx = 0;
				// Start from the most current one (the bottom)
				// Here leaf index is synonym of its freshness, that is the greater the index the
				// fresher it is.
				for (leaf_idx, leaf_hash) in leaves.iter().enumerate().rev() {
					let leaf_route =
						match sp_blockchain::tree_route(blockchain, *blk_hash, *leaf_hash) {
							Ok(route) => route,
							Err(e) => {
								log::warn!(
									target: "parachain",
									"Unable getting route from {} to {}: {}",
									blk_hash, leaf_hash, e,
								);
								continue
							},
						};
					if leaf_route.retracted().is_empty() {
						if leaf_idx > candidate_fresher_leaf_idx ||
							leaf_route.common_block().hash == best ||
							leaf_route.enacted().iter().any(|entry| entry.hash == best)
						{
							blk_leaves_routes.clear();
							break
						}

						if blk_fresher_leaf_idx == 0 {
							blk_fresher_leaf_idx = leaf_idx;
						}
						blk_leaves_routes.push(leaf_route);
					}
				}

				if !blk_leaves_routes.is_empty() {
					candidate_idx = blk_idx;
					candidate_routes = blk_leaves_routes;
					candidate_fresher_leaf_idx = blk_fresher_leaf_idx;
				}
			}

			// We now have all the routes to remove
			for route in candidate_routes.iter() {
				std::iter::once(route.common_block()).chain(route.enacted()).rev().for_each(
					|elem| {
						self.import_counters.remove(&elem.hash);
						if let Err(err) = self.backend.remove_leaf_block(&elem.hash) {
							log::warn!(target: "parachain", "Unable to remove block {}: {}", elem.hash, err);
						}
					},
				);
			}

			level.remove(candidate_idx);
		};

		// This is not the most efficient way to remove multiple entries,
		// but sould be considered that remove count is commonly 1.
		(0..remove_count).for_each(|_| remove_one());
	}

	pub fn restore(&mut self) {
		let leaves = self.backend.blockchain().leaves().unwrap_or_default();
		let finalized = self.backend.blockchain().last_finalized().unwrap_or_default();
		let mut counter_max = 0;

		for leaf in leaves {
			let route = sp_blockchain::tree_route(self.backend.blockchain(), finalized, leaf)
				.expect("Route from finalized to leaf should be present; qed");
			if !route.retracted().is_empty() {
				continue
			}
			std::iter::once(route.common_block()).chain(route.enacted()).for_each(|elem| {
				if !self.import_counters.contains_key(&elem.hash) {
					self.import_counter = elem.number.unique_saturated_into();
					self.update(elem.number, elem.hash);
				}
			});
			if self.import_counter > counter_max {
				counter_max = self.import_counter;
			}
		}
		self.import_counter = counter_max;
	}
}
