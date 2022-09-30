// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

//! Cumulus Collator implementation for Substrate.

use cumulus_client_network::WaitToAnnounce;
use cumulus_primitives_core::{
	relay_chain::Hash as PHash, CollationInfo, CollectCollationInfo, ParachainBlockData,
	PersistedValidationData, ValidationParams, ValidationResult,
};

use sc_client_api::BlockBackend;
use sc_executor_common::{
	runtime_blob::RuntimeBlob,
	wasm_runtime::{InvokeMethod, WasmModule},
};
use sc_executor_wasmtime::{Config, DeterministicStackLimit, Semantics};
use sp_api::{ApiExt, ProvideRuntimeApi};
use sp_consensus::BlockStatus;
use sp_core::traits::SpawnNamed;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, HashFor, Header as HeaderT, Zero},
};

use cumulus_client_consensus_common::ParachainConsensus;
use polkadot_node_primitives::{
	BlockData, Collation, CollationGenerationConfig, CollationResult, MaybeCompressedPoV, PoV,
};
use polkadot_node_subsystem::messages::{
	CollationGenerationMessage, CollatorProtocolMessage, RuntimeApiMessage, RuntimeApiRequest,
};
use polkadot_overseer::Handle as OverseerHandle;
use polkadot_primitives::v2::{CollatorPair, Id as ParaId, OccupiedCoreAssumption};

use codec::{Decode, Encode};
use futures::{channel::oneshot, FutureExt};
use parking_lot::Mutex;
use std::sync::Arc;
use tracing::Instrument;

/// The logging target.
const LOG_TARGET: &str = "cumulus-collator";

/// The implementation of the Cumulus `Collator`.
pub struct Collator<Block: BlockT, BS, RA> {
	block_status: Arc<BS>,
	parachain_consensus: Box<dyn ParachainConsensus<Block>>,
	wait_to_announce: Arc<Mutex<WaitToAnnounce<Block>>>,
	runtime_api: Arc<RA>,
	para_id: ParaId,
	spawner: Arc<dyn SpawnNamed + Send + Sync>,
	overseer_handle: OverseerHandle,
}

impl<Block: BlockT, BS, RA> Clone for Collator<Block, BS, RA> {
	fn clone(&self) -> Self {
		Self {
			block_status: self.block_status.clone(),
			wait_to_announce: self.wait_to_announce.clone(),
			parachain_consensus: self.parachain_consensus.clone(),
			runtime_api: self.runtime_api.clone(),
			para_id: self.para_id,
			spawner: self.spawner.clone(),
			overseer_handle: self.overseer_handle.clone(),
		}
	}
}

const DEFAULT_HEAP_PAGES_ESTIMATE: u64 = 32;
const EXTRA_HEAP_PAGES: u64 = 2048;

/// The number of bytes devoted for the stack during wasm execution of a PVF.
const NATIVE_STACK_MAX: u32 = 256 * 1024 * 1024;

const CONFIG: Config = Config {
	allow_missing_func_imports: true,
	cache_path: None,
	semantics: Semantics {
		extra_heap_pages: EXTRA_HEAP_PAGES,

		// NOTE: This is specified in bytes, so we multiply by WASM page size.
		max_memory_size: Some(((DEFAULT_HEAP_PAGES_ESTIMATE + EXTRA_HEAP_PAGES) * 65536) as usize),

		instantiation_strategy:
			sc_executor_wasmtime::InstantiationStrategy::RecreateInstanceCopyOnWrite,

		// Enable deterministic stack limit to pin down the exact number of items the wasmtime stack
		// can contain before it traps with stack overflow.
		//
		// Here is how the values below were chosen.
		//
		// At the moment of writing, the default native stack size limit is 1 MiB. Assuming a logical item
		// (see the docs about the field and the instrumentation algorithm) is 8 bytes, 1 MiB can
		// fit 2x 65536 logical items.
		//
		// Since reaching the native stack limit is undesirable, we halve the logical item limit and
		// also increase the native 256x. This hopefully should preclude wasm code from reaching
		// the stack limit set by the wasmtime.
		deterministic_stack_limit: Some(DeterministicStackLimit {
			logical_max: 65536,
			native_stack_max: NATIVE_STACK_MAX,
		}),
		canonicalize_nans: true,
		// Rationale for turning the multi-threaded compilation off is to make the preparation time
		// easily reproducible and as deterministic as possible.
		//
		// Currently the prepare queue doesn't distinguish between precheck and prepare requests.
		// On the one hand, it simplifies the code, on the other, however, slows down compile times
		// for execute requests. This behavior may change in future.
		parallel_compilation: false,
	},
};

impl<Block, BS, RA> Collator<Block, BS, RA>
where
	Block: BlockT,
	BS: BlockBackend<Block>,
	RA: ProvideRuntimeApi<Block>,
	RA::Api: CollectCollationInfo<Block>,
{
	/// Create a new instance.
	fn new(
		block_status: Arc<BS>,
		spawner: Arc<dyn SpawnNamed + Send + Sync>,
		announce_block: Arc<dyn Fn(Block::Hash, Option<Vec<u8>>) + Send + Sync>,
		runtime_api: Arc<RA>,
		parachain_consensus: Box<dyn ParachainConsensus<Block>>,
		para_id: ParaId,
		overseer_handle: OverseerHandle,
	) -> Self {
		let wait_to_announce =
			Arc::new(Mutex::new(WaitToAnnounce::new(spawner.clone(), announce_block)));

		Self {
			block_status,
			wait_to_announce,
			runtime_api,
			parachain_consensus,
			para_id,
			spawner,
			overseer_handle,
		}
	}

	/// Checks the status of the given block hash in the Parachain.
	///
	/// Returns `true` if the block could be found and is good to be build on.
	fn check_block_status(&self, hash: Block::Hash, header: &Block::Header) -> bool {
		match self.block_status.block_status(&BlockId::Hash(hash)) {
			Ok(BlockStatus::Queued) => {
				tracing::debug!(
					target: LOG_TARGET,
					block_hash = ?hash,
					"Skipping candidate production, because block is still queued for import.",
				);
				false
			},
			Ok(BlockStatus::InChainWithState) => true,
			Ok(BlockStatus::InChainPruned) => {
				tracing::error!(
					target: LOG_TARGET,
					"Skipping candidate production, because block `{:?}` is already pruned!",
					hash,
				);
				false
			},
			Ok(BlockStatus::KnownBad) => {
				tracing::error!(
					target: LOG_TARGET,
					block_hash = ?hash,
					"Block is tagged as known bad and is included in the relay chain! Skipping candidate production!",
				);
				false
			},
			Ok(BlockStatus::Unknown) => {
				if header.number().is_zero() {
					tracing::error!(
						target: LOG_TARGET,
						block_hash = ?hash,
						"Could not find the header of the genesis block in the database!",
					);
				} else {
					tracing::debug!(
						target: LOG_TARGET,
						block_hash = ?hash,
						"Skipping candidate production, because block is unknown.",
					);
				}
				false
			},
			Err(e) => {
				tracing::error!(
					target: LOG_TARGET,
					block_hash = ?hash,
					error = ?e,
					"Failed to get block status.",
				);
				false
			},
		}
	}

	/// Fetch the collation info from the runtime.
	///
	/// Returns `Ok(Some(_))` on success, `Err(_)` on error or `Ok(None)` if the runtime api isn't implemented by the runtime.
	fn fetch_collation_info(
		&self,
		block_hash: Block::Hash,
		header: &Block::Header,
	) -> Result<Option<CollationInfo>, sp_api::ApiError> {
		let runtime_api = self.runtime_api.runtime_api();
		let block_id = BlockId::Hash(block_hash);

		let api_version =
			match runtime_api.api_version::<dyn CollectCollationInfo<Block>>(&block_id)? {
				Some(version) => version,
				None => {
					tracing::error!(
						target: LOG_TARGET,
						"Could not fetch `CollectCollationInfo` runtime api version."
					);
					return Ok(None)
				},
			};

		tracing::error!(
			"RUNTIME API VERSION: {} === {:?}",
			api_version,
			sp_core::hexdisplay::HexDisplay::from(
				&<dyn CollectCollationInfo<Block> as sp_api::RuntimeApiInfo>::ID
			),
		);

		let collation_info = if api_version < 2 {
			#[allow(deprecated)]
			runtime_api
				.collect_collation_info_before_version_2(&block_id)?
				.into_latest(hex_literal::hex!("b30e836c46f09c3327b10a47aa90736e5bf664c17d25d3a05cbd0f07eee6d2a872d38800bbf39840dafaa6a29d365093d8b75cce2f5e927b2210eaa317125bc0aea5189a4e50f15acf13b101972b89777931780ba30a86e3227cc43d44c29bb83c55155b080661757261206e7544080000000005617572610101f4173486975580af86c9bf8ece3309a29808c6eb1f238c5f36fd9bb5c6c6811e34fa51c8e59fb35754ed92af1beda9555839de36cda49a85ac823d6ce94f778f").to_vec().into())
		} else {
			runtime_api.collect_collation_info(&block_id, header)?
		};

		Ok(Some(collation_info))
	}

	fn build_collation(
		&self,
		block: ParachainBlockData<Block>,
		block_hash: Block::Hash,
		pov: PoV,
	) -> Option<Collation> {
		let collation_info = self
			.fetch_collation_info(block_hash, block.header())
			.map_err(|e| {
				tracing::error!(
					target: LOG_TARGET,
					error = ?e,
					"Failed to collect collation info.",
				)
			})
			.ok()
			.flatten()?;

		Some(Collation {
			upward_messages: collation_info.upward_messages,
			new_validation_code: collation_info.new_validation_code,
			processed_downward_messages: collation_info.processed_downward_messages,
			horizontal_messages: collation_info.horizontal_messages,
			hrmp_watermark: collation_info.hrmp_watermark,
			head_data: collation_info.head_data,
			proof_of_validity: MaybeCompressedPoV::Compressed(pov),
		})
	}

	async fn produce_candidate(
		mut self,
		relay_parent: PHash,
		validation_data: PersistedValidationData,
	) -> Option<CollationResult> {
		tracing::trace!(
			target: LOG_TARGET,
			relay_parent = ?relay_parent,
			"Producing candidate",
		);

		let last_head = match Block::Header::decode(&mut &validation_data.parent_head.0[..]) {
			Ok(x) => x,
			Err(e) => {
				tracing::error!(
					target: LOG_TARGET,
					error = ?e,
					"Could not decode the head data."
				);
				return None
			},
		};

		let last_head_hash = last_head.hash();
		if !self.check_block_status(last_head_hash, &last_head) {
			return None
		}

		tracing::info!(
			target: LOG_TARGET,
			relay_parent = ?relay_parent,
			at = ?last_head_hash,
			"Starting collation.",
		);

		let candidate = self
			.parachain_consensus
			.produce_candidate(&last_head, relay_parent, &validation_data)
			.await?;

		let (header, extrinsics) = candidate.block.deconstruct();

		let compact_proof = match candidate
			.proof
			.into_compact_proof::<HashFor<Block>>(last_head.state_root().clone())
		{
			Ok(proof) => proof,
			Err(e) => {
				tracing::error!(target: "cumulus-collator", "Failed to compact proof: {:?}", e);
				return None
			},
		};

		// Create the parachain block data for the validators.
		let b = ParachainBlockData::<Block>::new(header, extrinsics, compact_proof);

		tracing::info!(
			target: LOG_TARGET,
			"PoV size {{ header: {}kb, extrinsics: {}kb, storage_proof: {}kb }}",
			b.header().encode().len() as f64 / 1024f64,
			b.extrinsics().encode().len() as f64 / 1024f64,
			b.storage_proof().encode().len() as f64 / 1024f64,
		);

		let pov =
			polkadot_node_primitives::maybe_compress_pov(PoV { block_data: BlockData(b.encode()) });

		tracing::info!(
			target: LOG_TARGET,
			"Compressed PoV size: {}kb",
			pov.block_data.0.len() as f64 / 1024f64,
		);

		let block_hash = b.header().hash();
		let collation = self.build_collation(b.clone(), block_hash, pov)?;

		self.validate_pov(
			BlockData(b.encode()),
			validation_data,
			relay_parent,
			collation.head_data.0.clone(),
		);

		let (result_sender, signed_stmt_recv) = oneshot::channel();

		self.wait_to_announce.lock().wait_to_announce(block_hash, signed_stmt_recv);

		tracing::info!(target: LOG_TARGET, ?block_hash, "Produced proof-of-validity candidate.",);

		Some(CollationResult { collation, result_sender: Some(result_sender) })
	}

	fn validate_pov(
		&self,
		pov: BlockData,
		validation_data: PersistedValidationData,
		relay_parent: PHash,
		expected_head: Vec<u8>,
	) {
		let validation_params = ValidationParams {
			parent_head: validation_data.parent_head,
			block_data: pov,
			relay_parent_number: validation_data.relay_parent_number,
			relay_parent_storage_root: validation_data.relay_parent_storage_root,
		};
		let mut handle = self.overseer_handle.clone();
		let para_id = self.para_id;

		self.spawner.spawn_blocking(
			"test",
			None,
			async move {
				let (code_tx, code_rx) = oneshot::channel();
				handle
					.send_msg(
						RuntimeApiMessage::Request(
							relay_parent,
							RuntimeApiRequest::ValidationCode(
								para_id,
								OccupiedCoreAssumption::TimedOut,
								code_tx,
							),
						),
						"Lol",
					)
					.await;

				let validation_code = code_rx.await.unwrap().unwrap().unwrap();

				type HostFunctions = (
					sp_io::misc::HostFunctions,
					sp_io::crypto::HostFunctions,
					sp_io::hashing::HostFunctions,
					sp_io::allocator::HostFunctions,
					sp_io::logging::HostFunctions,
					sp_io::trie::HostFunctions,
				);

				let runtime = sc_executor_wasmtime::create_runtime::<HostFunctions>(
					RuntimeBlob::uncompress_if_needed(&validation_code.0).unwrap(),
					CONFIG,
				)
				.unwrap();

				let mut instance = runtime.new_instance().unwrap();

				let start = std::time::Instant::now();
				let res = sp_io::TestExternalities::default().execute_with(|| {
					instance
						.call(InvokeMethod::Export("validate_block"), &validation_params.encode())
						.unwrap()
				});
				let end = std::time::Instant::now();

				let res = ValidationResult::decode(&mut &res[..]).unwrap();

				let res_header =
					<Block as BlockT>::Header::decode(&mut &res.head_data.0[..]).unwrap();
				let expected_header =
					<Block as BlockT>::Header::decode(&mut &expected_head[..]).unwrap();

				if res_header != expected_header {
					tracing::error!(?res_header, ?expected_header, "MISMATCH");
				}

				tracing::error!("VALIDATION TOOK: {:#?}", end.duration_since(start));
			}
			.boxed(),
		);
	}
}

/// Parameters for [`start_collator`].
pub struct StartCollatorParams<Block: BlockT, RA, BS, Spawner> {
	pub para_id: ParaId,
	pub runtime_api: Arc<RA>,
	pub block_status: Arc<BS>,
	pub announce_block: Arc<dyn Fn(Block::Hash, Option<Vec<u8>>) + Send + Sync>,
	pub overseer_handle: OverseerHandle,
	pub spawner: Spawner,
	pub key: CollatorPair,
	pub parachain_consensus: Box<dyn ParachainConsensus<Block>>,
}

/// Start the collator.
pub async fn start_collator<Block, RA, BS, Spawner>(
	StartCollatorParams {
		para_id,
		block_status,
		announce_block,
		mut overseer_handle,
		spawner,
		key,
		parachain_consensus,
		runtime_api,
	}: StartCollatorParams<Block, RA, BS, Spawner>,
) where
	Block: BlockT,
	BS: BlockBackend<Block> + Send + Sync + 'static,
	Spawner: SpawnNamed + Clone + Send + Sync + 'static,
	RA: ProvideRuntimeApi<Block> + Send + Sync + 'static,
	RA::Api: CollectCollationInfo<Block>,
{
	{
		let wasm_file = include_bytes!("../../../kilt_runtime.wasm");
		let wasm_file = hex::decode(wasm_file).unwrap();
		let version = sc_executor::read_embedded_version(
			&RuntimeBlob::uncompress_if_needed(&wasm_file).unwrap(),
		)
		.unwrap()
		.unwrap();

		tracing::error!("RUNTIME API VERSION: {:?}", version,);
	}

	let collator = Collator::new(
		block_status,
		Arc::new(spawner),
		announce_block,
		runtime_api,
		parachain_consensus,
		para_id,
		overseer_handle.clone(),
	);

	let span = tracing::Span::current();
	let config = CollationGenerationConfig {
		key,
		para_id,
		collator: Box::new(move |relay_parent, validation_data| {
			let collator = collator.clone();
			collator
				.produce_candidate(relay_parent, validation_data.clone())
				.instrument(span.clone())
				.boxed()
		}),
	};

	overseer_handle
		.send_msg(CollationGenerationMessage::Initialize(config), "StartCollator")
		.await;

	overseer_handle
		.send_msg(CollatorProtocolMessage::CollateOn(para_id), "StartCollator")
		.await;
}

#[cfg(test)]
mod tests {
	use super::*;
	use async_trait::async_trait;
	use cumulus_client_consensus_common::ParachainCandidate;
	use cumulus_test_client::{
		Client, ClientBlockImportExt, DefaultTestClientBuilderExt, InitBlockBuilder,
		TestClientBuilder, TestClientBuilderExt,
	};
	use cumulus_test_runtime::{Block, Header};
	use futures::{channel::mpsc, executor::block_on, StreamExt};
	use polkadot_node_subsystem_test_helpers::ForwardSubsystem;
	use polkadot_overseer::{dummy::dummy_overseer_builder, HeadSupportsParachains};
	use sp_consensus::BlockOrigin;
	use sp_core::{testing::TaskExecutor, Pair};
	use sp_runtime::traits::BlakeTwo256;
	use sp_state_machine::Backend;

	struct AlwaysSupportsParachains;

	#[async_trait]
	impl HeadSupportsParachains for AlwaysSupportsParachains {
		async fn head_supports_parachains(&self, _head: &PHash) -> bool {
			true
		}
	}

	#[derive(Clone)]
	struct DummyParachainConsensus {
		client: Arc<Client>,
	}

	#[async_trait::async_trait]
	impl ParachainConsensus<Block> for DummyParachainConsensus {
		async fn produce_candidate(
			&mut self,
			parent: &Header,
			_: PHash,
			validation_data: &PersistedValidationData,
		) -> Option<ParachainCandidate<Block>> {
			let block_id = BlockId::Hash(parent.hash());
			let builder = self.client.init_block_builder_at(
				&block_id,
				Some(validation_data.clone()),
				Default::default(),
			);

			let (block, _, proof) = builder.build().expect("Creates block").into_inner();

			self.client
				.import(BlockOrigin::Own, block.clone())
				.await
				.expect("Imports the block");

			Some(ParachainCandidate { block, proof: proof.expect("Proof is returned") })
		}
	}

	#[test]
	fn collates_produces_a_block_and_storage_proof_does_not_contains_code() {
		sp_tracing::try_init_simple();

		let spawner = TaskExecutor::new();
		let para_id = ParaId::from(100);
		let announce_block = |_, _| ();
		let client = Arc::new(TestClientBuilder::new().build());
		let header = client.header(&BlockId::Number(0)).unwrap().unwrap();

		let (sub_tx, sub_rx) = mpsc::channel(64);

		let (overseer, handle) =
			dummy_overseer_builder(spawner.clone(), AlwaysSupportsParachains, None)
				.expect("Creates overseer builder")
				.replace_collation_generation(|_| ForwardSubsystem(sub_tx))
				.build()
				.expect("Builds overseer");

		spawner.spawn("overseer", None, overseer.run().then(|_| async { () }).boxed());

		let collator_start = start_collator(StartCollatorParams {
			runtime_api: client.clone(),
			block_status: client.clone(),
			announce_block: Arc::new(announce_block),
			overseer_handle: OverseerHandle::new(handle),
			spawner,
			para_id,
			key: CollatorPair::generate().0,
			parachain_consensus: Box::new(DummyParachainConsensus { client: client.clone() }),
		});
		block_on(collator_start);

		let msg = block_on(sub_rx.into_future())
			.0
			.expect("message should be send by `start_collator` above.");

		let config = match msg {
			CollationGenerationMessage::Initialize(config) => config,
		};

		let mut validation_data = PersistedValidationData::default();
		validation_data.parent_head = header.encode().into();
		let relay_parent = Default::default();

		let collation = block_on((config.collator)(relay_parent, &validation_data))
			.expect("Collation is build")
			.collation;

		let pov = collation.proof_of_validity.into_compressed();

		let decompressed =
			sp_maybe_compressed_blob::decompress(&pov.block_data.0, 1024 * 1024 * 10).unwrap();

		let block =
			ParachainBlockData::<Block>::decode(&mut &decompressed[..]).expect("Is a valid block");

		assert_eq!(1, *block.header().number());

		// Ensure that we did not include `:code` in the proof.
		let proof = block.storage_proof();
		let db = proof
			.to_storage_proof::<BlakeTwo256>(Some(header.state_root()))
			.unwrap()
			.0
			.into_memory_db();

		let backend = sp_state_machine::new_in_mem_hash_key::<BlakeTwo256>()
			.update_backend(*header.state_root(), db);

		// Should return an error, as it was not included while building the proof.
		assert!(backend
			.storage(sp_core::storage::well_known_keys::CODE)
			.unwrap_err()
			.contains("Trie lookup error: Database missing expected key"));
	}
}
