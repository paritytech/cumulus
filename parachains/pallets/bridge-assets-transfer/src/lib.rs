// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Bridge Asset Transfer Pallet
//!
//! A utility which could help move assets through bridges, e.g. move assets between different global consensus...

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use xcm::prelude::*;

/// The log target of this pallet.
pub const LOG_TARGET: &str = "runtime::bridge-assets-transfer";

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// XCM sender which sends messages to the BridgeHub
		type BridgeXcmSender: SendXcm;

		// TODO: store as persistent and create add_bridge/remove_bridge - then we can have generic impl and dont need to hardcode NetworkId/ParaId in runtime
		/// Configuration for supported bridged networks
		type SupportedBridges: Get<
			sp_std::prelude::Vec<(NetworkId, MultiLocation, Option<MultiAsset>)>,
		>;

		/// Runtime's universal location
		type UniversalLocation: Get<InteriorMultiLocation>;
	}

	#[pallet::error]
	#[cfg_attr(test, derive(PartialEq))]
	pub enum Error<T> {
		InvalidConfiguration,
		UnsupportedDestination,
		BridgeCallError(#[codec(skip)] &'static str),
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		// TODO: add here xcm_hash?
		/// Transfer was successfully entered to the system (does not mean already delivered)
		TransferInitiated(XcmHash),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Transfer asset via bridge to different global consensus
		///
		/// Parameters:
		///
		/// * `assets`:
		/// * `destination`: Different consensus location, where the assets will be deposited, e.g. Polkadot's Statemint: `X2(GlobalConsensus(NetworkId::Polkadot), Parachain(1000))`
		///
		// TODO: correct weigth
		#[pallet::weight((T::DbWeight::get().reads_writes(1, 1), DispatchClass::Operational))]
		pub fn transfer_asset_via_bridge(
			origin: OriginFor<T>,
			assets: VersionedMultiAssets,
			destination: VersionedMultiLocation,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			// Check remote destination
			let remote_destination = Self::ensure_remote_destination(destination)?;

			// TODO: do some checks
			// TODO: check assets?
			// TODO: check enought fee?

			// Deposit assets into `AccountId` that corresponds to the bridge
			// hub. In this way, Statemine acts as a reserve location to the
			// bridge, such that it need not trust any consensus system from
			// `./Parent/Parent/...`. (It may trust Polkadot, but would
			// Polkadot trust Kusama with its DOT?)

			// TODO: xcm - withdraw and fire ReserveAssetDeposited to the other side

			// TODO: send message through bridge
			// Construct and send `Xcm(vec![Instruction])` to
			// `./Parent/BridgeHubParaId`.

			// TODO: prepare ReserveAssetDeposited msg to bridge to the other side?
			let xcm: Xcm<()> =
				sp_std::vec![Instruction::ReserveAssetDeposited(Default::default())].into();

			// TODO: how to compensate if this call fails?
			log::info!(
				target: LOG_TARGET,
				"[T::BridgeXcmSender] send to bridge, remote_destination: {:?}, xcm: {:?}",
				remote_destination,
				xcm,
			);
			// call bridge
			let (ticket, fees) =
				T::BridgeXcmSender::validate(&mut Some(remote_destination), &mut Some(xcm))
					.map_err(Self::convert_to_error)?;
			log::info!(
				target: LOG_TARGET,
				"[T::BridgeXcmSender::validate] (TODO: process) fees: {:?}",
				fees
			);
			// TODO: what to do with fees - we have fees here, pay here or ignore?
			// TODO: use fn send_msg
			let xcm_hash = T::BridgeXcmSender::deliver(ticket).map_err(Self::convert_to_error)?;

			Self::deposit_event(Event::TransferInitiated(xcm_hash));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Validates destination and check if we support bridging to this remote global consensus
		///
		/// Returns: correct remote location, where we should be able to bridge
		pub(crate) fn ensure_remote_destination(
			destination: VersionedMultiLocation,
		) -> Result<MultiLocation, Error<T>> {
			match destination {
				VersionedMultiLocation::V3(location) => {
					ensure!(location.parent_count() == 2, Error::<T>::UnsupportedDestination);
					let local_network = T::UniversalLocation::get()
						.global_consensus()
						.map_err(|_| Error::<T>::InvalidConfiguration)?;
					let remote_network = location
						.interior()
						.global_consensus()
						.map_err(|_| Error::<T>::UnsupportedDestination)?;
					ensure!(local_network != remote_network, Error::<T>::UnsupportedDestination);
					ensure!(
						T::SupportedBridges::get()
							.iter()
							.find(|sb| sb.0 == remote_network)
							.is_some(),
						Error::<T>::UnsupportedDestination
					);
					Ok(location)
				},
				_ => Err(Error::<T>::UnsupportedDestination),
			}
		}

		fn convert_to_error(error: SendError) -> Error<T> {
			log::error!(target: LOG_TARGET, "SendError occurred, error: {:?}", error);
			match error {
				SendError::NotApplicable => Error::<T>::BridgeCallError("NotApplicable"),
				SendError::Transport(error) => Error::<T>::BridgeCallError(error),
				SendError::Unroutable => Error::<T>::BridgeCallError("Unroutable"),
				SendError::DestinationUnsupported =>
					Error::<T>::BridgeCallError("DestinationUnsupported"),
				SendError::ExceedsMaxMessageSize =>
					Error::<T>::BridgeCallError("ExceedsMaxMessageSize"),
				SendError::MissingArgument => Error::<T>::BridgeCallError("MissingArgument"),
				SendError::Fees => Error::<T>::BridgeCallError("Fees"),
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate as bridge_assets_transfer;

	use frame_support::{parameter_types, sp_io, sp_tracing};
	use sp_runtime::{
		testing::{Header, H256},
		traits::{BlakeTwo256, IdentityLookup},
	};
	use sp_version::RuntimeVersion;
	use xcm_builder::{NetworkExportTable, UnpaidRemoteExporter};

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
	type Block = frame_system::mocking::MockBlock<TestRuntime>;

	frame_support::construct_runtime!(
		pub enum TestRuntime where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
			BridgeAssetsTransfer: bridge_assets_transfer::{Pallet, Call, Event<T>} = 52,
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
	}

	impl frame_system::Config for TestRuntime {
		type RuntimeOrigin = RuntimeOrigin;
		type RuntimeCall = RuntimeCall;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type RuntimeEvent = RuntimeEvent;
		type BlockHashCount = BlockHashCount;
		type BlockLength = ();
		type BlockWeights = ();
		type Version = Version;
		type PalletInfo = PalletInfo;
		type AccountData = ();
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type DbWeight = ();
		type BaseCallFilter = frame_support::traits::Everything;
		type SystemWeightInfo = ();
		type SS58Prefix = ();
		type OnSetCode = ();
		type MaxConsumers = frame_support::traits::ConstU32<16>;
	}

	parameter_types! {
		// UniversalLocation as statemine
		pub const RelayNetwork: NetworkId = NetworkId::Kusama;
		pub UniversalLocation: InteriorMultiLocation = X2(GlobalConsensus(RelayNetwork::get()), Parachain(1000));
		// Test bridge cfg
		pub TestBridgeTable: sp_std::prelude::Vec<(NetworkId, MultiLocation, Option<MultiAsset>)> = sp_std::vec![
			(NetworkId::Wococo, (Parent, Parachain(1013)).into(), None),
			(NetworkId::Polkadot, (Parent, Parachain(1003)).into(), None),
		];
	}

	std::thread_local! {
		static ROUTED_MESSAGE: std::cell::RefCell<Option<Xcm<()>>> = std::cell::RefCell::new(None);
	}

	pub struct ThreadLocalXcmRouter;
	impl SendXcm for ThreadLocalXcmRouter {
		type Ticket = Option<Xcm<()>>;

		fn validate(
			destination: &mut Option<MultiLocation>,
			message: &mut Option<Xcm<()>>,
		) -> SendResult<Self::Ticket> {
			log::info!(
				target: super::LOG_TARGET,
				"[ThreadLocalXcmRouter]: destination: {:?}, message: {:?}",
				destination,
				message
			);
			Ok((message.take(), MultiAssets::default()))
		}

		fn deliver(ticket: Self::Ticket) -> Result<XcmHash, SendError> {
			match ticket {
				Some(msg) => {
					ROUTED_MESSAGE.with(|rm| *rm.borrow_mut() = Some(msg));
					Ok([0u8; 32])
				},
				None => Err(SendError::MissingArgument),
			}
		}
	}

	/// Bridge router, which wraps and sends xcm to BridgeHub to be delivered to the different GlobalConsensus
	pub type TestBridgeXcmSender = UnpaidRemoteExporter<
		NetworkExportTable<TestBridgeTable>,
		ThreadLocalXcmRouter,
		UniversalLocation,
	>;

	impl Config for TestRuntime {
		type RuntimeEvent = RuntimeEvent;
		type BridgeXcmSender = TestBridgeXcmSender;
		type SupportedBridges = TestBridgeTable;
		type UniversalLocation = UniversalLocation;
	}

	pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
		sp_tracing::try_init_simple();
		frame_system::GenesisConfig::default()
			.build_storage::<TestRuntime>()
			.unwrap()
			.into()
	}

	#[test]
	fn test_ensure_remote_destination() {
		new_test_ext().execute_with(|| {
			// v2 not supported
			assert_eq!(
				BridgeAssetsTransfer::ensure_remote_destination(VersionedMultiLocation::V2(
					xcm::v2::MultiLocation::default()
				)),
				Err(Error::<TestRuntime>::UnsupportedDestination)
			);

			// v3 - "parent: 0" wrong
			assert_eq!(
				BridgeAssetsTransfer::ensure_remote_destination(VersionedMultiLocation::V3(
					MultiLocation::new(0, X2(GlobalConsensus(Wococo), Parachain(1000)))
				)),
				Err(Error::<TestRuntime>::UnsupportedDestination)
			);
			// v3 - "parent: 1" wrong
			assert_eq!(
				BridgeAssetsTransfer::ensure_remote_destination(VersionedMultiLocation::V3(
					MultiLocation::new(1, X2(GlobalConsensus(Wococo), Parachain(1000)))
				)),
				Err(Error::<TestRuntime>::UnsupportedDestination)
			);

			// v3 - Rococo is not supported
			assert_eq!(
				BridgeAssetsTransfer::ensure_remote_destination(VersionedMultiLocation::V3(
					MultiLocation::new(2, X2(GlobalConsensus(Rococo), Parachain(1000)))
				)),
				Err(Error::<TestRuntime>::UnsupportedDestination)
			);

			// v3 - ok
			assert_eq!(
				BridgeAssetsTransfer::ensure_remote_destination(VersionedMultiLocation::V3(
					MultiLocation::new(2, X2(GlobalConsensus(Wococo), Parachain(1000)))
				)),
				Ok(MultiLocation::new(2, X2(GlobalConsensus(Wococo), Parachain(1000))))
			);
		})
	}

	#[test]
	fn test_transfer_asset_via_bridge_works() {
		new_test_ext().execute_with(|| {
			assert!(ROUTED_MESSAGE.with(|r| r.borrow().is_none()));

			let assets = VersionedMultiAssets::V3(MultiAssets::default());
			let destination = VersionedMultiLocation::V3(MultiLocation::new(
				2,
				X2(GlobalConsensus(Wococo), Parachain(1000)),
			));

			let result = BridgeAssetsTransfer::transfer_asset_via_bridge(
				RuntimeOrigin::signed(1),
				assets,
				destination,
			);
			assert_eq!(result, Ok(()));
			assert!(ROUTED_MESSAGE.with(|r| r.borrow().is_some()));
		});
	}
}
