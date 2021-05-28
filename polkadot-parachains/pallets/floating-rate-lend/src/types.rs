use codec::{Decode, Encode};
use frame_support::{sp_runtime::FixedU128};
use frame_support::dispatch::DispatchResultWithPostInfo;
use sp_runtime::{FixedPointNumber, RuntimeDebug};
use sp_runtime::traits::{CheckedMul, One};
use sp_runtime::traits::{Saturating, Zero};
use sp_std::marker;
use sp_std::ops::{Add, Div, Mul, Sub};

use pallet_traits::PriceProvider;
use polkadot_parachain_primitives::FlowError;
use polkadot_parachain_primitives::{Overflown, PoolId};

use crate::{Config, PoolStorage, PoolUserDebts, PoolUserSupplies, UserDebtSet, UserSupplySet};
use crate::pool::Pool;

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
    amount: FixedU128,
    index: FixedU128,
}

impl UserData {
    pub fn accrue_interest(&mut self, pool_index: &FixedU128) -> Result<(), FlowError> {
        // TODO: maybe should consider throw an error here
        if pool_index.le(&self.index) { return Ok(()); }
        self.amount = pool_index
            .div(self.index)
            .checked_mul(&self.amount)
            .ok_or(FlowError{})?;
        self.index = *pool_index;
        Ok(())
    }

    pub fn increment(&mut self, amount: &FixedU128) {
        self.amount = self.amount.add(*amount);
    }

    pub fn decrement(&mut self, amount: &FixedU128) {
        self.amount = self.amount.sub(*amount);
    }

    pub fn amount(&self) -> FixedU128 { self.amount }

    pub fn index(&self) -> FixedU128 { self.index }

    pub fn new(amount: FixedU128) -> Self { UserData {amount, index: FixedU128::one()} }
}

pub const BLOCKS_IN_YEAR: u128 = 365 * 24 * 3600 / 6;

pub struct Convertor;

impl Convertor {
    pub fn convert_percentage_annum_to_per_block(a: u64) -> FixedU128 {
        Self::convert_percentage(a).div(FixedU128::from(BLOCKS_IN_YEAR))
    }

    pub fn convert_percentage(a: u64) -> FixedU128 {
        FixedU128::saturating_from_rational(a, 100)
    }
}

/// The util functions for user accounts
pub struct UserAccountUtil<T>(marker::PhantomData<T>);

impl <T: Config> UserAccountUtil<T> {
    /// Increment user supply.
    /// Ensure the amount is above pool.minimal_amount() before this function is actually called
    pub fn increment_user_supply(pool: &Pool<T>, account: T::AccountId, amount: FixedU128) -> DispatchResultWithPostInfo {
        if let Some(mut user_supply) = PoolUserSupplies::<T>::get(pool.id, account.clone()) {
            user_supply.accrue_interest(&pool.total_supply_index())?;
            user_supply.increment(&amount);
            PoolUserSupplies::<T>::insert(pool.id, account, user_supply);
        } else {
            Self::add_user_supply(&pool, account, amount);
        }

        Ok(().into())
    }

    /// Add to user supplies.
    /// This function contains write action, ensure the checks are performed in advance
    fn add_user_supply(pool: &Pool<T>, account: T::AccountId, amount: FixedU128) {
        let user_supply = UserData::new(amount);
        PoolUserSupplies::<T>::insert(pool.id, account.clone(), user_supply);

        // update user's supply asset set
        let mut pool_currency_tuples = UserSupplySet::<T>::get(account.clone());
        if !pool_currency_tuples.iter().any(|p| p.0 == pool.id) {
            pool_currency_tuples.push((pool.id, pool.currency_id));
            UserSupplySet::<T>::insert(account, pool_currency_tuples);
        }
    }


    /// Remove to user supplies.
    /// This function contains write action, ensure the checks are performed in advance
    pub fn remove_user_supply(pool_id: PoolId, account: T::AccountId) {
        let mut pool_currency_tuples = UserSupplySet::<T>::get(account.clone());
        pool_currency_tuples.retain(|p| p.0 != pool_id);
        UserSupplySet::<T>::insert(account, pool_currency_tuples);
    }

    /// Add to user debts.
    /// This function contains write action, ensure the checks are performed in advance
    pub fn add_user_debt(pool: &Pool<T>, account: T::AccountId, amount: FixedU128) {
        let user_debt = UserData::new(amount);
        PoolUserDebts::<T>::insert(pool.id, account.clone(), user_debt);

        // update user's supply asset set
        let mut pool_currency_tuples = UserDebtSet::<T>::get(account.clone());
        if !pool_currency_tuples.iter().any(|p| p.0 == pool.id) {
            pool_currency_tuples.push((pool.id, pool.currency_id));
            UserDebtSet::<T>::insert(account, pool_currency_tuples);
        }
    }

    /// Remove to user debts.
    /// This function contains write action, ensure the checks are performed in advance
    pub fn remove_user_debt(pool_id: PoolId, account: T::AccountId) {
        let mut pool_currency_tuples = UserDebtSet::<T>::get(account.clone());
        pool_currency_tuples.retain(|p| p.0 != pool_id);
        UserDebtSet::<T>::insert(account, pool_currency_tuples);
    }

    /// Get the user supply balance for the user in a pool
    pub fn user_supply_balance(pool_id: PoolId, user: T::AccountId) -> Result<FixedU128, Overflown> {
        if let Some(user_supply) = PoolUserSupplies::<T>::get(pool_id, user) {
            if let Some(mut pool) = PoolStorage::<T>::get(pool_id) {
                pool.accrue_interest(<frame_system::Pallet<T>>::block_number())?;
                return Ok(
                    pool.total_supply_index()
                        .mul(user_supply.index())
                        .mul(user_supply.amount())
                );
            }
        }
        Ok(FixedU128::zero())
    }

    /// Get the user debt balance for the user in a pool
    pub fn user_debt_balance(pool_id: PoolId, user: T::AccountId) -> Result<FixedU128, Overflown> {
        if let Some(user_debt) = PoolUserDebts::<T>::get(pool_id, user) {
            if let Some(mut pool) = PoolStorage::<T>::get(pool_id) {
                pool.accrue_interest(<frame_system::Pallet<T>>::block_number())?;
                return Ok(
                    pool.total_debt_index()
                        .mul(user_debt.index())
                        .mul(user_debt.amount())
                );
            }
        }
        Ok(FixedU128::zero())
    }

    // total supply balance; total converted supply balance; total debt balance;
    pub fn user_balances(user: T::AccountId) -> Result<UserBalanceStats, Overflown> {
        let mut supply_balance = FixedU128::zero();
        let mut collateral_balance = FixedU128::zero();

        for (pool_id, currency_id) in UserSupplySet::<T>::get(user.clone()).into_iter() {
            let pool = PoolStorage::<T>::get(pool_id).unwrap();
            if !pool.can_be_collateral { continue; }
            let price = T::PriceProvider::price(currency_id);
            // TODO: throw error
            if !price.price_ready() { continue; }

            let amount = Self::user_supply_balance(pool_id, user.clone())?;
            supply_balance = supply_balance.add(price.value().saturating_mul(amount));

            // TODO: consider using some sort of cache?
            let delta = (price.value() * pool.safe_factor).saturating_mul(amount);
            collateral_balance = collateral_balance.add(delta);
        }

        let mut debt_balance = FixedU128::zero();
        for (pool_id, currency_id) in UserDebtSet::<T>::get(user.clone()).into_iter() {
            let amount = Self::user_debt_balance(pool_id, user.clone())?;
            let price = T::PriceProvider::price(currency_id);
            if !price.price_ready() {
                // TODO: throw error
                continue
            }
            debt_balance = debt_balance.add(price.value().mul(amount));
        }
        Ok(UserBalanceStats {
            supply_balance,
            collateral_balance,
            debt_balance,
        })
    }
}