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
	ParachainBlockImportMarker, ParachainCandidate,
};
use cumulus_client_consensus_proposer::ProposerInterface;
use cumulus_client_collator::service::CollatorService;
use cumulus_primitives_core::{relay_chain::Hash as PHash, CollectCollationInfo, PersistedValidationData};
use cumulus_primitives_parachain_inherent::ParachainInherentData;
use cumulus_relay_chain_interface::RelayChainInterface;

use polkadot_overseer::Handle as OverseerHandle;
use polkadot_primitives::{CollatorPair, Id as ParaId};
use polkadot_node_primitives::{CollationResult, MaybeCompressedPoV};

use futures::prelude::*;
use sc_client_api::{backend::AuxStore, BlockBackend, BlockOf};
use sc_consensus::{BlockImport, BlockImportParams, ForkChoiceStrategy, StateAction};
use sc_consensus_aura::standalone as aura_internal;
use sc_telemetry::TelemetryHandle;
use sp_api::ProvideRuntimeApi;
use sp_application_crypto::AppPublic;
use sp_blockchain::HeaderBackend;
use sp_consensus::{BlockOrigin, SyncOracle};
use sp_consensus_aura::{AuraApi, Slot, SlotDuration};
use sp_core::crypto::Pair;
use sp_inherents::{CreateInherentDataProviders, InherentDataProvider};
use sp_keystore::KeystorePtr;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT, Member};
use std::{convert::TryFrom, error::Error, hash::Hash, sync::Arc};

/// Parameters of [`run`].
pub struct Params<Block: BlockT, BI, CIDP, Client, RClient, SO, Proposer> {
	pub create_inherent_data_providers: CIDP,
	pub block_import: BI,
	pub para_client: Arc<Client>,
	pub relay_client: Arc<RClient>,
	pub sync_oracle: SO,
	pub keystore: KeystorePtr,
	pub key: CollatorPair,
	pub para_id: ParaId,
	pub overseer_handle: OverseerHandle,
	pub slot_duration: SlotDuration,
	pub telemetry: Option<TelemetryHandle>,
	pub service: CollatorService<Block, Client, Client>,
	pub proposer: Proposer,
}

/// Run Aura consensus as a parachain.
pub async fn run<Block, P, BI, CIDP, Client, RClient, SO, Proposer>(
	params: Params<Block, BI, CIDP, Client, RClient, SO, Proposer>,
)
where
	Block: BlockT,
	Client:
		ProvideRuntimeApi<Block> + BlockOf + AuxStore + HeaderBackend<Block> + BlockBackend<Block> + Send + Sync + 'static,
	Client::Api: AuraApi<Block, P::Public> + CollectCollationInfo<Block>,
	RClient: RelayChainInterface,
	CIDP: CreateInherentDataProviders<Block, (PHash, PersistedValidationData)> + 'static,
	BI: BlockImport<Block>
		+ ParachainBlockImportMarker
		+ Send
		+ Sync
		+ 'static,
	SO: SyncOracle + Send + Sync + Clone + 'static,
	Proposer: ProposerInterface<Block, Transaction=BI::Transaction>,
	Proposer::Transaction: Sync,
	P: Pair + Send + Sync,
	P::Public: AppPublic + Hash + Member + Encode + Decode,
	P::Signature: TryFrom<Vec<u8>> + Hash + Member + Encode + Decode,
{
	let mut collator_service = params.service;
	let mut proposer = params.proposer;

	let mut collation_requests = cumulus_client_collator::relay_chain_driven::init(
		params.key,
		params.para_id,
		params.overseer_handle,
	).await;

	while let Some(request) = collation_requests.next().await {
		// TODO [now]: invoke `collate`.
	}
}

// Collate a block using the aura slot worker.
async fn collate<Block: BlockT, P, Client, BI, CIDP>(
	relay_parent: PHash,
	validation_data: &PersistedValidationData,
	para_id: ParaId,
	client: &Client,
	relay_chain_interface: &impl RelayChainInterface,
	parent_header: Block::Header,
	proposer: &mut impl ProposerInterface<Block, Transaction=BI::Transaction>,
	collator_service: CollatorService<Block, Client, Client>,
	slot_duration: SlotDuration,
	keystore: &KeystorePtr,
	block_import: BI,
	create_inherent_data_providers: &CIDP,
) -> Result<Option<CollationResult>, Box<dyn Error>>
where
	Block: BlockT,
	P: Pair,
	P::Public: AppPublic + Hash + Member + Encode + Decode,
	P::Signature: Encode + Decode + TryFrom<Vec<u8>>,
	Client: ProvideRuntimeApi<Block> + BlockOf + BlockBackend<Block> + Send + Sync + 'static,
	Client::Api: AuraApi<Block, P::Public> + CollectCollationInfo<Block>,
	BI: BlockImport<Block>
		+ ParachainBlockImportMarker
		+ Send
		+ Sync
		+ 'static,
	CIDP: CreateInherentDataProviders<Block, ()>,
{
	let parent_hash = parent_header.hash();
	if !collator_service.check_block_status(parent_hash, &parent_header) {
		return Ok(None)
	}

	// load authorities
	let authorities = client
		.runtime_api()
		.authorities(parent_hash)
		.map_err(Box::new)?;

	// Determine the current slot.
	let slot_now = {
		let timestamp = sp_timestamp::InherentDataProvider::from_system_time().timestamp();
		Slot::from_timestamp(timestamp, slot_duration)
	};

	// Try to claim the slot locally.
	let author_pub = {
		let res = aura_internal::claim_slot::<P>(slot_now, &authorities, keystore).await;
		match res {
			Some(p) => p,
			None => return Ok(None),
		}
	};

	// Produce the pre-digest.
	let pre_digest = aura_internal::pre_digest::<P>(slot_now);

	tracing::info!(
		target: crate::LOG_TARGET,
		relay_parent = ?relay_parent,
		at = ?parent_hash,
		"Starting collation.",
	);

	// propose the block.
	let mut proposal = {
		let paras_inherent_data = ParachainInherentData::create_at(
			relay_parent,
			relay_chain_interface,
			validation_data,
			para_id,
		).await;

		let paras_inherent_data = match paras_inherent_data {
			Some(p) => p,
			None => return Ok(None),
		};

		let other_inherent_data = create_inherent_data_providers
			.create_inherent_data_providers(parent_hash, ())
			.map_err(|e| e as Box<dyn Error>).await?
			.create_inherent_data().await.map_err(Box::new)?;

		proposer.propose(
			&parent_header,
			&paras_inherent_data,
			other_inherent_data,
			sp_runtime::generic::Digest { logs: vec![pre_digest] },
			unimplemented!(), // max duration
			// Set the block limit to 50% of the maximum PoV size.
			//
			// TODO: If we got benchmarking that includes the proof size,
			// we should be able to use the maximum pov size.
			Some((validation_data.max_pov_size / 2) as usize),
		)
			.await
			.map_err(Box::new)?
	};

	let (pre_header, body) = proposal.block.deconstruct();
	let pre_hash = pre_header.hash();

	// seal the block.
	let block_import_params = {
		let seal_digest = aura_internal::seal::<_, P>(&pre_hash, &author_pub, keystore).map_err(Box::new)?;
		let block_import_params = BlockImportParams::new(BlockOrigin::Own, pre_header);
		block_import_params.post_digests.push(seal_digest);
		block_import_params.body = Some(body.clone());
		block_import_params.state_action =
			StateAction::ApplyChanges(sc_consensus::StorageChanges::Changes(proposal.storage_changes));
		block_import_params.fork_choice = Some(ForkChoiceStrategy::LongestChain);
		block_import_params
	};
	let post_hash = block_import_params.post_hash();

	tracing::info!(
		target: crate::LOG_TARGET,
		"🔖 Pre-sealed block for proposal at {}. Hash now {:?}, previously {:?}.",
		pre_header.number(),
		post_hash,
		pre_hash,
	);

	// import the block.
	let block = Block::new(block_import_params.post_header(), body);
	match block_import.import_block(block_import_params).await {
		Ok(res) => {
			res.handle_justification::<Block>(
				&post_hash,
				*pre_header.number(),
				unimplemented!(), // TODO [now]: justification sync link.
			)
		}
		Err(err) => {
			tracing::warn!(
				target: crate::LOG_TARGET,
				"Error with block built on {:?}: {}", parent_hash, err,
			);

			return Ok(None);
		}
	}

	// build a collation and announce it with a barrier.
	let collation = collator_service.build_collation(
		&parent_header,
		post_hash,
		ParachainCandidate {
			block,
			proof: proposal.proof,
		}
	);

	Ok(match collation {
		Some((collation, b)) => {
			tracing::info!(
				target: crate::LOG_TARGET,
				"PoV size {{ header: {}kb, extrinsics: {}kb, storage_proof: {}kb }}",
				b.header().encode().len() as f64 / 1024f64,
				b.extrinsics().encode().len() as f64 / 1024f64,
				b.storage_proof().encode().len() as f64 / 1024f64,
			);

			if let MaybeCompressedPoV::Compressed(ref pov) = collation.proof_of_validity {
				tracing::info!(
					target: crate::LOG_TARGET,
					"Compressed PoV size: {}kb",
					pov.block_data.0.len() as f64 / 1024f64,
				);
			}

			let result_sender = collator_service.announce_with_barrier(post_hash);
			Some(CollationResult { collation, result_sender: Some(result_sender) })
		}
		None => None,
	})
}
