// use std::sync::Arc;
//
// use codec::Codec;
// use jsonrpc_core::{ErrorCode, Result, Error as RpcError};
// use jsonrpc_derive::rpc;
// use sp_api::ProvideRuntimeApi;
// use sp_blockchain::HeaderBackend;
// use sp_runtime::{
//     generic::BlockId,
//     traits::Block as BlockT,
// };
//
// pub use pallet_floating_rate_lend_rpc_runtime_api::LendingApi as LendingRuntimeApi;
// pub use pallet_floating_rate_lend_rpc_runtime_api::{SeqNumInfo, BalanceInfo, UserBalanceInfo};
// use sp_runtime::traits::{MaybeDisplay, MaybeFromStr};
// use std::convert::TryFrom;
// use sp_core::U256;
//
// #[rpc]
// pub trait LendingApi<BlockHash, PoolId, FixedU128, AccountId>
// {
//     #[rpc(name = "lending_supplyRate")]
//     fn supply_rate(
//         &self,
//         id: PoolId,
//         at: Option<BlockHash>
//     ) -> Result<FixedU128>;
//
//     #[rpc(name = "lending_debtRate")]
//     fn debt_rate(
//         &self,
//         id: PoolId,
//         at: Option<BlockHash>
//     ) -> Result<FixedU128>;
//
//     #[rpc(name = "lending_getUserInfo")]
//     fn user_balances(
//         &self,
//         user: AccountId,
//         at: Option<BlockHash>
//     ) -> Result<UserBalanceInfo<FixedU128>>;
//
//     #[rpc(name = "lending_getUserDebtWithInterest")]
//     fn user_debt_balance(
//         &self,
//         asset_id: PoolId,
//         user: AccountId,
//         at: Option<BlockHash>
//     ) -> Result<BalanceInfo<FixedU128>>;
//
//     #[rpc(name = "lending_getUserSupplyWithInterest")]
//     fn user_supply_balance(
//         &self,
//         asset_id: PoolId,
//         user: AccountId,
//         at: Option<BlockHash>
//     ) -> Result<BalanceInfo<FixedU128>>;
//
// }
//
// /// A struct that implements the `SumStorageApi`.
// pub struct Lending<C, M> {
//     client: Arc<C>,
//     _marker: std::marker::PhantomData<M>,
// }
//
// impl<C, M> Lending<C, M> {
//     /// Create new `SumStorage` instance with the given reference to the client.
//     pub fn new(client: Arc<C>) -> Self {
//         Self { client, _marker: Default::default() }
//     }
// }
//
// impl<C, Block, PoolId, FixedU128, AccountId> LendingApi<<Block as BlockT>::Hash, PoolId, FixedU128, AccountId>
// for Lending<C, Block>
//     where
//         Block: BlockT,
//         C: Send + Sync + 'static,
//         C: ProvideRuntimeApi<Block>,
//         C: HeaderBackend<Block>,
//         C::Api: LendingRuntimeApi<Block, PoolId, FixedU128, AccountId>,
//         PoolId: Codec,
//         FixedU128: Codec,
//         AccountId: Codec,
// {
//     fn supply_rate(
//         &self,
//         id: PoolId,
//         at: Option<<Block as BlockT>::Hash>
//     ) -> Result<FixedU128> {
//         let api = self.client.runtime_api();
//         let at = BlockId::hash(at.unwrap_or_else(||
//             // If the block hash is not supplied assume the best block.
//             self.client.info().best_hash
//         ));
//
//         let runtime_api_result = api.supply_rate(&at, id);
//         runtime_api_result.map_err(|e| RpcError {
//             code: ErrorCode::ServerError(9876), // No real reason for this value
//             message: "Something wrong".into(),
//             data: Some(format!("{:?}", e).into()),
//         })
//     }
//
//     fn debt_rate(
//         &self,
//         id: PoolId,
//         at: Option<<Block as BlockT>::Hash>
//     ) -> Result<FixedU128> {
//         let api = self.client.runtime_api();
//         let at = BlockId::hash(at.unwrap_or_else(||
//             // If the block hash is not supplied assume the best block.
//             self.client.info().best_hash
//         ));
//
//         let runtime_api_result = api.debt_rate(&at, id);
//         runtime_api_result.map_err(|e| RpcError {
//             code: ErrorCode::ServerError(9876), // No real reason for this value
//             message: "Something wrong".into(),
//             data: Some(format!("{:?}", e).into()),
//         })
//     }
//
//     fn user_balances(
//         &self,
//         user: AccountId,
//         at: Option<<Block as BlockT>::Hash>
//     ) -> Result<UserBalanceInfo<FixedU128>> {
//         let api = self.client.runtime_api();
//         let at = BlockId::hash(at.unwrap_or_else(||
//             // If the block hash is not supplied assume the best block.
//             self.client.info().best_hash
//         ));
//         let runtime_api_result = api.user_balances(&at, user);
//
//         runtime_api_result.map_err(|e| RpcError {
//             code: ErrorCode::ServerError(9876), // No real reason for this value
//             message: "Something wrong".into(),
//             data: Some(format!("{:?}", e).into()),
//         })
//     }
//
//     fn user_debt_balance(
//         &self,
//         asset_id: PoolId,
//         user: AccountId,
//         at: Option<<Block as BlockT>::Hash>
//     ) -> Result<BalanceInfo<u128>> {
//         let api = self.client.runtime_api();
//         let at = BlockId::hash(at.unwrap_or_else(||
//             // If the block hash is not supplied assume the best block.
//             self.client.info().best_hash
//         ));
//
//         let runtime_api_result = api.user_debt_balance(&at, asset_id, user);
//         runtime_api_result.map_err(|e| RpcError {
//             code: ErrorCode::ServerError(9876), // No real reason for this value
//             message: "Something wrong".into(),
//             data: Some(format!("{:?}", e).into()),
//         })
//     }
//
//     fn user_supply_balance(
//         &self,
//         asset_id: PoolId,
//         user: AccountId,
//         at: Option<<Block as BlockT>::Hash>
//     ) -> Result<BalanceInfo<FixedU128>> {
//         let api = self.client.runtime_api();
//         let at = BlockId::hash(at.unwrap_or_else(||
//             // If the block hash is not supplied assume the best block.
//             self.client.info().best_hash
//         ));
//
//         let runtime_api_result = api.user_supply_balance(&at, asset_id, user);
//         runtime_api_result.map_err(|e| RpcError {
//             code: ErrorCode::ServerError(9876), // No real reason for this value
//             message: "Something wrong".into(),
//             data: Some(format!("{:?}", e).into()),
//         })
//     }
// }