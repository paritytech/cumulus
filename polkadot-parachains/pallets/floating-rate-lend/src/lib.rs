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

pub use pallet::*;

mod types;
mod errors;
mod pool;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_support::error::BadOrigin;
    use frame_support::PalletId;
    use frame_system::pallet_prelude::*;
    use sp_runtime::{FixedPointNumber, FixedU128};
    use sp_runtime::traits::{AccountIdConversion, CheckedDiv, CheckedMul, Convert, One, Zero};
    use sp_std::{vec::Vec};
    use sp_std::collections::btree_map::BTreeMap;

    use pallet_traits::{MultiCurrency, PriceProvider};
    use polkadot_parachain_primitives::{InvalidParameters, PoolId, PriceValue, CustomError};

    use crate::pool::{Pool, PoolProxy, PoolRepository};
    use crate::types::{Convertor, UserAccountUtil, UserBalanceStats, UserData, UserSupplyDebtData};

    /* --------- Local Libs --------- */
    const PALLET_ID: PalletId = PalletId(*b"Floating");

    /// User supply struct
    pub(crate) type BalanceOf<T> =
    <<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type CurrencyIdOf<T> =
    <<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_sudo::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: MultiCurrency<Self::AccountId>;
        type PriceProvider: PriceProvider<Self, CurrencyId = CurrencyIdOf<Self>>;
        type Conversion: Convert<BalanceOf<Self>, FixedU128> + Convert<FixedU128, BalanceOf<Self>>;
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
        UserData
    >;

    #[pallet::storage]
    #[pallet::getter(fn pool_user_supplies)]
    pub(super) type PoolUserSupplies<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        PoolId,
        Twox64Concat,
        T::AccountId,
        UserData
    >;

    #[pallet::storage]
    #[pallet::getter(fn user_supply_set)]
    pub(super) type UserSupplySet<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<(PoolId, CurrencyIdOf<T>)>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn user_debt_set)]
    pub(super) type UserDebtSet<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<(PoolId, CurrencyIdOf<T>)>, ValueQuery>;

    /* ------- Pool Related ------- */
    #[pallet::storage]
    #[pallet::getter(fn next_pool_id)]
    pub(super) type NextPoolId<T: Config> = StorageValue<_, PoolId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pool_name)]
    pub(super) type PoolNameStorage<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, PoolId>;

    #[pallet::storage]
    #[pallet::getter(fn pool)]
    pub(super) type PoolStorage<T: Config> = StorageMap<_, Twox64Concat, PoolId, Pool<T>>;

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
            log::info!("triggered genesis");
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
        /// The balance has been supplied to the pool[pool_id, account_id, fixed_u_128]
        SupplySuccessful(PoolId, T::AccountId, FixedU128),
        /// The balance has been withdrawn to the pool[pool_id, account_id, fixed_u_128]
        WithdrawSuccessful(PoolId, T::AccountId, FixedU128),
        /// The balance has been borrowed from the pool[pool_id, account_id, fixed_u_128]
        BorrowSuccessful(PoolId, T::AccountId, FixedU128),
        /// The balance has been repaid to the pool[pool_id, account_id, fixed_u_128]
        ReplaySuccessful(PoolId, T::AccountId, FixedU128),
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
        /// Balance is too low.
        BalanceTooLow,
        /// User supply is too low.
        UserSupplyTooLow,
        /// User has not supply in the pool
        UserNoSupplyInPool,
        /// User has not debt in the pool
        UserNoDebtInPool,
        /// Pool is not collateral
        AssetNotCollateral,
        /// User is not under liquidation
        UserNotUnderLiquidation,
        /// Not enough liquidity in the pool for the transaction
        NotEnoughLiquidity,
        /// The requirement of not smaller than liquidation threshold is violated
        BelowLiquidationThreshold,
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
            currency_id: CurrencyIdOf<T>,
            can_be_collateral: bool,
            safe_factor_percentage: u64,
            close_factor_percentage: u64,
            discount_factor_percentage: u64,
            utilization_factor_percentage_annum: u64,
            initial_interest_rate_percentage_annum: u64,
        ) -> DispatchResultWithPostInfo {
            let origin = Self::ensure_signed_and_root(origin)?;

            // Check to ensure the floating-rate-pool name has not existed before
            if <PoolNameStorage<T>>::contains_key(name.clone()) {
                return Err(Error::<T>::PoolAlreadyExists.into());
            }
            let id = <NextPoolId<T>>::get();

            // TODO: ensure parameters range from 1 to 100
            let pool = Pool::new(
                id,
                name.clone(),
                currency_id,
                can_be_collateral,
                Convertor::convert_percentage(safe_factor_percentage),
                Convertor::convert_percentage(close_factor_percentage),
                Convertor::convert_percentage(discount_factor_percentage),
                Convertor::convert_percentage_annum_to_per_block(utilization_factor_percentage_annum),
                Convertor::convert_percentage_annum_to_per_block(initial_interest_rate_percentage_annum),
                Zero::zero(),
                origin,
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
            let origin = Self::ensure_signed_and_root(origin)?;

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
            let origin = Self::ensure_signed_and_root(origin)?;

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
            safe_factor_percentage: u64,
            close_factor_percentage: u64,
            discount_factor_percentage: u64,
            utilization_factor_percentage_annum: u64,
            initial_interest_rate_percentage_annum: u64,
        ) -> DispatchResultWithPostInfo {
            log::debug!("received request to update floating-rate-pool {:?}", pool_id);
            let origin = Self::ensure_signed_and_root(origin)?;

            let mut pool = PoolStorage::<T>::get(pool_id).ok_or(Error::<T>::PoolNotExist)?;
            pool.can_be_collateral = can_be_collateral;
            pool.safe_factor = Convertor::convert_percentage(safe_factor_percentage);
            pool.close_factor = Convertor::convert_percentage(close_factor_percentage);
            pool.discount_factor = Convertor::convert_percentage(discount_factor_percentage);
            pool.utilization_factor = Convertor::convert_percentage_annum_to_per_block(utilization_factor_percentage_annum);
            pool.initial_interest_rate = Convertor::convert_percentage_annum_to_per_block(initial_interest_rate_percentage_annum);

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
            let mut pool: PoolProxy<T> = PoolRepository::<T>::find_without_price(pool_id)?;
            if !pool.enabled() { return Err(Error::<T>::PoolNotEnabled.into()); }

            let amount_u128 = T::Conversion::convert(amount);
            if amount_u128 < pool.minimal_amount() { return Err(Error::<T>::BalanceTooLow.into()) }

            // transfer asset
            T::Currency::transfer(pool.currency_id(), &account, &Self::account_id(), amount)?;

            pool.accrue_interest()?;
            pool.increment_supply(&amount_u128);
            PoolRepository::<T>::save(pool.clone());
            UserAccountUtil::<T>::accrue_interest_and_increment_supply(&pool, account.clone(), &amount_u128)?;

            Self::deposit_event(Event::SupplySuccessful(pool_id, account, amount_u128));

            Ok(().into())
        }

        #[pallet::weight(1)]
        fn withdraw(origin: OriginFor<T>, pool_id: PoolId, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let account = ensure_signed(origin)?;

            if amount.is_zero() { return Err(Error::<T>::BalanceTooLow.into()); }

            // Check pool can withdraw
            let mut pool: PoolProxy<T> = PoolProxy::find(pool_id)?;
            if !pool.enabled() { return Err(Error::<T>::PoolNotEnabled.into()); }
            // This ensures the pool's supply will never be lower than 0
            let mut amount_fu128 = T::Conversion::convert(amount);
            if !pool.allow_amount_deduction(&amount_fu128) { return Err(Error::<T>::NotEnoughLiquidity.into()); }

            // Check user supply can withdraw
            let user_supply = PoolUserSupplies::<T>::get(pool.id(), account.clone())
                .ok_or(Error::<T>::UserNoSupplyInPool)?;
            let mut transfer_amount = amount;
            if user_supply.amount() < amount_fu128 {
                amount_fu128 = user_supply.amount();
                transfer_amount = T::Conversion::convert(user_supply.amount());
            }

            // Check if this pool is collateral
            if pool.can_be_collateral() {
                let (pool_map, user_supply_debt) = Self::prefetch_for_liquidation_check(account.clone())?;
                let r = UserAccountUtil::<T>::is_withdraw_trigger_liquidation(
                    (pool_id, amount_fu128),
                    user_supply_debt,
                    pool_map,
                    LiquidationThreshold::<T>::get(),
                )?;
                if r { return Err(Error::<T>::BelowLiquidationThreshold.into()); }
            }

            // Now the checks are done, we can prepare to transfer
            UserAccountUtil::<T>::decrement_supply(&pool, account.clone(), &amount_fu128, user_supply)?;
            pool.decrement_supply(&amount_fu128);
            PoolRepository::<T>::save(pool.clone());

            // Now perform the writes
            T::Currency::transfer(pool.currency_id(), &Self::account_id(), &account, transfer_amount)?;

            Self::deposit_event(Event::WithdrawSuccessful(pool_id, account, amount_fu128));

            Ok(().into())
        }

        #[pallet::weight(1)]
        fn borrow(origin: OriginFor<T>, pool_id: PoolId, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let account = ensure_signed(origin)?;

            // Check pool can borrow
            let mut pool: PoolProxy<T> = PoolRepository::<T>::find_without_price(pool_id)?;
            if !pool.enabled() { return Err(Error::<T>::PoolNotEnabled.into()); }
            // Check sufficient liquidity
            let amount_u128 = T::Conversion::convert(amount);
            pool.accrue_interest()?;
            if !pool.allow_amount_deduction(&amount_u128) { return Err(Error::<T>::NotEnoughLiquidity.into()); }

            let (mut pool_map, user_supply_debt) = Self::prefetch_for_liquidation_check(account.clone())?;
            if !pool_map.contains_key(&pool_id) { pool_map.insert(pool_id, pool.clone()); }
            let r = UserAccountUtil::<T>::is_borrow_trigger_liquidation(
                (pool_id,amount_u128),
                user_supply_debt,
                pool_map,
                LiquidationThreshold::<T>::get(),
            )?;
            if r { return Err(Error::<T>::BelowLiquidationThreshold.into()); }

            // Ready to perform the writes
            // TODO: add check can transfer
            T::Currency::transfer(pool.currency_id(), &Self::account_id(), &account, amount)?;
            UserAccountUtil::<T>::accrue_interest_and_increment_debt(&pool, account.clone(), &amount_u128)?;

            pool.increment_debt(&amount_u128);
            PoolRepository::<T>::save(pool);

            Self::deposit_event(Event::BorrowSuccessful(pool_id, account, amount_u128));

            Ok(().into())
        }

        #[pallet::weight(1)]
        fn repay(origin: OriginFor<T>, pool_id: PoolId, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let account = ensure_signed(origin)?;

            // Check pool can borrow
            let mut pool = PoolRepository::<T>::find_without_price(pool_id)?;
            if !pool.enabled() { return Err(Error::<T>::PoolNotEnabled.into()); }

            // Check user supply can withdraw
            let mut user_debt = PoolUserDebts::<T>::get(pool.id(), account.clone())
                .ok_or(Error::<T>::UserNoDebtInPool)?;

            let mut amount_fu128 = T::Conversion::convert(amount);
            let mut transfer_amount = amount;
            if user_debt.amount() < amount_fu128 {
                amount_fu128 = user_debt.amount();
                transfer_amount = T::Conversion::convert(amount_fu128);
            }

            pool.accrue_interest()?;
            pool.decrement_debt(&amount_fu128);
            PoolRepository::<T>::save(pool.clone());

            user_debt.accrue_interest(&pool.total_debt_index())?;
            UserAccountUtil::<T>::decrement_debt(&pool, account.clone(), &amount_fu128, user_debt)?;

            // Transfer currency
            T::Currency::transfer(pool.currency_id(), &account, &Self::account_id(), transfer_amount)?;

            Self::deposit_event(Event::ReplaySuccessful(pool_id, account, amount_fu128));

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
            let collateral_pool: PoolProxy<T> = PoolRepository::<T>::find(collateral_pool_id)?;
            if !collateral_pool.can_be_collateral() { return Err(Error::<T>::AssetNotCollateral.into()); }

            // Ensure the user has got the collateral and debt
            let mut user_debt = PoolUserDebts::<T>::get(debt_pool_id, account.clone())
                .ok_or(Error::<T>::UserNoDebtInPool)?;
            let mut user_collateral = PoolUserSupplies::<T>::get(collateral_pool_id, account.clone())
                .ok_or(Error::<T>::UserNoSupplyInPool)?;

            // Ensure the user is liquidated
            let (pool_map, user_supply_debt) = Self::prefetch_for_liquidation_check(target_user.clone())?;

            let balances = UserAccountUtil::<T>::user_balances(&user_supply_debt, &pool_map)?;
            let liquidation_threshold = LiquidationThreshold::<T>::get();
            if !balances.is_liquidated(liquidation_threshold) { return Err(Error::<T>::UserNotUnderLiquidation.into()); }

            let debt_pool = pool_map.get(&debt_pool_id).ok_or(CustomError::InconsistentState)?;
            let collateral_pool = pool_map.get(&collateral_pool_id).ok_or(CustomError::InconsistentState)?;
            user_debt.accrue_interest(&debt_pool.total_debt_index())?;
            user_collateral.accrue_interest(&collateral_pool.total_debt_index())?;

            // price should have been checked in liquidation checks
            let discounted_collateral_price = collateral_pool.discounted_price(&collateral_pool.price());

            // Now, we derive the amount for liquidation
            let arbitrageur_get_limit = collateral_pool.closable_amount(&user_collateral.amount(), &collateral_pool.price());
            let arbitrageur_pay_limit = Self::convert_amount(
                &arbitrageur_get_limit,
                &discounted_collateral_price,
                &debt_pool.price(),
            )?;

            // Now we calculate the total amount to transfer to arbitrageur
            let mut pay_amount = T::Conversion::convert(pay_amount);
            if pay_amount > arbitrageur_pay_limit { pay_amount = arbitrageur_pay_limit; }
            if pay_amount > user_debt.amount() { pay_amount = user_debt.amount(); }

            // TODO: check rounding errors due to discount_factor
            let get_amount = Self::convert_amount(
                &pay_amount,
                &debt_pool.price(),
                &discounted_collateral_price,
            )?;

            let debt_currency_id = debt_pool.currency_id();
            let collateral_currency_id = collateral_pool.currency_id();
            // Update user accounts
            UserAccountUtil::<T>::decrement_supply(&collateral_pool, account.clone(), &get_amount, user_collateral)?;
            UserAccountUtil::<T>::decrement_debt(&debt_pool, account.clone(), &pay_amount, user_debt)?;

            // update pools
            PoolRepository::<T>::save(debt_pool.clone());
            PoolRepository::<T>::save(collateral_pool.clone());

            // Now we can transfer debt from arbitrageur to pool
            let pay_amount_transfer = T::Conversion::convert(pay_amount);
            T::Currency::transfer(debt_currency_id, &account, &Self::account_id(), pay_amount_transfer)?;
            // Then the collateral to arbitrageur
            let get_amount_transfer = T::Conversion::convert(get_amount);
            T::Currency::transfer(collateral_currency_id, &Self::account_id(), &account, get_amount_transfer)?;

            Self::deposit_event(Event::LiquidationSuccessful);

            Ok(().into())
        }
    }

    impl<T:Config> Pallet<T> {
        /* ----- Runtime API ----- */
        pub fn debt_rate(id: PoolId) -> FixedU128{
            match PoolStorage::<T>::get(id) {
                Some(pool) => pool.debt_interest_rate().unwrap_or_else(|_| FixedU128::zero()),
                None => FixedU128::zero()
            }
        }

        pub fn supply_rate(id: PoolId) -> FixedU128 {
            match PoolStorage::<T>::get(id) {
                Some(pool) => pool.supply_interest_rate().unwrap_or_else(|_| FixedU128::zero()),
                None => FixedU128::zero()
            }
        }

        /// Get the user supply balance for the user in a pool
        pub fn user_supply_balance(pool_id: PoolId, user: T::AccountId) -> Result<FixedU128, CustomError> {
            UserAccountUtil::<T>::supply_balance_with_interest(pool_id, user)
        }

        /// Get the user debt balance for the user in a pool
        pub fn user_debt_balance(pool_id: PoolId, user: T::AccountId) -> Result<FixedU128, CustomError> {
            UserAccountUtil::<T>::debt_balance_with_interest(pool_id, user)
        }

        // total supply balance; total converted supply balance; total debt balance;
        pub fn user_balances(user: T::AccountId) -> Result<UserBalanceStats, CustomError> {
            let (pool_map, user_supply_debt) = Self::prefetch_for_liquidation_check(user.clone())?;
            UserAccountUtil::<T>::user_balances(&user_supply_debt, &pool_map)
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

        fn convert_amount(
            amount_of_source: &FixedU128,
            price_of_source: &PriceValue,
            price_of_target: &PriceValue
        ) -> Result<FixedU128, CustomError> {
            Ok(price_of_source
                .checked_div(price_of_target)
                .ok_or(CustomError::FlownError)?
                .checked_mul(amount_of_source)
                .ok_or(CustomError::FlownError)?)
        }

        /// Check the floating-rate-pool's parameters
        /// TODO: we need to tighten some of the checks
        fn check_pool(pool: &Pool<T>) -> Result<(), InvalidParameters> {
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

        fn prefetch_for_liquidation_check(account: T::AccountId) -> Result<(BTreeMap<PoolId, PoolProxy<T>>, UserSupplyDebtData), CustomError> {
            // Prefetch all the needed data
            let user_debts = UserAccountUtil::<T>::get_debt_pools(account.clone());
            let user_supplies = UserAccountUtil::<T>::get_supply_pools(account.clone());
            let pools = PoolRepository::<T>::find_pools(&user_supplies, &user_debts)?;

            // Accrue interest for all
            for mut p in pools.values().cloned() {
                p.accrue_interest()?;
                PoolRepository::<T>::save(p);
            }
            let supply_debt_map = UserAccountUtil::<T>::accrue_interest_for_user(account.clone(), &pools)?;

            Ok((pools, supply_debt_map))
        }
    }
}