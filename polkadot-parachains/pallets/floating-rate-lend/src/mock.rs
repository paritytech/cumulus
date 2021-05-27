//! Mocks for the floating rate lend module.

#![cfg(test)]

use super::*;
use frame_support::{construct_runtime, parameter_types};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::{IdentityLookup}, AccountId32, FixedU128, FixedPointNumber};

use crate as pallet_floating_rate_lend;
use polkadot_parachain_primitives::{Price, BALANCE_ONE};
use pallet_traits::{MultiCurrency, PriceProvider};
use frame_support::sp_runtime::traits::{One};
use frame_support::dispatch::DispatchResult;
use sp_runtime::traits::{Zero, Convert};
use frame_support::traits::GenesisBuild;
use sp_std::convert::TryInto;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

pub type AccountId = AccountId32;
impl frame_system::Config for Runtime {
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type BlockWeights = ();
    type BlockLength = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
}

// for sudo
impl pallet_sudo::Config for Runtime {
    type Call = Call;
    type Event = Event;
}

type CurrencyId = u32;
type Balance = u128;

pub struct MockMultiCurrency;
impl MultiCurrency<AccountId> for MockMultiCurrency {
    type CurrencyId = CurrencyId;
    type Balance = Balance;

    fn minimum_balance(_currency_id: Self::CurrencyId) -> Self::Balance {
        Balance::zero()
    }

    fn total_issuance(_currency_id: Self::CurrencyId) -> Self::Balance {
        Balance::one()
    }

    fn total_balance(_currency_id: Self::CurrencyId, _who: &AccountId) -> Self::Balance {
        Balance::one()
    }

    fn free_balance(_currency_id: Self::CurrencyId, _who: &AccountId) -> Self::Balance {
        Balance::zero()
    }

    fn ensure_can_withdraw(_currency_id: Self::CurrencyId, _who: &AccountId, _amount: Self::Balance) -> DispatchResult {
        Ok(())
    }

    fn transfer(_currency_id: Self::CurrencyId, _from: &AccountId, _to: &AccountId, _amount: Self::Balance) -> DispatchResult {
        Ok(())
    }

    fn deposit(_currency_id: Self::CurrencyId, _who: &AccountId, _amount: Self::Balance) -> DispatchResult {
        Ok(())
    }

    fn withdraw(_currency_id: Self::CurrencyId, _who: &AccountId, _amount: Self::Balance) -> DispatchResult {
        Ok(())
    }

    fn can_slash(_currency_id: Self::CurrencyId, _who: &AccountId, _value: Self::Balance) -> bool {
        false
    }

    fn slash(_currency_id: Self::CurrencyId, _who: &AccountId, _amount: Self::Balance) -> Self::Balance {
        Balance::zero()
    }
}

pub struct MockPriceProvider;
impl PriceProvider<Runtime> for MockPriceProvider {
    type CurrencyId = CurrencyId;

    fn price(_currency_id: Self::CurrencyId) -> Price<Runtime> {
        let default_price = FixedU128::from_float(1.2);
        Price::new(default_price, 1)
    }
}

pub struct Conversion;
impl Convert<Balance, FixedU128> for Conversion {
    fn convert(a: Balance) -> FixedU128 {
        let accuracy = FixedU128::accuracy() as u128;
        FixedU128::from_inner(a as u128 * (accuracy / BALANCE_ONE))
    }
}

impl Convert<FixedU128, Balance> for Conversion {
    fn convert(a: FixedU128) -> Balance {
        let accuracy = FixedU128::accuracy();
        (a.into_inner() / (accuracy / BALANCE_ONE)).try_into().unwrap()
    }
}

pub const CURRENCY_A: CurrencyId = 1;
pub const CURRENCY_B: CurrencyId = 2;

impl Config for Runtime {
    type Event = Event;
    type MultiCurrency = MockMultiCurrency;
    type PriceProvider = MockPriceProvider;
    type Conversion = Conversion;
}

// Runtime construction
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		FloatingRateLend: pallet_floating_rate_lend::{Pallet, Call, Storage, Event<T>} = 15,
		Sudo: pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

pub const ROOT: AccountId = AccountId32::new([1u8; 32]);

pub struct ExtBuilder {
    key: AccountId,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            key: ROOT,
        }
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        pallet_sudo::GenesisConfig::<Runtime> {
            key: self.key
        }
            .assimilate_storage(&mut t)
            .unwrap();
        t.into()
    }
}
