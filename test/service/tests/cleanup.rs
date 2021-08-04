// Copyright 2020-2021 Parity Technologies (UK) Ltd.
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
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

use cumulus_primitives_core::ParaId;
use cumulus_test_service::{initial_head_data, run_relay_chain_validator_node, Keyring::*};
use futures::join;
use sc_service::TaskExecutor;
use cumulus_test_client::{InitBlockBuilder, ClientBlockImportExt};
use substrate_test_client::sp_consensus::BlockOrigin;
use sp_runtime::generic::BlockId;

#[substrate_test_utils::test]
async fn cleanup(task_executor: TaskExecutor) {
	let mut builder = sc_cli::LoggerBuilder::new("");
	builder.with_colors(false);
	let _ = builder.init();

	let para_id = ParaId::from(100);

	// Build a node.
	let node = cumulus_test_service::TestNodeBuilder::new(para_id, task_executor.clone(), Charlie)
		.enable_collator()
		.build()
		.await;

	let mut client = node.client;

	for i in 0..100u32 {
		eprintln!("{}", i);

		let mut block_builder = client.init_block_builder_at(&BlockId::number(0), None, Default::default());

		block_builder.push(cumulus_test_client::transfer(&*client, 0, Alice, Bob, i.into())).unwrap();

		let res = block_builder.build().unwrap();

		client.import(BlockOrigin::Own, res.block).await.unwrap();
	}
}
