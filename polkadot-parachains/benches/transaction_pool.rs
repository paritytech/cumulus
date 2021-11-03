// This file is part of Substrate.

// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::sync::Arc;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use frame_benchmarking::frame_support::{sp_runtime::generic, sp_tracing};
use frame_system_rpc_runtime_api::AccountNonceApi;
use futures::{future, join, StreamExt};
use node_primitives::AccountId;
use node_runtime::{constants::currency::*, BalancesCall, SudoCall};
use polkadot_service::{polkadot_runtime::constants::currency::DOLLARS, ProvideRuntimeApi};
use sc_client_api::{execution_extensions::ExecutionStrategies, BlockBackend};
use sc_executor::NativeElseWasmExecutor;
use sc_service::{
	config::{
		DatabaseSource, KeepBlocks, KeystoreConfig, NetworkConfiguration, OffchainWorkerConfig,
		PruningMode, TransactionPoolOptions, TransactionStorageMode, WasmExecutionMethod,
	},
	BasePath, Configuration, Role,
};
use sc_transaction_pool::PoolLimit;
use sc_transaction_pool_api::{TransactionPool as _, TransactionSource, TransactionStatus};
use sp_core::{crypto::Pair, sr25519, Encode};
use sp_keyring::Sr25519Keyring;
use sp_runtime::{generic::BlockId, OpaqueExtrinsic, SaturatedConversion};

use cumulus_primitives_core::{relay_chain::Block, ParaId};
use cumulus_test_runtime::RuntimeApi;
use cumulus_test_service::{
	initial_head_data, Client, Keyring::*, RuntimeExecutor, TransactionPool,
};

pub fn fetch_nonce(client: &Client, account: sp_core::sr25519::Pair) -> u32 {
	let best_hash = client.chain_info().best_hash;
	client
		.runtime_api()
		.account_nonce(&generic::BlockId::Hash(best_hash), account.public().into())
		.expect("Fetching account nonce works; qed")
}

pub fn create_extrinsic(
	client: &Client,
	sender: sp_core::sr25519::Pair,
	function: impl Into<node_runtime::Call>,
	nonce: Option<u32>,
) -> node_runtime::UncheckedExtrinsic {
	let function = function.into();
	let genesis_hash = client.block_hash(0).ok().flatten().expect("Genesis block exists; qed");
	let best_hash = client.chain_info().best_hash;
	let best_block = client.chain_info().best_number;
	let nonce = nonce.unwrap_or_else(|| fetch_nonce(client, sender.clone()));

	let period = node_runtime::BlockHashCount::get()
		.checked_next_power_of_two()
		.map(|c| c / 2)
		.unwrap_or(2) as u64;
	let tip = 0;
	let extra: node_runtime::SignedExtra = (
		frame_system::CheckSpecVersion::<node_runtime::Runtime>::new(),
		frame_system::CheckTxVersion::<node_runtime::Runtime>::new(),
		frame_system::CheckGenesis::<node_runtime::Runtime>::new(),
		frame_system::CheckEra::<node_runtime::Runtime>::from(generic::Era::mortal(
			period,
			best_block.saturated_into(),
		)),
		frame_system::CheckNonce::<node_runtime::Runtime>::from(nonce),
		frame_system::CheckWeight::<node_runtime::Runtime>::new(),
		pallet_transaction_payment::ChargeTransactionPayment::<node_runtime::Runtime>::from(tip),
	);

	let raw_payload = node_runtime::SignedPayload::from_raw(
		function.clone(),
		extra.clone(),
		(
			node_runtime::VERSION.spec_version,
			node_runtime::VERSION.transaction_version,
			genesis_hash,
			best_hash,
			(),
			(),
			(),
		),
	);
	let signature = raw_payload.using_encoded(|e| sender.sign(e));

	node_runtime::UncheckedExtrinsic::new_signed(
		function.clone(),
		sp_runtime::AccountId32::from(sender.public()).into(),
		node_runtime::Signature::Sr25519(signature.clone()),
		extra.clone(),
	)
}

fn create_accounts(num: usize) -> Vec<sr25519::Pair> {
	(0..num)
		.map(|i| {
			Pair::from_string(&format!("{}/{}", Sr25519Keyring::Alice.to_seed(), i), None)
				.expect("Creates account pair")
		})
		.collect()
}

/// Create the extrinsics that will initialize the accounts from the sudo account (Alice).
///
/// `start_nonce` is the current nonce of Alice.
fn create_account_extrinsics(client: &Client, accounts: &[sr25519::Pair]) -> Vec<OpaqueExtrinsic> {
	let start_nonce = fetch_nonce(client, Sr25519Keyring::Alice.pair());

	accounts
		.iter()
		.enumerate()
		.map(|(i, a)| {
			vec![
				// Reset the nonce by removing any funds
				create_extrinsic(
					client,
					Sr25519Keyring::Alice.pair(),
					SudoCall::sudo {
						call: Box::new(
							BalancesCall::set_balance {
								who: AccountId::from(a.public()).into(),
								new_free: 0,
								new_reserved: 0,
							}
							.into(),
						),
					},
					Some(start_nonce + (i as u32) * 2),
				),
				// Give back funds
				create_extrinsic(
					client,
					Sr25519Keyring::Alice.pair(),
					SudoCall::sudo {
						call: Box::new(
							BalancesCall::set_balance {
								who: AccountId::from(a.public()).into(),
								new_free: 1_000_000 * DOLLARS,
								new_reserved: 0,
							}
							.into(),
						),
					},
					Some(start_nonce + (i as u32) * 2 + 1),
				),
			]
		})
		.flatten()
		.map(OpaqueExtrinsic::from)
		.collect()
}

fn create_benchmark_extrinsics(
	client: &Client,
	accounts: &[sr25519::Pair],
	extrinsics_per_account: usize,
) -> Vec<OpaqueExtrinsic> {
	accounts
		.iter()
		.map(|account| {
			(0..extrinsics_per_account).map(move |nonce| {
				create_extrinsic(
					client,
					account.clone(),
					BalancesCall::transfer {
						dest: Sr25519Keyring::Bob.to_account_id().into(),
						value: 1 * DOLLARS,
					},
					Some(nonce as u32),
				)
			})
		})
		.flatten()
		.map(OpaqueExtrinsic::from)
		.collect()
}

async fn submit_tx_and_wait_for_inclusion(
	tx_pool: &TransactionPool,
	tx: OpaqueExtrinsic,
	client: &Client,
	wait_for_finalized: bool,
) {
	let best_hash = client.chain_info().best_hash;

	let mut watch = tx_pool
		.submit_and_watch(&BlockId::Hash(best_hash), TransactionSource::External, tx.clone())
		.await
		.expect("Submits tx to pool")
		.fuse();

	loop {
		match watch.select_next_some().await {
			TransactionStatus::Finalized(_) => break,
			TransactionStatus::InBlock(_) if !wait_for_finalized => break,
			_ => {},
		}
	}
}

fn transaction_pool_benchmarks(c: &mut Criterion) {
	// sp_tracing::try_init_simple();
	let mut builder = sc_cli::LoggerBuilder::new("");
	builder.with_colors(false);
	let _ = builder.init();

	let para_id = ParaId::from(100);
	let runtime = tokio::runtime::Runtime::new().expect("Creates tokio runtime");
	let tokio_handle = runtime.handle().clone();

	// Start alice
	let alice = cumulus_test_service::run_relay_chain_validator_node(
		tokio_handle.clone(),
		Alice,
		|| {},
		vec![],
	);

	// Start bob
	let bob = cumulus_test_service::run_relay_chain_validator_node(
		tokio_handle.clone(),
		Bob,
		|| {},
		vec![alice.addr.clone()],
	);

	// Register parachain
	runtime
		.block_on(
			alice.register_parachain(
				para_id,
				cumulus_test_service::runtime::WASM_BINARY
					.expect("You need to build the WASM binary to run this test!")
					.to_vec(),
				initial_head_data(para_id),
			),
		)
		.unwrap();

	// Run charlie as parachain collator
	let charlie = runtime.block_on(
		cumulus_test_service::TestNodeBuilder::new(para_id, tokio_handle.clone(), Charlie)
			.enable_collator()
			.connect_to_relay_chain_nodes(vec![&alice, &bob])
			// .wrap_announce_block(|_| {
			// 	Never announce any block
			// Arc::new(|_, _| {})
			// })
			.build(),
	);

	// Run dave as parachain full node
	//
	// It will need to recover the pov blocks through availability recovery.
	let dave = Arc::new(
		runtime.block_on(
			cumulus_test_service::TestNodeBuilder::new(para_id, tokio_handle.clone(), Dave)
				.enable_collator()
				// .use_null_consensus()
				.connect_to_parachain_node(&charlie)
				.connect_to_relay_chain_nodes(vec![&alice, &bob])
				.build(),
		),
	);

	runtime.block_on(dave.wait_for_blocks(7));

	let mut group = c.benchmark_group("Transaction pool");
	let account_num = 10;
	let extrinsics_per_account = 20;
	group.sample_size(10);
	group.throughput(Throughput::Elements(account_num as u64 * extrinsics_per_account as u64));

	let handle2 = tokio_handle.clone();
	let accounts = create_accounts(account_num);
	let mut counter = 1;
	group.bench_function(
		format!("{} transfers from {} accounts", account_num * extrinsics_per_account, account_num),
		move |b| {
			b.iter_batched(
				|| {
					let prepare_extrinsics = create_account_extrinsics(&*dave.client, &accounts);

					handle2.block_on(future::join_all(prepare_extrinsics.into_iter().map(|tx| {
						submit_tx_and_wait_for_inclusion(
							&dave.transaction_pool,
							tx,
							&*dave.client,
							true,
						)
					})));

					create_benchmark_extrinsics(&*dave.client, &accounts, extrinsics_per_account)
				},
				|extrinsics| {
					handle2.block_on(future::join_all(extrinsics.into_iter().map(|tx| {
						submit_tx_and_wait_for_inclusion(
							&dave.transaction_pool,
							tx,
							&*dave.client,
							false,
						)
					})));

					println!("Finished {}", counter);
					counter += 1;
				},
				BatchSize::SmallInput,
			)
		},
	);

	runtime.block_on(alice.task_manager.clean_shutdown());
	runtime.block_on(bob.task_manager.clean_shutdown());
	runtime.block_on(charlie.task_manager.clean_shutdown());
	// runtime.block_on(dave.task_manager.clean_shutdown());
}

criterion_group!(benches, transaction_pool_benchmarks);
criterion_main!(benches);
