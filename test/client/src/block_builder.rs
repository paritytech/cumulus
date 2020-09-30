// Copyright 2020 Parity Technologies (UK) Ltd.
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

use crate::Client;
use sc_block_builder::{BlockBuilder, BlockBuilderApi};
use sc_client_api::backend;
use sp_api::{ApiExt, BlockT, ProvideRuntimeApi};
use sp_blockchain::Error;
use sp_core::ExecutionContext;
use sp_runtime::generic::BlockId;

/// Push the inherents to a block so you don't have to.
pub trait PushInherents {
	/// Push the inherents to a block so you don't have to.
	fn cumulus_inherents(&mut self, client: &Client) -> Vec<runtime::UncheckedExtrinsic>;
}

impl<'a, Block, A, B> PushInherents for BlockBuilder<'a, Block, A, B>
where
	Block: BlockT,
	A: ProvideRuntimeApi<Block> + 'a,
	A::Api: BlockBuilderApi<Block, Error = Error>
		+ ApiExt<Block, StateBackend = backend::StateBackendFor<B, Block>>,
	B: backend::Backend<Block>,
{
	fn cumulus_inherents(&mut self, client: &Client) -> Vec<runtime::UncheckedExtrinsic> {
		let mut inherent_data = sp_consensus::InherentData::new();
		let timestamp = runtime::MinimumPeriod::get();

		inherent_data
			.put_data(sp_timestamp::INHERENT_IDENTIFIER, &timestamp)
			.expect("Put timestamp failed");
		inherent_data
			.put_data(
				cumulus_primitives::inherents::VALIDATION_FUNCTION_PARAMS_IDENTIFIER,
				&cumulus_primitives::validation_function_params::ValidationFunctionParams::default(
				),
			)
			.expect("Put validation function params failed");

		client
			.runtime_api()
			.inherent_extrinsics_with_context(
				&BlockId::number(0),
				ExecutionContext::BlockConstruction,
				inherent_data,
			)
			.expect("Get inherents failed")
	}
}
