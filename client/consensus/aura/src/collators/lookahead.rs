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

//! A collator for Aura that looks ahead of the most recently included parachain block
//! when determining what to build upon.
//!
//! This collator also builds additional blocks when the maximum backlog is not saturated.
//! The size of the backlog is determined by invoking a runtime API. If that runtime API
//! is not supported, this assumes a maximum backlog size of 1.
//!
//! This takes more advantage of asynchronous backing, though not complete advantage.
//! When the backlog is not saturated, this approach lets the backlog temporarily 'catch up'
//! with periods of higher throughput. When the backlog is saturated, we typically
//! fall back to the limited cadence of a single parachain block per relay-chain block.
//!
//! Despite this, the fact that there is a backlog at all allows us to spend more time
//! building the block, as there is some buffer before it can get posted to the relay-chain.
//! The main limitation is block propagation time - i.e. the new blocks created by an author
//! must be propagated to the next author before their turn.

use codec::{Decode, Encode};
use cumulus_client_collator::service::ServiceInterface as CollatorServiceInterface;
use cumulus_client_consensus_common::{
	self as consensus_common,
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

use crate::collator as collator_util;

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
	pub authoring_duration: Duration,
}

/// Run async-backing-friendly Aura.
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

	let mut import_notifications = match params.relay_client.import_notification_stream().await {
		Ok(s) => s,
		Err(err) => {
			tracing::error!(
				target: crate::LOG_TARGET,
				?err,
				"Failed to initialize consensus: no relay chain import notification stream"
			);

			return
		},
	};

	let mut collator = {
		let params = collator_util::Params {
			create_inherent_data_providers: params.create_inherent_data_providers,
			block_import: params.block_import,
			relay_client: params.relay_client.clone(),
			keystore: params.keystore.clone(),
			para_id: params.para_id,
			proposer: params.proposer,
			collator_service: params.collator_service,
		};

		collator_util::Collator::<Block, P, _, _, _, _, _>::new(params)
	};

	while let Some(relay_parent_header) = import_notifications.next().await {
		let relay_parent = relay_parent_header.hash();

		// TODO [now]: get asynchronous backing parameters from the relay-chain
		// runtime. why? for the parent search parameters.

		let max_pov_size  = match params.relay_client.persisted_validation_data(
			relay_parent,
			params.para_id,
			OccupiedCoreAssumption::Included,
		).await {
			Ok(None) => continue,
			Ok(Some(pvd)) => pvd.max_pov_size,
			Err(err) => {
				tracing::error!(target: crate::LOG_TARGET, ?err, "Failed to gather information from relay-client");
				continue;
			}
		};

		let (slot_now, timestamp) = match consensus_common::relay_slot_and_timestamp(
			&relay_parent_header,
			params.relay_chain_slot_duration,
		) {
			None => continue,
			Some((s, t)) => (Slot::from_timestamp(t, params.slot_duration), t),
		};

		let parent_search_params = ParentSearchParams {
			relay_parent,
			para_id: params.para_id,
			ancestry_lookback: unimplemented!(),
			max_depth: unimplemented!(), // max unincluded segment len
			ignore_alternative_branches: true,
		};

		// TODO [now]: remove this in favor of one passed in as a parameter.
		let fake_hack: sc_client_api::in_mem::Blockchain<Block> = unimplemented!();

		let potential_parents = cumulus_client_consensus_common::find_potential_parents::<Block>(
			parent_search_params,
			&fake_hack, // sp_blockchain::Backend
			&params.relay_client,
		)
		.await;

		let mut potential_parents = match potential_parents {
			Err(e) => {
				tracing::error!(
					target: crate::LOG_TARGET,
					?relay_parent,
					err = ?e,
					"Could not fetch potential parents to build upon"
				);

				continue
			},
			Ok(x) => x,
		};

		let included_block = match potential_parents.iter().find(|x| x.depth == 0) {
			None => continue, // also serves as an `is_empty` check.
			Some(b) => b.hash,
		};

		let para_client = &*params.para_client;
		let keystore = &params.keystore;
		let can_build_upon = |block_hash| can_build_upon(
			slot_now,
			timestamp,
			block_hash,
			included_block,
			&para_client,
			&keystore,
		);

		// Sort by depth, ascending, to choose the longest chain.
		//
		// If the longest chain has space, build upon that. Otherwise, don't
		// build at all.
		potential_parents.sort_by_key(|a| &a.depth);
		let initial_parent = match potential_parents.pop() {
			None => continue,
			Some(p) => p,
		};

		// Build in a loop until not allowed. Note that the authorities can change
		// at any block, so we need to re-claim our slot every time.
		let mut parent_hash = initial_parent.hash;
		let mut parent_header = initial_parent.header;
		loop {
			let slot_claim = match can_build_upon(parent_hash).await {
				None => break,
				Some(c) => c,
			};

			let persisted_validation_data = PersistedValidationData {
				parent_head: parent_header.encode(),
				relay_parent_number: *relay_parent_header.number(),
				relay_parent_storage_root: *relay_parent_header.state_root(),
				max_pov_size,
			};

			// Build and announce collations recursively until
			// `can_build_upon` fails or building a collation fails.
			let (parachain_inherent_data, other_inherent_data) = match collator.create_inherent_data(
				relay_parent,
				&persisted_validation_data,
				parent_hash,
				slot_claim.timestamp(),
			).await {
				Err(err) => {
					tracing::error!(target: crate::LOG_TARGET, ?err);
					break;
				},
				Ok(x) => x,
			};

			let (new_block_hash, new_block_header) = match collator.collate(
				&parent_header,
				&slot_claim,
				None,
				(parachain_inherent_data, other_inherent_data),
				params.authoring_duration,
				// Set the block limit to 50% of the maximum PoV size.
				//
				// TODO: If we got benchmarking that includes the proof size,
				// we should be able to use the maximum pov size.
				(validation_data.max_pov_size / 2) as usize,
			).await {
				Ok((collation, block_data, new_block_hash)) => {
					parent_hash = new_block_hash;
					parent_header = block_data.header;

					// TODO [now]: announce to parachain sub-network

					// TODO [link to github issue when i have internet]:
					// announce collation to relay-chain validators.
				}
				Err(err) => {
					tracing::error!(target: crate::LOG_TARGET, ?err);
					break;
				}
			};
		}
	}
}

// Checks if we own the slot at the given block and whether there
// is space in the unincluded segment.
async fn can_build_upon<Block: BlockT, Client, P>(
	slot: Slot,
	timestamp: Timestamp,
	block_hash: Block::Hash,
	included_block: Block::Hash,
	client: &Client,
	keystore: &KeystorePtr,
) -> Option<SlotClaim<P::Public>>
where
	Client: ProvideRuntimeApi<Block>,
	Client::Api: AuraApi<Block, P::Public>,
	P: Pair,
	P::Public: Encode + Decode,
	P::Signature: Encode + Decode,
{
	let authorities = client.runtime_api().authorities(block_hash).ok()?;
	let author_pub = aura_internal::claim_slot::<P>(slot, &authorities, keystore).await?;

	// TODO [now]: new runtime API,
	// AuraUnincludedSegmentApi::has_space(included_block, slot) or something like it.
	unimplemented!()

	Some(SlotClaim::unchecked(author_pub, slot, timestamp))
}
