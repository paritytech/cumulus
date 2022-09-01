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
use sp_blockchain::TreeRoute;
use sp_runtime::traits::{Block as BlockT, NumberFor, Saturating, UniqueSaturatedInto, Zero};
use std::{
	collections::{HashMap, HashSet},
	sync::Arc,
};

/// Value good enough to be used with parachains using the current backend implementation
/// that ships with Substrate. This value may change in the future.
pub const MAX_LEAVES_PER_LEVEL_SENSIBLE_DEFAULT: usize = 32;

// Counter threshold after which we are going to eventually cleanup our internal data.
const CLEANUP_THRESHOLD: u64 = 32;

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

// Support structure to constrain the number of leaves at each level.
pub struct LevelMonitor<Block: BlockT, BE> {
	// Max number of leaves for each level.
	pub level_limit: usize,
	// Monotonic counter used to keep track of block freshness.
	pub import_counter: u64,
	// Map between blocks hashes and freshness.
	pub freshness: HashMap<Block::Hash, u64>,
	// Blockchain levels cache.
	pub levels: HashMap<NumberFor<Block>, HashSet<Block::Hash>>,
	// Lower level number stored by the levels map.
	pub lowest_level: NumberFor<Block>,
	// Backend reference to remove blocks on level saturation.
	pub backend: Arc<BE>,
}

impl<Block: BlockT, BE> LevelMonitor<Block, BE> {
	/// Instance a new monitor structure.
	pub fn new(level_limit: usize, backend: Arc<BE>) -> Self {
		LevelMonitor {
			level_limit,
			import_counter: 0,
			freshness: HashMap::new(),
			levels: HashMap::new(),
			lowest_level: Zero::zero(),
			backend,
		}
	}
}

impl<Block, BE> LevelMonitor<Block, BE>
where
	Block: BlockT,
	BE: Backend<Block>,
{
	/// Restore the structure using the backend.
	///
	/// Blocks freshness values are inferred from the height and not from the effective import
	/// moment. This is a not accurate but "good-enough" best effort solution.
	///
	/// Level limits are not enforced during this phase.
	fn restore(&mut self) {
		let mut counter_max = 0;
		let finalized = self.backend.blockchain().info().finalized_hash;

		for leaf in self.backend.blockchain().leaves().unwrap_or_default() {
			let route = sp_blockchain::tree_route(self.backend.blockchain(), finalized, leaf)
				.expect("Route from finalized to leaf should be available; qed");
			if !route.retracted().is_empty() {
				continue
			}
			std::iter::once(route.common_block()).chain(route.enacted()).for_each(|elem| {
				if !self.freshness.contains_key(&elem.hash) {
					// Use the block height value as the freshness.
					self.import_counter = elem.number.unique_saturated_into();
					self.update(elem.number, elem.hash);
				}
			});
			if self.import_counter > counter_max {
				counter_max = self.import_counter;
			}
		}

		self.import_counter = counter_max;
		self.lowest_level = self.backend.blockchain().info().finalized_number;
	}

	/// Check and enforce the limit bound at the given height.
	///
	/// In practice this will enforce the given height in having a number of blocks less than
	/// the limit passed to the constructor.
	///
	/// If the given level is found to have a number of blocks greater than or equal the limit
	/// then the limit is enforced by chosing one (or more) blocks to remove.
	/// The removal strategy is driven by the block freshness.
	///
	/// A block freshness is determined by the most recent leaf freshness descending from the block
	/// itself. In other words its freshness is equal to its more "fresh" descendant.
	///
	/// The least "fresh" blocks are eventually removed.
	pub fn enforce_limit(&mut self, number: NumberFor<Block>) {
		if self.import_counter == 0 {
			// First call detected, restore using the backend persistent information.
			self.restore();
		}

		let level = self.levels.entry(number).or_default();
		if level.len() < self.level_limit {
			return
		}

		log::debug!(
			target: "parachain",
			"Detected leaves overflow at height {}, removing obsolete blocks", number
		);

		let mut leaves = self.backend.blockchain().leaves().unwrap_or_default();
		// Sort leaves by freshness (most recent first).
		leaves.sort_unstable_by(|a, b| self.freshness.get(a).cmp(&self.freshness.get(b)));

		// This may not be the most efficient way to remove **multiple** entries, but is the clear
		// one. Should be considered that in normal conditions the number of blocks to remove is 0
		// or 1, it is not worth to complicate the code too much.
		let remove_count = level.len() - self.level_limit + 1;
		(0..remove_count).for_each(|_| self.remove_block(number, &leaves));
	}

	/// This will search for the less fresh block and removes it along with all its descendants.
	/// Leaves should have already been ordered by "freshness" (older first).
	fn remove_block(&mut self, number: NumberFor<Block>, leaves: &[Block::Hash]) {
		let mut candidate_fresher_leaf_idx = usize::MAX;
		let mut candidate_fresher_route = None;

		let blockchain = self.backend.blockchain();
		let best_hash = blockchain.info().best_hash;

		for blk_hash in self.levels.entry(number).or_default().iter() {
			// Start checking from the most fresh leaf (the last). Because of the leaves ordering,
			// here leaf index is synonym of its freshness, that is the greater the index the
			// fresher it is.
			for (leaf_idx, leaf_hash) in leaves.iter().enumerate().rev() {
				match sp_blockchain::tree_route(blockchain, *blk_hash, *leaf_hash) {
					Ok(route) if route.retracted().is_empty() => {
						// Skip this block if it is the ancestor of the best block or if it has
						// a descendant leaf fresher (or equally fresh) than the current candidate.
						if leaf_idx < candidate_fresher_leaf_idx &&
							route.common_block().hash != best_hash &&
							route.enacted().iter().all(|entry| entry.hash != best_hash)
						{
							// We have a new candidate
							candidate_fresher_leaf_idx = leaf_idx;
							candidate_fresher_route = Some(route);
						}
						break
					},
					Err(err) => {
						log::warn!(
							target: "parachain",
							"Unable getting route from {} to {}: {}",
							blk_hash, leaf_hash, err,
						);
					},
					_ => (),
				};
			}
			if candidate_fresher_leaf_idx == 0 {
				// We can't find a candidate with an older leaf.
				break
			}
		}

		let candidate_fresher_route = match candidate_fresher_route {
			Some(route) => route,
			None => return,
		};

		// We have a candidate, proceed removing the blocka and all its descendants.

		let candidate_hash = candidate_fresher_route.common_block().hash;

		// Takes care of route removal. Starts from the leaf and stops as soon as an error is
		// encountered (in this case this is interpreted as the block being not a leaf
		// and it will be removed while removing another route from the same block but to a
		// different leaf.
		let mut remove_route = |route: TreeRoute<Block>| {
			std::iter::once(route.common_block()).chain(route.enacted()).rev().all(|elem| {
				if self.backend.remove_leaf_block(&elem.hash).is_err() {
					return false
				}
				log::debug!(target: "parachain", "Removing block {}", elem.hash);
				self.levels
					.get_mut(&elem.number)
					.and_then(|level| level.remove(&elem.hash).then_some(()));
				self.freshness.remove(&elem.hash);
				true
			});
		};

		remove_route(candidate_fresher_route);

		let to_skip = leaves.len() - candidate_fresher_leaf_idx;
		leaves.iter().rev().skip(to_skip).for_each(|leaf_hash| {
			match sp_blockchain::tree_route(blockchain, candidate_hash, *leaf_hash) {
				Ok(route) if route.retracted().is_empty() => {
					remove_route(route);
				},
				Err(err) => {
					log::warn!(
						target: "parachain",
						"Unable getting route from {} to {}: {}",
						candidate_hash, leaf_hash, err,
					);
				},
				_ => (),
			};
		});
	}

	/// Add a new imported block information to the monitor.
	pub fn update(&mut self, number: NumberFor<Block>, hash: Block::Hash) {
		self.freshness.insert(hash, self.import_counter);
		self.levels.entry(number).or_default().insert(hash);
		self.import_counter += 1;

		if self.import_counter % CLEANUP_THRESHOLD == 0 {
			// Do cleanup once in a while, we are allowed to have some obsolete information.
			let finalized_num = self.backend.blockchain().info().finalized_number;
			let delta: u64 =
				finalized_num.saturating_sub(self.lowest_level).unique_saturated_into();

			for i in 0..delta {
				let number = finalized_num + i.unique_saturated_into();
				self.levels.remove(&number).unwrap_or_default().iter().for_each(|hash| {
					self.freshness.remove(hash);
				});
			}

			self.lowest_level = finalized_num;
		}
	}
}
