use codec::{Decode, Encode};
use sp_runtime::{RuntimeDebug, FixedU128, FixedPointNumber};
use frame_system::Config;
use sp_std::{vec::Vec};
use frame_support::sp_runtime::traits::Zero;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/* ---------- Price ------------ */
pub type PriceValue = FixedU128;

/// The price type used by all Pallets.
/// Please note that this struct is IMMUTABLE.
#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug)]
pub struct Price<T> where T: Config {
    price: PriceValue,
    updated_at: <T as Config>::BlockNumber,
}

impl <T> Price<T> where T: Config {
    /// Create a new price struct instance
    pub fn new(val: PriceValue, block_number: <T as Config>::BlockNumber) -> Price<T> {
        Price {
            price: val,
            updated_at: block_number,
        }
    }

    pub fn unready_price() -> Self {
        Price {
            price: PriceValue::zero(),
            updated_at: <T as Config>::BlockNumber::zero(),
        }
    }

    /// Checks if the price is ready to be used
    /// TODO: check the current block number is more than a certain
    /// TODO: threshold of the last updated block number
    pub fn price_ready(&self) -> bool {
        self.price.is_positive()
    }

    /// Get the value of the price
    pub fn value(&self) -> PriceValue {
        self.price
    }
}

/* ---------- errors Related ----------- */
pub type PoolId = u64;

/* ---------- Currency Related ------------ */
pub type CurrencyId = u64;

/// The representation of a currency in a multi-currency system
#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Currency {
    /// Basic
    Basic    {name: Vec<u8>, id: u64, address: Vec<u8>},
    /// Native DOT currency
    Polkadot {name: Vec<u8>, id: u64, address: Vec<u8>},
    /// ETH
    Erc20    {name: Vec<u8>, id: u64, address: Vec<u8>},
}

impl Currency {
    pub fn new(num: u64, id: u64, name: Vec<u8>, address: Vec<u8>) -> Currency {
        match num {
            1 => Currency::Polkadot{name, id, address},
            2 => Currency::Erc20{name, id, address},
            _ => Currency::Basic{name, id, address},
        }
    }
    pub fn is_polkadot_currency(&self) -> bool {
        match self {
            Currency::Polkadot {..} => true,
            _ => false
        }
    }

    pub fn is_erc20_currency(&self) -> bool {
        match self {
            Currency::Erc20 {..} => true,
            _ => false
        }
    }

    pub fn is_basic_currency(&self) -> bool {
        match self {
            Currency::Basic {..} => true,
            _ => false
        }
    }
}
