use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use frame_support::{sp_runtime::{FixedU128}};
use sp_std::{vec::Vec};
use polkadot_parachain_primitives::{CurrencyId, PoolId, Overflown, PriceValue, FlowError};
use frame_support::sp_runtime::{FixedPointNumber};
use sp_runtime::traits::{CheckedMul, CheckedAdd, One, Zero, CheckedDiv};
use sp_runtime::sp_std::convert::TryInto;
use sp_std::ops::{Add, Sub, Mul};

pub struct UserBalanceStats{
    /// The total supply balance of the user
    pub supply_balance: FixedU128,
    /// The total collateral balance of the user
    pub collateral_balance: FixedU128,
    /// The total debt balance of the user
    pub debt_balance: FixedU128,
}

impl UserBalanceStats {
    pub fn is_liquidated(&self, liquidation_threshold: FixedU128) -> bool {
        self.collateral_balance < liquidation_threshold * self.debt_balance
    }
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode)]
pub struct UserData {
    pub amount: FixedU128,
    pub index: FixedU128,
}

impl UserData {
    pub fn accrue_interest(&mut self, pool_index: &FixedU128) -> Result<(), FlowError> {
        self.amount = pool_index
            .checked_div(&self.index)
            .ok_or(FlowError{})?
            .checked_mul_int(self.amount)
            .ok_or(FlowError{})?;
        self.index = *pool_index;
        Ok(())
    }

    pub fn increment(&mut self, amount: &FixedU128) {
        self.amount.add(*amount);
    }

    pub fn decrement(&mut self, amount: &FixedU128) {
        self.amount.sub(*amount);
    }

    pub fn amount(&self) -> FixedU128 { self.amount }

    pub fn new(amount: FixedU128) -> Self { UserData {amount, index: FixedU128::one()} }
}