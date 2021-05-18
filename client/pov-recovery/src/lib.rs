// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! Parachain specific networking
//!
//! Provides a custom block announcement implementation for parachains
//! that use the relay chain provided consensus. See [`BlockAnnounceValidator`]
//! and [`WaitToAnnounce`] for more information about this implementation.

use sc_client_api::{
	Backend, BlockBackend, BlockImportNotification, BlockchainEvents, Finalizer, UsageProvider,
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::{Error as ClientError, Result as ClientResult};
use sp_consensus::{BlockImport, BlockImportParams, BlockOrigin, BlockStatus, ForkChoiceStrategy};
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, Header as HeaderT, NumberFor},
};

use polkadot_node_primitives::{AvailableData, POV_BOMB_LIMIT};
use polkadot_node_subsystem::messages::AvailabilityRecoveryMessage;
use polkadot_overseer::OverseerHandler;
use polkadot_primitives::v1::{
	Block as PBlock, CandidateReceipt, CommittedCandidateReceipt, Id as ParaId,
	OccupiedCoreAssumption, ParachainHost, SessionIndex,
};

use cumulus_primitives_core::ParachainBlockData;

use codec::Decode;
use futures::{
	channel::oneshot, future, select, stream::FuturesUnordered, Future, FutureExt, Stream,
	StreamExt,
};
use futures_timer::Delay;
use rand::{thread_rng, Rng};

use std::{
	collections::{HashMap, HashSet, VecDeque},
	pin::Pin,
	sync::Arc,
	time::Duration,
};

struct PendingCandidate<Block: BlockT> {
	receipt: CandidateReceipt,
	session_index: SessionIndex,
	block_number: NumberFor<Block>,
}

struct PoVRecovery<Block: BlockT, PC, BI> {
	pending_candidates: HashMap<Block::Hash, PendingCandidate<Block>>,
	next_candidate_to_recover: FuturesUnordered<Pin<Box<dyn Future<Output = Block::Hash> + Send>>>,
	active_candidate_recovery: FuturesUnordered<
		Pin<Box<dyn Future<Output = (Block::Hash, Option<AvailableData>)> + Send>>,
	>,
	recovering_candidates: HashSet<Block::Hash>,
	relay_chain_slot_duration: Duration,
	overseer_handler: OverseerHandler,
	waiting_for_parent: HashMap<Block::Hash, Vec<Block>>,
	parachain_client: Arc<PC>,
	parachain_block_import: Arc<BI>,
}

impl<Block: BlockT, PC: BlockBackend<Block>, BI> PoVRecovery<Block, PC, BI>
where
	for<'a> &'a BI: BlockImport<Block>,
{
	fn new(
		overseer_handler: OverseerHandler,
		relay_chain_slot_duration: Duration,
		parachain_client: Arc<PC>,
		parachain_block_import: Arc<BI>,
	) -> Self {
		Self {
			pending_candidates: HashMap::new(),
			next_candidate_to_recover: Default::default(),
			active_candidate_recovery: Default::default(),
			recovering_candidates: HashSet::new(),
			relay_chain_slot_duration,
			overseer_handler,
			waiting_for_parent: HashMap::new(),
			parachain_client,
			parachain_block_import,
		}
	}

	fn insert_pending_candidate(
		&mut self,
		hash: Block::Hash,
		block_number: NumberFor<Block>,
		receipt: CandidateReceipt,
		session_index: SessionIndex,
	) {
		if self
			.pending_candidates
			.insert(
				hash,
				PendingCandidate {
					block_number,
					receipt,
					session_index,
				},
			)
			.is_some()
		{
			return;
		}

		// Wait some random time, with the maximum being the slot duration of the relay chain
		// before we start to recover the candidate.
		let delay = Delay::new(self.relay_chain_slot_duration.mul_f64(thread_rng().gen()));
		self.next_candidate_to_recover.push(
			async move {
				delay.await;
				hash
			}
			.boxed(),
		);
	}

	/// Inform about an imported block.
	fn block_imported(&mut self, hash: &Block::Hash) {
		self.pending_candidates.remove(&hash);
	}

	// Inform about a finalized block with the given `block_number`.
	fn block_finalized(&mut self, block_number: NumberFor<Block>) {
		self.pending_candidates
			.retain(|_, pc| pc.block_number > block_number);
	}

	/// Recover the candidate for the given `block_hash`.
	async fn recover_candidate(&mut self, block_hash: Block::Hash) {
		let pending_candidate = match self.pending_candidates.remove(&block_hash) {
			Some(pending_candidate) => pending_candidate,
			None => return,
		};

		let (tx, rx) = oneshot::channel();

		self.overseer_handler
			.send_msg(AvailabilityRecoveryMessage::RecoverAvailableData(
				pending_candidate.receipt,
				pending_candidate.session_index,
				None,
				tx,
			))
			.await;

		self.recovering_candidates.insert(block_hash);

		self.active_candidate_recovery.push(
			async move {
				match rx.await {
					Ok(Ok(res)) => (block_hash, Some(res)),
					Ok(Err(error)) => {
						tracing::debug!(
							target: "cumulus-consensus",
							?error,
							?block_hash,
							"Availability recovery failed",
						);
						(block_hash, None)
					}
					Err(_) => {
						tracing::debug!(
							target: "cumulus-consensus",
							"Availability recovery oneshot channel closed",
						);
						(block_hash, None)
					}
				}
			}
			.boxed(),
		);
	}

	/// Handle a recovered candidate.
	async fn handle_candidate_recovered(
		&mut self,
		block_hash: Block::Hash,
		available_data: Option<AvailableData>,
	) {
		self.recovering_candidates.remove(&block_hash);

		let available_data = match available_data {
			Some(data) => data,
			//TODO: clear waiting childs
			None => return,
		};

		let raw_block_data = match sp_maybe_compressed_blob::decompress(
			&available_data.pov.block_data.0,
			POV_BOMB_LIMIT,
		) {
			Ok(r) => r,
			Err(error) => {
				tracing::debug!(target: "cumulus-consensus", ?error, "Failed to decompress PoV");

				//TODO: clear waiting childs

				return;
			}
		};

		let block_data = match ParachainBlockData::<Block>::decode(&mut &raw_block_data[..]) {
			Ok(d) => d,
			Err(error) => {
				tracing::warn!(
					target: "cumulus-consensus",
					?error,
					"Failed to decode parachain block data from recovered PoV",
				);
				//TODO: clear waiting childs

				return;
			}
		};

		let block = block_data.into_block();

		let parent = *block.header().parent_hash();

		match self.parachain_client.block_status(&BlockId::hash(parent)) {
			Ok(BlockStatus::Unknown) => {
				if self.recovering_candidates.contains(&parent) {
					tracing::debug!(
						target: "cumulus-consensus",
						?block_hash,
						parent_hash = ?parent,
						"Parent is still being recovered, waiting.",
					);

					self.waiting_for_parent
						.entry(parent)
						.or_default()
						.push(block);
					return;
				} else {
					tracing::debug!(
						target: "cumulus-consensus",
						?block_hash,
						parent_hash = ?parent,
						"Parent not found while trying to import recovered block.",
					);

					//TODO: clear waiting childs
					return;
				}
			}
			Err(error) => {
				tracing::debug!(
					target: "cumulus-consensus",
					block_hash = ?parent,
					"Error while checking block status",
				);
				//TODO: clear waiting childs
				return;
			}
			// Any other status is fine to "ignore/accept"
			_ => (),
		}

		self.import_block(block).await;
	}

	async fn import_block(&mut self, block: Block) {
		tracing::error!("IMPORTING: {:?}", block);
		let mut blocks = VecDeque::new();
		blocks.push_back(block);

		while let Some(block) = blocks.pop_front() {
			let block_hash = block.hash();
			let (header, body) = block.deconstruct();
			let mut block_import_params =
				BlockImportParams::new(BlockOrigin::ConsensusBroadcast, header);
			block_import_params.fork_choice = Some(ForkChoiceStrategy::Custom(false));
			block_import_params.body = Some(body);

			if let Err(err) = (&*self.parachain_block_import)
				.import_block(block_import_params, Default::default())
				.await
			{
				tracing::debug!(
					target: "cumulus-consensus",
					?block_hash,
					error = ?err,
					"Failed to import block",
				);
			}

			if let Some(waiting) = self.waiting_for_parent.remove(&block_hash) {
				blocks.extend(waiting);
			}
		}
	}

	async fn run(&mut self) {
		loop {
			if self.next_candidate_to_recover.is_empty()
				&& self.active_candidate_recovery.is_empty()
			{
				futures::pending!();
				continue;
			}

			select! {
				recover = self.next_candidate_to_recover.next() => {
					if let Some(block_hash) = recover {
						self.recover_candidate(block_hash).await;
					}
				},
				recovered = self.active_candidate_recovery.next() => {
					if let Some((block_hash, available_data)) = recovered {
						self.handle_candidate_recovered(block_hash, available_data).await;
					}
				},
			}
		}
	}
}
