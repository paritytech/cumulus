#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;

mod types;

pub use types::{ UserBalanceInfo, BalanceInfo, SeqNumInfo };
use sp_runtime::traits::{MaybeDisplay, MaybeFromStr};

sp_api::decl_runtime_apis! {
    pub trait LendingApi<AssetId, FixedU128, AccountId, Balance> where
        AssetId: Codec,
        FixedU128: Codec,
        AccountId: Codec,
        Balance: Codec  + MaybeDisplay + MaybeFromStr,
    {
        fn supply_rate(id: AssetId) -> FixedU128;

        fn debt_rate(id: AssetId) -> FixedU128;

        // effective supply balance; borrow balance
        fn get_user_info(user: AccountId) -> UserBalanceInfo<Balance>;

        fn get_user_debt_with_interest(asset_id: AssetId, user: AccountId) -> BalanceInfo<Balance>;

        fn get_user_supply_with_interest(asset_id: AssetId, user: AccountId) -> BalanceInfo<Balance>;

    }
}
