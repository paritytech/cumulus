// Copyright 2023 Parity Technologies (UK) Ltd.
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

//! The AuRa consensus algorithm for parachains.
//!
//! This extends the Substrate provided AuRa consensus implementation to make it compatible for
//! parachains. The main entry points for of this consensus algorithm are [`fn@run`]
//! and [`fn@import_queue`].
//!
//! For more information about AuRa, the Substrate crate should be checked.

use codec::{Decode, Encode};
use cumulus_client_consensus_common::{
	ParachainBlockImportMarker, ParachainCandidate, ParachainConsensus,
};
use cumulus_client_consensus_proposer::ProposerInterface;
use cumulus_client_collator::service::CollatorService;
use cumulus_primitives_core::{relay_chain::Hash as PHash, PersistedValidationData};

use polkadot_overseer::Handle as OverseerHandle;
use polkadot_primitives::{CollatorPair, Id as ParaId};

use futures::{prelude::*, lock::Mutex};
use sc_client_api::{backend::AuxStore, BlockBackend, BlockOf};
use sc_consensus::BlockImport;
use sc_consensus_aura::SlotProportion;
use sc_consensus_slots::{BackoffAuthoringBlocksStrategy, SimpleSlotWorker, SlotInfo};
use sc_telemetry::TelemetryHandle;
use sp_api::ProvideRuntimeApi;
use sp_application_crypto::AppPublic;
use sp_blockchain::HeaderBackend;
use sp_consensus::{EnableProofRecording, Environment, ProofRecording, Proposal, Proposer, SyncOracle};
use sp_consensus_aura::{AuraApi, SlotDuration};
use sp_core::crypto::Pair;
use sp_inherents::{CreateInherentDataProviders, InherentData};
use sp_keystore::KeystorePtr;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT, Member, NumberFor};
use sp_runtime::Digest;
use std::{convert::TryFrom, hash::Hash, marker::PhantomData, sync::Arc, time::Duration};

/// Parameters of [`run`].
pub struct Params<Block: BlockT, BI, CIDP, Client, SO, Proposer> {
	pub create_inherent_data_providers: CIDP,
	pub block_import: BI,
	pub para_client: Arc<Client>,
	pub sync_oracle: SO,
	pub keystore: KeystorePtr,
	pub key: CollatorPair,
	pub para_id: ParaId,
	pub overseer_handle: OverseerHandle,
	pub slot_duration: SlotDuration,
	pub telemetry: Option<TelemetryHandle>,
	pub block_proposal_slot_portion: SlotProportion,
	pub max_block_proposal_slot_portion: Option<SlotProportion>,
	pub service: CollatorService<Block, Client, Client>,
	pub proposer: Proposer,
}

/// Run Aura consensus as a parachain.
pub async fn run<Block, P, BI, CIDP, Client, SO, Proposer>(
	params: Params<Block, BI, CIDP, Client, SO, Proposer>,
)
where
	Block: BlockT,
	Client:
		ProvideRuntimeApi<Block> + BlockOf + AuxStore + HeaderBackend<Block> + BlockBackend<Block> + Send + Sync + 'static,
	Client::Api: AuraApi<Block, P::Public>,
	CIDP: CreateInherentDataProviders<B, (PHash, PersistedValidationData)> + 'static,
	CIDP::InherentDataProviders: InherentDataProviderExt,
	BI: BlockImport<Block, Transaction = sp_api::TransactionFor<Client, Block>>
		+ ParachainBlockImportMarker
		+ Send
		+ Sync
		+ 'static,
	SO: SyncOracle + Send + Sync + Clone + 'static,
	Proposer: ProposerInterface<Block, Transaction = sp_api::TransactionFor<Client, Block>>,
	Proposer::Transaction: Sync,
	P: Pair + Send + Sync,
	P::Public: AppPublic + Hash + Member + Encode + Decode,
	P::Signature: TryFrom<Vec<u8>> + Hash + Member + Encode + Decode,
{
	let mut worker = sc_consensus_aura::build_aura_worker::<P, _, _, _, _, _, _, _, _>(
		sc_consensus_aura::BuildAuraWorkerParams {
			client: params.para_client,
			block_import: params.block_import,
			justification_sync_link: (),
			proposer_factory: FakeSubstrateEnvironment(std::marker::PhantomData::<Proposer::Transaction>),
			sync_oracle: params.sync_oracle,
			force_authoring: false, // TODO [now]: how useful is this for parachains?
			backoff_authoring_blocks: Option::<()>::None, // TODO [now]: how useful is this for parachains?
			keystore: params.keystore,
			telemetry: params.telemetry,
			block_proposal_slot_portion: params.block_proposal_slot_portion,
			max_block_proposal_slot_portion: params.max_block_proposal_slot_portion,
			compatibility_mode: sc_consensus_aura::CompatibilityMode::None,
		},
	);

	let mut collator_service = params.service;
	let mut proposer = params.proposer;

	let mut collation_requests = cumulus_client_collator::relay_chain_driven::init(
		params.key,
		params.para_id,
		params.overseer_handle,
	).await;

	while let Some(request) = collation_requests.next().await {
		collate(&mut worker, &mut proposer).await;
	}
}

// Collate a block using the aura slot worker.
async fn collate<Block: BlockT, S, CIDP>(
	slot_worker: &mut S,
	proposer: &mut impl ProposerInterface<Block>,
	create_inherent_data_providers: &CIDP,
)
where
	Block: BlockT,
	S: SimpleSlotWorker<Block>,
	CIDP: CreateInherentDataProviders<Block, ()>,
{

}

// A fake environment which will never be invoked, as we don't use this part of the
// Aura slot worker.
struct FakeSubstrateEnvironment<T: Send + 'static>(std::marker::PhantomData<T>);
// A fake proposer which will never be invoked, as we don't use this part of the
// Aura slot worker.
struct FakeProposer<T: Send + 'static>(std::marker::PhantomData<T>);

impl<Block: BlockT, T: Default + Send + 'static> Environment<Block> for FakeSubstrateEnvironment<T> {
	type Proposer = FakeProposer<T>;
	type CreateProposer = futures::future::Pending<Result<Self::Proposer, Self::Error>>;
	type Error = sp_consensus::Error;

	fn init(&mut self, parent_header: &Block::Header) -> Self::CreateProposer {
		futures::future::pending()
	}
}

impl<Block: BlockT, T: Default + Send + 'static> Proposer<Block> for FakeProposer<T> {
		type Error = sp_consensus::Error;
		type Transaction = T;
		type Proposal = futures::future::Pending<Result<Proposal<Block, Self::Transaction, Self::Proof>, Self::Error>>;
		type ProofRecording = EnableProofRecording;
		type Proof = sp_state_machine::StorageProof;

		fn propose(
			self,
			_inherent_data: InherentData,
			_inherent_digests: Digest,
			_max_duration: Duration,
			_block_size_limit: Option<usize>,
		) -> Self::Proposal {
			futures::future::pending()
		}
}
