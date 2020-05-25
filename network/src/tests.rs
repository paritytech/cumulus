// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

#![allow(unused_imports)] // TODO

use super::*;
//use polkadot_test_runtime::RuntimeApi;
use polkadot_test_runtime_client::{
	DefaultTestClientBuilderExt, TestClientBuilder, TestClientBuilderExt, TestClient,
};
use cumulus_test_runtime::{Block, Header, Runtime};
use sp_block_builder::runtime_decl_for_BlockBuilder::BlockBuilder;
use sp_consensus::block_validation::BlockAnnounceValidator;
use polkadot_primitives::{
	parachain::{
		ParachainHost, ValidatorId, Id as ParaId, Chain, CollatorId, Retriable, DutyRoster,
		ValidationCode, GlobalValidationSchedule, LocalValidationData, AbridgedCandidateReceipt,
		SigningContext,
	},
	Block as PBlock, Hash as PHash,
};
use sp_api::{ApiRef, ProvideRuntimeApi};
use sp_blockchain::{Error as ClientError, HeaderBackend};
use sp_runtime::traits::{NumberFor, Block as BlockT, Zero};
use polkadot_service::PolkadotChainSpec;
use sp_state_machine::BasicExternalities;
use sc_chain_spec::ChainSpec;

#[test]
fn valid_if_no_data() {
	let builder = TestClientBuilder::new();
	let client = Arc::new(TestApi::new(Arc::new(builder.build())));
	let authorities = Vec::new();
	let mut validator = JustifiedBlockAnnounceValidator::<Block, _>::new(authorities, client);

	let genesis_config = PolkadotChainSpec::from_json_bytes(&include_bytes!("../../test/parachain/res/polkadot_chainspec.json")[..]).unwrap();
	let storage = genesis_config.as_storage_builder().build_storage().unwrap();
	let mut basic_ext = BasicExternalities::new(storage);
	let raw_payload = basic_ext.execute_with(|| {
		let header = Runtime::finalize_block();
		validator.validate(&header, &[]);
	});
}
#[derive(Default)]
struct ApiData {
	validators: Vec<ValidatorId>,
	duties: Vec<Chain>,
	active_parachains: Vec<(ParaId, Option<(CollatorId, Retriable)>)>,
}

#[derive(Clone)]
struct TestApi {
	data: Arc<Mutex<ApiData>>,
	client: Arc<TestClient>,
}

impl TestApi {
	fn new(client: Arc<TestClient>) -> Self {
		Self {
			client,
			data: Default::default(),
		}
	}
}

#[derive(Default)]
struct RuntimeApi {
	data: Arc<Mutex<ApiData>>,
}

impl ProvideRuntimeApi<PBlock> for TestApi {
	type Api = RuntimeApi;

	fn runtime_api<'a>(&'a self) -> ApiRef<'a, Self::Api> {
		RuntimeApi { data: self.data.clone() }.into()
	}
}

sp_api::mock_impl_runtime_apis! {
	impl ParachainHost<PBlock> for RuntimeApi {
		type Error = sp_blockchain::Error;

		fn validators(&self) -> Vec<ValidatorId> {
			self.data.lock().validators.clone()
		}

		fn duty_roster(&self) -> DutyRoster {
			DutyRoster {
				validator_duty: self.data.lock().duties.clone(),
			}
		}

		fn active_parachains(&self) -> Vec<(ParaId, Option<(CollatorId, Retriable)>)> {
			self.data.lock().active_parachains.clone()
		}

		fn parachain_code(_: ParaId) -> Option<ValidationCode> {
			Some(ValidationCode(Vec::new()))
		}

		fn global_validation_schedule() -> GlobalValidationSchedule {
			Default::default()
		}

		fn local_validation_data(_: ParaId) -> Option<LocalValidationData> {
			Some(Default::default())
		}

		fn get_heads(_: Vec<<PBlock as BlockT>::Extrinsic>) -> Option<Vec<AbridgedCandidateReceipt>> {
			Some(Vec::new())
		}

		fn signing_context() -> SigningContext {
			SigningContext {
				session_index: Default::default(),
				parent_hash: Default::default(),
			}
		}
	}
}

/// Blockchain database header backend. Does not perform any validation.
impl<Block: BlockT> HeaderBackend<Block> for TestApi {
	fn header(
		&self,
		_id: BlockId<Block>,
	) -> std::result::Result<Option<Block::Header>, sp_blockchain::Error> {
		Ok(None)
	}

	fn info(&self) -> sc_client_api::blockchain::Info<Block> {
		sc_client_api::blockchain::Info {
			best_hash: Default::default(),
			best_number: Zero::zero(),
			finalized_hash: Default::default(),
			finalized_number: Zero::zero(),
			genesis_hash: Default::default(),
			number_leaves: Default::default(),
		}
	}

	fn status(
		&self,
		_id: BlockId<Block>,
	) -> std::result::Result<sc_client_api::blockchain::BlockStatus, sp_blockchain::Error> {
		Ok(sc_client_api::blockchain::BlockStatus::Unknown)
	}

	fn number(
		&self,
		_hash: Block::Hash,
	) -> std::result::Result<Option<NumberFor<Block>>, sp_blockchain::Error> {
		Ok(None)
	}

	fn hash(
		&self,
		_number: NumberFor<Block>,
	) -> std::result::Result<Option<Block::Hash>, sp_blockchain::Error> {
		Ok(None)
	}
}
