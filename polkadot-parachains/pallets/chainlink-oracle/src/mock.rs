// //! Mocks for the chainlink oracle module.
//
// #![cfg(test)]
//
// use frame_support::{construct_runtime, parameter_types, PalletId};
// use frame_support::traits::GenesisBuild;
//
// use pallet_chainlink_feed;
// use pallet_balances;
// use crate as pallet_chainlink_oracle;
// use crate::Config;
// use sp_runtime::traits::{Convert};
// use sp_runtime::{AccountId32, testing::Header, traits::IdentityLookup};
// use pallet_chainlink_feed::{FeedOracle, Feed, FeedConfig};
// use sp_core::H256;
// use sp_core::sp_std::marker;
//
// parameter_types! {
// 	pub const BlockHashCount: u64 = 250;
// }
//
// pub type AccountId = AccountId32;
// impl frame_system::Config for Runtime {
//     type Origin = Origin;
//     type Call = Call;
//     type Index = u64;
//     type BlockNumber = u64;
//     type Hash = H256;
//     type Hashing = ::sp_runtime::traits::BlakeTwo256;
//     type AccountId = AccountId;
//     type Lookup = IdentityLookup<Self::AccountId>;
//     type Header = Header;
//     type Event = ();
//     type BlockHashCount = BlockHashCount;
//     type BlockWeights = ();
//     type BlockLength = ();
//     type Version = ();
//     type PalletInfo = PalletInfo;
//     type AccountData = ();
//     type OnNewAccount = ();
//     type OnKilledAccount = ();
//     type DbWeight = ();
//     type BaseCallFilter = ();
//     type SystemWeightInfo = ();
//     type SS58Prefix = ();
//     type OnSetCode = ();
// }
//
// type Balance = u64;
//
// parameter_types! {
// 	pub const ExistentialDeposit: u64 = 1;
// }
//
// impl pallet_balances::Config for Runtime {
//     type Balance = Balance;
//     type DustRemoval = ();
//     type Event = Event;
//     type ExistentialDeposit = ExistentialDeposit;
//     type AccountStore = frame_system::Pallet<Runtime>;
//     type MaxLocks = ();
//     type WeightInfo = ();
// }
//
// pub type FeedId = u64;
// pub type CurrencyId = u64;
// pub type Value = u128;
//
// pub struct Convertor;
// impl Convert<CurrencyId, Option<FeedId>> for Convertor {
//     fn convert(_: u64) -> Option<u64> {
//         Some(0)
//     }
// }
//
// pub struct Oracle<T>(marker::PhantomData<T>);
// impl <T: pallet_chainlink_feed::Config> FeedOracle<T> for Oracle<T> {
//     type FeedId = FeedId;
//     type Feed = Feed<T>;
//     type MutableFeed = ();
//
//     fn feed(id: Self::FeedId) -> Option<Self::Feed> {
//         let config = FeedConfig{
//             owner: (),
//             pending_owner: None,
//             submission_value_bounds: ((), ()),
//             submission_count_bounds: (0, 0),
//             payment: (),
//             timeout: (),
//             decimals: 0,
//             description: vec![],
//             restart_delay: 0,
//             reporting_round: 0,
//             latest_round: 1,
//             first_valid_round: None,
//             oracle_count: 0,
//             pruning_window: 0,
//             next_round_to_prune: 0,
//             debt: (),
//             max_debt: None
//         };
//         Some(Feed::new(id, config))
//     }
//
//     fn feed_mut(id: Self::FeedId) -> Option<Self::MutableFeed> {
//         unimplemented!()
//     }
// }
//
// parameter_types! {
// 	pub const FeedPalletId: PalletId = PalletId(*b"linkfeed");
// 	pub const MinimumReserve: Balance = ExistentialDeposit::get() * 1000;
// 	pub const OracleCountLimit: u32 = 25;
// 	pub const FeedLimit: FeedId = 100;
// 	pub const StringLimit: u32 = 50;
// }
//
// impl pallet_chainlink_feed::Config for Runtime {
//     type Event = Event;
//     type FeedId = FeedId;
//     type Value = Value;
//     type Currency = PalletBalances;
//     type PalletId = FeedPalletId;
//     type MinimumReserve = MinimumReserve;
//     type StringLimit = StringLimit;
//     type OnAnswerHandler = ();
//     type OracleCountLimit = OracleCountLimit;
//     type FeedLimit = FeedLimit;
//     type WeightInfo = ();
// }
//
// impl Config for Runtime {
//     type Oracle = Oracle<Runtime>;
//     type CurrencyFeedConvertor = Convertor;
// }
//
// // Runtime construction
// type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
// type Block = frame_system::mocking::MockBlock<Runtime>;
//
// construct_runtime!(
// 	pub enum Runtime where
// 		Block = Block,
// 		NodeBlock = Block,
// 		UncheckedExtrinsic = UncheckedExtrinsic,
// 	{
// 		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
// 		ChainlinkOracle: pallet_chainlink_oracle::{Pallet, Call, Storage},
// 		ChainlinkFeed: pallet_chainlink_feed::{Pallet, Call, Storage, Config<T>, Event<T>},
// 		PalletBalances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
// 	}
// );
//
// pub struct ExtBuilder {
//     key: AccountId,
// }
//
// impl ExtBuilder {
//     pub fn build(self) -> sp_io::TestExternalities {
//         let mut t = frame_system::GenesisConfig::default()
//             .build_storage::<Runtime>()
//             .unwrap();
//         t.into()
//     }
// }
