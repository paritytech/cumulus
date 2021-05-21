#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;

mod types;

pub use types::{ UserBalanceInfo, BalanceInfo, SeqNumInfo };
use sp_runtime::traits::{MaybeDisplay, MaybeFromStr};

sp_api::decl_runtime_apis! {
    pub trait LendingApi<PoolId, FixedU128, AccountId, Balance> where
        PoolId: Codec,
        FixedU128: Codec,
        AccountId: Codec,
        Balance: Codec  + MaybeDisplay + MaybeFromStr,
    {
        fn supply_rate(id: PoolId) -> FixedU128;

        fn debt_rate(id: PoolId) -> FixedU128;

        // effective supply balance; borrow balance
        fn user_balances(user: AccountId) -> UserBalanceInfo<Balance>;

        fn user_debt_balance(pool_id: PoolId, user: AccountId) -> BalanceInfo<Balance>;

        fn user_supply_balance(pool_id: PoolId, user: AccountId) -> BalanceInfo<Balance>;

    }
}
