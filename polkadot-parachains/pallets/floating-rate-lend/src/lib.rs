// This file is part of Konomi.
// Copyright (C) 2020-2021 Konomi Foundation.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Floating interest rate lending module.


#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
mod types;
mod errors;

pub use pallet::*;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{codec::{Decode, Encode}, PalletId};
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use sp_runtime::{RuntimeDebug, FixedPointNumber, FixedU128};
    use sp_runtime::traits::{AccountIdConversion, One, Zero, CheckedDiv};
    use sp_std::{vec::Vec};
    use frame_support::error::BadOrigin;

    /* --------- Local Libs --------- */
    use pallet_traits::{MultiCurrency, PriceProvider};
    use polkadot_parachain_primitives::{PoolId, CurrencyId, FlowError, InvalidParameters, Overflown, PriceValue};
    use crate::types::{UserBalanceStats, Pool};
    use crate::errors::{PoolNotEnabled, PoolPriceNotReady};

    const PALLET_ID: PalletId = PalletId(*b"Floating");

    pub(crate) type BalanceOf<T> =
    <<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

    /// User supply struct
    #[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode)]
    pub struct UserData<T: Config> {
        amount: BalanceOf<T>,
        index: FixedU128,
    }

    impl <T: Config> UserData<T> {
        pub fn accrue_interest(&mut self, pool_index: &FixedU128) -> Result<(), FlowError> {
            self.amount = pool_index
                .checked_div(&self.index)
                .ok_or(FlowError{})?
                .checked_mul_int(self.amount)
                .ok_or(FlowError{})?;
            self.index = *pool_index;
            Ok(())
        }

        pub fn increment(&mut self, amount: &BalanceOf<T>) {
            self.amount += *amount;
        }

        pub fn decrement(&mut self, amount: &BalanceOf<T>) {
            self.amount -= *amount;
        }

        pub fn amount(&self) -> BalanceOf<T> { self.amount.clone() }

        pub fn new(amount: BalanceOf<T>) -> Self { UserData {amount, index: FixedU128::one()} }
    }

    /// The floating-rate-pool type
    type PoolOf<T> = Pool<T, BalanceOf<T>>;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_sudo::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type MultiCurrency: MultiCurrency<Self::AccountId>;
        type PriceProvider: PriceProvider<Self>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn get_liquidation_threshold)]
    pub(super) type LiquidationThreshold<T: Config> = StorageValue<_, FixedU128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pool_user_debts)]
    pub(super) type PoolUserDebts<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        PoolId,
        Twox64Concat,
        T::AccountId,
        UserData<T>
    >;

    #[pallet::storage]
    #[pallet::getter(fn pool_user_supplies)]
    pub(super) type PoolUserSupplies<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        PoolId,
        Twox64Concat,
        T::AccountId,
        UserData<T>
    >;

    #[pallet::storage]
    #[pallet::getter(fn user_supply_set)]
    pub(super) type UserSupplySet<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<(PoolId, CurrencyId)>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn user_debt_set)]
    pub(super) type UserDebtSet<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<(PoolId, CurrencyId)>, ValueQuery>;

    /* ------- Pool Related ------- */
    #[pallet::storage]
    #[pallet::getter(fn next_pool_id)]
    pub(super) type NextPoolId<T: Config> = StorageValue<_, PoolId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pool_name)]
    pub(super) type PoolNameStorage<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, PoolId>;

    #[pallet::storage]
    #[pallet::getter(fn pool)]
    pub(super) type PoolStorage<T: Config> = StorageMap<_, Twox64Concat, PoolId, PoolOf<T>>;

    #[pallet::genesis_config]
    pub struct GenesisConfig {
    }

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self {
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            log::info!("triggered");
            LiquidationThreshold::<T>::put(FixedU128::from_float(1.0));
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config>  {
        /* ----- Pool Related ----- */
        /// The floating-rate-pool with the id has been created
        PoolCreated(PoolId),
        /// The floating-rate-pool with the given id is enabled
        PoolEnabled(PoolId),
        /// The floating-rate-pool with the given id is disabled
        PoolDisabled(PoolId),
        /// The floating-rate-pool with given id is updated
        PoolUpdated(PoolId),

        /* ----- Operational ----- */
        /// The balance has been supplied to the pool[pool_id, account_id, balance]
        SupplySuccessful(PoolId, T::AccountId, BalanceOf<T>),
        /// The balance has been withdrawn to the pool[pool_id, account_id, balance]
        WithdrawSuccessful(PoolId, T::AccountId, BalanceOf<T>),
        /// The balance has been borrowed from the pool[pool_id, account_id, balance]
        BorrowSuccessful(PoolId, T::AccountId, BalanceOf<T>),
        /// The balance has been repaid to the pool[pool_id, account_id, balance]
        ReplaySuccessful(PoolId, T::AccountId, BalanceOf<T>),
        /// Liquidation is successful
        LiquidationSuccessful,
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /* ----- Pool Related ----- */
        /// The floating-rate-pool does not exist
        PoolNotExist,
        /// The floating-rate-pool is not enabled, cannot perform any transaction
        PoolNotEnabled,
        /// The floating-rate-pool already exists, cannot create again
        PoolAlreadyExists,
        /// The floating-rate-pool is already enabled, cannot enable again
        PoolAlreadyEnabled,
        /// The price of the pool is not ready
        PoolPriceNotReady,
        /// Not authorized to perform the action
        NotAuthorized,

        /// Balance is too low.
        BalanceTooLow,
        /// User has not supply in the pool
        UserNoSupplyInPool,
        /// User has not debt in the pool
        UserNoDebtInPool,
        /// Pool is not collateral
        AssetNotCollateral,
        /// User is not under liquidation
        UserNotUnderLiquidation,
        TransferFailed,
        NotEnoughLiquidity,
        InsufficientCollateral,
        UserNotExist,
        AboveLiquidationThreshold,
        BelowLiquidationThreshold,
        UserNoSupply,
        UserNoDebt,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T:Config> Pallet<T> {
        /***************************************/
        /* ------ Admin Only Operations ------ */
        /***************************************/
        /// List a new floating-rate-pool, providing the necessary info for the floating-rate-pool
        #[pallet::weight(1)]
        pub fn list_new(
            origin: OriginFor<T>,
            name: Vec<u8>,
            currency_id: CurrencyId,
            can_be_collateral: bool,
            safe_factor: u64,
            close_factor: u64,
            discount_factor: u64,
            utilization_factor: u64,
            initial_interest_rate: u64,
        ) -> DispatchResultWithPostInfo {
            let origin = Self::ensure_signed_and_root(origin.clone())?;

            // Check to ensure the floating-rate-pool name has not existed before
            if <PoolNameStorage<T>>::contains_key(name.clone()) {
                return Err(Error::<T>::PoolAlreadyExists.into());
            }
            let id = <NextPoolId<T>>::get();
            let pool = Pool::new(
                id,
                name.clone(),
                currency_id,
                can_be_collateral,
                FixedU128::saturating_from_rational(safe_factor, 100),
                FixedU128::saturating_from_rational(close_factor, 100),
                FixedU128::saturating_from_rational(discount_factor, 100),
                FixedU128::saturating_from_rational(utilization_factor, 10000000000u64),
                FixedU128::saturating_from_rational(initial_interest_rate, 10000000000u64),
                Zero::zero(),
                origin.clone(),
                <frame_system::Pallet<T>>::block_number(),
            );

            Self::check_pool(&pool)?;

            // Actual creation process
            <PoolStorage<T>>::insert(id, pool);
            <PoolNameStorage<T>>::insert(name, id);

            <NextPoolId<T>>::mutate(|id| *id += PoolId::one());

            Self::deposit_event(Event::PoolCreated(id));

            Ok(().into())
        }

        #[pallet::weight(1)]
        pub fn enable_pool(origin: OriginFor<T>, pool_id: PoolId) -> DispatchResultWithPostInfo {
            log::debug!("received request to enable floating-rate-pool {:?}", pool_id);
            let origin = Self::ensure_signed_and_root(origin.clone())?;

            let mut pool = PoolStorage::<T>::get(pool_id).ok_or(Error::<T>::PoolNotExist)?;
            if pool.enabled {
                log::error!("floating-rate-pool with id: {:?} is already enabled", pool_id);
                return Err(Error::<T>::PoolAlreadyEnabled.into());
            }

            pool.enabled = true;
            pool.last_updated_by = origin;
            pool.last_updated = <frame_system::Pallet<T>>::block_number();
            PoolStorage::<T>::insert(pool_id, pool);
            log::debug!("enabled floating-rate-pool: {:?}", pool_id);

            Self::deposit_event(Event::PoolEnabled(pool_id));

            Ok(().into())
        }

        #[pallet::weight(1)]
        pub fn disable_pool(origin: OriginFor<T>, pool_id: PoolId) -> DispatchResultWithPostInfo {
            log::debug!("received request to disable floating-rate-pool {:?}", pool_id);
            let origin = Self::ensure_signed_and_root(origin.clone())?;

            let mut pool = PoolStorage::<T>::get(pool_id).ok_or(Error::<T>::PoolNotExist)?;
            if !pool.enabled {
                log::error!("floating-rate-pool with id: {:?} is not enabled", pool_id);
                return Err(Error::<T>::PoolNotEnabled.into());
            }

            pool.enabled = false;
            pool.last_updated_by = origin;
            pool.last_updated = <frame_system::Pallet<T>>::block_number();
            PoolStorage::<T>::insert(pool_id, pool);
            log::debug!("disabled floating-rate-pool: {:?}", pool_id);

            Self::deposit_event(Event::PoolDisabled(pool_id));

            Ok(().into())
        }

        #[pallet::weight(1)]
        fn update_pool(
            origin: OriginFor<T>,
            pool_id: PoolId,
            can_be_collateral: bool,
            safe_factor: FixedU128,
            close_factor: FixedU128,
            discount_factor: FixedU128,
            utilization_factor: FixedU128,
            initial_interest_rate: FixedU128,
        ) -> DispatchResultWithPostInfo {
            log::debug!("received request to update floating-rate-pool {:?}", pool_id);
            let origin = Self::ensure_signed_and_root(origin.clone())?;

            let mut pool = PoolStorage::<T>::get(pool_id).ok_or(Error::<T>::PoolNotExist)?;
            pool.can_be_collateral = can_be_collateral;
            pool.safe_factor = safe_factor;
            pool.close_factor = close_factor;
            pool.discount_factor = discount_factor;
            pool.utilization_factor = utilization_factor;
            pool.initial_interest_rate = initial_interest_rate;

            pool.last_updated_by = origin;
            pool.last_updated = <frame_system::Pallet<T>>::block_number();

            Self::check_pool(&pool)?;

            PoolStorage::<T>::insert(pool_id, pool);
            log::debug!("updated floating-rate-pool: {:?}", pool_id);

            Self::deposit_event(Event::PoolUpdated(pool_id));

            Ok(().into())
        }

        /// Update liquidation threshold, express in percentage
        /// E.g. if the threshold is 1.2, which equals 120%, then val should be 120
        #[pallet::weight(1)]
        fn update_liquidation_threshold(origin: OriginFor<T>, val: u64) -> DispatchResultWithPostInfo {
            Self::ensure_signed_and_root(origin)?;
            LiquidationThreshold::<T>::put(FixedU128::saturating_from_rational(val, 100));
            Ok(().into())
        }

        /*******************************/
        /* ------ For All Users ------ */
        /*******************************/
        /// Supply certain amount to the floating-rate-pool
        #[pallet::weight(1)]
        fn supply(origin: OriginFor<T>, pool_id: PoolId, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let account = ensure_signed(origin)?;
            if amount.is_zero() { return Err(Error::<T>::BalanceTooLow.into()); }

            // check floating-rate-pool exists and get floating-rate-pool instance
            let mut pool: PoolOf<T> = PoolStorage::<T>::get(pool_id).ok_or(Error::<T>::PoolNotExist)?;
            if !pool.enabled { return Err(PoolNotEnabled{}.into()); }
            if amount < pool.minimal_amount { return Err(Error::<T>::BalanceTooLow.into()) }

            pool.accrue_interest(<frame_system::Pallet<T>>::block_number())?;

            // transfer asset
            <T as Config>::MultiCurrency::transfer(
                pool.currency_id,
                &account,
                &Self::account_id(),
                amount,
            )?;

            Self::increment_user_supply(&pool, account.clone(), amount)?;
            pool.increment_supply(amount);
            PoolStorage::<T>::insert(pool_id, pool);

            Self::deposit_event(Event::SupplySuccessful(pool_id, account.clone(), amount));

            Ok(().into())
        }

        #[pallet::weight(1)]
        fn withdraw(origin: OriginFor<T>, pool_id: PoolId, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let account = ensure_signed(origin)?;

            if amount.is_zero() { return Err(Error::<T>::BalanceTooLow.into()); }

            // Check pool can withdraw
            let mut pool: PoolOf<T> = PoolStorage::<T>::get(pool_id).ok_or(Error::<T>::PoolNotExist)?;
            if !pool.enabled { return Err(Error::<T>::PoolNotEnabled.into()); }
            // This ensures the pool's supply will never be lower than 0
            if (pool.supply - pool.debt) < amount { return Err(Error::<T>::NotEnoughLiquidity.into()); }

            // Check user supply can withdraw
            let mut user_supply = PoolUserSupplies::<T>::get(pool.id, account.clone())
                .ok_or(Error::<T>::UserNoSupplyInPool)?;

            // Update interest
            user_supply.accrue_interest(&pool.total_supply_index)?;
            pool.accrue_interest(<frame_system::Pallet<T>>::block_number())?;

            let mut amount = amount;
            if user_supply.amount < amount { amount = user_supply.amount; }

            Self::check_withdraw_against_liquidation(account.clone(), pool.currency_id, pool.safe_factor, amount)?;

            // Now the checks are done, we can prepare to transfer
            user_supply.decrement(&amount);
            pool.decrement_supply(amount);

            let minimal_amount = pool.minimal_amount.clone();

            // Now perform the writes
            T::MultiCurrency::transfer(pool.currency_id, &Self::account_id(), &account, amount)?;
            PoolStorage::<T>::insert(pool_id, pool);

            // TODO: we need to handle dust here! Currently minimal_amount is just zero
            if user_supply.amount < minimal_amount {
                PoolUserSupplies::<T>::remove(pool_id, account.clone());
                Self::remove_user_supply(pool_id, account.clone());
            } else {
                PoolUserSupplies::<T>::insert(pool_id, account.clone(), user_supply);
            }

            Self::deposit_event(Event::WithdrawSuccessful(pool_id, account, amount));

            Ok(().into())
        }

        #[pallet::weight(1)]
        fn borrow(origin: OriginFor<T>, pool_id: PoolId, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let account = ensure_signed(origin)?;

            // Check pool can borrow
            let mut pool: PoolOf<T> = PoolStorage::<T>::get(pool_id).ok_or(Error::<T>::PoolNotExist)?;
            if !pool.enabled { return Err(Error::<T>::PoolNotEnabled.into()); }
            // This ensures the pool's supply will never be lower than 0
            if pool.debt + amount > pool.supply { return Err(Error::<T>::NotEnoughLiquidity.into()); }

            Self::check_borrow_against_liquidation(account.clone(), pool.currency_id, pool.safe_factor, amount)?;

            // Ready to perform the writes
            pool.accrue_interest(<frame_system::Pallet<T>>::block_number())?;

            T::MultiCurrency::transfer(pool.currency_id, &Self::account_id(), &account, amount)?;

            if let Some(mut user_debt) = PoolUserDebts::<T>::get(pool_id, account.clone()) {
                user_debt.accrue_interest(&pool.total_debt_index)?;
                PoolUserDebts::<T>::insert(pool_id, account.clone(), user_debt);
            } else {
                Self::add_user_debt(&pool, account.clone(), amount);
            }
            PoolStorage::<T>::insert(pool_id, pool);

            Self::deposit_event(Event::BorrowSuccessful(pool_id, account.clone(), amount));

            Ok(().into())
        }

        #[pallet::weight(1)]
        fn repay(origin: OriginFor<T>, pool_id: PoolId, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let account = ensure_signed(origin)?;

            // Check pool can borrow
            let mut pool: PoolOf<T> = PoolStorage::<T>::get(pool_id).ok_or(Error::<T>::PoolNotExist)?;
            if !pool.enabled { return Err(Error::<T>::PoolNotEnabled.into()); }

            // Check user supply can withdraw
            let mut user_debt = PoolUserDebts::<T>::get(pool.id, account.clone())
                .ok_or(Error::<T>::UserNoDebtInPool)?;

            pool.accrue_interest(<frame_system::Pallet<T>>::block_number())?;
            user_debt.accrue_interest(&pool.total_debt_index)?;

            let mut amount = amount;
            if user_debt.amount < amount { amount = user_debt.amount; }

            pool.decrement_debt(amount);
            user_debt.decrement(&amount);

            // Ready to perform the writes
            let minimal_amount = pool.minimal_amount.clone();

            // Now perform the writes
            T::MultiCurrency::transfer(pool.currency_id,&account,&Self::account_id(), amount)?;
            PoolStorage::<T>::insert(pool_id, pool);

            // TODO: we need to handle dust here! Currently minimal_amount is just zero
            if user_debt.amount < minimal_amount {
                PoolUserDebts::<T>::remove(pool_id, account.clone());
                Self::remove_user_debt(pool_id, account.clone());
            } else {
                PoolUserDebts::<T>::insert(pool_id, account.clone(), user_debt);
            }

            Self::deposit_event(Event::ReplaySuccessful(pool_id, account, amount));

            Ok(().into())
        }

        // arbitrager related
        #[pallet::weight(1)]
        fn liquidate(
            origin: OriginFor<T>,
            target_user: T::AccountId,
            debt_pool_id: PoolId,
            collateral_pool_id: PoolId,
            pay_amount: BalanceOf<T>
        ) -> DispatchResultWithPostInfo {
            let account = ensure_signed(origin)?;

            // check floating-rate-pool exists and get floating-rate-pool instances
            // check if get_asset_id is enabled as collateral
            let mut collateral_pool: PoolOf<T> = PoolStorage::<T>::get(collateral_pool_id).ok_or(Error::<T>::PoolNotExist)?;
            if !collateral_pool.can_be_collateral { return Err(Error::<T>::AssetNotCollateral.into()); }

            let mut debt_pool: PoolOf<T> = PoolStorage::<T>::get(debt_pool_id).ok_or(Error::<T>::PoolNotExist)?;

            let block_number = <frame_system::Pallet<T>>::block_number();
            debt_pool.accrue_interest(block_number.clone())?;
            collateral_pool.accrue_interest(block_number.clone())?;

            // Ensure the user has got the collateral and debt
            let mut user_debt = PoolUserDebts::<T>::get(debt_pool_id, account.clone())
                .ok_or(Error::<T>::UserNoDebtInPool)?;
            let mut user_collateral = PoolUserSupplies::<T>::get(collateral_pool_id, account.clone())
                .ok_or(Error::<T>::UserNoSupplyInPool)?;

            // Ensure the user is liquidated
            let liquidation_threshold = LiquidationThreshold::<T>::get();
            let user_stats = Self::user_balances(target_user.clone())?;
            if !user_stats.is_liquidated(liquidation_threshold) { return Err(Error::<T>::UserNotUnderLiquidation.into()); }

            user_debt.accrue_interest(&debt_pool.total_debt_index)?;
            user_collateral.accrue_interest(&collateral_pool.total_debt_index)?;

            // Next, we check the prices are ok to use
            let debt_pool_price = Self::pool_price(&debt_pool)?;
            let collateral_pool_price = Self::pool_price(&collateral_pool)?;
            let discounted_collateral_price = collateral_pool.discounted_price(collateral_pool_price.clone());

            // Now, we derive the amount for liquidation
            let arbitrageur_get_amount = collateral_pool.closable_amount(user_collateral.amount.clone());
            let arbitrageur_pay_amount = Self::convert_amount(
                arbitrageur_get_amount.clone(),
                &discounted_collateral_price,
                &debt_pool_price,
            )?;

            // Now we calculate the total amount to transfer to arbitrageur
            let mut pay_amount = pay_amount;
            if pay_amount > arbitrageur_pay_amount { pay_amount = arbitrageur_pay_amount; }
            if pay_amount > user_debt.amount { pay_amount = user_debt.amount; }

            // TODO: check rounding errors due to discount_factor
            let get_amount = Self::convert_amount(
                pay_amount.clone(),
                &debt_pool_price,
                &discounted_collateral_price,
            )?;

            // Now we can transfer debt from arbitrageur to pool
            T::MultiCurrency::transfer(debt_pool.currency_id,&account,&Self::account_id(), pay_amount)?;
            // Then the collateral to arbitrageur
            T::MultiCurrency::transfer(collateral_pool.currency_id,&Self::account_id(), &account, get_amount)?;

            user_collateral.decrement(&get_amount);
            user_debt.decrement(&pay_amount);

            let minimal_amount = collateral_pool.minimal_amount.clone();

            // update pools
            PoolStorage::<T>::insert(debt_pool_id, debt_pool);
            PoolStorage::<T>::insert(collateral_pool_id, collateral_pool);

            // TODO: shift the common code to a single function!
            if user_collateral.amount < minimal_amount {
                PoolUserSupplies::<T>::remove(collateral_pool_id, account.clone());
                Self::remove_user_supply(collateral_pool_id, account.clone());
            } else {
                PoolUserSupplies::<T>::insert(collateral_pool_id, account.clone(), user_collateral);
            }

            if user_debt.amount < minimal_amount {
                PoolUserDebts::<T>::remove(debt_pool_id, account.clone());
                Self::remove_user_debt(debt_pool_id, account.clone());
            } else {
                PoolUserDebts::<T>::insert(debt_pool_id, account.clone(), user_debt);
            }

            Self::deposit_event(Event::LiquidationSuccessful);

            Ok(().into())
        }
    }

    impl<T:Config> Pallet<T> {
        /* ----- Runtime API ----- */
        pub fn debt_rate(id: PoolId) -> FixedU128{
            match PoolStorage::<T>::get(id) {
                Some(pool) => pool.debt_interest_rate().unwrap_or(FixedU128::zero()),
                None => FixedU128::zero()
            }
        }

        pub fn supply_rate(id: PoolId) -> FixedU128 {
            match PoolStorage::<T>::get(id) {
                Some(pool) => pool.supply_interest_rate().unwrap_or(FixedU128::zero()),
                None => FixedU128::zero()
            }
        }

        /// Get the user supply balance for the user in a pool
        pub fn user_supply_balance(pool_id: PoolId, user: T::AccountId) -> Result<BalanceOf<T>, Overflown> {
            if let Some(user_supply) = PoolUserSupplies::<T>::get(pool_id, user) {
                if let Some(mut pool) = PoolStorage::<T>::get(pool_id) {
                    pool.accrue_interest(<frame_system::Pallet<T>>::block_number())?;
                    return Ok(
                        (pool.total_supply_index / user_supply.index)
                        .saturating_mul_int(user_supply.amount)
                    );
                }
            }
            Ok(BalanceOf::<T>::zero())
        }

        /// Get the user debt balance for the user in a pool
        pub fn user_debt_balance(pool_id: PoolId, user: T::AccountId) -> Result<BalanceOf<T>, Overflown> {
            if let Some(user_debt) = PoolUserDebts::<T>::get(pool_id, user) {
                if let Some(mut pool) = PoolStorage::<T>::get(pool_id) {
                    pool.accrue_interest(<frame_system::Pallet<T>>::block_number())?;
                    return Ok(
                        (pool.total_debt_index / user_debt.index)
                        .saturating_mul_int(user_debt.amount)
                    );
                }
            }
            Ok(BalanceOf::<T>::zero())
        }

        // total supply balance; total converted supply balance; total debt balance;
        pub fn user_balances(user: T::AccountId) -> Result<UserBalanceStats<BalanceOf<T>>, Overflown> {
            let mut supply_balance = BalanceOf::<T>::zero();
            let mut collateral_balance = BalanceOf::<T>::zero();

            for (pool_id, currency_id) in Self::user_supply_set(user.clone()).into_iter() {
                let amount = Self::user_supply_balance(pool_id, user.clone())?;
                let price = T::PriceProvider::price(currency_id);
                if !price.price_ready() {
                    // TODO: throw error
                    continue
                }
                supply_balance += price.value().saturating_mul_int(amount);

                // TODO: consider using some sort of cache?
                collateral_balance += (price.value() * PoolStorage::<T>::get(pool_id).unwrap().safe_factor).saturating_mul_int(amount);
            }

            let mut debt_balance = BalanceOf::<T>::zero();
            for (pool_id, currency_id) in Self::user_debt_set(user.clone()).into_iter() {
                let amount = Self::user_debt_balance(pool_id, user.clone())?;
                let price = T::PriceProvider::price(currency_id);
                if !price.price_ready() {
                    // TODO: throw error
                    continue
                }
                debt_balance += price.value().saturating_mul_int(amount);
            }
            Ok(UserBalanceStats {
                supply_balance,
                collateral_balance,
                debt_balance,
            })
        }

        /* -------- Internal Helper Functions ------------ */

        fn account_id() -> T::AccountId { PALLET_ID.into_account() }

        fn ensure_signed_and_root(origin: OriginFor<T>) -> Result<T::AccountId, BadOrigin> {
            // TODO: enable this after figuring out why this won't work
            // ensure_root(origin)?;
            let origin = ensure_signed(origin)?;
            if origin != <pallet_sudo::Pallet<T>>::key() {
                return Err(BadOrigin);
            }
            Ok(origin)
        }

        fn pool_price(pool: &PoolOf<T>) -> Result<PriceValue, PoolPriceNotReady> {
            let price = T::PriceProvider::price(pool.currency_id);
            if !price.price_ready() { return Err(PoolPriceNotReady{}); }
            Ok(price.value())
        }

        fn convert_amount(
            amount_of_source: BalanceOf<T>,
            price_of_source: &PriceValue,
            price_of_target: &PriceValue
        ) -> Result<BalanceOf<T>, FlowError> {
            Ok(price_of_source
                .checked_div(price_of_target)
                .ok_or(FlowError{})?
                .checked_mul_int(amount_of_source)
                .ok_or(FlowError{})?)
        }

        fn check_withdraw_against_liquidation(
            account: T::AccountId,
            currency_id: CurrencyId,
            safe_factor: FixedU128,
            amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let user_balance_stats = Self::user_balances(account.clone())?;
            let price = T::PriceProvider::price(currency_id);

            let mut collateral_remain = user_balance_stats.collateral_balance;
            collateral_remain -= (price.value() * safe_factor).saturating_mul_int(amount);

            let required_collateral = LiquidationThreshold::<T>::get()
                .saturating_mul_int(user_balance_stats.debt_balance);

            if collateral_remain < required_collateral {
                return Err(Error::<T>::BelowLiquidationThreshold.into());
            }

            Ok(().into())
        }

        fn check_borrow_against_liquidation(
            account: T::AccountId,
            currency_id: CurrencyId,
            safe_factor: FixedU128,
            amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let user_balance_stats = Self::user_balances(account.clone())?;
            let price = T::PriceProvider::price(currency_id);

            // First calculate the balance of the amount to borrow
            let mut required_collateral = (price.value() * safe_factor).saturating_mul_int(amount);
            // Then add to the existing debt balance
            required_collateral += user_balance_stats.debt_balance;
            // Finally multiply with the liquidation threshold
            required_collateral = LiquidationThreshold::<T>::get()
                .saturating_mul_int(required_collateral);

            if user_balance_stats.collateral_balance < required_collateral {
                return Err(Error::<T>::BelowLiquidationThreshold.into());
            }

            Ok(().into())
        }

        /// Increment user supply.
        /// Ensure the amount is above pool.minimal_amount() before this function is actually called
        fn increment_user_supply(pool: &PoolOf<T>, account: T::AccountId, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
            if let Some(mut user_supply) = PoolUserSupplies::<T>::get(pool.id, account.clone()) {
                user_supply.accrue_interest(&pool.total_supply_index)?;
                user_supply.increment(&amount);
                PoolUserSupplies::<T>::insert(pool.id, account.clone(), user_supply);
            } else {
                Self::add_user_supply(&pool, account.clone(), amount);
            }

            Ok(().into())
        }

        /// Add to user supplies.
        /// This function contains write action, ensure the checks are performed in advance
        fn add_user_supply(pool: &PoolOf<T>, account: T::AccountId, amount: BalanceOf<T>) {
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
        fn remove_user_supply(pool_id: PoolId, account: T::AccountId) {
            let mut pool_currency_tuples = UserSupplySet::<T>::get(account.clone());
            pool_currency_tuples.retain(|p| p.0 != pool_id);
            UserSupplySet::<T>::insert(account, pool_currency_tuples);
        }

        /// Add to user debts.
        /// This function contains write action, ensure the checks are performed in advance
        fn add_user_debt(pool: &PoolOf<T>, account: T::AccountId, amount: BalanceOf<T>) {
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
        fn remove_user_debt(pool_id: PoolId, account: T::AccountId) {
            let mut pool_currency_tuples = UserDebtSet::<T>::get(account.clone());
            pool_currency_tuples.retain(|p| p.0 != pool_id);
            UserDebtSet::<T>::insert(account, pool_currency_tuples);
        }

        /// Check the floating-rate-pool's parameters
        /// TODO: we need to tighten some of the checks
        fn check_pool(pool: &PoolOf<T>) -> Result<(), InvalidParameters> {
            Self::ensure_within_range(&pool.close_factor, FixedU128::zero(), FixedU128::one())?;
            Self::ensure_within_range(&pool.discount_factor, FixedU128::zero(), FixedU128::one())?;
            Self::ensure_within_range(&pool.utilization_factor, FixedU128::zero(), FixedU128::one())?;
            Self::ensure_within_range(&pool.safe_factor, FixedU128::zero(), FixedU128::one())?;
            Ok(())
        }

        fn ensure_within_range(num: &FixedU128, lower: FixedU128, upper: FixedU128) -> Result<(), InvalidParameters> {
            if lower <= *num && *num <= upper {
                return Ok(());
            }
            Err(InvalidParameters{})
        }
    }
}