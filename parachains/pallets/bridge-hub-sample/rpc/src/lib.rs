// Copyright 2020-2021 Parity Technologies (UK) Ltd.
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

//! RPC interface for the transaction payment pallet.
use std::sync::Arc;

use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

pub use pallet_bridge_hub_sample::ActualData;
pub use pallet_bridge_hub_sample_runtime_api::BridgeHubRuntimeApi;

#[rpc(client, server)]
pub trait BridgeHubRpcApi<BlockHash, ResponseType> {
	#[method(name = "bridgehub_getActualData")]
	fn get_actual_data(&self, at: Option<BlockHash>) -> RpcResult<ResponseType>;
}

/// Provides RPC methods to query a dispatchable's class, weight and fee.
pub struct BridgeHubRpc<C, P> {
	/// Shared reference to the client.
	client: Arc<C>,
	_marker: std::marker::PhantomData<P>,
}

impl<C, P> BridgeHubRpc<C, P> {
	/// Creates a new instance of the BridgeHubRpc helper.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

#[async_trait]
impl<C, Block> BridgeHubRpcApiServer<<Block as BlockT>::Hash, ActualData> for BridgeHubRpc<C, Block>
where
	Block: BlockT,
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
	C::Api: BridgeHubRuntimeApi<Block>,
{
	fn get_actual_data(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<ActualData> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		api.get_actual_data(&at).map_err(|e| {
			CallError::Custom(ErrorObject::owned(
				-1234,
				"Unable to get actual data.",
				Some(e.to_string()),
			))
			.into()
		})
	}
}
