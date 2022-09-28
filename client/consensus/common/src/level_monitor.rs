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
use sp_runtime::traits::{Block as BlockT, NumberFor, One, Saturating, UniqueSaturatedInto, Zero};
use std::{
	collections::{HashMap, HashSet},
	sync::Arc,
};

/// Value good enough to be used with parachains using the current backend implementation
/// that ships with Substrate. This value may change in the future.
pub const MAX_LEAVES_PER_LEVEL_SENSIBLE_DEFAULT: usize = 32;

// Counter threshold after which we are going to eventually cleanup our internal data.
const CLEANUP_THRESHOLD: u32 = 32;

/// Upper bound to the number of leaves allowed for each level of the blockchain.
///
/// If the limit is set and more leaves are detected on block import, then the older ones are
/// dropped to make space for the fresh blocks.
///
/// In environments where blocks confirmations from the relay chain may be "slow", then
/// setting an upper bound helps keeping the chain health by dropping old (presumably) stale
/// leaves and prevents discarding new blocks because we've reached the backend max value.
pub enum LevelLimit {
	/// Limit set to [`MAX_LEAVES_PER_LEVEL_SENSIBLE_DEFAULT`].
	Default,
	/// No explicit limit, however a limit may be implicitly imposed by the backend implementation.
	None,
	/// Custom value.
	Some(usize),
}

/// Support structure to constrain the number of leaves at each level.
pub struct LevelMonitor<Block: BlockT, BE> {
	// Max number of leaves for each level.
	level_limit: usize,
	// Monotonic counter used to keep track of block freshness.
	pub(crate) import_counter: NumberFor<Block>,
	// Map between blocks hashes and freshness.
	pub(crate) freshness: HashMap<Block::Hash, NumberFor<Block>>,
	// Blockchain levels cache.
	pub(crate) levels: HashMap<NumberFor<Block>, HashSet<Block::Hash>>,
	// Lower level number stored by the levels map.
	lowest_level: NumberFor<Block>,
	// Backend reference to remove blocks on level saturation.
	backend: Arc<BE>,
}

/// Contains information about the target scheduled for removal.
struct TargetInfo<Block: BlockT> {
	/// Index of freshest leaf in the leaves array.
	freshest_leaf_idx: usize,
	/// Route from target to its freshest leaf.
	/// `None` if they are equal.
	freshest_route: Option<TreeRoute<Block>>,
}

impl<Block, BE> LevelMonitor<Block, BE>
where
	Block: BlockT,
	BE: Backend<Block>,
{
	/// Instance a new monitor structure.
	pub fn new(level_limit: usize, backend: Arc<BE>) -> Self {
		let mut monitor = LevelMonitor {
			level_limit,
			import_counter: Zero::zero(),
			freshness: HashMap::new(),
			levels: HashMap::new(),
			lowest_level: Zero::zero(),
			backend,
		};
		monitor.restore();
		monitor
	}

	/// Restore the structure using the backend.
	///
	/// Blocks freshness values are inferred from the height and not from the effective import
	/// moment. This is a not accurate but "good-enough" best effort solution.
	///
	/// Level limits are not enforced during this phase.
	fn restore(&mut self) {
		let mut counter_max = Zero::zero();
		let info = self.backend.blockchain().info();

		self.import_counter = info.finalized_number;
		self.update(info.finalized_number, info.finalized_hash);

		for leaf in self.backend.blockchain().leaves().unwrap_or_default() {
			let route =
				sp_blockchain::tree_route(self.backend.blockchain(), info.finalized_hash, leaf)
					.expect("Route from finalized to leaf should be available; qed");
			if !route.retracted().is_empty() {
				continue
			}
			route.enacted().iter().for_each(|elem| {
				if !self.freshness.contains_key(&elem.hash) {
					// Use the block height value as the freshness.
					self.import_counter = elem.number;
					self.update(elem.number, elem.hash);
				}
			});
			counter_max = std::cmp::max(self.import_counter, counter_max);
		}

		self.import_counter = counter_max;
		self.lowest_level = info.finalized_number;
	}

	/// Check and enforce the limit bound at the given height.
	///
	/// In practice this will enforce the given height in having a number of blocks less than
	/// the limit passed to the constructor.
	///
	/// If the given level is found to have a number of blocks greater than or equal the limit
	/// then the limit is enforced by chosing one (or more) blocks to remove.
	///
	/// The removal strategy is driven by the block freshness.
	///
	/// A block freshness is determined by the most recent leaf freshness descending from the block
	/// itself. In other words its freshness is equal to its more "fresh" descendant.
	///
	/// The least "fresh" blocks are eventually removed.
	pub fn enforce_limit(&mut self, number: NumberFor<Block>) {
		let level_len = self.levels.get(&number).map(|l| l.len()).unwrap_or_default();
		if level_len < self.level_limit {
			return
		}

		log::debug!(
			target: "parachain",
			"Detected leaves overflow at height {}, removing obsolete blocks",
			number,
		);

		let mut leaves = self.backend.blockchain().leaves().unwrap_or_default();
		// Sort leaves by freshness (less fresh first).
		leaves.sort_unstable_by(|a, b| self.freshness.get(a).cmp(&self.freshness.get(b)));

		// This may not be the most efficient way to remove **multiple** entries, but is the easy
		// one :-). Should be considered that in "normal" conditions the number of blocks to remove
		// is 0 or 1, it is not worth to complicate the code too much. One condition that may
		// trigger multiple removals (2+) is if we restart the node using an existing db and a
		// smaller limit wrt the one previously used.
		let remove_count = level_len - self.level_limit + 1;
		(0..remove_count).all(|_| {
			self.find_target(number, &leaves).map_or(false, |target| {
				self.remove_target(target, number, &leaves);
				true
			})
		});
	}

	// Helper function to find the best candidate to be removed.
	//
	// Given a set of blocks with height equal to `number` (potential candidates)
	// 1. For each candidate fetch all the leaves that are descending from it.
	// 2. Set the candidate freshness equal to the fresher of its descending leaves.
	// 3. The target is set as the candidate that is less fresh.
	//
	// Input `leaves` are assumed to be already ordered by "freshness" (less fresh first).
	//
	// Returns the index of the target fresher leaf within `leaves` and the route from target to
	// such leaf.
	fn find_target(
		&self,
		number: NumberFor<Block>,
		leaves: &[Block::Hash],
	) -> Option<TargetInfo<Block>> {
		let mut target_info: Option<TargetInfo<Block>> = None;
		let blockchain = self.backend.blockchain();
		let best_hash = blockchain.info().best_hash;

		// Leaves that where already assigned to some node and thus can be skipped
		// during the search.
		let mut assigned_leaves = HashSet::new();

		let level = self.levels.get(&number)?;

		for blk_hash in level.iter().filter(|hash| **hash != best_hash) {
			// Search for the fresher leaf information for this block
			let candidate_info = leaves
				.iter()
				.enumerate()
				.filter(|(leaf_idx, _)| !assigned_leaves.contains(leaf_idx))
				.rev()
				.find_map(|(leaf_idx, leaf_hash)| {
					if blk_hash == leaf_hash {
						Some(TargetInfo { freshest_leaf_idx: leaf_idx, freshest_route: None })
					} else {
						match sp_blockchain::tree_route(blockchain, *blk_hash, *leaf_hash) {
							Ok(route) if route.retracted().is_empty() => Some(TargetInfo {
								freshest_leaf_idx: leaf_idx,
								freshest_route: Some(route),
							}),
							Err(err) => {
								log::warn!(
									target: "parachain",
									"Unable getting route from {:?} to {:?}: {}",
									blk_hash, leaf_hash, err,
								);
								None
							},
							_ => None,
						}
					}
				});

			let candidate_info = match candidate_info {
				Some(candidate_info) => {
					assigned_leaves.insert(candidate_info.freshest_leaf_idx);
					candidate_info
				},
				None => {
					log::warn!(
						target: "parachain",
						"Unable getting route to any leaf from {:?}",
						blk_hash,
					);
					continue
				},
			};

			// Found fresher leaf for this candidate.
			// This candidate is set as the new target if:
			// 1. its fresher leaf is less fresh than the previous target fresher leaf AND
			// 2. best block is not in its route

			let is_less_fresh = || {
				target_info
					.as_ref()
					.map(|ti| candidate_info.freshest_leaf_idx < ti.freshest_leaf_idx)
					.unwrap_or(true)
			};
			let not_contains_best = || {
				candidate_info
					.freshest_route
					.as_ref()
					.map(|tr| tr.enacted().iter().all(|entry| entry.hash != best_hash))
					.unwrap_or(true)
			};

			if is_less_fresh() && not_contains_best() {
				let early_stop = candidate_info.freshest_leaf_idx == 0;
				target_info = Some(candidate_info);
				if early_stop {
					// We will never find a candidate with an worst freshest leaf than this.
					break
				}
			}
		}

		target_info
	}

	// Remove the target block and all its descendants.
	//
	// Leaves should have already been ordered by "freshness" (less fresh first).
	fn remove_target(
		&mut self,
		target: TargetInfo<Block>,
		number: NumberFor<Block>,
		leaves: &[Block::Hash],
	) {
		let mut remove_leaf = |number, hash| {
			if self.backend.remove_leaf_block(&hash).is_err() {
				return false
			}
			log::debug!(target: "parachain", "Removing block {}", hash);
			self.levels.get_mut(&number).map(|level| level.remove(&hash));
			self.freshness.remove(&hash);
			true
		};

		let target_hash = match target.freshest_route {
			Some(freshest_route) => {
				// Takes care of route removal. Starts from the leaf and stops as soon as an error is
				// encountered. In this case an error is interpreted as the block being not a leaf
				// and it will be removed while removing another route from the same block but to a
				// different leaf.
				let mut remove_route = |route: TreeRoute<Block>| {
					route.enacted().iter().rev().all(|elem| remove_leaf(elem.number, elem.hash));
				};

				let target_hash = freshest_route.common_block().hash;
				debug_assert_eq!(
					freshest_route.common_block().number,
					number,
					"This is a bug in LevelMonitor::find_target() or the Backend is corrupted"
				);

				// Remove freshest (cached) route first.
				remove_route(freshest_route);

				// Don't bother trying with leaves we already found to not be our descendants.
				let to_skip = leaves.len() - target.freshest_leaf_idx;
				leaves.iter().rev().skip(to_skip).for_each(|leaf_hash| {
					match sp_blockchain::tree_route(
						self.backend.blockchain(),
						target_hash,
						*leaf_hash,
					) {
						Ok(route) if route.retracted().is_empty() => remove_route(route),
						Err(err) => {
							log::warn!(
								target: "parachain",
								"Unable getting route from {:?} to {:?}: {}",
								target_hash, leaf_hash, err,
							);
						},
						_ => (),
					};
				});

				target_hash
			},
			None => leaves[target.freshest_leaf_idx],
		};

		remove_leaf(number, target_hash);
	}

	/// Add a new imported block information to the monitor.
	pub fn update(&mut self, number: NumberFor<Block>, hash: Block::Hash) {
		self.freshness.insert(hash, self.import_counter);
		self.levels.entry(number).or_default().insert(hash);
		self.import_counter += One::one();

		// Do cleanup once in a while, we are allowed to have some obsolete information.
		let finalized_num = self.backend.blockchain().info().finalized_number;
		let delta: u32 = finalized_num.saturating_sub(self.lowest_level).unique_saturated_into();
		if delta >= CLEANUP_THRESHOLD {
			for i in 0..delta {
				let number = self.lowest_level + i.unique_saturated_into();
				self.levels.remove(&number).map(|level| {
					level.iter().for_each(|hash| {
						self.freshness.remove(hash);
					})
				});
			}

			self.lowest_level = finalized_num;
		}
	}
}
