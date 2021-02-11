// Copyright 2021 Parity Technologies (UK) Ltd.
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

//! The relay-chain provided consensus algoritm for parachains.
//!
//! This is the simplest consensus algorithm you can use when developing a parachain. It is a
//! permission-less consensus algorithm that doesn't require any staking or similar to join as a
//! collator. In this algorithm the consensus is provided by the relay-chain. This works in the
//! following way.
//!
//! 1. Each node that sees itself as a collator is free to build a parachain candidate.
//!
//! 2. This parachain candidate is send to the parachain validators that are part of the relay chain.
//!
//! 3. The parachain validators validate at most X different parachain candidates, where X is the
//! total number of parachain validators.
//!
//! 4. The parachain candidate with the highest number of approvals is choosen by the relay-chain
//! block producer to be backed.
//!
//! 5. After the parachain candidate got backed and included, all collators start at 1.

use cumulus_client_consensus_common::{ParachainConsensus, ParachainCandidate};
use sp_runtime::traits::Block as BlockT;
use sp_consensus::{
	BlockImport, BlockImportParams, BlockOrigin, BlockStatus, Environment, Error as ConsensusError,
	ForkChoiceStrategy, Proposal, Proposer, RecordProof,
};
use cumulus_primitives_core::relay_chain::v1::Hash as PHash;
use cumulus_primitives_parachain_inherent::ParachainInherentData;
use sp_inherents::{InherentData, InherentDataProviders};

const LOG_TARGET: &str = "cumulus-consensus-relay-chain";

/// The implementation of the relay-chain provided consensus for parachains.
pub struct RelayChainConsensus<B: BlockT, PF, BI, PClient, PBackend, PBackend2> {
	para_id: ParaId,
	_phantom: PhantomData<(Block, PBackend)>,
	proposer_factory: Arc<Mutex<PF>>,
	inherent_data_providers: InherentDataProviders,
	block_import: Arc<Mutex<BI>>,
	polkadot_client: Arc<PClient>,
	polkadot_backend: Arc<PBackend2>,
}

impl<B: BlockT> ParachainConsensus<B> {
	/// Get the inherent data with validation function parameters injected
	fn inherent_data(
		&mut self,
		validation_data: &PersistedValidationData,
		relay_parent: PHash,
	) -> Option<InherentData> {
		let mut inherent_data = self
			.inherent_data_providers
			.create_inherent_data()
			.map_err(|e| {
				tracing::error!(
					target: LOG_TARGET,
					error = ?e,
					"Failed to create inherent data.",
				)
			})
			.ok()?;

		let parachain_inherent_data = ParachainInherentData::create_at(
			relay_parent,
			&*self.polkadot_client,
			&*self.polkadot_backend,
			validation_data,
			self.para_id,
		)?;

		inherent_data
			.put_data(
				cumulus_primitives_parachain_inherent::INHERENT_IDENTIFIER,
				&parachain_inherent_data,
			)
			.map_err(|e| {
				error!(
					target: LOG_TARGET,
					"Failed to put the system inherent into inherent data: {:?}", e,
				)
			})
			.ok()?;

		Some(inherent_data)
	}
}

#[async_trait::async_trait]
impl<B: BlockT> ParachainConsensus for RelayChainConsensus<B> {
	async fn produce_candidate(
		&self,
		parent: &B::Header,
		relay_parent: PHash,
		validation_data: &PersistedValidationData,
	) -> ParachainCandidate<B> {
		let proposer_future = self.proposer_factory.lock().init(&parent);

		let proposer = proposer_future
			.await
			.map_err(|e|
				tracing::error!(target: LOG_TARGET, error = ?e, "Could not create proposer.")
			).ok()?;

		let inherent_data = self.inherent_data(&validation_data, relay_parent)?;

		let Proposal {
			block,
			storage_changes,
			proof,
		} = proposer
			.propose(
				inherent_data,
				Default::default(),
				//TODO: Fix this.
				Duration::from_millis(500),
				RecordProof::Yes,
			)
			.await
			.map_err(|e| tracing::error!(target: LOG_TARGET, error = ?e, "Proposing failed."))
			.ok()?;

		let proof = match proof {
			Some(proof) => proof,
			None => {
				tracing::error!(
					target: LOG_TARGET,
					"Proposer did not return the requested proof.",
				);

				return None;
			}
		};

		let (header, extrinsics) = block.clone().deconstruct();

		let mut block_import_params = BlockImportParams::new(BlockOrigin::Own, header);
		block_import_params.body = Some(extrinsics);
		// Best block is determined by the relay chain.
		block_import_params.fork_choice = Some(ForkChoiceStrategy::Custom(false));
		block_import_params.storage_changes = Some(storage_changes);

		if let Err(err) = self
			.block_import
			.lock()
			.import_block(block_import_params, Default::default())
		{
			tracing::error!(
				target: LOG_TARGET,
				at = ?parent.hash(),
				error = ?err,
				"Error importing build block.",
			);

			return None;
		}

		Some(ParachainCandidate { block, proof })
	}
}
