// Copyright 2021 Parity Technologies (UK) Ltd.
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

//! Parachain-specific RPCs implementation.

#![warn(missing_docs)]

use std::sync::Arc;

use sc_client_api::AuxStore;
pub use sc_rpc::{DenyUnsafe, SubscriptionTaskExecutor};
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

use pallet_encointer_bazaar_rpc::{Bazaar, BazaarApi};
use pallet_encointer_ceremonies_rpc::{Ceremonies, CeremoniesApi};
use pallet_encointer_communities_rpc::{Communities, CommunitiesApi};
use parachains_common::{AccountId, Balance, Block, BlockNumber, Index as Nonce};

/// A type representing all RPC extensions.
pub type RpcExtension = jsonrpc_core::IoHandler<sc_rpc::Metadata>;

/// Full client dependencies
pub struct FullDeps<C, P, Backend> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// backend instance used to access offchain storage (added by encointer).
	pub backend: Arc<Backend>,
	/// whether offchain-indexing is enabled (added by encointer).
	pub offchain_indexing_enabled: bool,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
}

/// Instantiate all RPC extensions.
pub fn create_full<C, P, TBackend>(deps: FullDeps<C, P, TBackend>) -> RpcExtension
where
	C: ProvideRuntimeApi<Block>
		+ HeaderBackend<Block>
		+ AuxStore
		+ HeaderMetadata<Block, Error = BlockChainError>
		+ Send
		+ Sync
		+ 'static,
	C::Api: frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: BlockBuilder<Block>,
	P: TransactionPool + Sync + Send + 'static,
	C::Api: pallet_encointer_ceremonies_rpc_runtime_api::CeremoniesApi<Block, AccountId>,
	C::Api:
		pallet_encointer_communities_rpc_runtime_api::CommunitiesApi<Block, AccountId, BlockNumber>,
	C::Api: pallet_encointer_bazaar_rpc_runtime_api::BazaarApi<Block, AccountId>,
	TBackend: sc_client_api::Backend<Block>, // added by encointer
	<TBackend as sc_client_api::Backend<Block>>::OffchainStorage: 'static, // added by encointer
{
	use frame_rpc_system::{FullSystem, SystemApi};
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};

	let mut io = jsonrpc_core::IoHandler::default();
	let FullDeps { client, pool, backend, offchain_indexing_enabled, deny_unsafe } = deps;

	io.extend_with(SystemApi::to_delegate(FullSystem::new(client.clone(), pool, deny_unsafe)));
	io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(client.clone())));

	io.extend_with(BazaarApi::to_delegate(Bazaar::new(client.clone(), deny_unsafe)));

	match backend.offchain_storage() {
		Some(storage) => {
			io.extend_with(CommunitiesApi::to_delegate(Communities::new(
				client.clone(),
				storage.clone(),
				offchain_indexing_enabled,
				deny_unsafe,
			)));
			io.extend_with(CeremoniesApi::to_delegate(Ceremonies::new(
				client.clone(),
				deny_unsafe,
				storage,
				offchain_indexing_enabled,
			)))
		},
		None => log::warn!(
			"Offchain caching disabled, due to lack of offchain storage support in backend."
		),
	};

	io
}

/// Instantiate reduced RPC extensions for launch runtime
pub fn create_launch_ext<C, P, TBackend>(deps: FullDeps<C, P, TBackend>) -> RpcExtension
where
	C: ProvideRuntimeApi<Block>
		+ HeaderBackend<Block>
		+ AuxStore
		+ HeaderMetadata<Block, Error = BlockChainError>
		+ Send
		+ Sync
		+ 'static,
	C::Api: frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: BlockBuilder<Block>,
	P: TransactionPool + Sync + Send + 'static,
{
	use frame_rpc_system::{FullSystem, SystemApi};
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};

	let mut io = jsonrpc_core::IoHandler::default();
	let FullDeps { client, pool, backend: _, offchain_indexing_enabled: _, deny_unsafe } = deps;

	io.extend_with(SystemApi::to_delegate(FullSystem::new(client.clone(), pool, deny_unsafe)));
	io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(client.clone())));

	io
}
