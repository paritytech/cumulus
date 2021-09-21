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

//! Parachain specific wrapper for the AuRa import queue.

use codec::Codec;
use sc_client_api::{backend::AuxStore, BlockOf, UsageProvider};
use sc_consensus::{import_queue::DefaultImportQueue, BlockImport};
use sc_consensus_aura::AuraVerifier;
use sc_consensus_slots::InherentDataProviderExt;
use sc_telemetry::TelemetryHandle;
use sp_api::{ApiExt, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_blockchain::{HeaderBackend, ProvideCache};
use sp_consensus::{CanAuthorWith, Error as ConsensusError};
use sp_consensus_aura::{digests::CompatibleDigestItem, AuraApi};
use sp_core::crypto::Pair;
use sp_inherents::CreateInherentDataProviders;
use sp_runtime::traits::{Block as BlockT, DigestItemFor};
use std::{fmt::Debug, hash::Hash, sync::Arc};
use substrate_prometheus_endpoint::Registry;

/// Parameters of [`import_queue`].
pub struct ImportQueueParams<'a, I, C, CIDP, S, CAW> {
	/// The block import to use.
	pub block_import: I,
	/// The client to interact with the chain.
	pub client: Arc<C>,
	/// The inherent data providers, to create the inherent data.
	pub create_inherent_data_providers: CIDP,
	/// The spawner to spawn background tasks.
	pub spawner: &'a S,
	/// The prometheus registry.
	pub registry: Option<&'a Registry>,
	/// Can we author with the current node?
	pub can_author_with: CAW,
	/// The telemetry handle.
	pub telemetry: Option<TelemetryHandle>,
}

/// Start an import queue for the Aura consensus algorithm.
pub fn import_queue<'a, P, Block, I, C, S, CAW, CIDP>(
	ImportQueueParams {
		block_import,
		client,
		create_inherent_data_providers,
		spawner,
		registry,
		can_author_with,
		telemetry,
	}: ImportQueueParams<'a, I, C, CIDP, S, CAW>,
) -> Result<DefaultImportQueue<Block, C>, sp_consensus::Error>
where
	Block: BlockT,
	C::Api: BlockBuilderApi<Block> + AuraApi<Block, P::Public> + ApiExt<Block>,
	C: 'static
		+ ProvideRuntimeApi<Block>
		+ BlockOf
		+ ProvideCache<Block>
		+ Send
		+ Sync
		+ AuxStore
		+ UsageProvider<Block>
		+ HeaderBackend<Block>,
	I: BlockImport<Block, Error = ConsensusError, Transaction = sp_api::TransactionFor<C, Block>>
		+ Send
		+ Sync
		+ 'static,
	DigestItemFor<Block>: CompatibleDigestItem<P::Signature>,
	P: Pair + Send + Sync + 'static,
	P::Public: Clone + Eq + Send + Sync + Hash + Debug + Codec,
	P::Signature: Codec,
	S: sp_core::traits::SpawnEssentialNamed,
	CAW: CanAuthorWith<Block> + Send + Sync + 'static,
	CIDP: CreateInherentDataProviders<Block, ()> + Sync + Send + 'static,
	CIDP::InherentDataProviders: InherentDataProviderExt + Send + Sync,
{
	sc_consensus_aura::import_queue::<P, _, _, _, _, _, _>(sc_consensus_aura::ImportQueueParams {
		block_import: cumulus_client_consensus_common::ParachainBlockImport::new(block_import),
		justification_import: None,
		client,
		create_inherent_data_providers,
		spawner,
		registry,
		can_author_with,
		check_for_equivocation: sc_consensus_aura::CheckForEquivocation::No,
		telemetry,
	})
}

/// Parameters of [`build_verifier`].
pub struct BuildVerifierParams<C, CIDP, CAW> {
	/// The client to interact with the chain.
	pub client: Arc<C>,
	/// The inherent data providers, to create the inherent data.
	pub create_inherent_data_providers: CIDP,
	/// Can we author with the current node?
	pub can_author_with: CAW,
	/// The telemetry handle.
	pub telemetry: Option<TelemetryHandle>,
}

/// Build the [`AuraVerifier`].
pub fn build_verifier<P, C, CIDP, CAW>(
	BuildVerifierParams {
		client,
		create_inherent_data_providers,
		can_author_with,
		telemetry,
	}: BuildVerifierParams<C, CIDP, CAW>,
) -> AuraVerifier<C, P, CAW, CIDP> {
	sc_consensus_aura::build_verifier(sc_consensus_aura::BuildVerifierParams {
		client,
		create_inherent_data_providers,
		can_author_with,
		telemetry,
		check_for_equivocation: sc_consensus_aura::CheckForEquivocation::No,
	})
}
