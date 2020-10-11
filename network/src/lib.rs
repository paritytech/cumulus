// Copyright 2019 Parity Technologies (UK) Ltd.
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

#[cfg(test)]
mod tests;

use sp_api::ProvideRuntimeApi;
use sp_blockchain::{Error as ClientError, HeaderBackend};
use sp_consensus::{
	block_validation::{BlockAnnounceValidator as BlockAnnounceValidatorT, Validation},
	SyncOracle,
};
use sp_core::traits::SpawnNamed;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, Header as HeaderT},
};

use polkadot_node_primitives::{SignedFullStatement, Statement};
use polkadot_node_subsystem::messages::StatementDistributionMessage;
use polkadot_overseer::OverseerHandler;
use polkadot_primitives::v1::{
	Block as PBlock, Hash as PHash, Id as ParaId, OccupiedCoreAssumption, ParachainHost,
	SigningContext,
};

use codec::{Decode, Encode};
use futures::{
	channel::{mpsc, oneshot},
	future::{ready, FutureExt},
	pin_mut, select, Future, StreamExt,
};
use log::{trace, warn};

use std::{marker::PhantomData, pin::Pin, sync::Arc};

/// Parachain specific block announce validator.
///
/// This block announce validator is required if the parachain is running
/// with the relay chain provided consensus to make sure each node only
/// imports a reasonable number of blocks per round. The relay chain provided
/// consensus doesn't have any authorities and so it could happen that without
/// this special block announce validator a node would need to import *millions*
/// of blocks per round, which is clearly not doable.
///
/// To solve this problem, each block announcement is delayed until a collator
/// has received a [`Statement::Seconded`] for its `PoV`. This message tells the
/// collator that its `PoV` was validated successfully by a parachain validator and
/// that it is very likely that this `PoV` will be included in the relay chain. Every
/// collator that doesn't receive the message for its `PoV` will not announce its block.
/// For more information on the block announcement, see [`WaitToAnnounce`].
///
/// For each block announcement that is received, the generic block announcement validation
/// will call this validator and provides the extra data that was attached to the announcement.
/// We call this extra data `justification`.
/// It is expected that the attached data is a SCALE encoded [`SignedFullStatement`]. The
/// statement is checked to be a [`Statement::Seconded`] and that it is signed by an active
/// parachain validator.
///
/// If no justification was provided we check if the block announcement is at the tip of the known
/// chain. If it is at the tip, it is required to provide a justification or otherwise we reject
/// it. However, if the announcement is for a block below the tip the announcement is accepted
/// as it probably comes from a node that is currently syncing the chain.
pub struct BlockAnnounceValidator<B, P> {
	phantom: PhantomData<B>,
	polkadot_client: Arc<P>,
	para_id: ParaId,
	polkadot_sync_oracle: Box<dyn SyncOracle + Send>,
}

impl<B, P> BlockAnnounceValidator<B, P> {
	pub fn new(
		polkadot_client: Arc<P>,
		para_id: ParaId,
		polkadot_sync_oracle: Box<dyn SyncOracle + Send>,
	) -> Self {
		Self {
			phantom: Default::default(),
			polkadot_client,
			para_id,
			polkadot_sync_oracle,
		}
	}
}

impl<B: BlockT, P> BlockAnnounceValidatorT<B> for BlockAnnounceValidator<B, P>
where
	P: ProvideRuntimeApi<PBlock> + HeaderBackend<PBlock> + 'static,
	P::Api: ParachainHost<PBlock>,
{
	fn validate(
		&mut self,
		header: &B::Header,
		mut data: &[u8],
	) -> Pin<Box<dyn Future<Output = Result<Validation, Box<dyn std::error::Error + Send>>> + Send>>
	{
		if self.polkadot_sync_oracle.is_major_syncing() {
			return ready(Ok(Validation::Success { is_new_best: false })).boxed();
		}

		let runtime_api = self.polkadot_client.runtime_api();
		let polkadot_info = self.polkadot_client.info();

		if data.is_empty() {
			let polkadot_client = self.polkadot_client.clone();
			let header = header.clone();
			let para_id = self.para_id;

			return async move {
				// Check if block is equal or higher than best (this requires a justification)
				let runtime_api_block_id = BlockId::Hash(polkadot_info.best_hash);
				let block_number = header.number();

				let local_validation_data = polkadot_client
					.runtime_api()
					.persisted_validation_data(
						&runtime_api_block_id,
						para_id,
						OccupiedCoreAssumption::TimedOut,
					)
					.map_err(|e| Box::new(ClientError::Msg(format!("{:?}", e))) as Box<_>)?
					.ok_or_else(|| {
						Box::new(ClientError::Msg(
							"Could not find parachain head in relay chain".into(),
						)) as Box<_>
					})?;
				let parent_head = B::Header::decode(&mut &local_validation_data.parent_head.0[..])
					.map_err(|e| {
						Box::new(ClientError::Msg(format!(
							"Failed to decode parachain head: {:?}",
							e
						))) as Box<_>
					})?;
				let known_best_number = parent_head.number();

				if block_number >= known_best_number {
					trace!(
						target: "cumulus-network",
						"validation failed because a justification is needed if the block at the top of the chain."
					);

					Ok(Validation::Failure)
				} else {
					Ok(Validation::Success { is_new_best: false })
				}
			}
			.boxed();
		}

		let signed_stmt = match SignedFullStatement::decode(&mut data) {
			Ok(r) => r,
			Err(_) => return ready(Err(Box::new(ClientError::BadJustification(
				"cannot decode block announcement justification, must be a `SignedFullStatement`"
					.to_string(),
			)) as Box<_>))
			.boxed(),
		};

		// Check statement is a candidate statement.
		let candidate_receipt = match signed_stmt.payload() {
			Statement::Seconded(ref candidate_receipt) => candidate_receipt,
			_ => {
				return ready(Err(Box::new(ClientError::BadJustification(
					"block announcement justification must be a `Statement::Seconded`".to_string(),
				)) as Box<_>))
				.boxed()
			}
		};

		// Check that the relay chain parent of the block is the relay chain head
		let best_number = polkadot_info.best_number;
		let validator_index = signed_stmt.validator_index();
		let relay_parent = &candidate_receipt.descriptor.relay_parent;

		match self.polkadot_client.number(*relay_parent) {
			Err(err) => {
				return ready(Err(Box::new(ClientError::Backend(format!(
					"could not find block number for {}: {}",
					relay_parent, err,
				))) as Box<_>))
				.boxed();
			}
			Ok(Some(x)) if x == best_number => {}
			Ok(None) => {
				return ready(Err(
					Box::new(ClientError::UnknownBlock(relay_parent.to_string())) as Box<_>,
				))
				.boxed();
			}
			Ok(Some(_)) => {
				trace!(
					target: "cumulus-network",
					"validation failed because the relay chain parent ({}) is not the relay chain \
					head ({})",
					relay_parent,
					best_number,
				);

				return ready(Ok(Validation::Failure)).boxed();
			}
		}

		let runtime_api_block_id = BlockId::Hash(*relay_parent);
		let session_index = match runtime_api.session_index_for_child(&runtime_api_block_id) {
			Ok(r) => r,
			Err(e) => {
				return ready(Err(Box::new(ClientError::Msg(format!("{:?}", e))) as Box<_>)).boxed()
			}
		};

		let signing_context = SigningContext {
			parent_hash: *relay_parent,
			session_index,
		};

		// Check that the signer is a legit validator.
		let authorities = match runtime_api.validators(&runtime_api_block_id) {
			Ok(r) => r,
			Err(e) => {
				return ready(Err(Box::new(ClientError::Msg(format!("{:?}", e))) as Box<_>)).boxed()
			}
		};
		let signer = match authorities.get(validator_index as usize) {
			Some(r) => r,
			None => {
				return ready(Err(Box::new(ClientError::BadJustification(
					"block accouncement justification signer is a validator index out of bound"
						.to_string(),
				)) as Box<_>))
				.boxed()
			}
		};

		// Check statement is correctly signed.
		if signed_stmt
			.check_signature(&signing_context, &signer)
			.is_err()
		{
			return ready(Err(Box::new(ClientError::BadJustification(
				"block announced justification signature is invalid".to_string(),
			)) as Box<_>))
			.boxed();
		}

		// Check the header in the candidate_receipt match header given header.
		if header.encode() != candidate_receipt.commitments.head_data.0 {
			return ready(Err(Box::new(ClientError::BadJustification(
				"block announced header does not match the one justified".to_string(),
			)) as Box<_>))
			.boxed();
		}

		ready(Ok(Validation::Success { is_new_best: true })).boxed()
	}
}

/// Wait before announcing a block that a candidate message has been received for this block, then
/// add this message as justification for the block announcement.
///
/// This object will spawn a new task every time the method `wait_to_announce` is called and cancel
/// the previous task running.
pub struct WaitToAnnounce<Block: BlockT> {
	spawner: Arc<dyn SpawnNamed + Send + Sync>,
	announce_block: Arc<dyn Fn(Block::Hash, Vec<u8>) + Send + Sync>,
	overseer_handler: OverseerHandler,
	current_trigger: oneshot::Sender<()>,
}

impl<Block: BlockT> WaitToAnnounce<Block> {
	/// Create the `WaitToAnnounce` object
	pub fn new(
		spawner: Arc<dyn SpawnNamed + Send + Sync>,
		announce_block: Arc<dyn Fn(Block::Hash, Vec<u8>) + Send + Sync>,
		overseer_handler: OverseerHandler,
	) -> WaitToAnnounce<Block> {
		let (tx, _rx) = oneshot::channel();

		WaitToAnnounce {
			spawner,
			announce_block,
			overseer_handler,
			current_trigger: tx,
		}
	}

	/// Wait for a candidate message for the block, then announce the block. The candidate
	/// message will be added as justification to the block announcement.
	pub fn wait_to_announce(&mut self, block_hash: <Block as BlockT>::Hash, pov_hash: PHash) {
		let (tx, rx) = oneshot::channel();
		let announce_block = self.announce_block.clone();
		let overseer_handler = self.overseer_handler.clone();

		self.current_trigger = tx;

		self.spawner.spawn(
			"cumulus-wait-to-announce",
			async move {
				let t1 = wait_to_announce::<Block>(
					block_hash,
					pov_hash,
					announce_block,
					overseer_handler,
				)
				.fuse();
				let t2 = rx.fuse();

				pin_mut!(t1, t2);

				trace!(
					target: "cumulus-network",
					"waiting for announce block in a background task...",
				);

				select! {
					_ = t1 => {
						trace!(
							target: "cumulus-network",
							"block announcement finished",
						);
					},
					_ = t2 => {
						trace!(
							target: "cumulus-network",
							"previous task that waits for announce block has been canceled",
						);
					}
				}
			}
			.boxed(),
		);
	}
}

async fn wait_to_announce<Block: BlockT>(
	block_hash: <Block as BlockT>::Hash,
	pov_hash: PHash,
	announce_block: Arc<dyn Fn(Block::Hash, Vec<u8>) + Send + Sync>,
	mut overseer_handler: OverseerHandler,
) {
	let (sender, mut receiver) = mpsc::channel(5);
	if overseer_handler
		.send_msg(StatementDistributionMessage::RegisterStatementListener(
			sender,
		))
		.await
		.is_err()
	{
		warn!(
			target: "cumulus-network",
			"Failed to register the statement listener!",
		);
		return;
	}

	while let Some(statement) = receiver.next().await {
		match &statement.payload() {
			Statement::Seconded(c) if &c.descriptor.pov_hash == &pov_hash => {
				announce_block(block_hash, statement.encode());

				break;
			}
			_ => {}
		}
	}
}
