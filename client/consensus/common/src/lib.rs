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
use sp_runtime::traits::{Block as BlockT, Header as _};

use std::sync::Arc;

mod parachain_consensus;
#[cfg(test)]
mod tests;

pub use parachain_consensus::run_parachain_consensus;

/// Value good enough to be use on parachains under the current backend implementation.
/// Note that this value may change in the future.
pub const MAX_LEAVES_PER_LEVEL_SENSIBLE_DEFAULT: usize = 3;

/// Upper bound for the number of leaves allowed for each chain level.
///
/// The a limit is set and more leaves are detected on block import, then the older ones are
/// eventually dropped to make space for more fresh blocks.
///
/// In environments where blocks confirmations from the relay chain are "slow", then
/// setting an upper bound helps keeping the chain health by dropping old (presumably) stale
/// leaves and prevents discarding new fresh blocks because we've reached the backend max value
/// according to the backend implementation.
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

/// Parachain specific block import.
///
/// This is used to set `block_import_params.fork_choice` to `false` as long as the block origin is
/// not `NetworkInitialSync`. The best block for parachains is determined by the relay chain. Meaning
/// we will update the best block, as it is included by the relay-chain.
pub struct ParachainBlockImport<BI, BE> {
	inner: BI,
	backend: Arc<BE>,
	level_leaves_max: Option<usize>,
}

impl<BI, BE> ParachainBlockImport<BI, BE> {
	/// Create a new instance.
	pub fn new(inner: BI, backend: Arc<BE>, level_leaves_max: LeavesLevelLimit) -> Self {
		let level_leaves_max = match level_leaves_max {
			LeavesLevelLimit::None => None,
			LeavesLevelLimit::Some(limit) => Some(limit),
			LeavesLevelLimit::Default => Some(MAX_LEAVES_PER_LEVEL_SENSIBLE_DEFAULT),
		};
		Self { inner, backend, level_leaves_max }
	}
}

#[async_trait::async_trait]
impl<Block, BI, BE> BlockImport<Block> for ParachainBlockImport<BI, BE>
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
		let number = params.header.number();

		let check_leaves = |level_limit| {
			let blockchain = self.backend.blockchain();
			let mut leaves = blockchain.leaves().unwrap_or_default();

			// First cheap check: the number of leaves at level `number` is always less than the total.
			if leaves.len() < level_limit {
				return
			}

			// Now focus on the leaves at the given height.
			leaves.retain(|hash| {
				blockchain
					.number(*hash)
					.ok()
					.flatten()
					.map(|n| n == *number)
					.unwrap_or_default()
			});
			if leaves.len() < level_limit {
				return
			}

			log::debug!(target: "parachain", "Detected leaves overflow, removing old blocks");

			// TODO: here we assuming that the leaves returned by the backend are sorted
			// by "age" with younger leaves in the back.
			// This is true in our backend implementation, we can add a constraint to the
			// Backend trait leaves() method signature.

			let best = blockchain.info().best_hash;
			let mut remove_count = (leaves.len() + 1) - level_limit;

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

		if let Some(limit) = self.level_leaves_max {
			check_leaves(limit);
		}

		// Best block is determined by the relay chain, or if we are doing the initial sync
		// we import all blocks as new best.
		params.fork_choice = Some(sc_consensus::ForkChoiceStrategy::Custom(
			params.origin == sp_consensus::BlockOrigin::NetworkInitialSync,
		));

		self.inner.import_block(params, cache).await
	}
}
