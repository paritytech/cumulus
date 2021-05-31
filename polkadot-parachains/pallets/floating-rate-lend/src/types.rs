use codec::{Decode, Encode};
use frame_support::{sp_runtime::FixedU128};
use frame_support::dispatch::DispatchResultWithPostInfo;
use sp_runtime::{FixedPointNumber, RuntimeDebug};
use sp_runtime::traits::{CheckedMul, One};
use sp_runtime::traits::{Saturating, Zero};
use sp_std::{vec::Vec};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::marker;
use sp_std::ops::{Add, Div, Mul, Sub};

use polkadot_parachain_primitives::CustomError;
use polkadot_parachain_primitives::PoolId;

use crate::{Config, Error, PoolStorage, PoolUserDebts, PoolUserSupplies, UserDebtSet, UserSupplySet};
use crate::pool::{PoolProxy};

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

    pub fn increment_debt(&mut self, amount: FixedU128) {
        self.debt_balance = self.debt_balance.add(amount);
    }

    pub fn decrement_collateral(&mut self, amount: FixedU128) {
        self.collateral_balance = self.collateral_balance.sub(amount);
    }
}

// TODO: urgent! Added a proxy layer and ensure the account is in the proxy
#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode)]
pub struct UserData {
    amount: FixedU128,
    index: FixedU128,
}

impl UserData {
    pub fn accrue_interest(&mut self, pool_index: &FixedU128) -> Result<(), CustomError> {
        // TODO: maybe should consider throw an error here
        if pool_index.le(&self.index) { return Ok(()); }
        self.amount = pool_index
            .div(self.index)
            .checked_mul(&self.amount)
            .ok_or(CustomError::FlownError)?;
        self.index = *pool_index;
        Ok(())
    }

    pub fn increment(&mut self, amount: &FixedU128) {
        self.amount = self.amount.add(*amount);
    }

    pub fn decrement(&mut self, amount: &FixedU128) {
        self.amount = self.amount.sub(*amount);
    }

    /// Checks if the amount below zero
    pub fn below_zero(&self) -> bool { self.amount.lt(&FixedU128::zero()) }

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
    /// Accrue interest for the user, including all the pools the user has participated in.
    /// Ensure the pool_map contains all the pools the user participated in
    pub fn accrue_interest_for_user(
        account: T::AccountId,
        pool_map: &BTreeMap<PoolId, PoolProxy<T>>,
    ) -> Result<UserSupplyDebtData, CustomError> {
        let mut supply_map = BTreeMap::new();
        for id in Self::get_supply_pools(account.clone()) {
            if let Some(pool) = pool_map.get(&id) {
                if let Some(mut supply) = PoolUserSupplies::<T>::get(pool.id(), account.clone()) {
                    supply.accrue_interest(&pool.total_supply_index())?;
                    PoolUserSupplies::<T>::insert(pool.id(), account.clone(), supply.clone());
                    supply_map.insert(pool.id(), supply);
                    continue;
                }
            }

            // TODO: more context here
            log::error!("Inconsistent state, check!");
            return Err(CustomError::InconsistentState);
        }

        let mut debt_map = BTreeMap::new();
        for id in Self::get_debt_pools(account.clone()) {
            if let Some(pool) = pool_map.get(&id) {
                if let Some(mut debt) = PoolUserDebts::<T>::get(pool.id(), account.clone()) {
                    debt.accrue_interest(&pool.total_debt_index())?;
                    PoolUserDebts::<T>::insert(pool.id(), account.clone(), debt.clone());
                    debt_map.insert(pool.id(), debt);
                    continue;
                }
            }

            // TODO: more context here
            log::error!("Inconsistent state, check!");
            return Err(CustomError::InconsistentState);
        }

        Ok(UserSupplyDebtData{ supply: supply_map, debt: debt_map })
    }

    pub fn get_supply_pools(account: T::AccountId) -> Vec<PoolId> {
        UserSupplySet::<T>::get(account)
            .into_iter()
            .map(|p| p.0)
            .collect::<Vec<PoolId>>()
    }

    pub fn get_debt_pools(account: T::AccountId) -> Vec<PoolId> {
        UserDebtSet::<T>::get(account)
            .into_iter()
            .map(|p| p.0)
            .collect::<Vec<PoolId>>()
    }

    /// Increment user supply.
    /// Ensure the amount is above pool.minimal_amount() before this function is actually called
    pub fn accrue_interest_and_increment_supply(pool: &PoolProxy<T>, account: T::AccountId, amount: &FixedU128) -> DispatchResultWithPostInfo {
        if let Some(mut user_supply) = PoolUserSupplies::<T>::get(pool.id(), account.clone()) {
            user_supply.accrue_interest(&pool.total_supply_index())?;
            user_supply.increment(&amount);
            PoolUserSupplies::<T>::insert(pool.id(), account, user_supply);
        } else {
            Self::add_user_supply(&pool, account, amount);
        }
        Ok(().into())
    }

    /// Decrement user supply.
    /// Ensure the amount is never zero before this function is actually called
    pub fn decrement_supply(pool: &PoolProxy<T>, account: T::AccountId, amount: &FixedU128, mut user_supply: UserData) -> DispatchResultWithPostInfo {
        // this should not have happened, caller should always ensure zero is never reached
        if user_supply.amount() < *amount { return Err(Error::<T>::UserSupplyTooLow.into()); }

        user_supply.decrement(amount);
        // TODO: we need to handle dust here! Currently minimal_amount is just zero
        if user_supply.amount() <= pool.minimal_amount() {
            PoolUserSupplies::<T>::remove(pool.id(), account.clone());
            Self::remove_user_supply(pool.id(), account);
        } else {
            PoolUserSupplies::<T>::insert(pool.id(), account, user_supply);
        }
        Ok(().into())
    }

    /// Increment user debt
    pub fn accrue_interest_and_increment_debt(pool: &PoolProxy<T>, account: T::AccountId, amount: &FixedU128) -> DispatchResultWithPostInfo {
        if let Some(mut user_debt) = PoolUserDebts::<T>::get(pool.id(), account.clone()) {
            user_debt.accrue_interest(&pool.total_debt_index())?;
            user_debt.increment(&amount);
            PoolUserDebts::<T>::insert(pool.id(), account.clone(), user_debt);
        } else {
            Self::new_debt(&pool, account.clone(), &amount);
        }
        Ok(().into())
    }

    /// Get the user supply balance for the user in a pool
    pub fn supply_balance_with_interest(pool_id: PoolId, user: T::AccountId) -> Result<FixedU128, CustomError> {
        if let Some(mut user_supply) = PoolUserSupplies::<T>::get(pool_id, user) {
            if let Some(mut pool) = PoolStorage::<T>::get(pool_id) {
                pool.accrue_interest(<frame_system::Pallet<T>>::block_number())?;
                user_supply.accrue_interest(&pool.total_supply_index())?;
                return Ok(user_supply.amount());
            }
        }
        Ok(FixedU128::zero())
    }

    /// Get the user debt balance for the user in a pool
    pub fn debt_balance_with_interest(pool_id: PoolId, user: T::AccountId) -> Result<FixedU128, CustomError> {
        if let Some(mut user_debt) = PoolUserDebts::<T>::get(pool_id, user) {
            if let Some(mut pool) = PoolStorage::<T>::get(pool_id) {
                pool.accrue_interest(<frame_system::Pallet<T>>::block_number())?;
                user_debt.accrue_interest(&pool.total_debt_index())?;
                return Ok(user_debt.amount());
            }
        }
        Ok(FixedU128::zero())
    }

    /// Calculate the user balances
    pub fn user_balances(
        user_supply_debt: &UserSupplyDebtData,
        pool_map: &BTreeMap<PoolId, PoolProxy<T>>,
    ) -> Result<UserBalanceStats, CustomError> {
        // calculate supply related
        let mut supply_balance = FixedU128::zero();
        let mut collateral_balance = FixedU128::zero();
        for pool_id in user_supply_debt.supply.keys() {
            let pool = pool_map.get(pool_id).ok_or(CustomError::InconsistentState)?;
            if !pool.price_ready() { return Err(CustomError::PriceNotReady); }

            let amount = user_supply_debt.supply
                .get(pool_id)
                .ok_or(CustomError::InconsistentState)?
                .amount();

            let mut balance = pool.price().saturating_mul(amount);
            supply_balance = supply_balance.add(balance);

            if pool.can_be_collateral() {
                balance = balance.mul(pool.safe_factor());
                collateral_balance = collateral_balance.add(balance);
            }
        }

        // calculate debt
        let mut debt_balance = FixedU128::zero();
        for pool_id in user_supply_debt.debt.keys() {
            let pool = pool_map.get(pool_id).ok_or(CustomError::InconsistentState)?;
            if !pool.price_ready() { return Err(CustomError::PriceNotReady); }
            let amount = user_supply_debt.debt
                .get(pool_id)
                .ok_or(CustomError::InconsistentState)?
                .amount();
            let balance = pool.price().saturating_mul(amount);
            debt_balance = debt_balance.add(balance);
        }

        Ok(UserBalanceStats {
            supply_balance,
            collateral_balance,
            debt_balance,
        })
    }

    /// Checks if the withdraw will not trigger liquidation
    pub fn is_withdraw_trigger_liquidation(
        to_withdraw: (PoolId, FixedU128),
        user_supply_debt: UserSupplyDebtData,
        pool_map: BTreeMap<PoolId, PoolProxy<T>>,
        liquidation_threshold: FixedU128,
    ) -> Result<bool, CustomError> {
        let mut user_balance_stats = Self::user_balances(&user_supply_debt, &pool_map)?;
        let (pool_id, amount) = to_withdraw;

        let pool = pool_map.get(&pool_id).ok_or(CustomError::InconsistentState)?;
        if !pool.price_ready() { return Err(CustomError::PriceNotReady.into()); }
        let price = pool.price();

        user_balance_stats.decrement_collateral((price * pool.safe_factor()).mul(amount));
        Ok(user_balance_stats.is_liquidated(liquidation_threshold))
    }

    pub fn is_borrow_trigger_liquidation(
        to_borrow: (PoolId, FixedU128),
        user_supply_debt: UserSupplyDebtData,
        pool_map: BTreeMap<PoolId, PoolProxy<T>>,
        liquidation_threshold: FixedU128,
    ) -> Result<bool, CustomError> {
        let mut user_balance_stats = Self::user_balances(&user_supply_debt, &pool_map)?;
        let (pool_id, amount) = to_borrow;

        let pool = pool_map.get(&pool_id).ok_or(CustomError::InconsistentState)?;
        if !pool.price_ready() { return Err(CustomError::PriceNotReady.into()); }
        let price = pool.price();

        user_balance_stats.increment_debt(amount.mul(price));
        Ok(user_balance_stats.is_liquidated(liquidation_threshold))
    }

    /* -------- Internal helper methods --------- */

    /// Add to user supplies.
    /// This function contains write action, ensure the checks are performed in advance
    fn add_user_supply(pool: &PoolProxy<T>, account: T::AccountId, amount: &FixedU128) {
        let user_supply = UserData::new(*amount);
        PoolUserSupplies::<T>::insert(pool.id(), account.clone(), user_supply);

        // update user's supply asset set
        let mut pool_currency_tuples = UserSupplySet::<T>::get(account.clone());
        if !pool_currency_tuples.iter().any(|p| p.0 == pool.id()) {
            pool_currency_tuples.push((pool.id(), pool.currency_id()));
            UserSupplySet::<T>::insert(account, pool_currency_tuples);
        }
    }

    /// Remove to user supplies.
    /// This function contains write action, ensure the checks are performed in advance
    fn remove_user_supply(pool_id: PoolId, account: T::AccountId) {
        let mut pool_currency_tuples = UserSupplySet::<T>::get(account.clone());
        pool_currency_tuples.retain(|p| p.0 != pool_id);
        UserSupplySet::<T>::insert(account, pool_currency_tuples);
    }

    /// Remove to user debts.
    /// This function contains write action, ensure the checks are performed in advance
    fn remove_user_debt(pool_id: PoolId, account: T::AccountId) {
        let mut pool_currency_tuples = UserDebtSet::<T>::get(account.clone());
        pool_currency_tuples.retain(|p| p.0 != pool_id);
        UserDebtSet::<T>::insert(account, pool_currency_tuples);
    }


    /// Add to user debts.
    /// This function contains write action, ensure the checks are performed in advance
    fn new_debt(pool: &PoolProxy<T>, account: T::AccountId, amount: &FixedU128) {
        let user_debt = UserData::new(*amount);
        PoolUserDebts::<T>::insert(pool.id(), account.clone(), user_debt);

        // update user's supply asset set
        let mut pool_currency_tuples = UserDebtSet::<T>::get(account.clone());
        if !pool_currency_tuples.iter().any(|p| p.0 == pool.id()) {
            pool_currency_tuples.push((pool.id(), pool.currency_id()));
            UserDebtSet::<T>::insert(account, pool_currency_tuples);
        }
    }

    /// Decrement user debt.
    /// Ensure the amount is above pool.minimal_amount() before this function is actually called
    pub fn decrement_debt(pool: &PoolProxy<T>, account: T::AccountId, amount: &FixedU128, mut user_debt: UserData) -> DispatchResultWithPostInfo {
        user_debt.decrement(amount);
        // TODO: we need to handle dust here! Currently minimal_amount is just zero
        if user_debt.amount() <= pool.minimal_amount() {
            PoolUserDebts::<T>::remove(pool.id(), account.clone());
            UserAccountUtil::<T>::remove_user_debt(pool.id(), account);
        } else {
            PoolUserDebts::<T>::insert(pool.id(), account, user_debt);
        }
        Ok(().into())
    }
}

pub struct UserSupplyDebtData {
    pub supply: BTreeMap<PoolId, UserData>,
    pub debt: BTreeMap<PoolId, UserData>,
}