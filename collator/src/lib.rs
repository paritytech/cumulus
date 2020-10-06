// Copyright 2019 Parity Technologies (UK) Ltd.
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

use cumulus_network::{
	DelayedBlockAnnounceValidator, JustifiedBlockAnnounceValidator, WaitToAnnounce,
};
use cumulus_primitives::{
	inherents::{DownwardMessagesType, DOWNWARD_MESSAGES_IDENTIFIER, VALIDATION_DATA_IDENTIFIER},
	well_known_keys, ValidationData,
};
use cumulus_runtime::ParachainBlockData;

use sc_client_api::{BlockBackend, Finalizer, StateBackend, UsageProvider};
use sp_api::ApiExt;
use sp_blockchain::HeaderBackend;
use sp_consensus::{
	BlockImport, BlockImportParams, BlockOrigin, BlockStatus, Environment, Error as ConsensusError,
	ForkChoiceStrategy, Proposal, Proposer, RecordProof, SyncOracle,
};
use sp_core::traits::SpawnNamed;
use sp_inherents::{InherentData, InherentDataProviders};
use sp_runtime::{
	generic::BlockId,
	traits::{BlakeTwo256, Block as BlockT, Header as HeaderT},
};
use sp_state_machine::InspectState;

use polkadot_node_primitives::{Collation, CollationGenerationConfig};
use polkadot_node_subsystem::messages::CollationGenerationMessage;
use polkadot_overseer::OverseerHandler;
use polkadot_primitives::v1::{
	Block as PBlock, BlockData, CollatorPair, Hash as PHash, HeadData, Id as ParaId, PoV,
	UpwardMessage,
};
use polkadot_service::{ClientHandle, RuntimeApiCollection};

use codec::{Decode, Encode};

use log::{debug, error, info, trace};

use futures::prelude::*;

use std::{marker::PhantomData, pin::Pin, sync::Arc, time::Duration};

use parking_lot::Mutex;

/// The implementation of the Cumulus `Collator`.
pub struct Collator<Block: BlockT, PF, BI, BS, Backend> {
	proposer_factory: Arc<Mutex<PF>>,
	_phantom: PhantomData<Block>,
	inherent_data_providers: InherentDataProviders,
	block_import: Arc<Mutex<BI>>,
	block_status: Arc<BS>,
	wait_to_announce: Arc<Mutex<WaitToAnnounce<Block>>>,
	backend: Arc<Backend>,
}

impl<Block: BlockT, PF, BI, BS, Backend> Clone for Collator<Block, PF, BI, BS, Backend> {
	fn clone(&self) -> Self {
		Self {
			proposer_factory: self.proposer_factory.clone(),
			inherent_data_providers: self.inherent_data_providers.clone(),
			_phantom: PhantomData,
			block_import: self.block_import.clone(),
			block_status: self.block_status.clone(),
			wait_to_announce: self.wait_to_announce.clone(),
			backend: self.backend.clone(),
		}
	}
}

impl<Block, PF, BI, BS, Backend> Collator<Block, PF, BI, BS, Backend>
where
	Block: BlockT,
	PF: Environment<Block> + 'static + Send,
	PF::Proposer: Send,
	BI: BlockImport<
			Block,
			Error = ConsensusError,
			Transaction = <PF::Proposer as Proposer<Block>>::Transaction,
		> + Send
		+ Sync
		+ 'static,
	BS: BlockBackend<Block>,
	Backend: sc_client_api::Backend<Block> + 'static,
{
	/// Create a new instance.
	fn new(
		proposer_factory: PF,
		inherent_data_providers: InherentDataProviders,
		overseer_handler: OverseerHandler,
		block_import: BI,
		block_status: Arc<BS>,
		spawner: Arc<dyn SpawnNamed + Send + Sync>,
		announce_block: Arc<dyn Fn(Block::Hash, Vec<u8>) + Send + Sync>,
		backend: Arc<Backend>,
	) -> Self {
		let wait_to_announce = Arc::new(Mutex::new(WaitToAnnounce::new(
			spawner,
			announce_block,
			overseer_handler,
		)));

		Self {
			proposer_factory: Arc::new(Mutex::new(proposer_factory)),
			inherent_data_providers,
			_phantom: PhantomData,
			block_import: Arc::new(Mutex::new(block_import)),
			block_status,
			wait_to_announce,
			backend,
		}
	}

	/// Get the inherent data with validation function parameters injected
	fn inherent_data(
		&mut self,
		validation_data: &ValidationData,
		downward_messages: DownwardMessagesType,
	) -> Option<InherentData> {
		let mut inherent_data = self
			.inherent_data_providers
			.create_inherent_data()
			.map_err(|e| {
				error!(
					target: "cumulus-collator",
					"Failed to create inherent data: {:?}",
					e,
				)
			})
			.ok()?;

		inherent_data
			.put_data(VALIDATION_DATA_IDENTIFIER, validation_data)
			.map_err(|e| {
				error!(
					target: "cumulus-collator",
					"Failed to put validation function params into inherent data: {:?}",
					e,
				)
			})
			.ok()?;

		inherent_data
			.put_data(DOWNWARD_MESSAGES_IDENTIFIER, &downward_messages)
			.map_err(|e| {
				error!(
					target: "cumulus-collator",
					"Failed to put downward messages into inherent data: {:?}",
					e,
				)
			})
			.ok()?;

		Some(inherent_data)
	}

	/// Checks the status of the given block hash in the Parachain.
	///
	/// Returns `true` if the block could be found and is good to be build on.
	fn check_block_status(&self, hash: Block::Hash) -> bool {
		match self.block_status.block_status(&BlockId::Hash(hash)) {
			Ok(BlockStatus::Queued) => {
				debug!(
					target: "cumulus-collator",
					"Skipping candidate production, because block `{:?}` is still queued for import.", hash,
				);
				false
			}
			Ok(BlockStatus::InChainWithState) => true,
			Ok(BlockStatus::InChainPruned) => {
				error!(
					target: "cumulus-collator",
					"Skipping candidate production, because block `{:?}` is already pruned!", hash,
				);
				false
			}
			Ok(BlockStatus::KnownBad) => {
				error!(
					target: "cumulus-collator",
					"Block `{}` is tagged as known bad and is included in the relay chain! Skipping candidate production!", hash,
				);
				false
			}
			Ok(BlockStatus::Unknown) => {
				debug!(
					target: "cumulus-collator",
					"Skipping candidate production, because block `{:?}` is unknown.", hash,
				);
				false
			}
			Err(e) => {
				error!(target: "cumulus-collator", "Failed to get block status of `{:?}`: {:?}", hash, e);
				false
			}
		}
	}

	fn build_collation(
		&mut self,
		block: ParachainBlockData<Block>,
		block_hash: Block::Hash,
	) -> Option<Collation> {
		let block_data = BlockData(block.encode());
		let header = block.into_header();
		let head_data = HeadData(header.encode());

		let state = match self.backend.state_at(BlockId::Hash(block_hash)) {
			Ok(state) => state,
			Err(e) => {
				error!(target: "cumulus-collator", "Failed to get state of the freshly build block: {:?}", e);
				return None;
			}
		};

		state.inspect_state(|| {
			let upward_messages = sp_io::storage::get(well_known_keys::UPWARD_MESSAGES);
			let upward_messages = match upward_messages.map(|v| Vec::<UpwardMessage>::decode(&mut &v[..])) {
				Some(Ok(msgs)) => msgs,
				Some(Err(e)) => {
					error!(target: "cumulus-collator", "Failed to decode upward messages from the build block: {:?}", e);
					return None
				},
				None => Vec::new(),
			};

			let new_validation_code = sp_io::storage::get(well_known_keys::NEW_VALIDATION_CODE);

			Some(Collation {
				//TODO: What should be do here?
				fees: 0,
				upward_messages,
				new_validation_code: new_validation_code.map(Into::into),
				head_data,
				proof_of_validity: PoV { block_data },
			})
		})
	}

	async fn produce_candidate(
		mut self,
		relay_parent: PHash,
		validation_data: ValidationData,
	) -> Option<Collation> {
		trace!(target: "cumulus-collator", "Producing candidate");

		let last_head =
			match Block::Header::decode(&mut &validation_data.persisted.parent_head.0[..]) {
				Ok(x) => x,
				Err(e) => {
					error!(target: "cumulus-collator", "Could not decode the head data: {:?}", e);
					return None;
				}
			};

		if !self.check_block_status(last_head.hash()) {
			return None;
		}

		let proposer_future = self.proposer_factory.lock().init(&last_head);

		let proposer = proposer_future
			.await
			.map_err(|e| {
				error!(
					target: "cumulus-collator",
					"Could not create proposer: {:?}",
					e,
				)
			})
			.ok()?;

		let inherent_data = self.inherent_data(
			&validation_data,
			// TODO get the downward messages
			Vec::new(),
		)?;

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
				RecordProof::Yes,
			)
			.await
			.map_err(|e| {
				error!(
					target: "cumulus-collator",
					"Proposing failed: {:?}",
					e,
				)
			})
			.ok()?;

		let proof = match proof {
			Some(proof) => proof,
			None => {
				error!(
					target: "cumulus-collator",
					"Proposer did not return the requested proof.",
				);

				return None;
			}
		};

		let (header, extrinsics) = block.deconstruct();
		let block_hash = header.hash();

		// Create the parachain block data for the validators.
		let b = ParachainBlockData::<Block>::new(header.clone(), extrinsics, proof);

		let mut block_import_params = BlockImportParams::new(BlockOrigin::Own, header);
		block_import_params.body = Some(b.extrinsics().to_vec());
		// Best block is determined by the relay chain.
		block_import_params.fork_choice = Some(ForkChoiceStrategy::Custom(false));
		block_import_params.storage_changes = Some(storage_changes);

		if let Err(err) = self
			.block_import
			.lock()
			.import_block(block_import_params, Default::default())
		{
			error!(
				target: "cumulus-collator",
				"Error importing build block (at {:?}): {:?}",
				b.header().parent_hash(),
				err,
			);

			return None;
		}

		let collation = self.build_collation(b, block_hash)?;
		let pov_hash = collation.proof_of_validity.hash();

		self.wait_to_announce
			.lock()
			.wait_to_announce(block_hash, pov_hash);

		info!(target: "cumulus-collator", "Produced proof-of-validity candidate `{:?}` from block `{:?}`.", pov_hash, block_hash);

		Some(collation)
	}
}

/// Parameters for [`start_collator`].
pub struct StartCollatorParams<Block: BlockT, PF, BI, Backend, Client, BS, Spawner> {
	pub proposer_factory: PF,
	pub inherent_data_providers: InherentDataProviders,
	pub backend: Backend,
	pub block_import: BI,
	pub block_status: BS,
	pub client: Arc<Client>,
	pub announce_block: Arc<dyn Fn(Block::Hash, Vec<u8>) + Send + Sync>,
	pub overseer_handler: OverseerHandler,
	pub spawner: Spawner,
	pub para_id: ParaId,
	pub key: CollatorPair,
	pub polkadot_client: polkadot_service::Client,
}

pub async fn start_collator<Block: BlockT, PF, BI, Backend, Client, BS, Spawner>(
	params: StartCollatorParams<Block, PF, BI, Backend, Client, BS, Spawner>,
) -> Result<(), String>
where
	PF: Environment<Block> + Send + 'static,
	BI: BlockImport<Block, Error = sp_consensus::Error, Transaction = TransactionFor<PF, Block>>
		+ Send
		+ Sync
		+ 'static,
	Backend: sc_client_api::Backend<Block> + 'static,
	Client: Finalizer<Block, Backend>
		+ UsageProvider<Block>
		+ HeaderBackend<Block>
		+ Send
		+ Sync
		+ BlockBackend<Block>
		+ 'static,
	for<'a> &'a Client: BlockImport<Block>,
	BS: BlockBackend<Block> + Send + Sync + 'static,
	Spawner: SpawnNamed + Clone + Send + Sync + 'static,
{
	params
		.polkadot_client
		.clone()
		.execute_with(StartCollator { params })?
		.await
}

struct StartCollator<Block: BlockT, PF, BI, Backend, Client, BS, Spawner> {
	params: StartCollatorParams<Block, PF, BI, Backend, Client, BS, Spawner>,
}

type TransactionFor<E, Block> =
	<<E as Environment<Block>>::Proposer as Proposer<Block>>::Transaction;

impl<Block: BlockT, PF, BI, Backend, Client, BS, Spawner> polkadot_service::ExecuteWithClient
	for StartCollator<Block, PF, BI, Backend, Client, BS, Spawner>
where
	PF: Environment<Block> + Send + 'static,
	BI: BlockImport<Block, Error = sp_consensus::Error, Transaction = TransactionFor<PF, Block>>
		+ Send
		+ Sync
		+ 'static,
	Backend: sc_client_api::Backend<Block> + 'static,
	Client: Finalizer<Block, Backend>
		+ UsageProvider<Block>
		+ HeaderBackend<Block>
		+ Send
		+ Sync
		+ BlockBackend<Block>
		+ 'static,
	for<'a> &'a Client: BlockImport<Block>,
	BS: BlockBackend<Block> + Send + Sync + 'static,
	Spawner: SpawnNamed + Clone + Send + Sync + 'static,
{
	type Output = Result<Pin<Box<dyn Future<Output = Result<(), String>>>>, String>;

	fn execute_with_client<PClient, Api, PBackend>(
		self,
		polkadot_client: Arc<PClient>,
	) -> Self::Output
	where
		<Api as ApiExt<PBlock>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
		PBackend: sc_client_api::Backend<PBlock>,
		PBackend::State: StateBackend<BlakeTwo256>,
		Api: RuntimeApiCollection<StateBackend = PBackend::State>,
		PClient: polkadot_service::AbstractClient<PBlock, PBackend, Api = Api> + 'static,
	{
		let follow = match cumulus_consensus::follow_polkadot(
			self.params.para_id,
			self.params.client,
			polkadot_client,
			self.params.announce_block.clone(),
		) {
			Ok(follow) => follow,
			Err(e) => return Err(format!("Could not start following polkadot: {:?}", e)),
		};

		self.params
			.spawner
			.spawn("cumulus-follow-polkadot", follow.map(|_| ()).boxed());

		let collator = Collator::new(
			self.params.proposer_factory,
			self.params.inherent_data_providers,
			self.params.overseer_handler.clone(),
			self.params.block_import,
			Arc::new(self.params.block_status),
			Arc::new(self.params.spawner),
			self.params.announce_block,
			Arc::new(self.params.backend),
		);

		let config = CollationGenerationConfig {
			key: self.params.key,
			para_id: self.params.para_id,
			collator: Box::new(move |relay_parent, validation_data| {
				let collator = collator.clone();
				collator
					.produce_candidate(relay_parent, validation_data.clone())
					.boxed()
			}),
		};

		let mut overseer_handler = self.params.overseer_handler;
		Ok(async move {
			overseer_handler
				.send_msg(CollationGenerationMessage::Initialize(config))
				.await
				.map_err(|e| format!("Failed to register collator: {:?}", e))
		}
		.boxed())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::time::Duration;

	use polkadot_collator::{collate, SignedStatement};
	use polkadot_primitives::v0::Id as ParaId;

	use sp_blockchain::Result as ClientResult;
	use sp_core::testing::TaskExecutor;
	use sp_inherents::InherentData;
	use sp_keyring::Sr25519Keyring;
	use sp_runtime::traits::{DigestFor, Header as HeaderT};
	use sp_state_machine::StorageProof;
	use substrate_test_client::{NativeExecutor, WasmExecutionMethod::Interpreted};

	use test_client::{DefaultTestClientBuilderExt, TestClientBuilder, TestClientBuilderExt};
	use test_runtime::{Block, Header};

	use futures::{future, Stream};

	#[derive(Debug)]
	struct Error;

	impl From<sp_consensus::Error> for Error {
		fn from(_: sp_consensus::Error) -> Self {
			unimplemented!("Not required in tests")
		}
	}

	struct DummyFactory;

	impl Environment<Block> for DummyFactory {
		type Proposer = DummyProposer;
		type Error = Error;
		type CreateProposer = Pin<
			Box<dyn Future<Output = Result<Self::Proposer, Self::Error>> + Send + Unpin + 'static>,
		>;

		fn init(&mut self, _: &Header) -> Self::CreateProposer {
			Box::pin(future::ready(Ok(DummyProposer)))
		}
	}

	struct DummyProposer;

	impl Proposer<Block> for DummyProposer {
		type Error = Error;
		type Proposal = future::Ready<Result<Proposal<Block, Self::Transaction>, Error>>;
		type Transaction = sc_client_api::TransactionFor<test_client::Backend, Block>;

		fn propose(
			self,
			_: InherentData,
			digest: DigestFor<Block>,
			_: Duration,
			_: RecordProof,
		) -> Self::Proposal {
			let header = Header::new(
				1337,
				Default::default(),
				Default::default(),
				Default::default(),
				digest,
			);

			future::ready(Ok(Proposal {
				block: Block::new(header, Vec::new()),
				storage_changes: Default::default(),
				proof: Some(StorageProof::empty()),
			}))
		}
	}

	#[derive(Clone)]
	struct DummySyncOracle;

	impl SyncOracle for DummySyncOracle {
		fn is_major_syncing(&mut self) -> bool {
			unimplemented!("Not required in tests")
		}

		fn is_offline(&mut self) -> bool {
			unimplemented!("Not required in tests")
		}
	}

	#[derive(Clone)]
	struct DummyPolkadotClient;

	impl cumulus_consensus::PolkadotClient for DummyPolkadotClient {
		type Error = Error;
		type HeadStream = Box<dyn futures::Stream<Item = Vec<u8>> + Send + Unpin>;

		fn new_best_heads(&self, _: ParaId) -> ClientResult<Self::HeadStream> {
			unimplemented!("Not required in tests")
		}

		fn finalized_heads(&self, _: ParaId) -> ClientResult<Self::HeadStream> {
			unimplemented!("Not required in tests")
		}

		fn parachain_head_at(
			&self,
			_: &BlockId<PBlock>,
			_: ParaId,
		) -> ClientResult<Option<Vec<u8>>> {
			unimplemented!("Not required in tests")
		}
	}

	#[test]
	fn collates_produces_a_block() {
		let id = ParaId::from(100);
		let _ = env_logger::try_init();
		let spawner = TaskExecutor::new();
		let announce_block = |_, _| ();
		let block_announce_validator = DelayedBlockAnnounceValidator::new();
		let client = Arc::new(TestClientBuilder::new().build());

		let builder = CollatorBuilder::new(
			DummyFactory,
			InherentDataProviders::default(),
			client.clone(),
			client.clone(),
			id,
			client.clone(),
			Arc::new(announce_block),
			block_announce_validator,
		);
		let context = builder
			.build(
				polkadot_service::Client::Polkadot(Arc::new(
					substrate_test_client::TestClientBuilder::<_, _, _, ()>::default()
						.build_with_native_executor::<polkadot_service::polkadot_runtime::RuntimeApi, _>(
							Some(NativeExecutor::<polkadot_service::PolkadotExecutor>::new(
								Interpreted,
								None,
								1,
							)),
						)
						.0,
				)),
				spawner,
				DummyCollatorNetwork,
			)
			.expect("Creates parachain context");

		let header = client.header(&BlockId::Number(0)).unwrap().unwrap();

		let collation = collate(
			Default::default(),
			id,
			GlobalValidationData {
				block_number: 0,
				max_code_size: 0,
				max_head_data_size: 0,
			},
			LocalValidationData {
				parent_head: parachain::HeadData(HeadData::<Block> { header }.encode()),
				balance: 10,
				code_upgrade_allowed: None,
			},
			Vec::new(),
			context,
			Arc::new(Sr25519Keyring::Alice.pair().into()),
		);

		let collation = futures::executor::block_on(collation).unwrap();

		let block_data = collation.pov.block_data;

		let block = Block::decode(&mut &block_data.0[..]).expect("Is a valid block");

		assert_eq!(1337, *block.header().number());
	}
}
