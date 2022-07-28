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
use sc_consensus::BlockImport;
use sp_runtime::traits::{Block as BlockT, Header as _};

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

/// Parachain specific block import.
///
/// This is used to set `block_import_params.fork_choice` to `false` as long as the block origin is
/// not `NetworkInitialSync`. The best block for parachains is determined by the relay chain. Meaning
/// we will update the best block, as it is included by the relay-chain.
pub struct ParachainBlockImport<I>(I);

impl<I> ParachainBlockImport<I> {
	/// Create a new instance.
	pub fn new(inner: I) -> Self {
		Self(inner)
	}
}

extern "Rust" {
	fn get_leaves() -> Vec<sp_core::H256>;
	fn del_leaf(hash: &sp_core::H256);
}

#[async_trait::async_trait]
impl<Block, I> BlockImport<Block> for ParachainBlockImport<I>
where
	Block: BlockT,
	I: BlockImport<Block> + Send,
{
	type Error = I::Error;
	type Transaction = I::Transaction;

	async fn check_block(
		&mut self,
		block: sc_consensus::BlockCheckParams<Block>,
	) -> Result<sc_consensus::ImportResult, Self::Error> {
		self.0.check_block(block).await
	}

	async fn import_block(
		&mut self,
		mut params: sc_consensus::BlockImportParams<Block, Self::Transaction>,
		cache: std::collections::HashMap<sp_consensus::CacheKeyId, Vec<u8>>,
	) -> Result<sc_consensus::ImportResult, Self::Error> {
		log::debug!(target: "parachain", ">>>>>>>>>>>>>>>>>>>>> Importing block @ {}", params.header.number());
		let leaves = unsafe { get_leaves() };
		log::debug!(target: "parachain", ">>>>>>>>>>>>>>>>>>>>> Leaves Number: {}", leaves.len());

		const MAX_LEAVES: usize = 2;
		if leaves.len() > MAX_LEAVES {
			// Per interface contract, we know that the leaves are ordered from the highest.
			// For this PoC just remove one of the lowers.
			// TODO: Here we have to check the num of leaves at level of the block we are importing
			// TODO: Better strategy, here we're just removin the first one
			let leaf = &leaves[leaves.len() - 1];
			log::debug!(target: "parachain", ">>>>>>>>>>>>>>>>>>>>> Removing block: {}", leaf);
			unsafe { del_leaf(leaf) };
			let leaves = unsafe { get_leaves() };
			log::debug!(target: "parachain", ">>>>>>>>>>>>>>>>>>>>> Leaves Number (post-del): {}", leaves.len());
		}

		// Best block is determined by the relay chain, or if we are doing the initial sync
		// we import all blocks as new best.
		params.fork_choice = Some(sc_consensus::ForkChoiceStrategy::Custom(
			params.origin == sp_consensus::BlockOrigin::NetworkInitialSync,
		));

		// Check if we require to prune a leaf before trying to import block

		let res = self.0.import_block(params, cache).await;
		if let Err(err) = &res {
			dbg!(&err);
			log::error!("RAW ERRROR: {:?}", err);
			let err_str = err.to_string();
			log::error!("ERROR IMPORTING: {}", err_str);
		}
		res
	}
}
