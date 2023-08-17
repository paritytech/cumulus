// Copyright Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Cumulus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

//! Test setup and helpers.

#![cfg(test)]

use super::*;

use codec::Encode;
use cumulus_primitives_core::{
	relay_chain::BlockNumber as RelayBlockNumber, InboundDownwardMessage, InboundHrmpMessage,
	PersistedValidationData,
};
use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;
use frame_support::{
	dispatch::UnfilteredDispatchable,
	inherent::{InherentData, ProvideInherent},
	parameter_types,
	traits::{OnFinalize, OnInitialize, ProcessMessage, ProcessMessageError},
	weights::{Weight, WeightMeter},
};
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};
use sp_version::RuntimeVersion;
use std::cell::RefCell;

use crate as parachain_system;

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test {
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		ParachainSystem: parachain_system::{Pallet, Call, Config<T>, Storage, Inherent, Event<T>, ValidateUnsigned},
		MessageQueue: pallet_message_queue::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub Version: RuntimeVersion = RuntimeVersion {
		spec_name: sp_version::create_runtime_str!("test"),
		impl_name: sp_version::create_runtime_str!("system-test"),
		authoring_version: 1,
		spec_version: 1,
		impl_version: 1,
		apis: sp_version::create_apis_vec!([]),
		transaction_version: 1,
		state_version: 1,
	};
	pub const ParachainId: ParaId = ParaId::new(200);
	pub const ReservedXcmpWeight: Weight = Weight::zero();
	pub const ReservedDmpWeight: Weight = Weight::zero();
}

impl frame_system::Config for Test {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type BlockLength = ();
	type BlockWeights = ();
	type Block = Block;
	type Version = Version;
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = frame_support::traits::Everything;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ParachainSetCode<Self>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl Config for Test {
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId = ParachainId;
	type OutboundXcmpMessageSource = FromThreadLocal;
	type DmpQueue = frame_support::traits::EnqueueWithOrigin<MessageQueue, RelayOrigin>;
	type ReservedDmpWeight = ReservedDmpWeight;
	type XcmpMessageHandler = SaveIntoThreadLocal;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
}

parameter_types! {
	pub const MaxWeight: Weight = Weight::MAX;
}

impl pallet_message_queue::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	#[cfg(feature = "runtime-benchmarks")]
	type MessageProcessor =
		pallet_message_queue::mock_helpers::NoopMessageProcessor<AggregateMessageOrigin>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type MessageProcessor = SaveIntoThreadLocal;
	type Size = u32;
	type QueueChangeHandler = ();
	type QueuePausedQuery = ();
	type HeapSize = sp_core::ConstU32<{ 64 * 1024 }>;
	type MaxStale = sp_core::ConstU32<8>;
	type ServiceWeight = MaxWeight;
	type WeightInfo = ();
}

pub struct FromThreadLocal;
pub struct SaveIntoThreadLocal;

std::thread_local! {
	pub static HANDLED_DMP_MESSAGES: RefCell<Vec<Vec<u8>>> = RefCell::new(Vec::new());
	pub static HANDLED_XCMP_MESSAGES: RefCell<Vec<(ParaId, relay_chain::BlockNumber, Vec<u8>)>> = RefCell::new(Vec::new());
	pub static SENT_MESSAGES: RefCell<Vec<(ParaId, Vec<u8>)>> = RefCell::new(Vec::new());
}

pub fn send_message(dest: ParaId, message: Vec<u8>) {
	SENT_MESSAGES.with(|m| m.borrow_mut().push((dest, message)));
}

impl XcmpMessageSource for FromThreadLocal {
	fn take_outbound_messages(maximum_channels: usize) -> Vec<(ParaId, Vec<u8>)> {
		let mut ids = std::collections::BTreeSet::<ParaId>::new();
		let mut taken = 0;
		let mut result = Vec::new();
		SENT_MESSAGES.with(|ms| {
			ms.borrow_mut().retain(|m| {
				let status = <Pallet<Test> as GetChannelInfo>::get_channel_status(m.0);
				let ready = matches!(status, ChannelStatus::Ready(..));
				if ready && !ids.contains(&m.0) && taken < maximum_channels {
					ids.insert(m.0);
					taken += 1;
					result.push(m.clone());
					false
				} else {
					true
				}
			})
		});
		result
	}
}

impl ProcessMessage for SaveIntoThreadLocal {
	type Origin = AggregateMessageOrigin;

	fn process_message(
		message: &[u8],
		origin: Self::Origin,
		_meter: &mut WeightMeter,
		_id: &mut [u8; 32],
	) -> Result<bool, ProcessMessageError> {
		assert_eq!(origin, Self::Origin::Parent);

		HANDLED_DMP_MESSAGES.with(|m| {
			m.borrow_mut().push(message.to_vec());
			Weight::zero()
		});
		Ok(true)
	}
}

impl XcmpMessageHandler for SaveIntoThreadLocal {
	fn handle_xcmp_messages<'a, I: Iterator<Item = (ParaId, RelayBlockNumber, &'a [u8])>>(
		iter: I,
		_max_weight: Weight,
	) -> Weight {
		HANDLED_XCMP_MESSAGES.with(|m| {
			for (sender, sent_at, message) in iter {
				m.borrow_mut().push((sender, sent_at, message.to_vec()));
			}
			Weight::zero()
		})
	}
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	HANDLED_DMP_MESSAGES.with(|m| m.borrow_mut().clear());
	HANDLED_XCMP_MESSAGES.with(|m| m.borrow_mut().clear());

	frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}

#[allow(dead_code)]
pub fn mk_dmp(sent_at: u32) -> InboundDownwardMessage {
	InboundDownwardMessage { sent_at, msg: format!("down{}", sent_at).into_bytes() }
}

pub fn mk_hrmp(sent_at: u32) -> InboundHrmpMessage {
	InboundHrmpMessage { sent_at, data: format!("{}", sent_at).into_bytes() }
}

pub struct ReadRuntimeVersion(pub Vec<u8>);

impl sp_core::traits::ReadRuntimeVersion for ReadRuntimeVersion {
	fn read_runtime_version(
		&self,
		_wasm_code: &[u8],
		_ext: &mut dyn sp_externalities::Externalities,
	) -> Result<Vec<u8>, String> {
		Ok(self.0.clone())
	}
}

pub fn wasm_ext() -> sp_io::TestExternalities {
	let version = RuntimeVersion {
		spec_name: "test".into(),
		spec_version: 2,
		impl_version: 1,
		..Default::default()
	};

	let mut ext = new_test_ext();
	ext.register_extension(sp_core::traits::ReadRuntimeVersionExt::new(ReadRuntimeVersion(
		version.encode(),
	)));
	ext
}

pub struct BlockTest {
	n: BlockNumberFor<Test>,
	within_block: Box<dyn Fn()>,
	after_block: Option<Box<dyn Fn()>>,
}

/// BlockTests exist to test blocks with some setup: we have to assume that
/// `validate_block` will mutate and check storage in certain predictable
/// ways, for example, and we want to always ensure that tests are executed
/// in the context of some particular block number.
#[derive(Default)]
pub struct BlockTests {
	tests: Vec<BlockTest>,
	pending_upgrade: Option<RelayChainBlockNumber>,
	ran: bool,
	without_externalities: bool,
	relay_sproof_builder_hook:
		Option<Box<dyn Fn(&BlockTests, RelayChainBlockNumber, &mut RelayStateSproofBuilder)>>,
	persisted_validation_data_hook: Option<Box<dyn Fn(&BlockTests, &mut PersistedValidationData)>>,
	inherent_data_hook:
		Option<Box<dyn Fn(&BlockTests, RelayChainBlockNumber, &mut ParachainInherentData)>>,
}

impl BlockTests {
	pub fn new() -> BlockTests {
		Default::default()
	}

	pub fn new_without_externalities() -> BlockTests {
		let mut tests = BlockTests::new();
		tests.without_externalities = true;
		tests
	}

	pub fn add_raw(mut self, test: BlockTest) -> Self {
		self.tests.push(test);
		self
	}

	pub fn add<F>(self, n: BlockNumberFor<Test>, within_block: F) -> Self
	where
		F: 'static + Fn(),
	{
		self.add_raw(BlockTest { n, within_block: Box::new(within_block), after_block: None })
	}

	pub fn add_with_post_test<F1, F2>(
		self,
		n: BlockNumberFor<Test>,
		within_block: F1,
		after_block: F2,
	) -> Self
	where
		F1: 'static + Fn(),
		F2: 'static + Fn(),
	{
		self.add_raw(BlockTest {
			n,
			within_block: Box::new(within_block),
			after_block: Some(Box::new(after_block)),
		})
	}

	pub fn with_relay_sproof_builder<F>(mut self, f: F) -> Self
	where
		F: 'static + Fn(&BlockTests, RelayChainBlockNumber, &mut RelayStateSproofBuilder),
	{
		self.relay_sproof_builder_hook = Some(Box::new(f));
		self
	}

	pub fn with_validation_data<F>(mut self, f: F) -> Self
	where
		F: 'static + Fn(&BlockTests, &mut PersistedValidationData),
	{
		self.persisted_validation_data_hook = Some(Box::new(f));
		self
	}

	pub fn with_inherent_data<F>(mut self, f: F) -> Self
	where
		F: 'static + Fn(&BlockTests, RelayChainBlockNumber, &mut ParachainInherentData),
	{
		self.inherent_data_hook = Some(Box::new(f));
		self
	}

	pub fn run(&mut self) {
		wasm_ext().execute_with(|| {
			self.run_without_ext();
		});
	}

	pub fn run_without_ext(&mut self) {
		self.ran = true;

		for BlockTest { n, within_block, after_block } in self.tests.iter() {
			// clear pending updates, as applicable
			if let Some(upgrade_block) = self.pending_upgrade {
				if n >= &upgrade_block.into() {
					self.pending_upgrade = None;
				}
			}

			// begin initialization
			System::reset_events();
			System::initialize(&n, &Default::default(), &Default::default());

			// now mess with the storage the way validate_block does
			let mut sproof_builder = RelayStateSproofBuilder::default();
			if let Some(ref hook) = self.relay_sproof_builder_hook {
				hook(self, *n as RelayChainBlockNumber, &mut sproof_builder);
			}
			let (relay_parent_storage_root, relay_chain_state) =
				sproof_builder.into_state_root_and_proof();
			let mut vfp = PersistedValidationData {
				relay_parent_number: *n as RelayChainBlockNumber,
				relay_parent_storage_root,
				..Default::default()
			};
			if let Some(ref hook) = self.persisted_validation_data_hook {
				hook(self, &mut vfp);
			}

			<ValidationData<Test>>::put(&vfp);
			NewValidationCode::<Test>::kill();

			// It is insufficient to push the validation function params
			// to storage; they must also be included in the inherent data.
			let inherent_data = {
				let mut inherent_data = InherentData::default();
				let mut system_inherent_data = ParachainInherentData {
					validation_data: vfp.clone(),
					relay_chain_state,
					downward_messages: Default::default(),
					horizontal_messages: Default::default(),
				};
				if let Some(ref hook) = self.inherent_data_hook {
					hook(self, *n as RelayChainBlockNumber, &mut system_inherent_data);
				}
				inherent_data
					.put_data(
						cumulus_primitives_parachain_inherent::INHERENT_IDENTIFIER,
						&system_inherent_data,
					)
					.expect("failed to put VFP inherent");
				inherent_data
			};

			// execute the block
			ParachainSystem::on_initialize(*n);
			ParachainSystem::create_inherent(&inherent_data)
				.expect("got an inherent")
				.dispatch_bypass_filter(RawOrigin::None.into())
				.expect("dispatch succeeded");
			MessageQueue::on_initialize(*n);

			within_block();

			MessageQueue::on_finalize(*n);
			ParachainSystem::on_finalize(*n);

			// did block execution set new validation code?
			if NewValidationCode::<Test>::exists() && self.pending_upgrade.is_some() {
				panic!("attempted to set validation code while upgrade was pending");
			}

			// clean up
			System::finalize();
			if let Some(after_block) = after_block {
				after_block();
			}
		}
	}
}

impl Drop for BlockTests {
	fn drop(&mut self) {
		if !self.ran {
			if self.without_externalities {
				self.run_without_ext();
			} else {
				self.run();
			}
		}
	}
}
