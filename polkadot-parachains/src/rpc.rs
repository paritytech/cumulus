// //! A collection of node-specific RPC methods.
// //! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
// //! used by Substrate nodes. This file extends those RPC definitions with
// //! capabilities that are specific to this project's runtime configuration.
//
// #![warn(missing_docs)]
//
// use std::sync::Arc;
//
// use polkadot_parachain_primitives::{PoolId};
// use rococo_parachain_runtime::{Block, AccountId, Balance };
// use sp_api::ProvideRuntimeApi;
// use sp_blockchain::{Error as BlockChainError, HeaderMetadata, HeaderBackend};
// use sp_block_builder::BlockBuilder;
// use sp_transaction_pool::TransactionPool;
// use sp_runtime::FixedU128;
// use sc_rpc_api::DenyUnsafe;
//
//
// /// Full client dependencies.
// pub struct FullDeps<C> {
// 	/// The client instance to use.
// 	pub client: Arc<C>,
// }
//
// /// Instantiate all full RPC extensions.
// pub fn create_full<C>(
// 	deps: FullDeps<C>,
// ) -> jsonrpc_core::IoHandler<sc_rpc::Metadata> where
// 	C: ProvideRuntimeApi<Block>,
// 	C: HeaderBackend<Block>,
// 	C: Send + Sync + 'static,
// 	C::Api: pallet_floating_rate_lend_rpc::LendingRuntimeApi<Block, PoolId, FixedU128, AccountId, Balance>,
// 	C::Api: BlockBuilder<Block>,
// {
// 	use pallet_floating_rate_lend_rpc::{Lending as FloatingRateLending, LendingApi as FloatingRateLendingApi};
//
// 	let mut io = jsonrpc_core::IoHandler::default();
// 	let FullDeps {
// 		client,
// 	} = deps;
//
// 	io.extend_with(
// 		FloatingRateLendingApi::to_delegate(FloatingRateLending::new(client))
// 	);
//
// 	io
// }
