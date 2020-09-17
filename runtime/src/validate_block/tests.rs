// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

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

use crate::ParachainBlockData;

use parachain::primitives::{BlockData, HeadData, ValidationParams, ValidationResult};
use sc_block_builder::BlockBuilderProvider;
use sc_executor::{
	error::Result, sp_wasm_interface::HostFunctions, WasmExecutionMethod, WasmExecutor,
};
use sp_blockchain::HeaderBackend;
use sp_consensus::SelectChain;
use sp_core::traits::CallInWasm;
use sp_io::TestExternalities;
use sp_keyring::AccountKeyring;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, Header as HeaderT},
};
use test_client::{
	runtime::{Block, Hash, Header, WASM_BINARY},
	Client, DefaultTestClientBuilderExt, LongestChain, TestClientBuilder, TestClientBuilderExt,
};

use codec::{Decode, Encode};

fn call_validate_block(
	parent_head: Header,
	block_data: ParachainBlockData<Block>,
) -> Result<Header> {
	let mut ext = TestExternalities::default();
	let mut ext_ext = ext.ext();
	let params = ValidationParams {
		block_data: BlockData(block_data.encode()),
		parent_head: HeadData(parent_head.encode()),
		code_upgrade_allowed: None,
		max_code_size: 1024,
		max_head_data_size: 1024,
		relay_chain_height: 1,
	}
	.encode();

	let executor = WasmExecutor::new(
		WasmExecutionMethod::Interpreted,
		Some(1024),
		sp_io::SubstrateHostFunctions::host_functions(),
		1,
	);

	executor
		.call_in_wasm(
			&WASM_BINARY.expect("You need to build the WASM binaries to run the tests!"),
			None,
			"validate_block",
			&params,
			&mut ext_ext,
			sp_core::traits::MissingHostFunctions::Disallow,
		)
		.map(|v| ValidationResult::decode(&mut &v[..]).expect("Decode `ValidationResult`."))
		.map(|v| Header::decode(&mut &v.head_data.0[..]).expect("Decode `Header`."))
		.map_err(|err| err.into())
}

fn create_extrinsics() -> Vec<<Block as BlockT>::Extrinsic> {
	todo!();
	/*
	use test_client::runtime::Transfer;
	vec![
		Transfer {
			from: AccountKeyring::Alice.into(),
			to: AccountKeyring::Bob.into(),
			amount: 69,
			nonce: 0,
		}
		.into_signed_tx(),
		Transfer {
			from: AccountKeyring::Alice.into(),
			to: AccountKeyring::Charlie.into(),
			amount: 100,
			nonce: 1,
		}
		.into_signed_tx(),
		Transfer {
			from: AccountKeyring::Bob.into(),
			to: AccountKeyring::Charlie.into(),
			amount: 100,
			nonce: 0,
		}
		.into_signed_tx(),
		Transfer {
			from: AccountKeyring::Charlie.into(),
			to: AccountKeyring::Alice.into(),
			amount: 500,
			nonce: 0,
		}
		.into_signed_tx(),
	]
	*/
}

fn create_test_client() -> (Client, LongestChain) {
	TestClientBuilder::new()
		.set_execution_strategy(sc_client_api::ExecutionStrategy::NativeWhenPossible)
		.build_with_longest_chain()
}

fn build_block_with_proof(
	client: &Client,
	extrinsics: Vec<<Block as BlockT>::Extrinsic>,
) -> (Block, sp_trie::StorageProof) {
	let block_id = BlockId::Hash(client.info().best_hash);
	let mut builder = client
		.new_block_at(&block_id, Default::default(), true)
		.expect("Initializes new block");

	/*
	let caller = AccountKeyring::Alice;
	let function = cumulus_test_runtime::Call::Sudo(pallet_sudo::Call::sudo(Box::new(
		cumulus_test_runtime::Call::ParachainUpgrade(cumulus_parachain_upgrade::Call::set_validation_function_parameters(
			Default::default(),
		)),
	)));

	use sp_arithmetic::traits::SaturatedConversion;
	let current_block_hash = client.info().best_hash;
	let current_block = client.info().best_number.saturated_into();
	let genesis_block = client.hash(0).unwrap().unwrap();
	let nonce = 0;
	let period = cumulus_test_runtime::BlockHashCount::get()
		.checked_next_power_of_two()
		.map(|c| c / 2)
		.unwrap_or(2) as u64;
	let tip = 0;
	let extra: cumulus_test_runtime::SignedExtra = (
		frame_system::CheckSpecVersion::<cumulus_test_runtime::Runtime>::new(),
		frame_system::CheckGenesis::<cumulus_test_runtime::Runtime>::new(),
		frame_system::CheckEra::<cumulus_test_runtime::Runtime>::from(sp_runtime::generic::Era::mortal(period, current_block)),
		frame_system::CheckNonce::<cumulus_test_runtime::Runtime>::from(nonce),
		frame_system::CheckWeight::<cumulus_test_runtime::Runtime>::new(),
		pallet_transaction_payment::ChargeTransactionPayment::<cumulus_test_runtime::Runtime>::from(tip),
	);
	let raw_payload = cumulus_test_runtime::SignedPayload::from_raw(
		function.clone(),
		extra.clone(),
		(
			cumulus_test_runtime::VERSION.spec_version,
			genesis_block,
			current_block_hash,
			(),
			(),
			(),
		),
	);
	let signature = raw_payload.using_encoded(|e| caller.sign(e));
	let extrinsic = cumulus_test_runtime::UncheckedExtrinsic::new_signed(
		function.clone(),
		//cumulus_test_runtime::Address::Id(caller.public().into()),
		caller.public().into(),
		//cumulus_primitives::v0::Signature::Sr25519(signature.clone()),
		test_primitives::Signature::Sr25519(signature.clone()),
		extra.clone(),
	);

	builder.push(extrinsic.into()).expect("Push VFP");
	*/
	let mut inherent_data = sp_consensus::InherentData::new();
	inherent_data.put_data(
		cumulus_primitives::inherents::VALIDATION_FUNCTION_PARAMS_IDENTIFIER,
		&cumulus_primitives::validation_function_params::ValidationFunctionParams::default(),
	).expect("Put validation function params failed");
	let timestamp = cumulus_test_runtime::MinimumPeriod::get();
	inherent_data.put_data(sp_timestamp::INHERENT_IDENTIFIER, &timestamp)
		.expect("Put timestamp failed");
	builder.create_inherents(inherent_data).expect("create inherents");

	extrinsics
		.into_iter()
		.for_each(|e| builder.push(e).expect("Pushes an extrinsic"));

	let built_block = builder.build().expect("Creates block");

	(
		built_block.block,
		built_block
			.proof
			.expect("We enabled proof recording before."),
	)
}

#[test]
fn validate_block_with_no_extrinsics() {
	let (client, longest_chain) = create_test_client();
	let parent_head = longest_chain.best_chain().expect("Best block exists");
	let (block, witness_data) = build_block_with_proof(&client, Vec::new());
	let (header, extrinsics) = block.deconstruct();

	let block_data = ParachainBlockData::new(header.clone(), extrinsics, witness_data);

	let res_header = call_validate_block(parent_head, block_data).expect("Calls `validate_block`");
	assert_eq!(header, res_header);
}

#[test]
#[ignore]
fn validate_block_with_extrinsics() {
	let (client, longest_chain) = create_test_client();
	let parent_head = longest_chain.best_chain().expect("Best block exists");
	let (block, witness_data) = build_block_with_proof(&client, create_extrinsics());
	let (header, extrinsics) = block.deconstruct();

	let block_data = ParachainBlockData::new(header.clone(), extrinsics, witness_data);

	let res_header = call_validate_block(parent_head, block_data).expect("Calls `validate_block`");
	assert_eq!(header, res_header);
}

#[test]
#[ignore]
#[should_panic]
fn validate_block_invalid_parent_hash() {
	let (client, longest_chain) = create_test_client();
	let parent_head = longest_chain.best_chain().expect("Best block exists");
	let (block, witness_data) = build_block_with_proof(&client, Vec::new());
	let (mut header, extrinsics) = block.deconstruct();
	header.set_parent_hash(Hash::from_low_u64_be(1));

	let block_data = ParachainBlockData::new(header, extrinsics, witness_data);
	call_validate_block(parent_head, block_data).expect("Calls `validate_block`");
}
