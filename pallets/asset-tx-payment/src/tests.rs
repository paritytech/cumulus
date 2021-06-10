
use super::*;
use crate as pallet_asset_tx_payment;
use frame_system as system;
use frame_system::pallet_prelude::*;
use frame_system::EnsureRoot;
use codec::Encode;
use frame_support::{
	assert_noop, assert_ok, parameter_types,
	pallet_prelude::*,
	weights::{
		DispatchClass, DispatchInfo, PostDispatchInfo, GetDispatchInfo, Weight,
		WeightToFeePolynomial, WeightToFeeCoefficients, WeightToFeeCoefficient,
	},
	traits::{Currency, FindAuthor},
	ConsensusEngineId,
};
use pallet_balances::Call as BalancesCall;
use pallet_transaction_payment::CurrencyAdapter;
use sp_core::H256;
use sp_runtime::{Perbill, testing::{Header, TestXt}, traits::{BlakeTwo256, ConvertInto, IdentityLookup, One}, transaction_validity::InvalidTransaction};
use std::cell::RefCell;
use smallvec::smallvec;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;
type Balance = u64;
type AccountId = u64;

frame_support::construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage},
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
		Authorship: pallet_authorship::{Pallet, Call, Storage},
		AssetTxPayment: pallet_asset_tx_payment::{Pallet},
	}
);

const CALL: &<Runtime as frame_system::Config>::Call =
	&Call::Balances(BalancesCall::transfer(2, 69));

thread_local! {
	static EXTRINSIC_BASE_WEIGHT: RefCell<u64> = RefCell::new(0);
}

pub struct BlockWeights;
impl Get<frame_system::limits::BlockWeights> for BlockWeights {
	fn get() -> frame_system::limits::BlockWeights {
		frame_system::limits::BlockWeights::builder()
			.base_block(0)
			.for_class(DispatchClass::all(), |weights| {
				weights.base_extrinsic = EXTRINSIC_BASE_WEIGHT.with(|v| *v.borrow()).into();
			})
			.for_class(DispatchClass::non_mandatory(), |weights| {
				weights.max_total = 1024.into();
			})
			.build_or_panic()
	}
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub static TransactionByteFee: u64 = 1;
	pub static WeightToFee: u64 = 1;
}

impl frame_system::Config for Runtime {
	type BaseCallFilter = ();
	type BlockWeights = BlockWeights;
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Call = Call;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = ();
	type WeightInfo = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
}

impl WeightToFeePolynomial for WeightToFee {
	type Balance = u64;

	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		smallvec![WeightToFeeCoefficient {
			degree: 1,
			coeff_frac: Perbill::zero(),
			coeff_integer: WEIGHT_TO_FEE.with(|v| *v.borrow()),
			negative: false,
		}]
	}
}

impl pallet_transaction_payment::Config for Runtime {
	type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = WeightToFee;
	type FeeMultiplierUpdate = ();
}

parameter_types! {
	pub const AssetDeposit: u64 = 2;
	pub const MetadataDeposit: u64 = 0;
	pub const StringLimit: u32 = 20;
}

impl pallet_assets::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type AssetId = u32;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDeposit;
	type MetadataDepositPerByte = MetadataDeposit;
	type ApprovalDeposit = MetadataDeposit;
	type StringLimit = StringLimit;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = ();
}

pub struct HardcodedAuthor;
const BLOCK_AUTHOR: AccountId = 1234;
impl FindAuthor<AccountId> for HardcodedAuthor {
	fn find_author<'a, I>(_: I) -> Option<AccountId>
		where I: 'a + IntoIterator<Item=(ConsensusEngineId, &'a [u8])>
	{
		Some(BLOCK_AUTHOR)
	}
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = HardcodedAuthor;
	type UncleGenerations = ();
	type FilterUncle = ();
	type EventHandler = ();
}

impl Config for Runtime {
	type BalanceConversion = pallet_assets::BalanceToAssetBalance<Balances, Runtime, ConvertInto>;
	type Fungibles = Assets;
}

pub struct ExtBuilder {
	balance_factor: u64,
	base_weight: u64,
	byte_fee: u64,
	weight_to_fee: u64
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			balance_factor: 1,
			base_weight: 0,
			byte_fee: 1,
			weight_to_fee: 1,
		}
	}
}

impl ExtBuilder {
	pub fn base_weight(mut self, base_weight: u64) -> Self {
		self.base_weight = base_weight;
		self
	}
	pub fn byte_fee(mut self, byte_fee: u64) -> Self {
		self.byte_fee = byte_fee;
		self
	}
	pub fn weight_fee(mut self, weight_to_fee: u64) -> Self {
		self.weight_to_fee = weight_to_fee;
		self
	}
	pub fn balance_factor(mut self, factor: u64) -> Self {
		self.balance_factor = factor;
		self
	}
	fn set_constants(&self) {
		EXTRINSIC_BASE_WEIGHT.with(|v| *v.borrow_mut() = self.base_weight);
		TRANSACTION_BYTE_FEE.with(|v| *v.borrow_mut() = self.byte_fee);
		WEIGHT_TO_FEE.with(|v| *v.borrow_mut() = self.weight_to_fee);
	}
	pub fn build(self) -> sp_io::TestExternalities {
		self.set_constants();
		let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
		pallet_balances::GenesisConfig::<Runtime> {
			balances: if self.balance_factor > 0 {
				vec![
					(1, 10 * self.balance_factor),
					(2, 20 * self.balance_factor),
					(3, 30 * self.balance_factor),
					(4, 40 * self.balance_factor),
					(5, 50 * self.balance_factor),
					(6, 60 * self.balance_factor)
				]
			} else {
				vec![]
			},
		}.assimilate_storage(&mut t).unwrap();
		t.into()
	}
}

/// create a transaction info struct from weight. Handy to avoid building the whole struct.
pub fn info_from_weight(w: Weight) -> DispatchInfo {
	// pays_fee: Pays::Yes -- class: DispatchClass::Normal
	DispatchInfo { weight: w, ..Default::default() }
}

fn post_info_from_weight(w: Weight) -> PostDispatchInfo {
	PostDispatchInfo {
		actual_weight: Some(w),
		pays_fee: Default::default(),
	}
}

fn post_info_from_pays(p: Pays) -> PostDispatchInfo {
	PostDispatchInfo {
		actual_weight: None,
		pays_fee: p,
	}
}

fn default_post_info() -> PostDispatchInfo {
	PostDispatchInfo {
		actual_weight: None,
		pays_fee: Default::default(),
	}
}

#[test]
fn transaction_payment_in_native_possible() {
	ExtBuilder::default()
		.balance_factor(10)
		.base_weight(5)
		.build()
		.execute_with(||
	{
		let len = 10;
		let pre = ChargeAssetTxPayment::<Runtime>::from(0, None)
			.pre_dispatch(&1, CALL, &info_from_weight(5), len)
			.unwrap();
		assert_eq!(Balances::free_balance(1), 100 - 5 - 5 - 10);

		assert_ok!(
			ChargeAssetTxPayment::<Runtime>
				::post_dispatch(pre, &info_from_weight(5), &default_post_info(), len, &Ok(()))
		);
		assert_eq!(Balances::free_balance(1), 100 - 5 - 5 - 10);

		let pre = ChargeAssetTxPayment::<Runtime>::from(5 /* tipped */, None)
			.pre_dispatch(&2, CALL, &info_from_weight(100), len)
			.unwrap();
		assert_eq!(Balances::free_balance(2), 200 - 5 - 10 - 100 - 5);

		assert_ok!(
			ChargeAssetTxPayment::<Runtime>
				::post_dispatch(pre, &info_from_weight(100), &post_info_from_weight(50), len, &Ok(()))
		);
		assert_eq!(Balances::free_balance(2), 200 - 5 - 10 - 50 - 5);
	});
}

#[test]
fn transaction_payment_in_asset_possible() {
	ExtBuilder::default()
		.balance_factor(10)
		.base_weight(5)
		.build()
		.execute_with(||
	{
		use pallet_assets::Call as AssetsCall;
		use sp_runtime::traits::StaticLookup;
		let force_create = |asset_id, owner, is_sufficient, min_balance| {
			Call::Assets(AssetsCall::force_create(asset_id, owner, is_sufficient, min_balance))
				.dispatch(Origin::root())
		};
		let mint = |owner, asset_id, beneficiary, amount| {
			Call::Assets(AssetsCall::mint(asset_id, beneficiary, amount))
				.dispatch(Origin::signed(owner))
		};
		let len = 10;
		let asset_id = 1;
		let owner = 42;
		let min_balance = 2;
		let caller = 1;
		let beneficiary = <Runtime as system::Config>::Lookup::unlookup(caller);
		assert_ok!(force_create(asset_id, owner, true, min_balance));
		assert_ok!(mint(owner, asset_id, beneficiary, 100));
		assert_eq!(Assets::balance(asset_id, caller), 100);
		let pre = ChargeAssetTxPayment::<Runtime>::from(0, Some(asset_id))
			.pre_dispatch(&caller, CALL, &info_from_weight(5), len)
			.unwrap();
		assert_eq!(Balances::free_balance(caller), 100);
		assert_eq!(Assets::balance(asset_id, caller), 100 - 10 - 10 -20);

		assert_ok!(
			ChargeAssetTxPayment::<Runtime>
				::post_dispatch(pre, &info_from_weight(5), &default_post_info(), len, &Ok(()))
		);
		assert_eq!(Balances::free_balance(caller), 100);
		assert_eq!(Assets::balance(asset_id, caller), 100 - 10 - 10 -20);

		// let pre = ChargeAssetTxPayment::<Runtime>::from(5 /* tipped */, Some(asset_id))
		// 	.pre_dispatch(&2, CALL, &info_from_weight(100), len)
		// 	.unwrap();
		// assert_eq!(Balances::free_balance(2), 200 - 5 - 10 - 100 - 5);

		// assert_ok!(
		// 	ChargeAssetTxPayment::<Runtime>
		// 		::post_dispatch(pre, &info_from_weight(100), &post_info_from_weight(50), len, &Ok(()))
		// );
		// assert_eq!(Balances::free_balance(2), 200 - 5 - 10 - 50 - 5);
	});
}

// 	#[test]
// 	fn signed_extension_transaction_payment_multiplied_refund_works() {
// 		ExtBuilder::default()
// 			.balance_factor(10)
// 			.base_weight(5)
// 			.build()_
// 			.execute_with(||
// 		{
// 			let len = 10;
// 			NextFeeMultiplier::put(Multiplier::saturating_from_rational(3, 2));

// 			let pre = ChargeTransactionPayment::<Runtime>::from(5 /* tipped */)
// 				.pre_dispatch(&2, CALL, &info_from_weight(100), len)
// 				.unwrap();
// 			// 5 base fee, 10 byte fee, 3/2 * 100 weight fee, 5 tip
// 			assert_eq!(Balances::free_balance(2), 200 - 5 - 10 - 150 - 5);

// 			assert_ok!(
// 				ChargeTransactionPayment::<Runtime>
// 					::post_dispatch(pre, &info_from_weight(100), &post_info_from_weight(50), len, &Ok(()))
// 			);
// 			// 75 (3/2 of the returned 50 units of weight) is refunded
// 			assert_eq!(Balances::free_balance(2), 200 - 5 - 10 - 75 - 5);
// 		});
// 	}

// 	#[test]
// 	fn signed_extension_transaction_payment_is_bounded() {
// 		ExtBuilder::default()
// 			.balance_factor(1000)
// 			.byte_fee(0)
// 			.build()
// 			.execute_with(||
// 		{
// 			// maximum weight possible
// 			assert_ok!(
// 				ChargeTransactionPayment::<Runtime>::from(0)
// 					.pre_dispatch(&1, CALL, &info_from_weight(Weight::max_value()), 10)
// 			);
// 			// fee will be proportional to what is the actual maximum weight in the runtime.
// 			assert_eq!(
// 				Balances::free_balance(&1),
// 				(10000 - <Runtime as frame_system::Config>::BlockWeights::get().max_block) as u64
// 			);
// 		});
// 	}

// 	#[test]
// 	fn signed_extension_allows_free_transactions() {
// 		ExtBuilder::default()
// 			.base_weight(100)
// 			.balance_factor(0)
// 			.build()
// 			.execute_with(||
// 		{
// 			// 1 ain't have a penny.
// 			assert_eq!(Balances::free_balance(1), 0);

// 			let len = 100;

// 			// This is a completely free (and thus wholly insecure/DoS-ridden) transaction.
// 			let operational_transaction = DispatchInfo {
// 				weight: 0,
// 				class: DispatchClass::Operational,
// 				pays_fee: Pays::No,
// 			};
// 			assert_ok!(
// 				ChargeTransactionPayment::<Runtime>::from(0)
// 					.validate(&1, CALL, &operational_transaction , len)
// 			);

// 			// like a InsecureFreeNormal
// 			let free_transaction = DispatchInfo {
// 				weight: 0,
// 				class: DispatchClass::Normal,
// 				pays_fee: Pays::Yes,
// 			};
// 			assert_noop!(
// 				ChargeTransactionPayment::<Runtime>::from(0)
// 					.validate(&1, CALL, &free_transaction , len),
// 				TransactionValidityError::Invalid(InvalidTransaction::Payment),
// 			);
// 		});
// 	}

// 	#[test]
// 	fn signed_ext_length_fee_is_also_updated_per_congestion() {
// 		ExtBuilder::default()
// 			.base_weight(5)
// 			.balance_factor(10)
// 			.build()
// 			.execute_with(||
// 		{
// 			// all fees should be x1.5
// 			NextFeeMultiplier::put(Multiplier::saturating_from_rational(3, 2));
// 			let len = 10;

// 			assert_ok!(
// 				ChargeTransactionPayment::<Runtime>::from(10) // tipped
// 					.pre_dispatch(&1, CALL, &info_from_weight(3), len)
// 			);
// 			assert_eq!(
// 				Balances::free_balance(1),
// 				100 // original
// 				- 10 // tip
// 				- 5 // base
// 				- 10 // len
// 				- (3 * 3 / 2) // adjusted weight
// 			);
// 		})
// 	}

// 	#[test]
// 	fn query_info_works() {
// 		let call = Call::Balances(BalancesCall::transfer(2, 69));
// 		let origin = 111111;
// 		let extra = ();
// 		let xt = TestXt::new(call, Some((origin, extra)));
// 		let info  = xt.get_dispatch_info();
// 		let ext = xt.encode();
// 		let len = ext.len() as u32;
// 		ExtBuilder::default()
// 			.base_weight(5)
// 			.weight_fee(2)
// 			.build()
// 			.execute_with(||
// 		{
// 			// all fees should be x1.5
// 			NextFeeMultiplier::put(Multiplier::saturating_from_rational(3, 2));

// 			assert_eq!(
// 				TransactionPayment::query_info(xt, len),
// 				RuntimeDispatchInfo {
// 					weight: info.weight,
// 					class: info.class,
// 					partial_fee:
// 						5 * 2 /* base * weight_fee */
// 						+ len as u64  /* len * 1 */
// 						+ info.weight.min(BlockWeights::get().max_block) as u64 * 2 * 3 / 2 /* weight */
// 				},
// 			);

// 		});
// 	}

// 	#[test]
// 	fn compute_fee_works_without_multiplier() {
// 		ExtBuilder::default()
// 			.base_weight(100)
// 			.byte_fee(10)
// 			.balance_factor(0)
// 			.build()
// 			.execute_with(||
// 		{
// 			// Next fee multiplier is zero
// 			assert_eq!(NextFeeMultiplier::get(), Multiplier::one());

// 			// Tip only, no fees works
// 			let dispatch_info = DispatchInfo {
// 				weight: 0,
// 				class: DispatchClass::Operational,
// 				pays_fee: Pays::No,
// 			};
// 			assert_eq!(Module::<Runtime>::compute_fee(0, &dispatch_info, 10), 10);
// 			// No tip, only base fee works
// 			let dispatch_info = DispatchInfo {
// 				weight: 0,
// 				class: DispatchClass::Operational,
// 				pays_fee: Pays::Yes,
// 			};
// 			assert_eq!(Module::<Runtime>::compute_fee(0, &dispatch_info, 0), 100);
// 			// Tip + base fee works
// 			assert_eq!(Module::<Runtime>::compute_fee(0, &dispatch_info, 69), 169);
// 			// Len (byte fee) + base fee works
// 			assert_eq!(Module::<Runtime>::compute_fee(42, &dispatch_info, 0), 520);
// 			// Weight fee + base fee works
// 			let dispatch_info = DispatchInfo {
// 				weight: 1000,
// 				class: DispatchClass::Operational,
// 				pays_fee: Pays::Yes,
// 			};
// 			assert_eq!(Module::<Runtime>::compute_fee(0, &dispatch_info, 0), 1100);
// 		});
// 	}

// 	#[test]
// 	fn compute_fee_works_with_multiplier() {
// 		ExtBuilder::default()
// 			.base_weight(100)
// 			.byte_fee(10)
// 			.balance_factor(0)
// 			.build()
// 			.execute_with(||
// 		{
// 			// Add a next fee multiplier. Fees will be x3/2.
// 			NextFeeMultiplier::put(Multiplier::saturating_from_rational(3, 2));
// 			// Base fee is unaffected by multiplier
// 			let dispatch_info = DispatchInfo {
// 				weight: 0,
// 				class: DispatchClass::Operational,
// 				pays_fee: Pays::Yes,
// 			};
// 			assert_eq!(Module::<Runtime>::compute_fee(0, &dispatch_info, 0), 100);

// 			// Everything works together :)
// 			let dispatch_info = DispatchInfo {
// 				weight: 123,
// 				class: DispatchClass::Operational,
// 				pays_fee: Pays::Yes,
// 			};
// 			// 123 weight, 456 length, 100 base
// 			assert_eq!(
// 				Module::<Runtime>::compute_fee(456, &dispatch_info, 789),
// 				100 + (3 * 123 / 2) + 4560 + 789,
// 			);
// 		});
// 	}

// 	#[test]
// 	fn compute_fee_works_with_negative_multiplier() {
// 		ExtBuilder::default()
// 			.base_weight(100)
// 			.byte_fee(10)
// 			.balance_factor(0)
// 			.build()
// 			.execute_with(||
// 		{
// 			// Add a next fee multiplier. All fees will be x1/2.
// 			NextFeeMultiplier::put(Multiplier::saturating_from_rational(1, 2));

// 			// Base fee is unaffected by multiplier.
// 			let dispatch_info = DispatchInfo {
// 				weight: 0,
// 				class: DispatchClass::Operational,
// 				pays_fee: Pays::Yes,
// 			};
// 			assert_eq!(Module::<Runtime>::compute_fee(0, &dispatch_info, 0), 100);

// 			// Everything works together.
// 			let dispatch_info = DispatchInfo {
// 				weight: 123,
// 				class: DispatchClass::Operational,
// 				pays_fee: Pays::Yes,
// 			};
// 			// 123 weight, 456 length, 100 base
// 			assert_eq!(
// 				Module::<Runtime>::compute_fee(456, &dispatch_info, 789),
// 				100 + (123 / 2) + 4560 + 789,
// 			);
// 		});
// 	}

// 	#[test]
// 	fn compute_fee_does_not_overflow() {
// 		ExtBuilder::default()
// 			.base_weight(100)
// 			.byte_fee(10)
// 			.balance_factor(0)
// 			.build()
// 			.execute_with(||
// 		{
// 			// Overflow is handled
// 			let dispatch_info = DispatchInfo {
// 				weight: Weight::max_value(),
// 				class: DispatchClass::Operational,
// 				pays_fee: Pays::Yes,
// 			};
// 			assert_eq!(
// 				Module::<Runtime>::compute_fee(
// 					<u32>::max_value(),
// 					&dispatch_info,
// 					<u64>::max_value()
// 				),
// 				<u64>::max_value()
// 			);
// 		});
// 	}

// 	#[test]
// 	fn refund_does_not_recreate_account() {
// 		ExtBuilder::default()
// 			.balance_factor(10)
// 			.base_weight(5)
// 			.build()
// 			.execute_with(||
// 		{
// 			// So events are emitted
// 			System::set_block_number(10);
// 			let len = 10;
// 			let pre = ChargeTransactionPayment::<Runtime>::from(5 /* tipped */)
// 				.pre_dispatch(&2, CALL, &info_from_weight(100), len)
// 				.unwrap();
// 			assert_eq!(Balances::free_balance(2), 200 - 5 - 10 - 100 - 5);

// 			// kill the account between pre and post dispatch
// 			assert_ok!(Balances::transfer(Some(2).into(), 3, Balances::free_balance(2)));
// 			assert_eq!(Balances::free_balance(2), 0);

// 			assert_ok!(
// 				ChargeTransactionPayment::<Runtime>
// 					::post_dispatch(pre, &info_from_weight(100), &post_info_from_weight(50), len, &Ok(()))
// 			);
// 			assert_eq!(Balances::free_balance(2), 0);
// 			// Transfer Event
// 			assert!(System::events().iter().any(|event| {
// 				event.event == Event::pallet_balances(pallet_balances::Event::Transfer(2, 3, 80))
// 			}));
// 			// Killed Event
// 			assert!(System::events().iter().any(|event| {
// 				event.event == Event::system(system::Event::KilledAccount(2))
// 			}));
// 		});
// 	}

// 	#[test]
// 	fn actual_weight_higher_than_max_refunds_nothing() {
// 		ExtBuilder::default()
// 			.balance_factor(10)
// 			.base_weight(5)
// 			.build()
// 			.execute_with(||
// 		{
// 			let len = 10;
// 			let pre = ChargeTransactionPayment::<Runtime>::from(5 /* tipped */)
// 				.pre_dispatch(&2, CALL, &info_from_weight(100), len)
// 				.unwrap();
// 			assert_eq!(Balances::free_balance(2), 200 - 5 - 10 - 100 - 5);

// 			assert_ok!(
// 				ChargeTransactionPayment::<Runtime>
// 					::post_dispatch(pre, &info_from_weight(100), &post_info_from_weight(101), len, &Ok(()))
// 			);
// 			assert_eq!(Balances::free_balance(2), 200 - 5 - 10 - 100 - 5);
// 		});
// 	}

// 	#[test]
// 	fn zero_transfer_on_free_transaction() {
// 		ExtBuilder::default()
// 			.balance_factor(10)
// 			.base_weight(5)
// 			.build()
// 			.execute_with(||
// 		{
// 			// So events are emitted
// 			System::set_block_number(10);
// 			let len = 10;
// 			let dispatch_info = DispatchInfo {
// 				weight: 100,
// 				pays_fee: Pays::No,
// 				class: DispatchClass::Normal,
// 			};
// 			let user = 69;
// 			let pre = ChargeTransactionPayment::<Runtime>::from(0)
// 				.pre_dispatch(&user, CALL, &dispatch_info, len)
// 				.unwrap();
// 			assert_eq!(Balances::total_balance(&user), 0);
// 			assert_ok!(
// 				ChargeTransactionPayment::<Runtime>
// 					::post_dispatch(pre, &dispatch_info, &default_post_info(), len, &Ok(()))
// 			);
// 			assert_eq!(Balances::total_balance(&user), 0);
// 			// No events for such a scenario
// 			assert_eq!(System::events().len(), 0);
// 		});
// 	}

// 	#[test]
// 	fn refund_consistent_with_actual_weight() {
// 		ExtBuilder::default()
// 			.balance_factor(10)
// 			.base_weight(7)
// 			.build()
// 			.execute_with(||
// 		{
// 			let info = info_from_weight(100);
// 			let post_info = post_info_from_weight(33);
// 			let prev_balance = Balances::free_balance(2);
// 			let len = 10;
// 			let tip = 5;

// 			NextFeeMultiplier::put(Multiplier::saturating_from_rational(5, 4));

// 			let pre = ChargeTransactionPayment::<Runtime>::from(tip)
// 				.pre_dispatch(&2, CALL, &info, len)
// 				.unwrap();

// 			ChargeTransactionPayment::<Runtime>
// 				::post_dispatch(pre, &info, &post_info, len, &Ok(()))
// 				.unwrap();

// 			let refund_based_fee = prev_balance - Balances::free_balance(2);
// 			let actual_fee = Module::<Runtime>
// 				::compute_actual_fee(len as u32, &info, &post_info, tip);

// 			// 33 weight, 10 length, 7 base, 5 tip
// 			assert_eq!(actual_fee, 7 + 10 + (33 * 5 / 4) + 5);
// 			assert_eq!(refund_based_fee, actual_fee);
// 		});
// 	}

// 	#[test]
// 	fn post_info_can_change_pays_fee() {
// 		ExtBuilder::default()
// 			.balance_factor(10)
// 			.base_weight(7)
// 			.build()
// 			.execute_with(||
// 		{
// 			let info = info_from_weight(100);
// 			let post_info = post_info_from_pays(Pays::No);
// 			let prev_balance = Balances::free_balance(2);
// 			let len = 10;
// 			let tip = 5;

// 			NextFeeMultiplier::put(Multiplier::saturating_from_rational(5, 4));

// 			let pre = ChargeTransactionPayment::<Runtime>::from(tip)
// 				.pre_dispatch(&2, CALL, &info, len)
// 				.unwrap();

// 			ChargeTransactionPayment::<Runtime>
// 				::post_dispatch(pre, &info, &post_info, len, &Ok(()))
// 				.unwrap();

// 			let refund_based_fee = prev_balance - Balances::free_balance(2);
// 			let actual_fee = Module::<Runtime>
// 				::compute_actual_fee(len as u32, &info, &post_info, tip);

// 			// Only 5 tip is paid
// 			assert_eq!(actual_fee, 5);
// 			assert_eq!(refund_based_fee, actual_fee);
// 		});
// 	}
// }
