use crate::{Trait, Module};
use sp_core::H256;
use frame_support::{impl_outer_origin, parameter_types, weights::Weight};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
};
use frame_system as system;
use pallet_assets as assets;

impl_outer_origin! {
	pub enum Origin for Test {}
}

type Balance = u128;
type AccountId = u64;
type AssetId = u64;

// Configure a mock runtime to test the pallet.
#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl system::Trait for Test {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Call = ();
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type PalletInfo = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

impl assets::Trait for Test {
    type Event = ();
    /// The units in which we record balances.
    type Balance = Balance;
    /// The arithmetic type of asset identifier.
    type AssetId = AssetId;
}

impl Trait for Test {
	type Balance = Balance;
    /// The arithmetic type of asset identifier.
    type AssetId =  AssetId;

    type Event = ();

    type Oracle = assets::Module<Test>;

    type MultiAsset = assets::Module<Test>;

}

pub type System = system::Module<Test>;
pub type Assets = assets::Module<Test>;
pub type Lending = Module<Test>;

pub struct ExtBuilder {
	assets: Vec<(AccountId, Balance, u64)>,
	owner: AccountId,
	pools: Vec<AssetId>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			assets: vec![
				(1, 1000_000_000_000_000_000, 100),
				(1, 1000000000000000000, 60),
			],
			owner: 2,
			pools: vec![0, 1],
		}
	}
}

// Build genesis storage according to the mock runtime.
impl ExtBuilder {
	// builds genesis config
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t =  system::GenesisConfig::default().build_storage::<Test>().unwrap();

		pallet_assets::GenesisConfig::<Test> {
			owner: self.owner,
			assets: self.assets,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		crate::GenesisConfig::<Test> {
			pools: self.pools,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()

	}
}