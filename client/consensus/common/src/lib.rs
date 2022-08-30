// Copyright 2019-2021 Parity Technologies (UK) Ltd.
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

use polkadot_primitives::v2::{Hash as PHash, PersistedValidationData};

use sc_client_api::{
	blockchain::{Backend as _, HeaderBackend as _},
	Backend,
};
use sc_consensus::BlockImport;
use sp_api::NumberFor;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT, UniqueSaturatedInto};

use std::{collections::HashMap, sync::Arc};

mod parachain_consensus;
#[cfg(test)]
mod tests;

pub use parachain_consensus::run_parachain_consensus;

/// The result of [`ParachainConsensus::produce_candidate`].
pub struct ParachainCandidate<B> {
	/// The block that was built for this candidate.
	pub block: B,
	/// The proof that was recorded while building the block.
	pub proof: sp_trie::StorageProof,
}

/// A specific parachain consensus implementation that can be used by a collator to produce candidates.
///
/// The collator will call [`Self::produce_candidate`] every time there is a free core for the parachain
/// this collator is collating for. It is the job of the consensus implementation to decide if this
/// specific collator should build a candidate for the given relay chain block. The consensus
/// implementation could, for example, check whether this specific collator is part of a staked set.
#[async_trait::async_trait]
pub trait ParachainConsensus<B: BlockT>: Send + Sync + dyn_clone::DynClone {
	/// Produce a new candidate at the given parent block and relay-parent blocks.
	///
	/// Should return `None` if the consensus implementation decided that it shouldn't build a
	/// candidate or if there occurred any error.
	///
	/// # NOTE
	///
	/// It is expected that the block is already imported when the future resolves.
	async fn produce_candidate(
		&mut self,
		parent: &B::Header,
		relay_parent: PHash,
		validation_data: &PersistedValidationData,
	) -> Option<ParachainCandidate<B>>;
}

dyn_clone::clone_trait_object!(<B> ParachainConsensus<B> where B: BlockT);

#[async_trait::async_trait]
impl<B: BlockT> ParachainConsensus<B> for Box<dyn ParachainConsensus<B> + Send + Sync> {
	async fn produce_candidate(
		&mut self,
		parent: &B::Header,
		relay_parent: PHash,
		validation_data: &PersistedValidationData,
	) -> Option<ParachainCandidate<B>> {
		(*self).produce_candidate(parent, relay_parent, validation_data).await
	}
}

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
struct LevelMonitor<Block: BlockT, BE> {
	// Max number of leaves for each level.
	level_limit: usize,
	// Monotonic counter used to keep track of block import age (used as a timestamp).
	import_counter: u64,
	// Map between leaves hashes and insertion timestamp.
	import_counters: HashMap<Block::Hash, u64>,
	// Blockhain levels cache.
	levels: HashMap<NumberFor<Block>, Vec<Block::Hash>>,
	// Backend reference to remove leaves on level saturation.
	backend: Arc<BE>,
}

// // Threshold after which we are going to cleanup our internal leaves cache by removing hashes that
// // doesn't belong to leaf blocks anymore.
// // This is a farly arbitrary value that can be changed in the future without breaking anything.
// const CLEANUP_THRESHOLD: usize = 64;

impl<Block, BE> LevelMonitor<Block, BE>
where
	Block: BlockT,
	BE: Backend<Block>,
{
	fn update(&mut self, number: NumberFor<Block>, hash: Block::Hash) {
		self.import_counters.insert(hash, self.import_counter);
		self.levels.entry(number).or_default().push(hash);
		self.import_counter += 1;
	}

	fn check(&mut self, number: NumberFor<Block>) {
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

	fn restore(&mut self) {
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

/// Parachain specific block import.
///
/// This is used to set `block_import_params.fork_choice` to `false` as long as the block origin is
/// not `NetworkInitialSync`. The best block for parachains is determined by the relay chain. Meaning
/// we will update the best block, as it is included by the relay-chain.
pub struct ParachainBlockImport<Block: BlockT, BI, BE> {
	inner: BI,
	level_monitor: Option<LevelMonitor<Block, BE>>,
}

impl<Block: BlockT, BI, BE> ParachainBlockImport<Block, BI, BE> {
	/// Create a new instance.
	pub fn new(inner: BI, backend: Arc<BE>, level_leaves_max: LevelLimit) -> Self {
		let level_limit = match level_leaves_max {
			LevelLimit::None => None,
			LevelLimit::Some(limit) => Some(limit),
			LevelLimit::Default => Some(MAX_LEAVES_PER_LEVEL_SENSIBLE_DEFAULT),
		};

		let level_monitor = level_limit.map(|level_limit| LevelMonitor {
			level_limit,
			import_counter: 0,
			import_counters: HashMap::new(),
			levels: HashMap::new(),
			backend,
		});

		Self { inner, level_monitor }
	}
}

#[async_trait::async_trait]
impl<Block, BI, BE> BlockImport<Block> for ParachainBlockImport<Block, BI, BE>
where
	Block: BlockT,
	BI: BlockImport<Block> + Send,
	BE: Backend<Block>,
{
	type Error = BI::Error;
	type Transaction = BI::Transaction;

	async fn check_block(
		&mut self,
		block: sc_consensus::BlockCheckParams<Block>,
	) -> Result<sc_consensus::ImportResult, Self::Error> {
		self.inner.check_block(block).await
	}

	async fn import_block(
		&mut self,
		mut params: sc_consensus::BlockImportParams<Block, Self::Transaction>,
		cache: std::collections::HashMap<sp_consensus::CacheKeyId, Vec<u8>>,
	) -> Result<sc_consensus::ImportResult, Self::Error> {
		let number = *params.header.number();
		let hash = params.header.hash();

		if let Some(ref mut monitor) = self.level_monitor {
			monitor.check(number);
		}

		// Best block is determined by the relay chain, or if we are doing the initial sync
		// we import all blocks as new best.
		params.fork_choice = Some(sc_consensus::ForkChoiceStrategy::Custom(
			params.origin == sp_consensus::BlockOrigin::NetworkInitialSync,
		));

		let res = self.inner.import_block(params, cache).await?;

		if let Some(ref mut monitor) = self.level_monitor {
			monitor.update(number, hash);
		}

		Ok(res)
	}
}
