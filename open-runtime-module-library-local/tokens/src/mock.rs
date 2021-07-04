//! Mocks for the tokens module.

#![cfg(test)]

use super::*;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ChangeMembers, ContainsLengthBound, SaturatingCurrencyToVote, SortedMembers},
};
use orml_traits::parameter_type_with_key;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, AccountId32, Permill};
use sp_std::cell::RefCell;

pub type AccountId = AccountId32;
pub type CurrencyId = u32;
pub type Balance = u64;

pub const DOT: CurrencyId = 1;
pub const BTC: CurrencyId = 2;
pub const ETH: CurrencyId = 3;
pub const ALICE: AccountId = AccountId32::new([0u8; 32]);
pub const BOB: AccountId = AccountId32::new([1u8; 32]);
pub const CHARLIE: AccountId = AccountId32::new([2u8; 32]);
pub const TREASURY_ACCOUNT: AccountId = AccountId32::new([3u8; 32]);
pub const ID_1: LockIdentifier = *b"1       ";
pub const ID_2: LockIdentifier = *b"2       ";
pub const ID_3: LockIdentifier = *b"3       ";

use crate as tokens;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

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

thread_local! {
	static TEN_TO_FOURTEEN: RefCell<Vec<AccountId>> = RefCell::new(vec![
		AccountId32::new([10u8; 32]),
		AccountId32::new([11u8; 32]),
		AccountId32::new([12u8; 32]),
		AccountId32::new([13u8; 32]),
		AccountId32::new([14u8; 32]),
	]);
}

pub struct TenToFourteen;
impl SortedMembers<AccountId> for TenToFourteen {
	fn sorted_members() -> Vec<AccountId> {
		TEN_TO_FOURTEEN.with(|v| v.borrow().clone())
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn add(new: &AccountId) {
		TEN_TO_FOURTEEN.with(|v| {
			let mut members = v.borrow_mut();
			members.push(new.clone());
			members.sort();
		})
	}
}

impl ContainsLengthBound for TenToFourteen {
	fn max_len() -> usize {
		TEN_TO_FOURTEEN.with(|v| v.borrow().len())
	}
	fn min_len() -> usize {
		0
	}
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: u64 = 1;
	pub const SpendPeriod: u64 = 2;
	pub const Burn: Permill = Permill::from_percent(50);
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const GetTokenId: CurrencyId = DOT;
	pub const MaxApprovals: u32 = 100;
}

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryPalletId;
	type Currency = CurrencyAdapter<Runtime, GetTokenId>;
	type ApproveOrigin = frame_system::EnsureRoot<AccountId>;
	type RejectOrigin = frame_system::EnsureRoot<AccountId>;
	type Event = Event;
	type OnSlash = ();
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type SpendFunds = ();
	type WeightInfo = ();
	type MaxApprovals = MaxApprovals;
}

thread_local! {
	pub static MEMBERS: RefCell<Vec<AccountId>> = RefCell::new(vec![]);
	pub static PRIME: RefCell<Option<AccountId>> = RefCell::new(None);
}

pub struct TestChangeMembers;
impl ChangeMembers<AccountId> for TestChangeMembers {
	fn change_members_sorted(incoming: &[AccountId], outgoing: &[AccountId], new: &[AccountId]) {
		// new, incoming, outgoing must be sorted.
		let mut new_sorted = new.to_vec();
		new_sorted.sort();
		assert_eq!(new, &new_sorted[..]);

		let mut incoming_sorted = incoming.to_vec();
		incoming_sorted.sort();
		assert_eq!(incoming, &incoming_sorted[..]);

		let mut outgoing_sorted = outgoing.to_vec();
		outgoing_sorted.sort();
		assert_eq!(outgoing, &outgoing_sorted[..]);

		// incoming and outgoing must be disjoint
		for x in incoming.iter() {
			assert!(outgoing.binary_search(x).is_err());
		}

		let mut old_plus_incoming = MEMBERS.with(|m| m.borrow().to_vec());
		old_plus_incoming.extend_from_slice(incoming);
		old_plus_incoming.sort();

		let mut new_plus_outgoing = new.to_vec();
		new_plus_outgoing.extend_from_slice(outgoing);
		new_plus_outgoing.sort();

		assert_eq!(
			old_plus_incoming, new_plus_outgoing,
			"change members call is incorrect!"
		);

		MEMBERS.with(|m| *m.borrow_mut() = new.to_vec());
		PRIME.with(|p| *p.borrow_mut() = None);
	}

	fn set_prime(who: Option<AccountId>) {
		PRIME.with(|p| *p.borrow_mut() = who);
	}
}

parameter_types! {
	pub const ElectionsPhragmenPalletId: LockIdentifier = *b"phrelect";
	pub const CandidacyBond: u64 = 3;
	pub const VotingBond: u64 = 2;
	pub const DesiredMembers: u32 = 2;
	pub const DesiredRunnersUp: u32 = 2;
	pub const TermDuration: u64 = 5;
	pub const VotingBondBase: u64 = 2;
	pub const VotingBondFactor: u64 = 0;
}

impl pallet_elections_phragmen::Config for Runtime {
	type PalletId = ElectionsPhragmenPalletId;
	type Event = Event;
	type Currency = CurrencyAdapter<Runtime, GetTokenId>;
	type CurrencyToVote = SaturatingCurrencyToVote;
	type ChangeMembers = TestChangeMembers;
	type InitializeMembers = ();
	type CandidacyBond = CandidacyBond;
	type VotingBondBase = VotingBondBase;
	type VotingBondFactor = VotingBondFactor;
	type TermDuration = TermDuration;
	type DesiredMembers = DesiredMembers;
	type DesiredRunnersUp = DesiredRunnersUp;
	type LoserCandidate = ();
	type KickedMember = ();
	type WeightInfo = ();
}

parameter_type_with_key! {
	pub ExistentialDeposits: |currency_id: CurrencyId| -> Balance {
		match currency_id {
			&BTC => 1,
			&DOT => 2,
			_ => 0,
		}
	};
}

parameter_types! {
	pub DustAccount: AccountId = PalletId(*b"orml/dst").into_account();
	pub MaxLocks: u32 = 2;
}

impl Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = i64;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = TransferDust<Runtime, DustAccount>;
	type MaxLocks = MaxLocks;
}
pub type TreasuryCurrencyAdapter = <Runtime as pallet_treasury::Config>::Currency;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		Tokens: tokens::{Pallet, Storage, Event<T>, Config<T>},
		Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>},
		ElectionsPhragmen: pallet_elections_phragmen::{Pallet, Call, Storage, Event<T>},
	}
);

pub struct ExtBuilder {
	balances: Vec<(AccountId, CurrencyId, Balance)>,
	treasury_genesis: bool,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			balances: vec![],
			treasury_genesis: false,
		}
	}
}

impl ExtBuilder {
	pub fn balances(mut self, balances: Vec<(AccountId, CurrencyId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub fn one_hundred_for_alice_n_bob(self) -> Self {
		self.balances(vec![(ALICE, DOT, 100), (BOB, DOT, 100)])
	}

	pub fn one_hundred_for_treasury_account(mut self) -> Self {
		self.treasury_genesis = true;
		self.balances(vec![(TREASURY_ACCOUNT, DOT, 100)])
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		tokens::GenesisConfig::<Runtime> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		if self.treasury_genesis {
			pallet_treasury::GenesisConfig::default()
				.assimilate_storage::<Runtime, _>(&mut t)
				.unwrap();

			pallet_elections_phragmen::GenesisConfig::<Runtime> {
				members: vec![(TREASURY_ACCOUNT, 10)],
			}
			.assimilate_storage(&mut t)
			.unwrap();
		}

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
