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
use sc_consensus::{BlockImport, ImportResult};
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};

use std::{collections::HashMap, sync::Arc};

mod parachain_consensus;
#[cfg(test)]
mod tests;

pub use parachain_consensus::run_parachain_consensus;

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
pub enum LeavesLevelLimit {
	/// Limit set to `MAX_LEAVES_PER_LEVEL_SENSIBLE_DEFAULT`.
	Default,
	/// No explicit limit, however a limit may be implicitly imposed by the backend implementation.
	None,
	/// Custom value.
	Some(usize),
}

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

type BlockHash<Block> = <<Block as BlockT>::Header as HeaderT>::Hash;

struct LeavesCache<Block: BlockT> {
	level_limit: usize,
	import_counter: u64,
	import_map: HashMap<BlockHash<Block>, u64>,
}

/// Parachain specific block import.
///
/// This is used to set `block_import_params.fork_choice` to `false` as long as the block origin is
/// not `NetworkInitialSync`. The best block for parachains is determined by the relay chain. Meaning
/// we will update the best block, as it is included by the relay-chain.
pub struct ParachainBlockImport<Block: BlockT, BI, BE> {
	inner: BI,
	backend: Arc<BE>,
	leaves_cache: Option<LeavesCache<Block>>,
}

impl<Block: BlockT, BI, BE> ParachainBlockImport<Block, BI, BE> {
	/// Create a new instance.
	pub fn new(inner: BI, backend: Arc<BE>, level_leaves_max: LeavesLevelLimit) -> Self {
		let level_limit = match level_leaves_max {
			LeavesLevelLimit::None => None,
			LeavesLevelLimit::Some(limit) => Some(limit),
			LeavesLevelLimit::Default => Some(MAX_LEAVES_PER_LEVEL_SENSIBLE_DEFAULT),
		};
		let leaves_cache = level_limit.map(|limit| LeavesCache {
			level_limit: limit,
			import_map: HashMap::new(),
			import_counter: 0,
		});
		Self { inner, backend, leaves_cache }
	}
}

use std::collections::HashSet;

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
		let hash = params.header.hash();
		let number = *params.header.number();

		let check_leaves = |leaves_cache: &mut LeavesCache<_>| {
			let blockchain = self.backend.blockchain();
			let mut leaves = blockchain.leaves().unwrap_or_default();

			// First cheap check: the number of leaves at level `number` is always less than the total.
			if leaves.len() < leaves_cache.level_limit {
				return
			}

			let mut leaves_set = HashSet::with_capacity(leaves.len());

			// Now focus on the leaves at the given height.
			leaves.retain(|hash| {
				// Pick all the leaves in our temporary leaves set to cleanup the leaves cache.
				leaves_set.insert(*hash);

				// Exloit this iteration to cleanup our cache
				blockchain.number(*hash).ok().flatten().map(|n| n == number).unwrap_or_default()
			});

			// Take the opportunity to cleanup our leaves cache.
			// We use the leaves HashSet to have a O(1) complexity.
			leaves_cache.import_map.retain(|hash, _| leaves_set.contains(hash));

			if leaves.len() < leaves_cache.level_limit {
				return
			}

			log::debug!(
                target: "parachain",
                "Detected leaves overflow at height {}, removing old blocks", number);

			let best = blockchain.info().best_hash;
			let mut remove_count = (leaves.len() + 1) - leaves_cache.level_limit;

			// Sort by import chronological order
			// With Substrate the leaves for one level are already given ordered by
			// import time, so this will be cheap.
			leaves.sort_unstable_by(|a, b| {
				leaves_cache.import_map.get(a).cmp(&leaves_cache.import_map.get(b))
			});

			for hash in leaves.into_iter().filter(|hash| *hash != best) {
				if self.backend.remove_leaf_block(&hash).is_err() {
					log::warn!(target: "parachain", "Unable to remove block {}, skipping it...", hash);
					continue
				}
				remove_count -= 1;
				if remove_count == 0 {
					break
				}
			}
		};

		if let Some(leaves_cache) = &mut self.leaves_cache {
			check_leaves(leaves_cache);
		}

		// Best block is determined by the relay chain, or if we are doing the initial sync
		// we import all blocks as new best.
		params.fork_choice = Some(sc_consensus::ForkChoiceStrategy::Custom(
			params.origin == sp_consensus::BlockOrigin::NetworkInitialSync,
		));

		let res = self.inner.import_block(params, cache).await;
		if let Ok(ImportResult::Imported(_)) = res {
			if let Some(leaves_cache) = &mut self.leaves_cache {
				leaves_cache.import_counter += 1;
				leaves_cache.import_map.insert(hash, leaves_cache.import_counter);
			}
		}

		res
	}
}
