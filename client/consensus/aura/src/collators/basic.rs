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

//! This provides the option to run a basic relay-chain driven Aura implementation
//!
//! For more information about AuRa, the Substrate crate should be checked.

use codec::{Decode, Encode};
use cumulus_client_collator::service::ServiceInterface as CollatorServiceInterface;
use cumulus_client_consensus_common::{
	ParachainBlockImportMarker, ParachainCandidate, ParentSearchParams,
};
use cumulus_client_consensus_proposer::ProposerInterface;
use cumulus_primitives_core::{
	relay_chain::Hash as PHash, CollectCollationInfo, PersistedValidationData,
};
use cumulus_primitives_parachain_inherent::ParachainInherentData;
use cumulus_relay_chain_interface::RelayChainInterface;

use polkadot_node_primitives::{CollationResult, MaybeCompressedPoV};
use polkadot_overseer::Handle as OverseerHandle;
use polkadot_primitives::{Block as PBlock, CollatorPair, Header as PHeader, Id as ParaId};

use futures::prelude::*;
use sc_client_api::{backend::AuxStore, BlockBackend, BlockOf};
use sc_consensus::{
	import_queue::{BasicQueue, Verifier as VerifierT},
	BlockImport, BlockImportParams, ForkChoiceStrategy, StateAction,
};
use sc_consensus_aura::standalone as aura_internal;
use sc_telemetry::{telemetry, TelemetryHandle, CONSENSUS_DEBUG, CONSENSUS_TRACE};
use sp_api::ProvideRuntimeApi;
use sp_application_crypto::AppPublic;
use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_blockchain::HeaderBackend;
use sp_consensus::{error::Error as ConsensusError, BlockOrigin, SyncOracle};
use sp_consensus_aura::{AuraApi, Slot, SlotDuration};
use sp_core::crypto::Pair;
use sp_inherents::{CreateInherentDataProviders, InherentData, InherentDataProvider};
use sp_keystore::KeystorePtr;
use sp_runtime::{
	generic::Digest,
	traits::{Block as BlockT, HashFor, Header as HeaderT, Member},
};
use sp_state_machine::StorageChanges;
use sp_timestamp::Timestamp;
use std::{convert::TryFrom, error::Error, fmt::Debug, hash::Hash, sync::Arc, time::Duration};

/// Parameters for [`run`].
pub struct Params<BI, CIDP, Client, RClient, SO, Proposer, CS> {
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
	pub relay_chain_slot_duration: SlotDuration,
	pub proposer: Proposer,
	pub collator_service: CS,
}

/// Run bare Aura consensus as a relay-chain-driven collator.
pub async fn run<Block, P, BI, CIDP, Client, RClient, SO, Proposer, CS>(
	params: Params<BI, CIDP, Client, RClient, SO, Proposer, CS>,
) where
	Block: BlockT,
	Client: ProvideRuntimeApi<Block>
		+ BlockOf
		+ AuxStore
		+ HeaderBackend<Block>
		+ BlockBackend<Block>
		+ Send
		+ Sync
		+ 'static,
	Client::Api: AuraApi<Block, P::Public> + CollectCollationInfo<Block>,
	RClient: RelayChainInterface,
	CIDP: CreateInherentDataProviders<Block, ()> + 'static,
	BI: BlockImport<Block> + ParachainBlockImportMarker + Send + Sync + 'static,
	SO: SyncOracle + Send + Sync + Clone + 'static,
	Proposer: ProposerInterface<Block, Transaction = BI::Transaction>,
	Proposer::Transaction: Sync,
	CS: CollatorServiceInterface<Block>,
	P: Pair + Send + Sync,
	P::Public: AppPublic + Hash + Member + Encode + Decode,
	P::Signature: TryFrom<Vec<u8>> + Hash + Member + Encode + Decode,
{
	let mut params = params;

	let mut collation_requests = cumulus_client_collator::relay_chain_driven::init(
		params.key,
		params.para_id,
		params.overseer_handle,
	)
	.await;

	while let Some(request) = collation_requests.next().await {
		macro_rules! reject_with_error {
			($err:expr) => {{
				request.complete(None);
				tracing::error!(target: crate::LOG_TARGET, err = ?{ $err });
				continue;
			}};
		}

		macro_rules! try_request {
			($x:expr) => {{
				match $x {
					Ok(x) => x,
					Err(e) => reject_with_error!(e),
				}
			}};
		}

		let validation_data = request.persisted_validation_data();

		let parent_header =
			try_request!(Block::Header::decode(&mut &validation_data.parent_head.0[..]));

		let parent_hash = parent_header.hash();

		if !params.collator_service.check_block_status(parent_hash, &parent_header) {
			continue
		}

		let relay_parent_header = match params.relay_client.header(*request.relay_parent()).await {
			Err(e) => reject_with_error!(e),
			Ok(None) => continue, // sanity: would be inconsistent to get `None` here
			Ok(Some(h)) => h,
		};

		let claim = match claim_slot::<_, _, P>(
			&*params.para_client,
			parent_hash,
			&relay_parent_header,
			params.slot_duration,
			params.relay_chain_slot_duration,
			&params.keystore,
		)
		.await
		{
			Ok(None) => continue,
			Ok(Some(c)) => c,
			Err(e) => reject_with_error!(e),
		};

		let (parachain_inherent_data, other_inherent_data) = try_request!(
			create_inherent_data(
				*request.relay_parent(),
				&validation_data,
				parent_hash,
				params.para_id,
				claim.timestamp,
				&params.relay_client,
				&params.create_inherent_data_providers,
			)
			.await
		);

		let proposal = try_request!(
			params
				.proposer
				.propose(
					&parent_header,
					&parachain_inherent_data,
					other_inherent_data,
					Digest { logs: vec![claim.pre_digest] },
					// TODO [https://github.com/paritytech/cumulus/issues/2439]
					// We should call out to a pluggable interface that provides
					// the proposal duration.
					Duration::from_millis(500),
					// Set the block limit to 50% of the maximum PoV size.
					//
					// TODO: If we got benchmarking that includes the proof size,
					// we should be able to use the maximum pov size.
					Some((validation_data.max_pov_size / 2) as usize),
				)
				.await
		);

		let sealed_importable = try_request!(seal::<_, _, P>(
			proposal.block,
			proposal.storage_changes,
			&claim.author_pub,
			&params.keystore,
		));

		let post_hash = sealed_importable.post_hash();
		let block = Block::new(
			sealed_importable.post_header(),
			sealed_importable
				.body
				.as_ref()
				.expect("body always created with this `propose` fn; qed")
				.clone(),
		);

		try_request!(params.block_import.import_block(sealed_importable).await);

		let response = if let Some((collation, b)) = params.collator_service.build_collation(
			&parent_header,
			post_hash,
			ParachainCandidate { block, proof: proposal.proof },
		) {
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

			let result_sender = params.collator_service.announce_with_barrier(post_hash);
			Some(CollationResult { collation, result_sender: Some(result_sender) })
		} else {
			None
		};

		request.complete(response);
	}
}

/// A claim on an Aura slot.
struct SlotClaim<Pub> {
	author_pub: Pub,
	pre_digest: sp_runtime::DigestItem,
	timestamp: Timestamp,
}

async fn claim_slot<B, C, P>(
	client: &C,
	parent_hash: B::Hash,
	relay_parent_header: &PHeader,
	slot_duration: SlotDuration,
	relay_chain_slot_duration: SlotDuration,
	keystore: &KeystorePtr,
) -> Result<Option<SlotClaim<P::Public>>, Box<dyn Error>>
where
	B: BlockT,
	C: ProvideRuntimeApi<B> + Send + Sync + 'static,
	C::Api: AuraApi<B, P::Public>,
	P: Pair,
	P::Public: Encode + Decode,
	P::Signature: Encode + Decode,
{
	// load authorities
	let authorities = client.runtime_api().authorities(parent_hash).map_err(Box::new)?;

	// Determine the current slot and timestamp based on the relay-parent's.
	let (slot_now, timestamp) =
		match sc_consensus_babe::find_pre_digest::<PBlock>(relay_parent_header) {
			Ok(babe_pre_digest) => {
				let t =
					Timestamp::new(relay_chain_slot_duration.as_millis() * *babe_pre_digest.slot());
				let slot = Slot::from_timestamp(t, slot_duration);

				(slot, t)
			},
			Err(_) => return Ok(None),
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

	Ok(Some(SlotClaim { author_pub, pre_digest, timestamp }))
}

// This explicitly creates the inherent data for parachains, as well as overriding the
// timestamp based on the slot number.
async fn create_inherent_data<B: BlockT>(
	relay_parent: PHash,
	validation_data: &PersistedValidationData,
	parent_hash: B::Hash,
	para_id: ParaId,
	timestamp: Timestamp,
	relay_chain_interface: &impl RelayChainInterface,
	create_inherent_data_providers: &impl CreateInherentDataProviders<B, ()>,
) -> Result<(ParachainInherentData, InherentData), Box<dyn Error>> {
	let paras_inherent_data = ParachainInherentData::create_at(
		relay_parent,
		relay_chain_interface,
		validation_data,
		para_id,
	)
	.await;

	let paras_inherent_data = match paras_inherent_data {
		Some(p) => p,
		None =>
			return Err(format!("Could not create paras inherent data at {:?}", relay_parent).into()),
	};

	let mut other_inherent_data = create_inherent_data_providers
		.create_inherent_data_providers(parent_hash, ())
		.map_err(|e| e as Box<dyn Error>)
		.await?
		.create_inherent_data()
		.await
		.map_err(Box::new)?;

	other_inherent_data.replace_data(sp_timestamp::INHERENT_IDENTIFIER, &timestamp);

	Ok((paras_inherent_data, other_inherent_data))
}

fn seal<B: BlockT, T, P>(
	pre_sealed: B,
	storage_changes: StorageChanges<T, HashFor<B>>,
	author_pub: &P::Public,
	keystore: &KeystorePtr,
) -> Result<BlockImportParams<B, T>, Box<dyn Error>>
where
	P: Pair,
	P::Signature: Encode + Decode + TryFrom<Vec<u8>>,
	P::Public: AppPublic,
{
	let (pre_header, body) = pre_sealed.deconstruct();
	let pre_hash = pre_header.hash();
	let block_number = *pre_header.number();

	// seal the block.
	let block_import_params = {
		let seal_digest =
			aura_internal::seal::<_, P>(&pre_hash, &author_pub, keystore).map_err(Box::new)?;
		let mut block_import_params = BlockImportParams::new(BlockOrigin::Own, pre_header);
		block_import_params.post_digests.push(seal_digest);
		block_import_params.body = Some(body.clone());
		block_import_params.state_action =
			StateAction::ApplyChanges(sc_consensus::StorageChanges::Changes(storage_changes));
		block_import_params.fork_choice = Some(ForkChoiceStrategy::LongestChain);
		block_import_params
	};
	let post_hash = block_import_params.post_hash();

	tracing::info!(
		target: crate::LOG_TARGET,
		"🔖 Pre-sealed block for proposal at {}. Hash now {:?}, previously {:?}.",
		block_number,
		post_hash,
		pre_hash,
	);

	Ok(block_import_params)
}
