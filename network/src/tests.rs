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

use super::*;
use polkadot_test_runtime_client::{
	DefaultTestClientBuilderExt, TestClientBuilder, TestClientBuilderExt, TestClient,
};
use cumulus_test_runtime::{Block, Header};
use sp_consensus::block_validation::BlockAnnounceValidator;
use polkadot_primitives::{
	parachain::{
		ParachainHost, ValidatorId, Id as ParaId, Chain, CollatorId, Retriable, DutyRoster,
		ValidationCode, GlobalValidationSchedule, LocalValidationData, AbridgedCandidateReceipt,
		SigningContext,
	},
	Block as PBlock, Hash as PHash, Header as PHeader,
};
use sp_api::{ApiRef, ProvideRuntimeApi};
use sp_blockchain::{Error as ClientError, HeaderBackend};
use sp_runtime::traits::{NumberFor, Block as BlockT, Zero};
use polkadot_validation::{Statement, sign_table_statement};
use sp_core::H256;
use sp_keyring::Sr25519Keyring;

fn make_validator(authorities: Vec<ValidatorId>) -> JustifiedBlockAnnounceValidator<Block, TestApi> {
	let (validator, _client) = make_validator_and_client(authorities);

	validator
}

fn make_validator_and_client(authorities: Vec<ValidatorId>) -> (JustifiedBlockAnnounceValidator<Block, TestApi>, Arc<TestApi>) {
	let builder = TestClientBuilder::new();
	let client = Arc::new(TestApi::new(Arc::new(builder.build())));

	(JustifiedBlockAnnounceValidator::new(authorities, client.clone()), client)
}

fn default_header() -> Header {
	Header {
		number: Default::default(),
		digest: Default::default(),
		extrinsics_root: Default::default(),
		parent_hash: Default::default(),
		state_root: Default::default(),
	}
}

fn make_gossip_message_and_header(client: Arc<TestApi>, relay_chain_leaf: H256)
-> (GossipMessage, Header) {
	let key = Sr25519Keyring::Alice.pair().into();
	let signing_context = client
		.runtime_api()
		.signing_context(&BlockId::Hash(relay_chain_leaf))
		.unwrap();
	let header = default_header();
	let candidate_receipt = AbridgedCandidateReceipt {
		head_data: header.encode().into(),
		..AbridgedCandidateReceipt::default()
	};
	let statement = Statement::Candidate(candidate_receipt);
	let signature = sign_table_statement(&statement, &key, &signing_context);
	let sender = 0;
	let gossip_statement = GossipStatement {
		relay_chain_leaf,
		signed_statement: SignedStatement {
			statement,
			signature,
			sender,
		},
	};

	(GossipMessage::Statement(gossip_statement), header)
}

#[test]
fn valid_if_no_data() {
	let mut validator = make_validator(Vec::new());
	let header = default_header();

	assert!(
		matches!(validator.validate(&header, &[]), Ok(Validation::Success)),
		"validating without data is always a success",
	);
}

#[test]
fn check_gossip_message_is_valid() {
	let mut validator = make_validator(Vec::new());
	let header = default_header();

	let res = validator.validate(&header, &[0x42]).err();
	assert!(res.is_some(), "only data that are gossip message are allowed");
	assert!(matches!(
		*res.unwrap().downcast::<ClientError>().unwrap(),
		ClientError::BadJustification(x) if x.contains("must be a gossip message")
	));
}

#[test]
fn check_relay_parent_is_head() {
	let (mut validator, client) = make_validator_and_client(Vec::new());
	let relay_chain_leaf = H256::zero();
	let (gossip_message, header) = make_gossip_message_and_header(client, relay_chain_leaf);
	let data = gossip_message.encode();
	let res = validator.validate(&header, data.as_slice());

	assert_eq!(
		res.unwrap(),
		Validation::Failure,
		"validation fails if the relay chain parent is not the relay chain head",
	);
}

#[test]
fn check_relay_parent_actually_exists() {
	let (mut validator, client) = make_validator_and_client(Vec::new());
	let relay_chain_leaf = H256::from_low_u64_be(42);
	let (gossip_message, header) = make_gossip_message_and_header(client, relay_chain_leaf);
	let data = gossip_message.encode();
	let res = validator.validate(&header, data.as_slice()).err();

	assert!(res.is_some(), "validation should fail if the relay chain leaf does not exist");
	assert!(matches!(
		*res.unwrap().downcast::<ClientError>().unwrap(),
		ClientError::UnknownBlock(_)
	));
}

#[test]
fn check_relay_parent_does_not_fail() {
	let (mut validator, client) = make_validator_and_client(Vec::new());
	let relay_chain_leaf = H256::from_low_u64_be(0xdead);
	let (gossip_message, header) = make_gossip_message_and_header(client, relay_chain_leaf);
	let data = gossip_message.encode();
	let res = validator.validate(&header, data.as_slice()).err();

	assert!(res.is_some(), "validation should fail if the relay chain leaf does not exist");
	assert!(matches!(
		*res.unwrap().downcast::<ClientError>().unwrap(),
		ClientError::Backend(_)
	));
}

#[test]
fn check_signer_is_legit_validator() {
	let (mut validator, client) = make_validator_and_client(Vec::new());
	let relay_chain_leaf = H256::from_low_u64_be(1);
	let (gossip_message, header) = make_gossip_message_and_header(client, relay_chain_leaf);
	let data = gossip_message.encode();
	let res = validator.validate(&header, data.as_slice()).err();

	assert!(res.is_some(), "validation should fail if the signer is not a legit validator");
	assert!(matches!(
		*res.unwrap().downcast::<ClientError>().unwrap(),
		ClientError::BadJustification(x) if x.contains("signer is a validator")
	));
}

#[test]
fn check_statement_is_correctly_signed() {
	let (mut validator, client) = make_validator_and_client(vec![
		Sr25519Keyring::Alice.public().into(),
	]);
	let header = default_header();
	let relay_chain_leaf = H256::from_low_u64_be(1);

	let key = Sr25519Keyring::Alice.pair().into();
	let signing_context = client
		.runtime_api()
		.signing_context(&BlockId::Hash(relay_chain_leaf))
		.unwrap();
	let mut candidate_receipt = AbridgedCandidateReceipt::default();
	let statement = Statement::Candidate(candidate_receipt.clone());
	let signature = sign_table_statement(&statement, &key, &signing_context);

	// alterate statement so the signature doesn't match anymore
	candidate_receipt = AbridgedCandidateReceipt {
		parachain_index: candidate_receipt.parachain_index + 1,
		..candidate_receipt
	};
	let statement = Statement::Candidate(candidate_receipt);

	let sender = 0;
	let gossip_statement = GossipStatement {
		relay_chain_leaf,
		signed_statement: SignedStatement {
			statement,
			signature,
			sender,
		},
	};
	let gossip_message = GossipMessage::Statement(gossip_statement);

	let data = gossip_message.encode();
	let res = validator.validate(&header, data.as_slice()).err();

	assert!(res.is_some(), "validation should fail if the statement is not correctly signed");
	assert!(matches!(
		*res.unwrap().downcast::<ClientError>().unwrap(),
		ClientError::BadJustification(x) if x.contains("signature is invalid")
	));
}

#[test]
fn check_statement_is_a_candidate_message() {
	let (mut validator, client) = make_validator_and_client(vec![
		Sr25519Keyring::Alice.public().into(),
	]);
	let header = default_header();
	let relay_chain_leaf = H256::from_low_u64_be(1);

	let key = Sr25519Keyring::Alice.pair().into();
	let signing_context = client
		.runtime_api()
		.signing_context(&BlockId::Hash(relay_chain_leaf))
		.unwrap();
	let statement = Statement::Valid(H256::zero());
	let signature = sign_table_statement(&statement, &key, &signing_context);
	let sender = 0;
	let gossip_statement = GossipStatement {
		relay_chain_leaf,
		signed_statement: SignedStatement {
			statement,
			signature,
			sender,
		},
	};
	let gossip_message = GossipMessage::Statement(gossip_statement);

	let data = gossip_message.encode();
	let res = validator.validate(&header, data.as_slice()).err();

	assert!(res.is_some(), "validation should fail if the statement is not a candidate message");
	assert!(matches!(
		*res.unwrap().downcast::<ClientError>().unwrap(),
		ClientError::BadJustification(x) if x.contains("must be a candidate statement")
	));
}

#[test]
fn check_header_match_candidate_receipt_header() {
	let (mut validator, client) = make_validator_and_client(vec![
		Sr25519Keyring::Alice.public().into(),
	]);
	let relay_chain_leaf = H256::from_low_u64_be(1);

	let key = Sr25519Keyring::Alice.pair().into();
	let signing_context = client
		.runtime_api()
		.signing_context(&BlockId::Hash(relay_chain_leaf))
		.unwrap();
	let header = default_header();
	let candidate_receipt = AbridgedCandidateReceipt::default();
	let statement = Statement::Candidate(candidate_receipt);
	let signature = sign_table_statement(&statement, &key, &signing_context);
	let sender = 0;
	let gossip_statement = GossipStatement {
		relay_chain_leaf,
		signed_statement: SignedStatement {
			statement,
			signature,
			sender,
		},
	};
	let gossip_message = GossipMessage::Statement(gossip_statement);

	let data = gossip_message.encode();
	let res = validator.validate(&header, data.as_slice()).err();

	assert!(res.is_some(), "validation should fail if the header in the candidate_receipt doesn't \
		match the header provided");
	assert!(matches!(
		*res.unwrap().downcast::<ClientError>().unwrap(),
		ClientError::BadJustification(x) if x.contains("header does not match")
	));
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
impl HeaderBackend<PBlock> for TestApi {
	fn header(
		&self,
		_id: BlockId<PBlock>,
	) -> std::result::Result<Option<PHeader>, sp_blockchain::Error> {
		Ok(None)
	}

	fn info(&self) -> sc_client_api::blockchain::Info<PBlock> {
		let best_hash = H256::from_low_u64_be(1);

		sc_client_api::blockchain::Info {
			best_hash,
			best_number: 1,
			finalized_hash: Default::default(),
			finalized_number: Zero::zero(),
			genesis_hash: Default::default(),
			number_leaves: Default::default(),
		}
	}

	fn status(
		&self,
		_id: BlockId<PBlock>,
	) -> std::result::Result<sc_client_api::blockchain::BlockStatus, sp_blockchain::Error> {
		Ok(sc_client_api::blockchain::BlockStatus::Unknown)
	}

	fn number(
		&self,
		hash: PHash,
	) -> std::result::Result<Option<NumberFor<PBlock>>, sp_blockchain::Error> {
		Ok(match hash.as_fixed_bytes() {
			&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] => Some(0),
			&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1] => Some(1),
			&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xde, 0xad] => {
				return Err(sp_blockchain::Error::Backend("dead".to_string()));
			},
			_ => None,
		})
	}

	fn hash(
		&self,
		_number: NumberFor<PBlock>,
	) -> std::result::Result<Option<PHash>, sp_blockchain::Error> {
		Ok(None)
	}
}
