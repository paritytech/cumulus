// Copyright 2019 Parity Technologies (UK) Ltd.
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

use std::{marker::PhantomData, sync::Arc};

use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_blockchain::Result as ClientResult;
use sp_consensus::{
	error::Error as ConsensusError,
	import_queue::{BasicQueue, CacheKeyId, Verifier as VerifierT},
	BlockImport, BlockImportParams, BlockOrigin,
};
use sp_inherents::{CreateInherentDataProviders, InherentDataProvider};
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, Header as HeaderT},
	Justifications,
	DigestItem,
};
use nimbus_primitives::{NimbusId, NimbusSignature, NimbusPair};
use sp_application_crypto::{TryFrom, Pair as _, Public as _};
use log::debug;

/// The Nimbus verifier strips the seal digest, and checks that it is a valid signature by
/// the same key that was injected into the runtime and noted in the Seal digest.
/// From Nimbu's perspective any block that faithfully reports its authorship to the runtime
/// is valid. The intention is that the runtime itself may then put further restrictions on
/// the identity of the author.
struct Verifier<Client, Block, CIDP> {
	client: Arc<Client>,
	create_inherent_data_providers: CIDP,
	_marker: PhantomData<Block>,
}

#[async_trait::async_trait]
impl<Client, Block, CIDP> VerifierT<Block> for Verifier<Client, Block, CIDP>
where
	Block: BlockT,
	Client: ProvideRuntimeApi<Block> + Send + Sync,
	<Client as ProvideRuntimeApi<Block>>::Api: BlockBuilderApi<Block>,
	CIDP: CreateInherentDataProviders<Block, ()> ,
{
	async fn verify(
		&mut self,
		origin: BlockOrigin,
		mut header: Block::Header,
		justifications: Option<Justifications>,
		mut body: Option<Vec<Block::Extrinsic>>,
	) -> Result<
		(
			BlockImportParams<Block, ()>,
			Option<Vec<(CacheKeyId, Vec<u8>)>>,
		),
		String,
	> {

		debug!(target: crate::LOG_TARGET, "🪲 Header hash before popping digest {:?}", header.hash());
		// Grab the digest from the seal
		//TODO use CompatibleDigest trait here once I write it. For now assume the seal is last.
		let seal = header.digest_mut().pop().expect("Block should have at least one digest on it");

		let sig = match seal {
			DigestItem::Seal(id, ref sig) if id == *b"nmbs" => sig.clone(),
			_ => return Err("HeaderUnsealed".into()),
		};

		debug!(target: crate::LOG_TARGET, "🪲 Header hash after popping digest {:?}", header.hash());

		debug!(target: crate::LOG_TARGET, "🪲 Signature according to verifier is {:?}", sig);

		// Grab the digest from the runtime
		//TODO use the trait. Maybe this code should move to the trait.
		let consensus_digest = header
			.digest()
			.logs
			.iter()
			.find(|digest| {
				match *digest {
					DigestItem::Consensus(id, _) if id == b"nmbs" => true,
					_ => false,
				}
			})
			.expect("A single consensus digest should be added by the runtime when executing the author inherent.");
		
		let claimed_author = match *consensus_digest {
			DigestItem::Consensus(id, ref author_id) if id == *b"nmbs" => author_id.clone(),
			_ => panic!("Expected consensus digest to contains author id bytes"),
		};

		debug!(target: crate::LOG_TARGET, "🪲 Claimed Author according to verifier is {:?}", claimed_author);

		// Verify the signature
		let valid_signature = NimbusPair::verify(
			&NimbusSignature::try_from(sig).expect("Bytes should convert to signature correctly"),
			header.hash(),
			&NimbusId::from_slice(&claimed_author),
		);

		debug!(target: crate::LOG_TARGET, "🪲 Valid signature? {:?}", valid_signature);

		if !valid_signature{
			return Err("Block signature invalid".into());
		}

		// This part copied from RelayChainConsensus. I guess this is the inherent checking.
		if let Some(inner_body) = body.take() {
			let inherent_data_providers = self
				.create_inherent_data_providers
				.create_inherent_data_providers(*header.parent_hash(), ())
				.await
				.map_err(|e| e.to_string())?;

			let inherent_data = inherent_data_providers
				.create_inherent_data()
				.map_err(|e| format!("{:?}", e))?;

			let block = Block::new(header.clone(), inner_body);

			let inherent_res = self
				.client
				.runtime_api()
				.check_inherents(
					&BlockId::Hash(*header.parent_hash()),
					block.clone(),
					inherent_data,
				)
				.map_err(|e| format!("{:?}", e))?;

			if !inherent_res.ok() {
				for (i, e) in inherent_res.into_errors() {
					match inherent_data_providers.try_handle_error(&i, &e).await {
						Some(r) => r.map_err(|e| format!("{:?}", e))?,
						None => Err(format!(
							"Unhandled inherent error from `{}`.",
							String::from_utf8_lossy(&i)
						))?,
					}
				}
			}

			let (_, inner_body) = block.deconstruct();
			body = Some(inner_body);
		}

		let mut block_import_params = BlockImportParams::new(origin, header);
		block_import_params.post_digests.push(seal);
		block_import_params.body = body;
		block_import_params.justifications = justifications;

		debug!(target: crate::LOG_TARGET, "🪲 Just finished verifier. posthash from params is {:?}", &block_import_params.post_hash());

		Ok((block_import_params, None))
	}
}

/// Start an import queue for a Cumulus collator that does not uses any special authoring logic.
pub fn import_queue<Client, Block: BlockT, I, CIDP>(
	client: Arc<Client>,
	block_import: I,
	create_inherent_data_providers: CIDP,
	spawner: &impl sp_core::traits::SpawnEssentialNamed,
	registry: Option<&substrate_prometheus_endpoint::Registry>,
) -> ClientResult<BasicQueue<Block, I::Transaction>>
where
	I: BlockImport<Block, Error = ConsensusError> + Send + Sync + 'static,
	I::Transaction: Send,
	Client: ProvideRuntimeApi<Block> + Send + Sync + 'static,
	<Client as ProvideRuntimeApi<Block>>::Api: BlockBuilderApi<Block>,
	CIDP: CreateInherentDataProviders<Block, ()> + 'static,
{
	let verifier = Verifier {
		client,
		create_inherent_data_providers,
		_marker: PhantomData,
	};

	Ok(BasicQueue::new(
		verifier,
		Box::new(cumulus_client_consensus_common::ParachainBlockImport::new(
			block_import,
		)),
		None,
		spawner,
		registry,
	))
}
