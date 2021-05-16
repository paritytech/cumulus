use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use frame_support::{sp_runtime::{FixedU128}, Parameter};
use sp_std::{vec::Vec};
use rococo_parachain_primitives::{CurrencyId, PoolId, Overflown, PriceValue};
use frame_support::pallet_prelude::{Member, MaybeSerializeDeserialize};
use frame_support::sp_runtime::{FixedPointOperand, FixedPointNumber};
use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedMul, CheckedAdd, Saturating, One, Zero};
use sp_runtime::sp_std::convert::TryInto;


/// The floating-rate-pool for different lending transactions.
/// Each floating-rate-pool needs to be associated with a currency id.
#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug)]
pub struct Pool<T, Balance> where
    T: frame_system::Config,
    Balance: Member + Parameter + FixedPointOperand + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize,
{
    pub id: u64,
    pub name: Vec<u8>,
    pub currency_id: CurrencyId,

    /// indicates whether this floating-rate-pool can be used as collateral
    pub can_be_collateral: bool,
    /// This flag indicated whether this floating-rate-pool is enabled for transactions
    pub enabled: bool,

    /* --- Supply And Debt --- */
    pub supply: Balance,
    pub total_supply_index: FixedU128,
    pub debt: Balance,
    pub total_debt_index: FixedU128,

    /* ----- Parameters ----- */
    /// Minimum amount required for each transaction
    pub minimal_amount: Balance,
    /// When this is used as collateral, the value is multiplied by safe_factor
    pub safe_factor: FixedU128,
    /// Only a close_factor of the collateral can be liquidated at a time
    pub close_factor: FixedU128,
    /// The discount given to the arbitrageur
    pub discount_factor: FixedU128,
    /// The multiplier to the utilization ratio
    pub utilization_factor: FixedU128,
    /// The constant for interest rate
    pub initial_interest_rate: FixedU128,

    /* ----- Metadata Related ----- */
    /// The block number when this floating-rate-pool is last updated
    pub last_updated: T::BlockNumber,
    /// The account that lastly modified this floating-rate-pool
    pub last_updated_by: T::AccountId,
    /// The account that created this floating-rate-pool
    pub created_by: T::AccountId,
    /// The block number when this floating-rate-pool is created
    pub created_at: T::BlockNumber,
}

impl <T, Balance> Pool<T, Balance> where
    T: frame_system::Config,
    Balance: Member + Parameter + FixedPointOperand + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize,
{
    /// Creates a new floating-rate-pool from the input parameters
    pub fn new(id: PoolId,
               name: Vec<u8>,
               currency_id: CurrencyId,
               can_be_collateral: bool,
               safe_factor: FixedU128,
               close_factor: FixedU128,
               discount_factor: FixedU128,
               utilization_factor: FixedU128,
               initial_interest_rate: FixedU128,
               minimal_amount: Balance,
               owner: T::AccountId,
               block_number: T::BlockNumber,
    ) -> Pool<T, Balance>{
        Pool{
            id,
            name,
            currency_id,
            can_be_collateral,
            enabled: false,
            supply: Balance::zero(),
            total_supply_index: FixedU128::one(),
            debt: Balance::zero(),
            total_debt_index: FixedU128::one(),
            minimal_amount,
            safe_factor,
            close_factor,
            discount_factor,
            utilization_factor,
            initial_interest_rate,
            last_updated: block_number.clone(),
            last_updated_by: owner.clone(),
            created_by: owner.clone(),
            created_at: block_number.clone(),
        }
    }

    /// Accrue interest for the floating-rate-pool. The block_number is the block number when the floating-rate-pool is updated
    /// TODO: update and check all the overflow here
    pub fn accrue_interest(&mut self, block_number: T::BlockNumber) -> Result<(), Overflown>{
        // Not updating if the time is the same or lagging
        // TODO: maybe should raise exception if last_updated is greater than block_number
        if self.last_updated >= block_number {
            return Ok(());
        }


        // get time span
        let interval_block_number = block_number - self.last_updated;
        let elapsed_time_u32 = TryInto::<u32>::try_into(interval_block_number)
            .ok()
            .expect("blockchain will not exceed 2^32 blocks; qed");

        // get rates and calculate interest
        let s_rate = self.supply_interest_rate()?;
        let d_rate = self.debt_interest_rate()?;
        let supply_multiplier = FixedU128::one() + s_rate * FixedU128::saturating_from_integer(elapsed_time_u32);
        let debt_multiplier = FixedU128::one() + d_rate * FixedU128::saturating_from_integer(elapsed_time_u32);

        self.supply = supply_multiplier.saturating_mul_int(self.supply.clone());
        self.total_supply_index = self.total_supply_index * supply_multiplier;

        self.debt = debt_multiplier.saturating_mul_int(self.debt.clone());
        self.total_debt_index = self.total_debt_index * debt_multiplier;

        self.last_updated = block_number;

        Ok(())
    }

    /// Increment the supply of the pool
    pub fn increment_supply(&mut self, amount: Balance) { self.supply += amount; }

    /// Decrement the supply of the pool
    pub fn decrement_supply(&mut self, amount: Balance) { self.supply -= amount; }

    /// Increment the debt of the pool
    pub fn increment_debt(&mut self, amount: Balance) { self.debt += amount; }

    /// Decrement the debt of the pool
    pub fn decrement_debt(&mut self, amount: Balance) { self.debt -= amount; }

    /// The amount that can be close given the input
    /// TODO: handle dust values
    pub fn closable_amount(&self, amount: Balance) -> Balance { self.close_factor.saturating_mul_int(amount) }

    /// The discounted price of the pool given the current price of the currency
    pub fn discounted_price(&self, price: PriceValue) -> PriceValue { self.discount_factor.saturating_mul(price) }

    pub fn supply_interest_rate(&self) -> Result<FixedU128, Overflown> {
        if self.supply == Balance::zero() {
            return Ok(FixedU128::zero());
        }

        let utilization_ratio = FixedU128::saturating_from_rational(self.debt.clone(), self.supply.clone());
        self.debt_interest_rate()?.checked_mul(&utilization_ratio).ok_or(Overflown{})
    }

    pub fn debt_interest_rate(&self) -> Result<FixedU128, Overflown> {
        if self.supply == Balance::zero() {
            return Ok(self.initial_interest_rate.clone());
        }

        let utilization_ratio = FixedU128::saturating_from_rational(self.debt.clone(), self.supply.clone());
        let rate = self.utilization_factor.checked_mul(&utilization_ratio).ok_or(Overflown{})?;
        self.initial_interest_rate.checked_add(&rate).ok_or(Overflown{})
    }
}

pub struct UserBalanceStats<Balance>{
    /// The total supply balance of the user
    pub supply_balance: Balance,
    /// The total collateral balance of the user
    pub collateral_balance: Balance,
    /// The total debt balance of the user
    pub debt_balance: Balance,
}

impl <Balance> UserBalanceStats<Balance> where Balance: Clone + FixedPointOperand {
    pub fn is_liquidated(&self, liquidation_threshold: FixedU128) -> bool {
        self.collateral_balance < liquidation_threshold.saturating_mul_int(self.debt_balance.clone())
    }
}