use frame_support::{
	assert_noop, assert_ok,
	traits::{GenesisBuild, PalletInfo},
	weights::WeightToFee as WeightToFeeT,
};
use codec::Encode;
use sp_runtime::{Digest, DigestItem};
use parachains_common::{AccountId, Balance, StatemintAuraId as AuraId};
use sp_consensus_aura::AURA_ENGINE_ID;
pub use statemint_runtime::{
	constants::fee::WeightToFee, xcm_config::XcmConfig, Assets, Balances, ExistentialDeposit,
	Runtime, SessionKeys, System,
};
use xcm::latest::prelude::*;
use xcm_executor::traits::WeightTrader;

pub struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
	// collators to test block prod
	collators: Vec<AccountId>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder { balances: vec![], collators: vec![] }
	}
}
impl ExtBuilder {
	pub fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}
	pub fn with_collators(mut self, collators: Vec<AccountId>) -> Self {
		self.collators = collators;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

		pallet_balances::GenesisConfig::<Runtime> { balances: self.balances }
			.assimilate_storage(&mut t)
			.unwrap();

		pallet_collator_selection::GenesisConfig::<Runtime> {
			invulnerables: self.collators.clone(),
			candidacy_bond: Default::default(),
			desired_candidates: Default::default(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let keys: Vec<(AccountId, AccountId, SessionKeys)> = self
			.collators
			.iter()
			.map(|account| {
				let bytearray: &[u8; 32] = account.as_ref();
				(
					account.clone(),
					account.clone(),
					SessionKeys {
						aura: AuraId::from(sp_core::ed25519::Public::from_raw(*bytearray)),
					},
				)
			})
			.collect();
		pallet_session::GenesisConfig::<Runtime> { keys }
			.assimilate_storage(&mut t)
			.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);

		ext.execute_with(|| {
			System::set_block_number(1);
		});

		ext
	}
}

/// Utility function that advances the chain to the desired block number.
/// If an author is provided, that author information is injected to all the blocks in the meantime.
pub fn run_to_block(n: u32, author: Option<AccountId>) {
	while System::block_number() < n {
		// Set the new block number and author
		match author {
			Some(ref author) => {
				let pre_digest =
					Digest { logs: vec![DigestItem::PreRuntime(AURA_ENGINE_ID, author.encode())] };
				System::reset_events();
				System::initialize(
					&(System::block_number() + 1),
					&System::parent_hash(),
					&pre_digest,
				);
			},
			None => {
				System::set_block_number(System::block_number() + 1);
			},
		}
	}
}

pub const ALICE: [u8; 32] = [1u8; 32];

pub fn root_origin() -> <Runtime as frame_system::Config>::Origin {
	<Runtime as frame_system::Config>::Origin::root()
}

pub fn origin_of(account_id: AccountId) -> <Runtime as frame_system::Config>::Origin {
	<Runtime as frame_system::Config>::Origin::signed(account_id)
}

#[test]
fn test_asset_xcm_trader_does_not_work_in_statemine() {
	ExtBuilder::default()
		.with_collators(vec![AccountId::from(ALICE)])
		.build()
		.execute_with(|| {
			// We need root origin to create a sufficient asset
			// We set existential deposit to be identical to the one for Balances first
			assert_ok!(Assets::force_create(
				root_origin(),
				1,
				AccountId::from(ALICE).into(),
				true,
				ExistentialDeposit::get()
			));

			let mut trader = <XcmConfig as xcm_executor::Config>::Trader::new();

			// Set Alice as block author, who will receive fees
			run_to_block(2, Some(AccountId::from(ALICE)));

			// We are going to buy 400e9 weight
			// Because of the ED being higher in statemine
			// and not to complicate things, we use a little
			// bit more of weight
			let bought = 400_000_000_000u64;

			// lets calculate amount needed
			let amount_needed = WeightToFee::weight_to_fee(&bought);

			let asset_multilocation = MultiLocation::new(
				0,
				X2(
					PalletInstance(
						<Runtime as frame_system::Config>::PalletInfo::index::<Assets>().unwrap()
							as u8,
					),
					GeneralIndex(1),
				),
			);

			let asset: MultiAsset = (asset_multilocation, amount_needed).into();

			// Buy weight should return an error, since asset trader not installed
			assert_noop!(trader.buy_weight(bought, asset.into()), XcmError::TooExpensive);

			// not credited since the ED is higher than this value
			assert_eq!(Assets::balance(1, AccountId::from(ALICE)), 0);

			// We also need to ensure the total supply did not increase
			assert_eq!(Assets::total_supply(1), 0);
		});
}
