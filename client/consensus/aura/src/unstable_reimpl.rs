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
use cumulus_client_collator::service::ServiceInterface as CollatorServiceInterface;
use cumulus_primitives_core::{relay_chain::Hash as PHash, CollectCollationInfo, PersistedValidationData};
use cumulus_primitives_parachain_inherent::ParachainInherentData;
use cumulus_relay_chain_interface::RelayChainInterface;

use polkadot_overseer::Handle as OverseerHandle;
use polkadot_primitives::{CollatorPair, Id as ParaId};
use polkadot_node_primitives::{CollationResult, MaybeCompressedPoV};

use futures::prelude::*;
use sc_client_api::{backend::AuxStore, BlockBackend, BlockOf};
use sc_consensus::{BlockImport, BlockImportParams, ForkChoiceStrategy, StateAction};
use sc_consensus::import_queue::{BasicQueue, Verifier as VerifierT};
use sc_consensus_aura::standalone as aura_internal;
use sc_telemetry::TelemetryHandle;
use sp_api::ProvideRuntimeApi;
use sp_application_crypto::AppPublic;
use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_blockchain::HeaderBackend;
use sp_consensus::{error::Error as ConsensusError, BlockOrigin, SyncOracle};
use sp_consensus_aura::{AuraApi, Slot, SlotDuration};
use sp_core::crypto::Pair;
use sp_inherents::{CreateInherentDataProviders, InherentDataProvider};
use sp_keystore::KeystorePtr;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT, Member};
use std::{convert::TryFrom, error::Error, fmt::Debug, hash::Hash, sync::Arc, time::Duration};

/// Parameters of [`run`].
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
	pub telemetry: Option<TelemetryHandle>,
	pub proposer: Proposer,
	pub collator_service: CS,
}

/// Run Aura consensus as a parachain.
pub async fn run<Block, P, BI, CIDP, Client, RClient, SO, Proposer, CS>(
	params: Params<BI, CIDP, Client, RClient, SO, Proposer, CS>,
)
where
	Block: BlockT,
	Client:
		ProvideRuntimeApi<Block> + BlockOf + AuxStore + HeaderBackend<Block> + BlockBackend<Block> + Send + Sync + 'static,
	Client::Api: AuraApi<Block, P::Public> + CollectCollationInfo<Block>,
	RClient: RelayChainInterface,
	CIDP: CreateInherentDataProviders<Block, ()> + 'static,
	BI: BlockImport<Block>
		+ ParachainBlockImportMarker
		+ Send
		+ Sync
		+ 'static,
	SO: SyncOracle + Send + Sync + Clone + 'static,
	Proposer: ProposerInterface<Block, Transaction=BI::Transaction>,
	Proposer::Transaction: Sync,
	CS: CollatorServiceInterface<Block>,
	P: Pair + Send + Sync,
	P::Public: AppPublic + Hash + Member + Encode + Decode,
	P::Signature: TryFrom<Vec<u8>> + Hash + Member + Encode + Decode,
{
	let mut proposer = params.proposer;
	let mut block_import = params.block_import;

	let mut collation_requests = cumulus_client_collator::relay_chain_driven::init(
		params.key,
		params.para_id,
		params.overseer_handle,
	).await;

	while let Some(request) = collation_requests.next().await {
		let res = collate::<P, _, _, _, _>(
			CollationConstants {
				para_id: params.para_id,
				slot_duration: params.slot_duration,
				keystore: &params.keystore,
			},
			CollationParams {
				relay_parent: *request.relay_parent(),
				validation_data: request.persisted_validation_data(),
			},
			CollationEnvironment {
				client: &*params.para_client,
				relay_chain_interface: &params.relay_client,
				proposer: &mut proposer,
				collator_service: &params.collator_service,
				block_import: &mut block_import,
				create_inherent_data_providers: &params.create_inherent_data_providers,
			},
		).await;

		match res {
			Ok(maybe_collation) => request.complete(maybe_collation),
			Err(e) => {
				request.complete(None);
				tracing::error!(
					target: crate::LOG_TARGET,
					err = ?e,
					"Collation failed"
				);
			}
		}
	}
}

fn slot_now(slot_duration: SlotDuration) -> Slot {
	let timestamp = sp_timestamp::InherentDataProvider::from_system_time().timestamp();
	Slot::from_timestamp(timestamp, slot_duration)
}

struct CollationConstants<'a> {
	para_id: ParaId,
	slot_duration: SlotDuration,
	keystore: &'a KeystorePtr,
}

struct CollationParams<'a> {
	relay_parent: PHash,
	validation_data: &'a PersistedValidationData,
}

struct CollationEnvironment<'a, C: 'a, RCI: 'a, P: 'a, CS: 'a, BI, CIDP: 'a> {
	client: &'a C,
	relay_chain_interface: &'a RCI,
	proposer: &'a mut P,
	collator_service: &'a CS,
	block_import: &'a mut BI,
	create_inherent_data_providers: &'a CIDP,
}

// Collate a block using the aura slot worker.
async fn collate<P, Block: BlockT, Client, BI, CIDP>(
	const_params: CollationConstants<'_>,
	params: CollationParams<'_>,
	environment: CollationEnvironment<
		'_,
		Client,
		impl RelayChainInterface,
		impl ProposerInterface<Block, Transaction=BI::Transaction>,
		impl CollatorServiceInterface<Block>,
		BI,
		CIDP,
	>,
) -> Result<Option<CollationResult>, Box<dyn Error>>
where
	P: Pair,
	P::Public: AppPublic + Hash + Member + Encode + Decode,
	P::Signature: Encode + Decode + TryFrom<Vec<u8>>,
	Block: BlockT,
	Client: ProvideRuntimeApi<Block> + Send + Sync + 'static,
	Client::Api: AuraApi<Block, P::Public> + CollectCollationInfo<Block>,
	BI: BlockImport<Block>
		+ ParachainBlockImportMarker
		+ Send
		+ Sync
		+ 'static,
	CIDP: CreateInherentDataProviders<Block, ()>,
{
	let CollationConstants { para_id, slot_duration, keystore } = const_params;
	let CollationParams { relay_parent, validation_data } = params;
	let CollationEnvironment {
		client,
		relay_chain_interface,
		proposer,
		collator_service,
		block_import,
		create_inherent_data_providers,
	} = environment;

	let parent_header = match Block::Header::decode(&mut &validation_data.parent_head.0[..]) {
		Ok(x) => x,
		Err(e) => {
			return Err(format!("Header decoding error during collation ({:?})", e).into());
		}
	};

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
	let slot_now = slot_now(slot_duration);

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
	let proposal = {
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
			.map_err(Box::new)?
	};

	let (pre_header, body) = proposal.block.deconstruct();
	let pre_hash = pre_header.hash();
	let block_number = *pre_header.number();

	// seal the block.
	let block_import_params = {
		let seal_digest = aura_internal::seal::<_, P>(&pre_hash, &author_pub, keystore).map_err(Box::new)?;
		let mut block_import_params = BlockImportParams::new(BlockOrigin::Own, pre_header);
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
		block_number,
		post_hash,
		pre_hash,
	);

	// import the block.
	let block = Block::new(block_import_params.post_header(), body);
	match block_import.import_block(block_import_params).await {
		Ok(_) => {}
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

struct Verifier<P, Client, Block, CIDP> {
	client: Arc<Client>,
	create_inherent_data_providers: CIDP,
	slot_duration: SlotDuration,
	_marker: std::marker::PhantomData<(Block, P)>,
}

#[async_trait::async_trait]
impl<P, Client, Block, CIDP> VerifierT<Block> for Verifier<P, Client, Block, CIDP>
where
	P: Pair,
	P::Signature: Encode + Decode,
	P::Public: Encode + Decode + PartialEq + Clone + Debug,
	Block: BlockT,
	Client: ProvideRuntimeApi<Block> + Send + Sync,
	<Client as ProvideRuntimeApi<Block>>::Api: BlockBuilderApi<Block> + AuraApi<Block, P::Public>,

	CIDP: CreateInherentDataProviders<Block, ()>,
{
	async fn verify(
		&mut self,
		mut block_params: BlockImportParams<Block, ()>,
	) -> Result<BlockImportParams<Block, ()>, String> {
		// Skip checks that include execution, if being told so, or when importing only state.
		//
		// This is done for example when gap syncing and it is expected that the block after the gap
		// was checked/chosen properly, e.g. by warp syncing to this block using a finality proof.
		if block_params.state_action.skip_execution_checks() || block_params.with_state() {
			return Ok(block_params)
		}

		let post_hash = block_params.header.hash();
		let parent_hash = *block_params.header.parent_hash();

		// check seal and update pre-hash/post-hash
		{
			let authorities = aura_internal::fetch_authorities(
				self.client.as_ref(),
				parent_hash,
			).map_err(|e| format!("Could not fetch authorities at {:?}: {}", parent_hash, e))?;

			let slot_now = slot_now(self.slot_duration);
			let res = aura_internal::check_header_slot_and_seal::<Block, P>(
				slot_now,
				block_params.header,
				&authorities,
			);

			match res {
				Ok((pre_header, _slot, seal_digest)) => {
					block_params.header = pre_header;
					block_params.post_digests.push(seal_digest);
					block_params.fork_choice = Some(ForkChoiceStrategy::LongestChain);
					block_params.post_hash = Some(post_hash);

					// TODO [now] telemetry
				}
				Err(aura_internal::SealVerificationError::Deferred(_hdr, slot)) => {
					// TODO [now]: telemetry

					return Err(format!(
						"Rejecting block ({:?}) from future slot {:?}",
						post_hash,
						slot
					));
				}
				Err(e) => {
					return Err(
						format!("Rejecting block ({:?}) with invalid seal ({:?})",
						post_hash,
						e
					));
				}
			}
		}

		// check inherents.
		if let Some(body) = block_params.body.clone() {
			let block = Block::new(block_params.header.clone(), body);
			let create_inherent_data_providers = self
				.create_inherent_data_providers
				.create_inherent_data_providers(parent_hash, ())
				.await
				.map_err(|e| format!("Could not create inherent data {:?}", e))?;

			let inherent_data = create_inherent_data_providers
				.create_inherent_data()
				.await
				.map_err(|e| format!("Could not create inherent data {:?}", e))?;

			let inherent_res = self
				.client
				.runtime_api()
				.check_inherents_with_context(
					parent_hash,
					block_params.origin.into(),
					block,
					inherent_data,
				)
				.map_err(|e| format!("Unable to check block inherents {:?}", e))?;

			if !inherent_res.ok() {
				for (i, e) in inherent_res.into_errors() {
					match create_inherent_data_providers.try_handle_error(&i, &e).await {
						Some(res) => res.map_err(|e| format!("Inherent Error {:?}", e))?,
						None => return Err(format!(
							"Unknown inherent error, source {:?}",
							String::from_utf8_lossy(&i[..])
						)),
					}
				}
			}
		}

		Ok(block_params)
	}
}

/// Start an import queue for a Cumulus node which checks blocks' seals and inherent data.
///
/// Pass in only inherent data providers which don't include aura or parachain consensus inherents,
/// e.g. things like timestamp and custom inherents for the runtime.
///
/// The others are generated explicitly internally.
///
/// This should only be used for runtimes where the runtime does not check all inherents and
/// seals in `execute_block` (see https://github.com/paritytech/cumulus/issues/2436)
pub fn fully_verifying_import_queue<P, Client, Block: BlockT, I, CIDP>(
	client: Arc<Client>,
	block_import: I,
	create_inherent_data_providers: CIDP,
	slot_duration: SlotDuration,
	spawner: &impl sp_core::traits::SpawnEssentialNamed,
	registry: Option<&substrate_prometheus_endpoint::Registry>,
) -> BasicQueue<Block, I::Transaction>
where
	P: Pair,
	P::Signature: Encode + Decode,
	P::Public: Encode + Decode + PartialEq + Clone + Debug,
	I: BlockImport<Block, Error = ConsensusError>
		+ ParachainBlockImportMarker
		+ Send
		+ Sync
		+ 'static,
	I::Transaction: Send,
	Client: ProvideRuntimeApi<Block> + Send + Sync + 'static,
	<Client as ProvideRuntimeApi<Block>>::Api: BlockBuilderApi<Block> + AuraApi<Block, P::Public>,
	CIDP: CreateInherentDataProviders<Block, ()> + 'static,
{
	let verifier = Verifier::<P, _, _, _> {
		client,
		create_inherent_data_providers,
		slot_duration,
		_marker: std::marker::PhantomData,
	};

	BasicQueue::new(verifier, Box::new(block_import), None, spawner, registry)
}
