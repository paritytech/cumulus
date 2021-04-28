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

//! The nimbus consensus client-side worker
//!
//! This engine is driven by relay-chain based slots. It queries the in-runtime filter to determine
//! whether any keys stored in its keystore are eligible to author at this slot. If it has an eligible
//! ley it authors.
//!
//! Like the relay chain consensus is manually inserts the parachain inherent.
//! In addition it manually inserts the author inherent. So be sure not to use this
//! worker and the simple author inherent data provider together.
//!
//! Idea: I wonder if we could simplify this by having a separate worker that just had a stream called
//! like `slot-beacon` or something. We could use the relay chain or a simple timer.

use author_filter_api::AuthorFilterAPI;
use codec::Codec;
use cumulus_client_consensus_common::{ParachainCandidate, ParachainConsensus};
use cumulus_primitives_core::{
	relay_chain::v1::{Block as PBlock, Hash as PHash, ParachainHost},
	ParaId, PersistedValidationData,
};
use cumulus_primitives_parachain_inherent::ParachainInherentData;
pub use import_queue::import_queue;
use log::{info, warn};
use parking_lot::Mutex;
use polkadot_service::ClientHandle;
use sc_client_api::Backend;
use sp_api::{ProvideRuntimeApi, BlockId};
use sp_consensus::{
	BlockImport, BlockImportParams, BlockOrigin, EnableProofRecording, Environment,
	ForkChoiceStrategy, ProofRecording, Proposal, Proposer,
};
use sp_inherents::{InherentData, InherentDataProviders};
use sp_runtime::traits::{Block as BlockT, HashFor, Header as HeaderT};
use sp_runtime::KeyTypeId;
use std::{marker::PhantomData, sync::Arc, time::Duration};
use tracing::error;
use sp_keystore::{SyncCryptoStorePtr, SyncCryptoStore};
// woof the naming
use author_filter_api::{NIMBUS_KEY_ID, NimbusId};
use sp_application_crypto::{AppKey, AppPublic, Pair, TryFrom, Public};
use sp_runtime::traits::{Member, Hash};
use codec::{Encode, Decode};
use core::fmt::Debug;

mod import_queue;

const LOG_TARGET: &str = "filtering-consensus";

/// The implementation of the relay-chain provided consensus for parachains.
pub struct FilteringConsensus<B, PF, BI, RClient, RBackend, ParaClient, AuthorId> {
	para_id: ParaId,
	_phantom: PhantomData<B>,
	proposer_factory: Arc<Mutex<PF>>,
	inherent_data_providers: InherentDataProviders,
	block_import: Arc<futures::lock::Mutex<BI>>,
	relay_chain_client: Arc<RClient>,
	relay_chain_backend: Arc<RBackend>,
	parachain_client: Arc<ParaClient>,
	author: AuthorId,
	keystore: SyncCryptoStorePtr,
}

impl<B, PF, BI, RClient, RBackend, ParaClient, AuthorId: Clone> Clone for FilteringConsensus<B, PF, BI, RClient, RBackend, ParaClient, AuthorId> {
	fn clone(&self) -> Self {
		Self {
			para_id: self.para_id,
			_phantom: PhantomData,
			proposer_factory: self.proposer_factory.clone(),
			inherent_data_providers: self.inherent_data_providers.clone(),
			block_import: self.block_import.clone(),
			relay_chain_backend: self.relay_chain_backend.clone(),
			relay_chain_client: self.relay_chain_client.clone(),
			parachain_client: self.parachain_client.clone(),
			author: self.author.clone(),
			keystore: self.keystore.clone(),
		}
	}
}

impl<B, PF, BI, RClient, RBackend, ParaClient, AuthorId> FilteringConsensus<B, PF, BI, RClient, RBackend, ParaClient, AuthorId>
where
	B: BlockT,
	RClient: ProvideRuntimeApi<PBlock>,
	RClient::Api: ParachainHost<PBlock>,
	RBackend: Backend<PBlock>,
	ParaClient: ProvideRuntimeApi<B>,
{
	/// Create a new instance of relay-chain provided consensus.
	pub fn new(
		para_id: ParaId,
		proposer_factory: PF,
		inherent_data_providers: InherentDataProviders,
		block_import: BI,
		polkadot_client: Arc<RClient>,
		polkadot_backend: Arc<RBackend>,
		parachain_client: Arc<ParaClient>,
		author: AuthorId,
		keystore: SyncCryptoStorePtr,
	) -> Self {
		Self {
			para_id,
			proposer_factory: Arc::new(Mutex::new(proposer_factory)),
			inherent_data_providers,
			block_import: Arc::new(futures::lock::Mutex::new(block_import)),
			relay_chain_backend: polkadot_backend,
			relay_chain_client: polkadot_client,
			parachain_client,
			author,
			keystore,
			_phantom: PhantomData,
		}
	}

	/// Get the inherent data with validation function parameters injected
	fn inherent_data(
		&self,
		validation_data: &PersistedValidationData,
		relay_parent: PHash,
		author_id: &NimbusId,
	) -> Option<InherentData> {
		// Build the inherents that use normal inherent data providers.
		let mut inherent_data = self
			.inherent_data_providers
			.create_inherent_data()
			.map_err(|e| {
				error!(
					target: LOG_TARGET,
					error = ?e,
					"Failed to create inherent data.",
				)
			})
			.ok()?;

		// Now manually build and attach the parachain one.
		// This is the same as in RelayChainConsensus.
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
				error!(
					target: LOG_TARGET,
					error = ?e,
					"Failed to put the system inherent into inherent data.",
				)
			})
			.ok()?;

		// Now manually attach the author one.
		inherent_data
			//TODO import the inherent id from somewhere. Currently it is defined in the pallet.
			.put_data(*b"author__", author_id)
			.map_err(|e| {
				error!(
					target: LOG_TARGET,
					error = ?e,
					"Failed to put the author inherent into inherent data.",
				)
			})
			.ok()?;

		println!("On client side. Inherent data is");
		// Grrr debug isn't implemented
		// println!("{:?}", inherent_data);

		Some(inherent_data)
	}
}

#[async_trait::async_trait]
impl<B, PF, BI, RClient, RBackend, ParaClient, AuthorId> ParachainConsensus<B>
	for FilteringConsensus<B, PF, BI, RClient, RBackend, ParaClient, AuthorId>
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
	ParaClient: ProvideRuntimeApi<B> + Send + Sync,
	ParaClient::Api: AuthorFilterAPI<B>,
	AuthorId: Send + Sync + Clone + Codec,
{
	async fn produce_candidate(
		&mut self,
		parent: &B::Header,
		relay_parent: PHash,
		validation_data: &PersistedValidationData,
	) -> Option<ParachainCandidate<B>> {
		// Design decision: We will check the keystore for any available keys. Then we will iterate
		// those keys until we find one that is eligible. If none are eligible, we skip this slot.
		// If multiple are eligible, we only author with the first one.
		// The accountId that comes in from the service will no longer be used, and will ultimately
		// be stripped out.
		// We will insert the inherent manually here as Basti does for the parachain inherent. That
		// may improve when/if his bkchr-inherent-something-future branch is merged, but it
		// honestly doesn't feel that bad to me the way it is.
		// Whew, I think that's al lthe stuff I was stuck on yesterday.

		// We're using the Trait names when calling keystore methods because of this.
		// Hopefully switching to the async keystore will make the code sexier:
		// error[E0034]: multiple applicable items in scope
		//    --> client/consensus/filtering/src/lib.rs:213:38
		//     |
		// 213 |         let have_key: bool = self.keystore.has_keys(
		//     |                                            ^^^^^^^^ multiple `has_keys` found
		//     |
		// note: candidate #1 is defined in the trait `SyncCryptoStore`
		//    --> /home/joshy/.cargo/git/checkouts/substrate-7e08433d4c370a21/a8c2bc6/primitives/keystore/src/lib.rs:278:2
		//     |
		// 278 |     fn has_keys(&self, public_keys: &[(Vec<u8>, KeyTypeId)]) -> bool;
		//     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
		// note: candidate #2 is defined in the trait `CryptoStore`
		//    --> /home/joshy/.cargo/git/checkouts/substrate-7e08433d4c370a21/a8c2bc6/primitives/keystore/src/lib.rs:118:2
		//     |
		// 118 |     async fn has_keys(&self, public_keys: &[(Vec<u8>, KeyTypeId)]) -> bool;
		//     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
		// help: disambiguate the associated function for candidate #1
		//     |
		// 213 |         let have_key: bool = SyncCryptoStore::has_keys(self.keystore, &vec![(self.author.encode(), KeyTypeId(*b"nmbs"))]);
		//     |                              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
		// help: disambiguate the associated function for candidate #2
		//     |
		// 213 |         let have_key: bool = CryptoStore::has_keys(self.keystore, &vec![(self.author.encode(), KeyTypeId(*b"nmbs"))]);
		//     |                              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^


		// Get allthe available keys
		// I don't know why I had to call the crypto-specific method as opposed to just "keys",
		// but this is the one that returns the `Public` type I (think I ) need.
		// Ughhh, it still doesn't return NimbusId. I hope this will work somehow.
		// note: expected struct `Vec<author_filter_api::app::Public>`
    	//          found struct `Vec<sp_core::sr25519::Public>`
		let available_keys/*: Vec<NimbusId>*/ =
			SyncCryptoStore::sr25519_public_keys(&*self.keystore, NIMBUS_KEY_ID);

		// Print a more helpful message than "not eligible" when there are no keys at all.
		if available_keys.is_empty() {
			warn!(target: LOG_TARGET, "No Nimbus keys available. We will not be able to author.");
		}

		// Iterate keys until we find an eligible one, or run out of candidates.
		let maybe_key = available_keys.into_iter().find(|k| {
			self.parachain_client.runtime_api()
				.can_author(
					&BlockId::Hash(parent.hash()),
					//Holy shit I think that worked!
					From::from(*k),
					validation_data.relay_parent_number
				)
				.expect("Author API should not return error")
		});

		// If there are no eligible keys, print the log, and exit early.
		let author_id: NimbusId = match maybe_key {
			Some(key) => From::from(key),
			None => {
				info!(
					target: LOG_TARGET,
					"🔮 Skipping candidate production because we are not eligible"
				);
				return None;
			}
		};

		let proposer_future = self.proposer_factory.lock().init(&parent);

		let proposer = proposer_future
			.await
			.map_err(
				|e| error!(target: LOG_TARGET, error = ?e, "Could not create proposer."),
			)
			.ok()?;

		let inherent_data = self.inherent_data(&validation_data, relay_parent, &author_id)?;

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
				// Set the block limit to 50% of the maximum PoV size.
				//
				// TODO: If we got benchmarking that includes that encapsulates the proof size,
				// we should be able to use the maximum pov size.
				Some((validation_data.max_pov_size / 2) as usize),
			)
			.await
			.map_err(|e| error!(target: LOG_TARGET, error = ?e, "Proposing failed."))
			.ok()?;

		let (header, extrinsics) = block.clone().deconstruct();

		let pre_hash = header.hash();

		// To sign I need the "CryptoTypePublicPair" I can't make it because I don't know wft the crypto type is
		// I can't just grab it to begin with, because I can't convert the unsized [u8] it goves me to a NimbusId
		// I wish the jeystore interface amde a little more sense. Maybe I should be passing opaque data
		// into the runtime api afterall?
		// Observation: Aura has a type paramaeter P that is set (explicitly) to `AuraPair` when called
		// From the service. Maybe I need to do that here as well. P has several trait bounds that may help.
		//
		// let type_public_pair =
		// 	SyncCryptoStore::keys(&*self.keystore, NIMBUS_KEY_ID)
		// 	.expect("fetching the keys succeeded")
		// 	.iter()
		// 	.find(|p| {
		// 		&p.1 == author_id.0
		// 	})
		// 	.expect("The key should be there, because we already found it earlier.");
		//
		// let sig = SyncCryptoStore::sign_with(
		// 	&*self.keystore,
		// 	NIMBUS_KEY_ID,
		// 	type_public_pair,
		// 	pre_hash,
		// ).ok_or_else(|| {
		// 	println!("Signing failed!!");
		// 	return None;
		// });
		//
		// println!("The signature is \n{:?}", sig);

		// Add a silly test digest, just to get familiar with how it works
		// TODO better identifier and stuff.
		let test_digest = sp_runtime::generic::DigestItem::Seal(*b"test", Vec::new());

		let mut block_import_params = BlockImportParams::new(BlockOrigin::Own, header.clone());
		// Add the test digest to the block import params
		block_import_params.post_digests.push(test_digest.clone());
		block_import_params.body = Some(extrinsics.clone());
		// Best block is determined by the relay chain.
		block_import_params.fork_choice = Some(ForkChoiceStrategy::Custom(false));
		block_import_params.storage_changes = Some(storage_changes);

		// Print the same log line as slots (aura and babe) to see if this is working the same way
		// It seems to be working the same way now.
		info!(
			"🔖 Sealed block for proposal at {}. Hash now {:?}, previously {:?}.",
			*header.number(),
			block_import_params.post_hash(),
			pre_hash,
		);

		if let Err(err) = self
			.block_import
			.lock()
			.await
			.import_block(block_import_params, Default::default())
			.await
		{
			error!(
				target: LOG_TARGET,
				at = ?parent.hash(),
				error = ?err,
				"Error importing built block.",
			);

			return None;
		}

		// Compute info about the block after the digest is added
		let mut post_header = header.clone();
		post_header.digest_mut().logs.push(test_digest.clone());
		let post_block = B::new(post_header, extrinsics);

		// Returning the block WITH the seal for distribution around the network.
		Some(ParachainCandidate { block: post_block, proof })
	}
}

/// Paramaters of [`build_relay_chain_consensus`].
/// TODO can this be moved into common and shared with relay chain conensus builder?
/// I bet my head would explode from thinking about generic types.
///
/// I'm going to start trying to add the keystore here. I briefly tried the async approach, but
/// decided t ogo sync so I can copy code from Aura. Maybe after it is working, Jeremy can help me
/// go async.
pub struct BuildFilteringConsensusParams<PF, BI, RBackend, ParaClient, AuthorId> {
	pub para_id: ParaId,
	pub proposer_factory: PF,
	pub inherent_data_providers: InherentDataProviders,
	pub block_import: BI,
	pub relay_chain_client: polkadot_service::Client,
	pub relay_chain_backend: Arc<RBackend>,
	pub parachain_client: Arc<ParaClient>,
	pub author: AuthorId,
	pub keystore: SyncCryptoStorePtr,

}

/// Build the [`FilteringConsensus`].
///
/// Returns a boxed [`ParachainConsensus`].
pub fn build_filtering_consensus<Block, PF, BI, RBackend, ParaClient, AuthorId>(
	BuildFilteringConsensusParams {
		para_id,
		proposer_factory,
		inherent_data_providers,
		block_import,
		relay_chain_client,
		relay_chain_backend,
		parachain_client,
		author,
		keystore,
	}: BuildFilteringConsensusParams<PF, BI, RBackend, ParaClient, AuthorId>,
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
	ParaClient: ProvideRuntimeApi<Block> + Send + Sync + 'static,
	ParaClient::Api: AuthorFilterAPI<Block>,
	AuthorId: Send + Sync + Clone + 'static + Codec,
{
	FilteringConsensusBuilder::new(
		para_id,
		proposer_factory,
		block_import,
		inherent_data_providers,
		relay_chain_client,
		relay_chain_backend,
		parachain_client,
		author,
		keystore,
	)
	.build()
}

/// Relay chain consensus builder.
///
/// Builds a [`FilteringConsensus`] for a parachain. As this requires
/// a concrete relay chain client instance, the builder takes a [`polkadot_service::Client`]
/// that wraps this concrete instanace. By using [`polkadot_service::ExecuteWithClient`]
/// the builder gets access to this concrete instance.
struct FilteringConsensusBuilder<Block, PF, BI, RBackend, ParaClient, AuthorId> {
	para_id: ParaId,
	_phantom: PhantomData<Block>,
	proposer_factory: PF,
	inherent_data_providers: InherentDataProviders,
	block_import: BI,
	relay_chain_backend: Arc<RBackend>,
	relay_chain_client: polkadot_service::Client,
	parachain_client: Arc<ParaClient>,
	author: AuthorId,
	keystore: SyncCryptoStorePtr,
}

impl<Block, PF, BI, RBackend, ParaClient, AuthorId> FilteringConsensusBuilder<Block, PF, BI, RBackend, ParaClient, AuthorId>
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
	ParaClient: ProvideRuntimeApi<Block> + Send + Sync + 'static,
	AuthorId: Send + Sync + Clone + Codec + 'static,
{
	/// Create a new instance of the builder.
	fn new(
		para_id: ParaId,
		proposer_factory: PF,
		block_import: BI,
		inherent_data_providers: InherentDataProviders,
		relay_chain_client: polkadot_service::Client,
		relay_chain_backend: Arc<RBackend>,
		parachain_client: Arc<ParaClient>,
		author: AuthorId,
		keystore: SyncCryptoStorePtr,
	) -> Self {
		Self {
			para_id,
			_phantom: PhantomData,
			proposer_factory,
			block_import,
			inherent_data_providers,
			relay_chain_backend,
			relay_chain_client,
			parachain_client,
			author,
			keystore,
		}
	}

	/// Build the relay chain consensus.
	fn build(self) -> Box<dyn ParachainConsensus<Block>>
	where
		ParaClient::Api: AuthorFilterAPI<Block>,
	{
		self.relay_chain_client.clone().execute_with(self)
	}
}

impl<Block, PF, BI, RBackend, ParaClient, AuthorId> polkadot_service::ExecuteWithClient
	for FilteringConsensusBuilder<Block, PF, BI, RBackend, ParaClient, AuthorId>
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
	ParaClient: ProvideRuntimeApi<Block> + Send + Sync + 'static,
	ParaClient::Api: AuthorFilterAPI<Block>,
	AuthorId: Send + Sync + Clone + Codec + 'static,
{
	type Output = Box<dyn ParachainConsensus<Block>>;

	fn execute_with_client<PClient, Api, PBackend>(self, client: Arc<PClient>) -> Self::Output
	where
		<Api as sp_api::ApiExt<PBlock>>::StateBackend: sp_api::StateBackend<HashFor<PBlock>>,
		PBackend: Backend<PBlock>,
		PBackend::State: sp_api::StateBackend<sp_runtime::traits::BlakeTwo256>,
		Api: polkadot_service::RuntimeApiCollection<StateBackend = PBackend::State>,
		PClient: polkadot_service::AbstractClient<PBlock, PBackend, Api = Api> + 'static,
		ParaClient::Api: AuthorFilterAPI<Block>,
	{
		Box::new(FilteringConsensus::new(
			self.para_id,
			self.proposer_factory,
			self.inherent_data_providers,
			self.block_import,
			client.clone(),
			self.relay_chain_backend,
			self.parachain_client,
			self.author,
			self.keystore,
		))
	}
}
