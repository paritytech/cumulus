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
//! 4. The parachain candidate that is backed by the most validators is choosen by the relay-chain
//! block producer to be added as backed candidate on chain.
//!
//! 5. After the parachain candidate got backed and included, all collators start at 1.

use cumulus_client_consensus_common::{ParachainCandidate, ParachainConsensus};
use cumulus_primitives_core::{
	relay_chain::v1::{Block as PBlock, Hash as PHash, ParachainHost},
	ParaId, PersistedValidationData,
};
use cumulus_primitives_parachain_inherent::ParachainInherentData;
pub use import_queue::import_queue;
use parking_lot::Mutex;
use polkadot_service::ClientHandle;
use sc_client_api::Backend;
use sp_api::ProvideRuntimeApi;
use sp_consensus::{
	BlockImport, BlockImportParams, BlockOrigin, EnableProofRecording, Environment,
	ForkChoiceStrategy, ProofRecording, Proposal, Proposer,
};
use sp_inherents::{InherentData, InherentDataProviders};
use sp_runtime::traits::{Block as BlockT, HashFor, Header as HeaderT};
use std::{marker::PhantomData, sync::Arc, time::Duration};

mod import_queue;

const LOG_TARGET: &str = "cumulus-consensus-relay-chain";

/// The implementation of the relay-chain provided consensus for parachains.
pub struct RelayChainConsensus<B, PF, BI, RClient, RBackend> {
	para_id: ParaId,
	_phantom: PhantomData<B>,
	proposer_factory: Arc<Mutex<PF>>,
	inherent_data_providers: InherentDataProviders,
	block_import: Arc<futures::lock::Mutex<BI>>,
	relay_chain_client: Arc<RClient>,
	relay_chain_backend: Arc<RBackend>,
}

impl<B, PF, BI, RClient, RBackend> Clone for RelayChainConsensus<B, PF, BI, RClient, RBackend> {
	fn clone(&self) -> Self {
		Self {
			para_id: self.para_id,
			_phantom: PhantomData,
			proposer_factory: self.proposer_factory.clone(),
			inherent_data_providers: self.inherent_data_providers.clone(),
			block_import: self.block_import.clone(),
			relay_chain_backend: self.relay_chain_backend.clone(),
			relay_chain_client: self.relay_chain_client.clone(),
		}
	}
}

impl<B, PF, BI, RClient, RBackend> RelayChainConsensus<B, PF, BI, RClient, RBackend>
where
	B: BlockT,
	RClient: ProvideRuntimeApi<PBlock>,
	RClient::Api: ParachainHost<PBlock>,
	RBackend: Backend<PBlock>,
{
	/// Create a new instance of relay-chain provided consensus.
	pub fn new(
		para_id: ParaId,
		proposer_factory: PF,
		inherent_data_providers: InherentDataProviders,
		block_import: BI,
		polkadot_client: Arc<RClient>,
		polkadot_backend: Arc<RBackend>,
	) -> Self {
		Self {
			para_id,
			proposer_factory: Arc::new(Mutex::new(proposer_factory)),
			inherent_data_providers,
			block_import: Arc::new(futures::lock::Mutex::new(block_import)),
			relay_chain_backend: polkadot_backend,
			relay_chain_client: polkadot_client,
			_phantom: PhantomData,
		}
	}

	/// Get the inherent data with validation function parameters injected
	fn inherent_data(
		&self,
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
			&*self.relay_chain_client,
			&*self.relay_chain_backend,
			validation_data,
			self.para_id,
		)?;

		inherent_data
			.put_data(
				cumulus_primitives_parachain_inherent::INHERENT_IDENTIFIER,
				&parachain_inherent_data,
			)
			.map_err(|e| {
				tracing::error!(
					target: LOG_TARGET,
					error = ?e,
					"Failed to put the system inherent into inherent data.",
				)
			})
			.ok()?;

		Some(inherent_data)
	}
}

#[async_trait::async_trait]
impl<B, PF, BI, RClient, RBackend> ParachainConsensus<B>
	for RelayChainConsensus<B, PF, BI, RClient, RBackend>
where
	B: BlockT,
	RClient: ProvideRuntimeApi<PBlock> + Send + Sync,
	RClient::Api: ParachainHost<PBlock>,
	RBackend: Backend<PBlock>,
	BI: BlockImport<B> + Send + Sync,
	PF: Environment<B> + Send + Sync,
	PF::Proposer: Proposer<
		B,
		Transaction = BI::Transaction,
		ProofRecording = EnableProofRecording,
		Proof = <EnableProofRecording as ProofRecording>::Proof,
	>,
{
	async fn produce_candidate(
		&mut self,
		parent: &B::Header,
		relay_parent: PHash,
		validation_data: &PersistedValidationData,
	) -> Option<ParachainCandidate<B>> {
		let proposer_future = self.proposer_factory.lock().init(&parent);

		let proposer = proposer_future
			.await
			.map_err(
				|e| tracing::error!(target: LOG_TARGET, error = ?e, "Could not create proposer."),
			)
			.ok()?;

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
			)
			.await
			.map_err(|e| tracing::error!(target: LOG_TARGET, error = ?e, "Proposing failed."))
			.ok()?;

		let (header, extrinsics) = block.clone().deconstruct();

		let mut block_import_params = BlockImportParams::new(BlockOrigin::Own, header);
		block_import_params.body = Some(extrinsics);
		// Best block is determined by the relay chain.
		block_import_params.fork_choice = Some(ForkChoiceStrategy::Custom(false));
		block_import_params.storage_changes = Some(storage_changes);

		if let Err(err) = self
			.block_import
			.lock()
			.await
			.import_block(block_import_params, Default::default())
			.await
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

/// Paramaters of [`build_relay_chain_consensus`].
pub struct BuildRelayChainConsensusParams<PF, BI, RBackend> {
	pub para_id: ParaId,
	pub proposer_factory: PF,
	pub inherent_data_providers: InherentDataProviders,
	pub block_import: BI,
	pub relay_chain_client: polkadot_service::Client,
	pub relay_chain_backend: Arc<RBackend>,
}

/// Build the [`RelayChainConsensus`].
///
/// Returns a boxed [`ParachainConsensus`].
pub fn build_relay_chain_consensus<Block, PF, BI, RBackend>(
	BuildRelayChainConsensusParams {
		para_id,
		proposer_factory,
		inherent_data_providers,
		block_import,
		relay_chain_client,
		relay_chain_backend,
	}: BuildRelayChainConsensusParams<PF, BI, RBackend>,
) -> Box<dyn ParachainConsensus<Block>>
where
	Block: BlockT,
	PF: Environment<Block> + Send + Sync + 'static,
	PF::Proposer: Proposer<
		Block,
		Transaction = BI::Transaction,
		ProofRecording = EnableProofRecording,
		Proof = <EnableProofRecording as ProofRecording>::Proof,
	>,
	BI: BlockImport<Block> + Send + Sync + 'static,
	RBackend: Backend<PBlock> + 'static,
	// Rust bug: https://github.com/rust-lang/rust/issues/24159
	sc_client_api::StateBackendFor<RBackend, PBlock>: sc_client_api::StateBackend<HashFor<PBlock>>,
{
	RelayChainConsensusBuilder::new(
		para_id,
		proposer_factory,
		block_import,
		inherent_data_providers,
		relay_chain_client,
		relay_chain_backend,
	)
	.build()
}

/// Relay chain consensus builder.
///
/// Builds a [`RelayChainConsensus`] for a parachain. As this requires
/// a concrete relay chain client instance, the builder takes a [`polkadot_service::Client`]
/// that wraps this concrete instanace. By using [`polkadot_service::ExecuteWithClient`]
/// the builder gets access to this concrete instance.
struct RelayChainConsensusBuilder<Block, PF, BI, RBackend> {
	para_id: ParaId,
	_phantom: PhantomData<Block>,
	proposer_factory: PF,
	inherent_data_providers: InherentDataProviders,
	block_import: BI,
	relay_chain_backend: Arc<RBackend>,
	relay_chain_client: polkadot_service::Client,
}

impl<Block, PF, BI, RBackend> RelayChainConsensusBuilder<Block, PF, BI, RBackend>
where
	Block: BlockT,
	// Rust bug: https://github.com/rust-lang/rust/issues/24159
	sc_client_api::StateBackendFor<RBackend, PBlock>: sc_client_api::StateBackend<HashFor<PBlock>>,
	PF: Environment<Block> + Send + Sync + 'static,
	PF::Proposer: Proposer<
		Block,
		Transaction = BI::Transaction,
		ProofRecording = EnableProofRecording,
		Proof = <EnableProofRecording as ProofRecording>::Proof,
	>,
	BI: BlockImport<Block> + Send + Sync + 'static,
	RBackend: Backend<PBlock> + 'static,
{
	/// Create a new instance of the builder.
	fn new(
		para_id: ParaId,
		proposer_factory: PF,
		block_import: BI,
		inherent_data_providers: InherentDataProviders,
		relay_chain_client: polkadot_service::Client,
		relay_chain_backend: Arc<RBackend>,
	) -> Self {
		Self {
			para_id,
			_phantom: PhantomData,
			proposer_factory,
			block_import,
			inherent_data_providers,
			relay_chain_backend,
			relay_chain_client,
		}
	}

	/// Build the relay chain consensus.
	fn build(self) -> Box<dyn ParachainConsensus<Block>> {
		self.relay_chain_client.clone().execute_with(self)
	}
}

impl<Block, PF, BI, RBackend> polkadot_service::ExecuteWithClient
	for RelayChainConsensusBuilder<Block, PF, BI, RBackend>
where
	Block: BlockT,
	// Rust bug: https://github.com/rust-lang/rust/issues/24159
	sc_client_api::StateBackendFor<RBackend, PBlock>: sc_client_api::StateBackend<HashFor<PBlock>>,
	PF: Environment<Block> + Send + Sync + 'static,
	PF::Proposer: Proposer<
		Block,
		Transaction = BI::Transaction,
		ProofRecording = EnableProofRecording,
		Proof = <EnableProofRecording as ProofRecording>::Proof,
	>,
	BI: BlockImport<Block> + Send + Sync + 'static,
	RBackend: Backend<PBlock> + 'static,
{
	type Output = Box<dyn ParachainConsensus<Block>>;

	fn execute_with_client<PClient, Api, PBackend>(self, client: Arc<PClient>) -> Self::Output
	where
		<Api as sp_api::ApiExt<PBlock>>::StateBackend: sp_api::StateBackend<HashFor<PBlock>>,
		PBackend: Backend<PBlock>,
		PBackend::State: sp_api::StateBackend<sp_runtime::traits::BlakeTwo256>,
		Api: polkadot_service::RuntimeApiCollection<StateBackend = PBackend::State>,
		PClient: polkadot_service::AbstractClient<PBlock, PBackend, Api = Api> + 'static,
	{
		Box::new(RelayChainConsensus::new(
			self.para_id,
			self.proposer_factory,
			self.inherent_data_providers,
			self.block_import,
			client.clone(),
			self.relay_chain_backend,
		))
	}
}
