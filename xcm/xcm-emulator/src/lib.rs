// Copyright 2023 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

pub use codec::Encode;
pub use paste;
pub use casey::pascal;

pub use frame_support::{
	traits::{Get, Hooks},
	weights::Weight,
	sp_runtime::BuildStorage,
};
pub use pallet_balances::AccountData;
pub use frame_system::AccountInfo;
pub use sp_arithmetic::traits::Bounded;
pub use sp_io::TestExternalities;
pub use sp_std::{cell::RefCell, collections::{vec_deque::VecDeque}, marker::PhantomData};
pub use sp_trie::StorageProof;
pub use sp_core::storage::Storage;

pub use cumulus_test_service::get_account_id_from_seed;
pub use cumulus_pallet_dmp_queue;
pub use cumulus_pallet_parachain_system;
pub use cumulus_pallet_xcmp_queue;
pub use cumulus_primitives_core::{
	self, relay_chain::BlockNumber as RelayBlockNumber, DmpMessageHandler, ParaId,
	PersistedValidationData, XcmpMessageHandler,
};
pub use cumulus_primitives_parachain_inherent::ParachainInherentData;
pub use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;
pub use parachain_info;
pub use parachains_common::{BlockNumber, AccountId};

pub use polkadot_primitives;
pub use polkadot_runtime_parachains::{
	dmp,
	ump::{MessageId, UmpSink, XcmSink},
};
pub use xcm::{v3::prelude::*, VersionedXcm};
pub use xcm_executor::XcmExecutor;
pub use std::{thread::LocalKey, collections::HashMap};

thread_local! {
	/// Downward messages, each message is: `(to_para_id, [(relay_block_number, msg)])`
	#[allow(clippy::type_complexity)]
	pub static DOWNWARD_MESSAGES: RefCell<HashMap<String, VecDeque<(u32, Vec<(RelayBlockNumber, Vec<u8>)>)>>>
		= RefCell::new(HashMap::new());
	/// Downward messages that already processed by parachains, each message is: `(to_para_id, relay_block_number, Vec<u8>)`
	#[allow(clippy::type_complexity)]
	pub static DMP_DONE: RefCell<HashMap<String, VecDeque<(u32, RelayBlockNumber, Vec<u8>)>>>
		= RefCell::new(HashMap::new());
	/// Horizontal messages, each message is: `(to_para_id, [(from_para_id, relay_block_number, msg)])`
	#[allow(clippy::type_complexity)]
	pub static HORIZONTAL_MESSAGES: RefCell<HashMap<String, VecDeque<(u32, Vec<(ParaId, RelayBlockNumber, Vec<u8>)>)>>>
		= RefCell::new(HashMap::new());
	/// Upward messages, each message is: `(from_para_id, msg)
	pub static UPWARD_MESSAGES: RefCell<HashMap<String, VecDeque<(u32, Vec<u8>)>>> = RefCell::new(HashMap::new());
	/// Global incremental relay chain block number
	pub static RELAY_BLOCK_NUMBER: RefCell<HashMap<String, u32>> = RefCell::new(HashMap::new());
	/// Parachains Ids a the Network
	pub static PARA_IDS: RefCell<HashMap<String, Vec<u32>>> = RefCell::new(HashMap::new());
	/// Flag indicating if global variables have been initialized for a certain Network
	pub static INITIALIZED: RefCell<HashMap<String, bool>> = RefCell::new(HashMap::new());
}

pub trait TestExt {
	fn build_new_ext(storage: Storage) -> sp_io::TestExternalities;
	fn new_ext() -> sp_io::TestExternalities;
	fn reset_ext();
	fn execute_with<R>(execute: impl FnOnce() -> R) -> R;
	fn ext_wrapper<R>(func: impl FnOnce() -> R) -> R;
}

pub trait RelayChain: TestExt + UmpSink {
	type Runtime;
	type RuntimeOrigin;
	type RuntimeEvent;
	type XcmConfig;
	type System;
	type XcmPallet;
	type Balances;
	type SovereignAccountOf;
}

pub trait Parachain: TestExt + XcmpMessageHandler + DmpMessageHandler {
	type Runtime;
	type RuntimeOrigin;
	type RuntimeEvent;
	type XcmpMessageHandler;
	type DmpMessageHandler;
	type System;
	type ParachainSystem;
	type ParachainInfo;
	type XcmPallet;
	type Balances;
	type LocationToAccountId;
}

// Relay Chain Implementation
#[macro_export]
macro_rules! decl_test_relay_chains {
	(
		$(
			pub struct $name:ident {
				Runtime = $runtime:path,
				RuntimeOrigin = $runtime_origin:path,
				RuntimeEvent = $runtime_event:path,
				XcmConfig = $xcm_config:path,
				System = $system:path,
				XcmPallet = $xcm_pallet:path,
				Balances = $balances_pallet:path,
				SovereignAccountOf = $sovereign_account_of:path,
				genesis = $genesis:expr,
				on_init = $on_init:expr,
			}
		),
		+
	) => {
		$(
			pub struct $name;

			impl RelayChain for $name {
				type Runtime = $runtime;
				type RuntimeOrigin = $runtime_origin;
				type RuntimeEvent = $runtime_event;
				type XcmConfig = $xcm_config;
				type System = $system;
				type XcmPallet = $xcm_pallet;
				type Balances = $balances_pallet;
				type SovereignAccountOf = $sovereign_account_of;

			}

			$crate::__impl_xcm_handlers_for_relay_chain!($name);
			$crate::__impl_test_ext_for_relay_chain!($name, $genesis, $on_init);
		)+
	};
}

#[macro_export]
macro_rules! __impl_xcm_handlers_for_relay_chain {
	($name:ident) => {
		impl $crate::UmpSink for $name {
			fn process_upward_message(
				origin: $crate::ParaId,
				msg: &[u8],
				max_weight: $crate::Weight,
			) -> Result<$crate::Weight, ($crate::MessageId, $crate::Weight)> {
				use $crate::{TestExt, UmpSink};

				Self::execute_with(|| {
					$crate::XcmSink::<$crate::XcmExecutor<<Self as RelayChain>::XcmConfig>, <Self as RelayChain>::Runtime>::process_upward_message(origin, msg, max_weight)
				})
			}
		}
	}
}

#[macro_export]
macro_rules! __impl_test_ext_for_relay_chain {
	// entry point: generate ext name
	($name:ident, $genesis:expr, $on_init:expr) => {
		$crate::paste::paste! {
			$crate::__impl_test_ext_for_relay_chain!(@impl $name, $genesis, $on_init, [<EXT_ $name:upper>]);
		}
	};
	// impl
	(@impl $name:ident, $genesis:expr, $on_init:expr, $ext_name:ident) => {
		thread_local! {
			pub static $ext_name: $crate::RefCell<$crate::TestExternalities>
				= $crate::RefCell::new(<$name>::build_new_ext($genesis));
		}

		impl $crate::TestExt for $name {
			fn build_new_ext(storage: $crate::Storage) -> $crate::TestExternalities {
				let mut ext = sp_io::TestExternalities::new(storage);
				ext.execute_with(|| {
					$on_init;
					sp_tracing::try_init_simple();
					<Self as RelayChain>::System::set_block_number(1);
				});
				ext
			}

			fn new_ext() -> $crate::TestExternalities {
				<$name>::build_new_ext($genesis)
			}

			fn reset_ext() {
				$ext_name.with(|v| *v.borrow_mut() = <$name>::build_new_ext($genesis));
			}

			fn execute_with<R>(execute: impl FnOnce() -> R) -> R {
				// Make sure the Network is initialized
				<$name>::init();

				let r = $ext_name.with(|v| v.borrow_mut().execute_with(execute));

				// send messages if needed
				$ext_name.with(|v| {
					v.borrow_mut().execute_with(|| {
						use $crate::polkadot_primitives::runtime_api::runtime_decl_for_parachain_host::ParachainHostV4;

						//TODO: mark sent count & filter out sent msg
						for para_id in <$name>::para_ids() {
							// downward messages
							let downward_messages = <Self as RelayChain>::Runtime::dmq_contents(para_id.into())
								.into_iter()
								.map(|inbound| (inbound.sent_at, inbound.msg));
							if downward_messages.len() == 0 {
								continue;
							}
							<$name>::send_downward_messages(para_id, downward_messages.into_iter());

							// Note: no need to handle horizontal messages, as the
							// simulator directly sends them to dest (not relayed).
						}
					})
				});

				<$name>::process_messages();

				r
			}

			fn ext_wrapper<R>(func: impl FnOnce() -> R) -> R {
				$ext_name.with(|v| {
					v.borrow_mut().execute_with(|| {
						func()
					})
				})
			}
		}
	};
}

#[macro_export]
macro_rules! __impl_relay {
	($network_name:ident, $relay_chain:ty) => {

		impl $relay_chain {
			pub fn network_name() -> &'static str {
				stringify!($network_name)
			}

			fn init() {
				<$network_name>::_init();
			}

			pub fn relay_block_number() -> u32 {
				<$network_name>::_relay_block_number()
			}

			fn para_ids() -> Vec<u32> {
				<$network_name>::_para_ids()
			}

			pub fn child_location_of(id: $crate::ParaId) -> MultiLocation {
				(Ancestor(0), Parachain(id.into())).into()
			}

			pub fn account_id_of(seed: &str) -> $crate::AccountId {
				$crate::get_account_id_from_seed::<sr25519::Public>(seed)
			}

			pub fn account_data_of(account: AccountId) -> $crate::AccountData<Balance> {
				Self::ext_wrapper(|| <Self as RelayChain>::System::account(account).data)
			}

			pub fn sovereign_account_id_of(location: $crate::MultiLocation) -> $crate::AccountId {
				<Self as RelayChain>::SovereignAccountOf::convert(location.into()).unwrap()
			}

			pub fn fund_accounts(accounts: Vec<(AccountId, Balance)>) {
				Self::ext_wrapper(|| {
					for account in accounts {
						let _ = <Self as RelayChain>::Balances::force_set_balance(
							<Self as RelayChain>::RuntimeOrigin::root(),
							account.0.into(),
							account.1.into(),
						);
					}
				});
			}

			fn send_downward_messages(to_para_id: u32, iter: impl Iterator<Item = ($crate::RelayBlockNumber, Vec<u8>)>) {
				$crate::DOWNWARD_MESSAGES.with(|b| b.borrow_mut().get_mut(Self::network_name()).unwrap().push_back((to_para_id, iter.collect())));
			}

			fn process_messages() {
				<$network_name>::_process_messages();
			}
		}
	}
}

// Parachain Implementation
#[macro_export]
macro_rules! decl_test_parachains {
	(
		$(
			pub struct $name:ident {
				Runtime = $runtime:path,
				RuntimeOrigin = $runtime_origin:path,
				RuntimeEvent = $runtime_event:path,
				XcmpMessageHandler = $xcmp_message_handler:path,
				DmpMessageHandler = $dmp_message_handler:path,
				System = $system:path,
				ParachainSystem = $parachain_system:path,
				ParachainInfo = $parachain_info:path,
				XcmPallet = $xcm_pallet:path,
				Balances = $balances_pallet:path,
				LocationToAccountId = $location_to_account:path,
				genesis = $genesis:expr,
				on_init = $on_init:expr,
			}
		),
		+
	) => {
		$(
			pub struct $name;

			impl Parachain for $name {
				type Runtime = $runtime;
				type RuntimeOrigin = $runtime_origin;
				type RuntimeEvent = $runtime_event;
				type XcmpMessageHandler = $xcmp_message_handler;
				type DmpMessageHandler = $dmp_message_handler;
				type System = $system;
				type ParachainSystem = $parachain_system;
				type ParachainInfo = $parachain_info;
				type XcmPallet = $xcm_pallet;
				type Balances = $balances_pallet;
				type LocationToAccountId = $location_to_account;
			}

			$crate::__impl_xcm_for_parachain!($name);
			$crate::__impl_test_ext_for_parachain!($name, $genesis, $on_init);
		)+
	};
}

#[macro_export]
macro_rules! __impl_xcm_for_parachain {
	($name:ident) => {
		impl $crate::XcmpMessageHandler for $name {
			fn handle_xcmp_messages<
				'a,
				I: Iterator<Item = ($crate::ParaId, $crate::RelayBlockNumber, &'a [u8])>,
			>(
				iter: I,
				max_weight: $crate::Weight,
			) -> $crate::Weight {
				use $crate::{TestExt, XcmpMessageHandler};

				$name::execute_with(|| {
					<Self as Parachain>::XcmpMessageHandler::handle_xcmp_messages(iter, max_weight)
				})
			}
		}

		impl $crate::DmpMessageHandler for $name {
			fn handle_dmp_messages(
				iter: impl Iterator<Item = ($crate::RelayBlockNumber, Vec<u8>)>,
				max_weight: $crate::Weight,
			) -> $crate::Weight {
				use $crate::{DmpMessageHandler, TestExt};

				$name::execute_with(|| {
					<Self as Parachain>::DmpMessageHandler::handle_dmp_messages(iter, max_weight)
				})
			}
		}
	}
}

#[macro_export]
macro_rules! __impl_test_ext_for_parachain {
	// entry point: generate ext name
	($name:ident, $genesis:expr, $on_init:expr) => {
		$crate::paste::paste! {
			$crate::__impl_test_ext_for_parachain!(@impl $name, $genesis, $on_init, [<EXT_ $name:upper>]);
		}
	};
	// impl
	(@impl $name:ident, $genesis:expr, $on_init:expr, $ext_name:ident) => {
		thread_local! {
			pub static $ext_name: $crate::RefCell<$crate::TestExternalities>
				= $crate::RefCell::new(<$name>::build_new_ext($genesis));
		}

		impl $crate::TestExt for $name {
			fn build_new_ext(storage: $crate::Storage) -> $crate::TestExternalities {
				let mut ext = sp_io::TestExternalities::new(storage);
				ext.execute_with(|| {
					$on_init;
					sp_tracing::try_init_simple();
					<Self as Parachain>::System::set_block_number(1);
				});
				ext
			}

			fn new_ext() -> $crate::TestExternalities {
				<$name>::build_new_ext($genesis)
			}

			fn reset_ext() {
				$ext_name.with(|v| *v.borrow_mut() = <$name>::build_new_ext($genesis));
			}

			fn execute_with<R>(execute: impl FnOnce() -> R) -> R {
				use $crate::{Get, Hooks};

				// Make sure the Network is initialized
				<$name>::init();

				let mut relay_block_number = <$name>::relay_block_number();
				relay_block_number += 1;
				<$name>::set_relay_block_number(relay_block_number);

				let para_id = <$name>::para_id().into();

				$ext_name.with(|v| {
					v.borrow_mut().execute_with(|| {
						// Make sure it has been recorded properly
						let relay_block_number = <$name>::relay_block_number();
						let _ = <Self as Parachain>::ParachainSystem::set_validation_data(
							<Self as Parachain>::RuntimeOrigin::none(),
							<$name>::hrmp_channel_parachain_inherent_data(para_id, relay_block_number),
						);
					})
				});


				let r = $ext_name.with(|v| v.borrow_mut().execute_with(execute));

				// send messages if needed
				$ext_name.with(|v| {
					v.borrow_mut().execute_with(|| {
						use sp_runtime::traits::Header as HeaderT;

						let block_number = <Self as Parachain>::System::block_number();
						let mock_header = HeaderT::new(
							0,
							Default::default(),
							Default::default(),
							Default::default(),
							Default::default(),
						);

						// get messages
						<Self as Parachain>::ParachainSystem::on_finalize(block_number);
						let collation_info = <Self as Parachain>::ParachainSystem::collect_collation_info(&mock_header);

						// send upward messages
						let relay_block_number = <$name>::relay_block_number();
						for msg in collation_info.upward_messages.clone() {
							<$name>::send_upward_message(para_id, msg);
						}

						// send horizontal messages
						for msg in collation_info.horizontal_messages {
							<$name>::send_horizontal_messages(
								msg.recipient.into(),
								vec![(para_id.into(), relay_block_number, msg.data)].into_iter(),
							);
						}

						// clean messages
						<Self as Parachain>::ParachainSystem::on_initialize(block_number);
					})
				});

				<$name>::process_messages();

				r
			}

			fn ext_wrapper<R>(func: impl FnOnce() -> R) -> R {
				$ext_name.with(|v| {
					v.borrow_mut().execute_with(|| {
						func()
					})
				})
			}
		}
	};
}

#[macro_export]
macro_rules! __impl_parachain {
	($network_name:ident, $parachain:ty) => {

		impl $parachain {
			pub fn network_name() -> &'static str {
				stringify!($network_name)
			}

			fn init() {
				<$network_name>::_init();
			}

			pub fn para_id() -> $crate::ParaId {
				Self::ext_wrapper(|| <Self as Parachain>::ParachainInfo::get())
			}

			pub fn parent_location() -> $crate::MultiLocation {
				(Parent).into()
			}

			fn relay_block_number() -> u32 {
				<$network_name>::_relay_block_number()
			}

			fn set_relay_block_number(block_number: u32) {
				<$network_name>::_set_relay_block_number(block_number);
			}

			pub fn account_id_of(seed: &str) -> $crate::AccountId {
				$crate::get_account_id_from_seed::<sr25519::Public>(seed)
			}

			pub fn account_data_of(account: AccountId) -> $crate::AccountData<Balance> {
				Self::ext_wrapper(|| <Self as Parachain>::System::account(account).data)
			}

			pub fn sovereign_account_id_of(location: $crate::MultiLocation) -> $crate::AccountId {
				<Self as Parachain>::LocationToAccountId::convert(location.into()).unwrap()
			}

			pub fn fund_accounts(accounts: Vec<(AccountId, Balance)>) {
				Self::ext_wrapper(|| {
					for account in accounts {
						let _ = <Self as Parachain>::Balances::force_set_balance(
							<Self as Parachain>::RuntimeOrigin::root(),
							account.0.into(),
							account.1.into(),
						);
					}
				});
			}

			fn send_horizontal_messages<
				I: Iterator<Item = ($crate::ParaId, $crate::RelayBlockNumber, Vec<u8>)>,
			>(to_para_id: u32, iter: I) {
				$crate::HORIZONTAL_MESSAGES.with(|b| b.borrow_mut().get_mut(Self::network_name()).unwrap().push_back((to_para_id, iter.collect())));
			}

			fn send_upward_message(from_para_id: u32, msg: Vec<u8>) {
				$crate::UPWARD_MESSAGES.with(|b| b.borrow_mut().get_mut(Self::network_name()).unwrap().push_back((from_para_id, msg)));
			}

			fn hrmp_channel_parachain_inherent_data(
				para_id: u32,
				relay_parent_number: u32,
			) -> $crate::ParachainInherentData {
				<$network_name>::_hrmp_channel_parachain_inherent_data(para_id, relay_parent_number)
			}

			fn prepare_for_xcmp() {
				let para_id = Self::para_id();

				<Self as TestExt>::ext_wrapper(|| {
					use $crate::{Get, Hooks};

					let block_number = <Self as Parachain>::System::block_number();
					// let para_id = Self::para_id();

					let _ = <Self as Parachain>::ParachainSystem::set_validation_data(
						<Self as Parachain>::RuntimeOrigin::none(),
						Self::hrmp_channel_parachain_inherent_data(para_id.into(), 1),
					);
					// set `AnnouncedHrmpMessagesPerCandidate`
					<Self as Parachain>::ParachainSystem::on_initialize(block_number);
				});
			}

			fn process_messages() {
				<$network_name>::_process_messages();
			}
		}
	}
}

// Network Implementation
#[macro_export]
macro_rules! decl_test_networks {
	(
		$(
			pub struct $name:ident {
				relay_chain = $relay_chain:ty,
				parachains = vec![ $( $parachain:ty, )* ],
			}
		),
		+
	) => {
		$(
			pub struct $name;

			impl $name {
				pub fn reset() {
					use $crate::{TestExt, VecDeque};

					<$relay_chain>::reset_ext();
					$( <$parachain>::reset_ext(); )*

					$( <$parachain>::prepare_for_xcmp(); )*

					$crate::INITIALIZED.with(|b| b.borrow_mut().remove(stringify!($name)));
					$crate::DOWNWARD_MESSAGES.with(|b| b.borrow_mut().remove(stringify!($name)));
					$crate::DMP_DONE.with(|b| b.borrow_mut().remove(stringify!($name)));
					$crate::UPWARD_MESSAGES.with(|b| b.borrow_mut().remove(stringify!($name)));
					$crate::HORIZONTAL_MESSAGES.with(|b| b.borrow_mut().remove(stringify!($name)));
					$crate::RELAY_BLOCK_NUMBER.with(|b| b.borrow_mut().remove(stringify!($name)));
				}

				fn _init() {
					// If Network has not been itialized yet, it gets initialized
					if $crate::INITIALIZED.with(|b| b.borrow_mut().get(stringify!($name)).is_none()) {
						$crate::INITIALIZED.with(|b| b.borrow_mut().insert(stringify!($name).to_string(), true));
						$crate::DOWNWARD_MESSAGES.with(|b| b.borrow_mut().insert(stringify!($name).to_string(), $crate::VecDeque::new()));
						$crate::DMP_DONE.with(|b| b.borrow_mut().insert(stringify!($name).to_string(), $crate::VecDeque::new()));
						$crate::UPWARD_MESSAGES.with(|b| b.borrow_mut().insert(stringify!($name).to_string(), $crate::VecDeque::new()));
						$crate::HORIZONTAL_MESSAGES.with(|b| b.borrow_mut().insert(stringify!($name).to_string(), $crate::VecDeque::new()));
						$crate::RELAY_BLOCK_NUMBER.with(|b| b.borrow_mut().insert(stringify!($name).to_string(), 1));
						$crate::PARA_IDS.with(|b| b.borrow_mut().insert(stringify!($name).to_string(), Self::_para_ids()));
					}
				}

				fn _para_ids() -> Vec<u32> {
					vec![$(
						<$parachain>::para_id().into(),
					)*]
				}

				fn _relay_block_number() -> u32 {
					$crate::RELAY_BLOCK_NUMBER.with(|v| *v.clone().borrow().get(stringify!($name)).unwrap())
				}

				fn _set_relay_block_number(block_number: u32) {
					$crate::RELAY_BLOCK_NUMBER.with(|v| v.borrow_mut().insert(stringify!($name).to_string(), block_number));
				}

				fn _process_messages() {
					while Self::_has_unprocessed_messages() {
						Self::_process_upward_messages();
						Self::_process_horizontal_messages();
						Self::_process_downward_messages();
					}
				}

				fn _has_unprocessed_messages() -> bool {
					$crate::DOWNWARD_MESSAGES.with(|b| !b.borrow_mut().get_mut(stringify!($name)).unwrap().is_empty())
					|| $crate::HORIZONTAL_MESSAGES.with(|b| !b.borrow_mut().get_mut(stringify!($name)).unwrap().is_empty())
					|| $crate::UPWARD_MESSAGES.with(|b| !b.borrow_mut().get_mut(stringify!($name)).unwrap().is_empty())
				}

				fn _process_downward_messages() {
					use $crate::{DmpMessageHandler, Bounded};
					use polkadot_parachain::primitives::RelayChainBlockNumber;

					while let Some((to_para_id, messages))
						= $crate::DOWNWARD_MESSAGES.with(|b| b.borrow_mut().get_mut(stringify!($name)).unwrap().pop_front()) {
						$(
							if $crate::PARA_IDS.with(|b| b.borrow_mut().get_mut(stringify!($name)).unwrap().contains(&to_para_id)) {
								let mut msg_dedup: Vec<(RelayChainBlockNumber, Vec<u8>)> = Vec::new();
								for m in &messages {
									msg_dedup.push((m.0, m.1.clone()));
								}
								msg_dedup.dedup();

								let msgs = msg_dedup.clone().into_iter().filter(|m| {
									!$crate::DMP_DONE.with(|b| b.borrow_mut().get_mut(stringify!($name)).unwrap_or(&mut $crate::VecDeque::new()).contains(&(to_para_id, m.0, m.1.clone())))
								}).collect::<Vec<(RelayChainBlockNumber, Vec<u8>)>>();
								if msgs.len() != 0 {
									<$parachain>::handle_dmp_messages(msgs.clone().into_iter(), $crate::Weight::max_value());
									for m in msgs {
										$crate::DMP_DONE.with(|b| b.borrow_mut().get_mut(stringify!($name)).unwrap().push_back((to_para_id, m.0, m.1)));
									}
								}
							} else {
								unreachable!();
							}
						)*
					}
				}

				fn _process_horizontal_messages() {
					use $crate::{XcmpMessageHandler, Bounded};

					while let Some((to_para_id, messages))
						= $crate::HORIZONTAL_MESSAGES.with(|b| b.borrow_mut().get_mut(stringify!($name)).unwrap().pop_front()) {
						let iter = messages.iter().map(|(p, b, m)| (*p, *b, &m[..])).collect::<Vec<_>>().into_iter();
						$(
							if $crate::PARA_IDS.with(|b| b.borrow_mut().get_mut(stringify!($name)).unwrap().contains(&to_para_id)) {
								<$parachain>::handle_xcmp_messages(iter.clone(), $crate::Weight::max_value());
							}
						)*
					}
				}

				fn _process_upward_messages() {
					use $crate::{UmpSink, Bounded};
					while let Some((from_para_id, msg)) = $crate::UPWARD_MESSAGES.with(|b| b.borrow_mut().get_mut(stringify!($name)).unwrap().pop_front()) {
						let _ =  <$relay_chain>::process_upward_message(
							from_para_id.into(),
							&msg[..],
							$crate::Weight::max_value(),
						);
					}
				}

				fn _hrmp_channel_parachain_inherent_data(
					para_id: u32,
					relay_parent_number: u32,
				) -> $crate::ParachainInherentData {
					use $crate::cumulus_primitives_core::{relay_chain::HrmpChannelId, AbridgedHrmpChannel};

					let mut sproof = $crate::RelayStateSproofBuilder::default();
					sproof.para_id = para_id.into();

					// egress channel
					let e_index = sproof.hrmp_egress_channel_index.get_or_insert_with(Vec::new);
					for recipient_para_id in $crate::PARA_IDS.with(|b| b.borrow_mut().get_mut(stringify!($name)).unwrap().clone()) {
						let recipient_para_id = $crate::ParaId::from(recipient_para_id);
						if let Err(idx) = e_index.binary_search(&recipient_para_id) {
							e_index.insert(idx, recipient_para_id);
						}

						sproof
							.hrmp_channels
							.entry(HrmpChannelId {
								sender: sproof.para_id,
								recipient: recipient_para_id,
							})
							.or_insert_with(|| AbridgedHrmpChannel {
								max_capacity: 1024,
								max_total_size: 1024 * 1024,
								max_message_size: 1024 * 1024,
								msg_count: 0,
								total_size: 0,
								mqc_head: Option::None,
							});
					}

					let (relay_storage_root, proof) = sproof.into_state_root_and_proof();

					$crate::ParachainInherentData {
						validation_data: $crate::PersistedValidationData {
							parent_head: Default::default(),
							relay_parent_number,
							relay_parent_storage_root: relay_storage_root,
							max_pov_size: Default::default(),
						},
						relay_chain_state: proof,
						downward_messages: Default::default(),
						horizontal_messages: Default::default(),
					}
				}
			}

			$crate::__impl_relay!($name, $relay_chain);

			$(
				$crate::__impl_parachain!($name, $parachain);
			)*
		)+
	};
}

#[macro_export]
macro_rules! bx {
    ($e:expr) => {
        Box::new($e)
    };
}
