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

//! # Bridge Transfer Pallet
//!
//! A utility which could help transfer through bridges, e.g. move assets between different global consensus...

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::boxed::Box;

pub use pallet::*;
use xcm::prelude::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

/// The log target of this pallet.
pub const LOG_TARGET: &str = "runtime::bridge-assets-transfer";

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct BridgeConfig {
	/// Contains location, which is able to bridge XCM messages to bridged network
	pub bridge_location: MultiLocation,

	/// Contains target destination on bridged network. E.g.: MultiLocation of Statemine/t on different consensus
	// TODO:check-parameter - lets start with 1..1, maybe later we could extend this with BoundedVec
	// TODO: bridged bridge-hub should have router for this
	pub allowed_target_location: MultiLocation,

	/// Fee which could be needed to pay in `bridge_location`
	pub fee: Option<MultiAsset>,
}

impl From<BridgeConfig> for (MultiLocation, Option<MultiAsset>) {
	fn from(bridge_config: BridgeConfig) -> (MultiLocation, Option<MultiAsset>) {
		(bridge_config.bridge_location, bridge_config.fee)
	}
}

/// Trait for constructing ping message.
pub trait PingMessageBuilder {
	fn try_build(
		local_origin: &MultiLocation,
		network: &NetworkId,
		remote_destination: &MultiLocation,
	) -> Option<Xcm<()>>;
}

impl PingMessageBuilder for () {
	fn try_build(_: &MultiLocation, _: &NetworkId, _: &MultiLocation) -> Option<Xcm<()>> {
		None
	}
}

/// Builder creates xcm message just with `Trap` instruction.
pub struct UnpaidTrapMessageBuilder<TrapCode>(sp_std::marker::PhantomData<TrapCode>);
impl<TrapCode: frame_support::traits::Get<u64>> PingMessageBuilder
	for UnpaidTrapMessageBuilder<TrapCode>
{
	fn try_build(_: &MultiLocation, _: &NetworkId, _: &MultiLocation) -> Option<Xcm<()>> {
		Some(Xcm(sp_std::vec![Trap(TrapCode::get())]))
	}
}

#[frame_support::pallet]
pub mod pallet {
	pub use crate::weights::WeightInfo;

	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use xcm::latest::Error as XcmError;
	use xcm_builder::ExporterFor;
	use xcm_executor::traits::TransactAsset;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Everything we need to run benchmarks.
	#[cfg(feature = "runtime-benchmarks")]
	pub trait BenchmarkHelper<RuntimeOrigin> {
		/// Returns proper bridge configuration, supported by the runtime.
		///
		/// We expect that the XCM environment (`BridgeXcmSender`) has everything enabled
		/// to support transfer to this destination **after** `prepare_asset_transfer` call.
		fn bridge_config() -> (NetworkId, BridgeConfig);

		/// Prepare environment for assets transfer and return transfer origin and assets
		/// to transfer. After this function is called, we expect `transfer_asset_via_bridge`
		/// to succeed, so in proper environment, it should:
		///
		/// - deposit enough funds (fee from `bridge_config()` and transferred assets) to the sender account;
		///
		/// - ensure that the `BridgeXcmSender` is properly configured for the transfer;
		///
		/// - be close to the worst possible scenario - i.e. if some account may need to be created during
		///   the assets transfer, it should be created. If there are multiple bridges, the "worst possible"
		///   (in terms of performance) bridge must be selected for the transfer.
		fn prepare_asset_transfer(
			assets_count: u32,
		) -> (RuntimeOrigin, VersionedMultiAssets, VersionedMultiLocation);

		/// Prepare environment for ping transfer and return transfer origin and assets
		/// to transfer. After this function is called, we expect `ping_via_bridge`
		/// to succeed, so in proper environment, it should:
		///
		/// - deposit enough funds (fee from `bridge_config()`) to the sender account;
		///
		/// - ensure that the `BridgeXcmSender` is properly configured for the transfer;
		///
		/// - be close to the worst possible scenario - i.e. if some account may need to be created during
		///  it should be created. If there are multiple bridges, the "worst possible"
		///   (in terms of performance) bridge must be selected for the transfer.
		fn prepare_ping() -> (RuntimeOrigin, VersionedMultiLocation);
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// XCM sender which sends messages to the BridgeHub
		type BridgeXcmSender: SendXcm;

		/// Runtime's universal location
		type UniversalLocation: Get<InteriorMultiLocation>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// How to withdraw and deposit an asset for reserve.
		type AssetTransactor: TransactAsset;

		/// The configurable origin to allow bridges configuration management
		type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Required origin for asset transfer. If successful, it resolves to `MultiLocation`.
		type TransferAssetOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = MultiLocation>;

		/// Required origin for ping transfer. If successful, it resolves to `MultiLocation`.
		type TransferPingOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = MultiLocation>;

		/// Configurable ping message, `None` means no message will be transferred.
		type PingMessageBuilder: PingMessageBuilder;

		/// Benchmarks helper.
		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper: BenchmarkHelper<Self::RuntimeOrigin>;
	}

	/// Details of configured bridges which are allowed for transfer.
	#[pallet::storage]
	#[pallet::getter(fn bridges)]
	pub(super) type Bridges<T: Config> = StorageMap<_, Blake2_128Concat, NetworkId, BridgeConfig>;

	#[pallet::error]
	#[cfg_attr(test, derive(PartialEq))]
	pub enum Error<T> {
		InvalidConfiguration,
		InvalidAssets,
		MaxAssetsLimitReached,
		UnsupportedDestination,
		BridgeCallError,
		FailedToReserve,
		UnsupportedPing,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Transfer was successfully entered to the system (does not mean already delivered)
		TransferInitiated(XcmHash),

		/// New bridge configuration was added
		BridgeAdded,
		/// Bridge configuration was removed
		BridgeRemoved,
		/// Bridge configuration was updated
		BridgeUpdated,

		/// Reserve asset passed
		ReserveAssetsDeposited { from: MultiLocation, to: MultiLocation, assets: MultiAssets },
		/// Reserve asset failed
		FailedToReserve(XcmError),

		/// Bridge transfer failed
		BridgeCallError(SendError),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Transfer asset via bridge to different global consensus
		///
		/// Parameters:
		///
		/// * `assets`:
		/// * `destination`: Different consensus location, where the assets will be deposited, e.g. Polkadot's Statemint: `2, X2(GlobalConsensus(NetworkId::Polkadot), Parachain(1000))`
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::transfer_asset_via_bridge())]
		pub fn transfer_asset_via_bridge(
			origin: OriginFor<T>,
			assets: Box<VersionedMultiAssets>,
			destination: Box<VersionedMultiLocation>,
		) -> DispatchResult {
			let origin_location = T::TransferAssetOrigin::ensure_origin(origin)?;

			// Check remote destination + bridge_config
			let (_, bridge_config, remote_destination) =
				Self::ensure_remote_destination(*destination)?;

			// Check reserve account - sovereign account of bridge
			let reserve_account = bridge_config.bridge_location;
			let allowed_target_location = bridge_config.allowed_target_location;

			// TODO: do some checks - balances, can_withdraw, ...
			// TODO:check-parameter - check assets:
			// TODO:check-parameter - check assets - allow just fungible or non-fungible? allow mix?
			// TODO:check-parameter - check assets - allow Abstract assets - reanchor ignores them?
			// TODO:check-parameter - check enought fee?
			// TODO:check-parameter - reserve_account has enought for existential deposit?

			// TODO: fix this for multiple assets
			let assets: MultiAssets =
				(*assets).try_into().map_err(|()| Error::<T>::InvalidAssets)?;
			ensure!(assets.len() == 1, Error::<T>::MaxAssetsLimitReached);
			let asset = assets.get(0).unwrap();

			// Deposit assets into `AccountId` that corresponds to the bridge
			// hub. In this way, Statemine acts as a reserve location to the
			// bridge, such that it need not trust any consensus system from
			// `./Parent/Parent/...`. (It may trust Polkadot, but would
			// Polkadot trust Kusama with its DOT?)

			// Move asset to reserve account for selected bridge
			let mut asset = T::AssetTransactor::transfer_asset(
				asset,
				&origin_location,
				&reserve_account,
				// We aren't able to track the XCM that initiated the fee deposit, so we create a
				// fake message hash here
				&XcmContext::with_message_hash([0; 32]),
			)
			.and_then(|assets| {
				Self::deposit_event(Event::ReserveAssetsDeposited {
					from: origin_location,
					to: reserve_account,
					assets: assets.clone().into(),
				});
				Ok(assets)
			})
			.map_err(|e| {
				log::error!(
					target: LOG_TARGET,
					"AssetTransactor failed to reserve assets from origin_location: {:?} to reserve_account: {:?} for assets: {:?}, error: {:?}",
					origin_location,
					reserve_account,
					asset,
					e
				);
				Self::deposit_event(Event::FailedToReserve(e));
				Error::<T>::FailedToReserve
			})?;

			// TODO: asset.clone for compensation + add test for compensation

			// prepare ReserveAssetDeposited msg to bridge to the other side - reanchor stuff
			// We need to convert local asset's id/MultiLocation to format, that could be understood by different consensus and from their point-of-view
			// assets.prepend_location(&T::UniversalLocation::get().into_location());
			asset.reanchor(&allowed_target_location, T::UniversalLocation::get(), None);
			let remote_destination = remote_destination
				.reanchored(&allowed_target_location, T::UniversalLocation::get())
				.expect("TODO: handle compenstaion?");

			let xcm: Xcm<()> = sp_std::vec![
				// TODO:check-parameter - setup fees
				UnpaidExecution { weight_limit: Unlimited, check_origin: None },
				ReserveAssetDeposited(asset.into()),
				ClearOrigin,
				DepositAsset { assets: All.into(), beneficiary: remote_destination }
			]
			.into();

			// TODO: how to compensate if this call fails?
			Self::initiate_bridge_transfer(allowed_target_location, xcm).map_err(Into::into)
		}

		/// Transfer `ping` via bridge to different global consensus.
		///
		/// - can be used for testing purposes that bridge transfer is working and configured for `destination`
		///
		/// Parameters:
		///
		/// * `destination`: Different consensus location, e.g. Polkadot's Statemint: `2, X2(GlobalConsensus(NetworkId::Polkadot), Parachain(1000))`
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::ping_via_bridge())]
		pub fn ping_via_bridge(
			origin: OriginFor<T>,
			destination: Box<VersionedMultiLocation>,
		) -> DispatchResult {
			let origin_location = T::TransferPingOrigin::ensure_origin(origin)?;

			// Check remote destination + bridge_config
			let (network, bridge_config, remote_destination) =
				Self::ensure_remote_destination(*destination)?;

			// Check reserve account - sovereign account of bridge
			let allowed_target_location = bridge_config.allowed_target_location;

			// Prepare `ping` message
			let xcm: Xcm<()> =
				T::PingMessageBuilder::try_build(&origin_location, &network, &remote_destination)
					.ok_or(Error::<T>::UnsupportedPing)?;

			// Initiate bridge transfer
			Self::initiate_bridge_transfer(allowed_target_location, xcm).map_err(Into::into)
		}

		/// Adds new bridge configuration, which allows transfer to this `bridged_network`.
		///
		/// Parameters:
		///
		/// * `bridged_network`: Network where we want to allow transfer funds
		/// * `bridge_config`: contains location for BridgeHub in our network + fee
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::add_bridge_config())]
		pub fn add_bridge_config(
			origin: OriginFor<T>,
			bridged_network: NetworkId,
			bridge_config: Box<BridgeConfig>,
		) -> DispatchResult {
			let _ = T::AdminOrigin::ensure_origin(origin)?;
			ensure!(!Bridges::<T>::contains_key(bridged_network), Error::<T>::InvalidConfiguration);
			let allowed_target_location_network = bridge_config
				.allowed_target_location
				.interior()
				.global_consensus()
				.map_err(|_| Error::<T>::InvalidConfiguration)?;
			ensure!(
				bridged_network == allowed_target_location_network,
				Error::<T>::InvalidConfiguration
			);

			Bridges::<T>::insert(bridged_network, bridge_config);
			Self::deposit_event(Event::BridgeAdded);
			Ok(())
		}

		/// Remove bridge configuration for specified `bridged_network`.
		///
		/// Parameters:
		///
		/// * `bridged_network`: Network where we want to remove
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::remove_bridge_config())]
		pub fn remove_bridge_config(
			origin: OriginFor<T>,
			bridged_network: NetworkId,
		) -> DispatchResult {
			let _ = T::AdminOrigin::ensure_origin(origin)?;
			ensure!(Bridges::<T>::contains_key(bridged_network), Error::<T>::InvalidConfiguration);

			Bridges::<T>::remove(bridged_network);
			Self::deposit_event(Event::BridgeRemoved);
			Ok(())
		}

		/// Updates bridge configuration for specified `bridged_network`.
		///
		/// Parameters:
		///
		/// * `bridged_network`: Network where we want to remove
		/// * `fee`: New fee to update
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::update_bridge_config())]
		pub fn update_bridge_config(
			origin: OriginFor<T>,
			bridged_network: NetworkId,
			fee: Option<MultiAsset>,
		) -> DispatchResult {
			let _ = T::AdminOrigin::ensure_origin(origin)?;
			ensure!(Bridges::<T>::contains_key(bridged_network), Error::<T>::InvalidConfiguration);

			Bridges::<T>::try_mutate_exists(bridged_network, |bridge_config| {
				let deposit = bridge_config.as_mut().ok_or(Error::<T>::InvalidConfiguration)?;
				deposit.fee = fee;
				Self::deposit_event(Event::BridgeUpdated);
				Ok(())
			})
		}
	}

	impl<T: Config> Pallet<T> {
		/// Validates destination and check if we support bridging to this remote global consensus
		///
		/// Returns: correct remote location, where we should be able to bridge
		pub(crate) fn ensure_remote_destination(
			remote_destination: VersionedMultiLocation,
		) -> Result<(NetworkId, BridgeConfig, MultiLocation), Error<T>> {
			match remote_destination {
				VersionedMultiLocation::V3(remote_location) => {
					ensure!(
						remote_location.parent_count() == 2,
						Error::<T>::UnsupportedDestination
					);
					let local_network = T::UniversalLocation::get()
						.global_consensus()
						.map_err(|_| Error::<T>::InvalidConfiguration)?;
					let remote_network = remote_location
						.interior()
						.global_consensus()
						.map_err(|_| Error::<T>::UnsupportedDestination)?;
					ensure!(local_network != remote_network, Error::<T>::UnsupportedDestination);
					match Bridges::<T>::get(remote_network) {
						Some(bridge_config) => {
							ensure!(
								remote_location.starts_with(&bridge_config.allowed_target_location),
								Error::<T>::UnsupportedDestination
							);
							Ok((remote_network, bridge_config, remote_location))
						},
						None => return Err(Error::<T>::UnsupportedDestination),
					}
				},
				_ => Err(Error::<T>::UnsupportedDestination),
			}
		}

		fn get_bridge_for(network: &NetworkId) -> Option<BridgeConfig> {
			Bridges::<T>::get(network)
		}

		fn initiate_bridge_transfer(
			allowed_target_location: MultiLocation,
			xcm: Xcm<()>,
		) -> Result<(), Error<T>> {
			// call bridge
			log::info!(
				target: LOG_TARGET,
				"[T::BridgeXcmSender] send to bridge, allowed_target_location: {:?}, xcm: {:?}",
				allowed_target_location,
				xcm,
			);
			// TODO: use fn send_msg - which does: validate + deliver - but find out what to do with the fees?
			let (ticket, fees) =
				T::BridgeXcmSender::validate(&mut Some(allowed_target_location), &mut Some(xcm))
					.map_err(|e| {
						log::error!(
							target: LOG_TARGET,
							"[BridgeXcmSender::validate] SendError occurred, error: {:?}",
							e
						);
						Self::deposit_event(Event::BridgeCallError(e));
						Error::<T>::BridgeCallError
					})?;
			log::info!(
				target: LOG_TARGET,
				"[T::BridgeXcmSender::validate] (TODO: process) fees: {:?}",
				fees
			);
			// TODO: what to do with fees - we have fees here, pay here or ignore?
			let xcm_hash = T::BridgeXcmSender::deliver(ticket).map_err(|e| {
				log::error!(
					target: LOG_TARGET,
					"[BridgeXcmSender::deliver] SendError occurred, error: {:?}",
					e
				);
				Self::deposit_event(Event::BridgeCallError(e));
				Error::<T>::BridgeCallError
			})?;

			Self::deposit_event(Event::TransferInitiated(xcm_hash));
			Ok(())
		}
	}

	impl<T: Config> ExporterFor for Pallet<T> {
		fn exporter_for(
			network: &NetworkId,
			_remote_location: &InteriorMultiLocation,
			_message: &Xcm<()>,
		) -> Option<(MultiLocation, Option<MultiAsset>)> {
			Pallet::<T>::get_bridge_for(network).map(Into::into)
		}
	}
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use crate as bridge_transfer;
	use frame_support::traits::Currency;

	use frame_support::{
		assert_noop, assert_ok, dispatch::DispatchError, parameter_types, sp_io, sp_tracing,
	};
	use frame_system::EnsureRoot;
	use polkadot_parachain::primitives::Sibling;
	use sp_runtime::{
		testing::{Header, H256},
		traits::{BlakeTwo256, IdentityLookup},
		AccountId32, ModuleError,
	};
	use sp_version::RuntimeVersion;
	use xcm_builder::{
		AccountId32Aliases, CurrencyAdapter, EnsureXcmOrigin, ExporterFor, IsConcrete,
		SiblingParachainConvertsVia, SignedToAccountId32, UnpaidRemoteExporter,
	};
	use xcm_executor::traits::Convert;

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
	type Block = frame_system::mocking::MockBlock<TestRuntime>;

	frame_support::construct_runtime!(
		pub enum TestRuntime where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
			Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
			BridgeTransfer: bridge_transfer::{Pallet, Call, Event<T>} = 52,
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

	pub type AccountId = AccountId32;

	impl frame_system::Config for TestRuntime {
		type RuntimeOrigin = RuntimeOrigin;
		type RuntimeCall = RuntimeCall;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = AccountId;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type RuntimeEvent = RuntimeEvent;
		type BlockHashCount = BlockHashCount;
		type BlockLength = ();
		type BlockWeights = ();
		type Version = Version;
		type PalletInfo = PalletInfo;
		type AccountData = pallet_balances::AccountData<u64>;
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
		pub const ExistentialDeposit: u64 = 5;
		pub const MaxReserves: u32 = 50;
	}

	impl pallet_balances::Config for TestRuntime {
		type Balance = u64;
		type RuntimeEvent = RuntimeEvent;
		type DustRemoval = ();
		type ExistentialDeposit = ExistentialDeposit;
		type AccountStore = System;
		type WeightInfo = ();
		type MaxLocks = ();
		type MaxReserves = MaxReserves;
		type ReserveIdentifier = [u8; 8];
	}

	parameter_types! {
		// UniversalLocation as statemine
		pub const RelayNetwork: NetworkId = NetworkId::Kusama;
		pub UniversalLocation: InteriorMultiLocation = X2(GlobalConsensus(RelayNetwork::get()), Parachain(1000));
		// Test bridge cfg
		pub TestBridgeTable: sp_std::prelude::Vec<(NetworkId, MultiLocation, Option<MultiAsset>)> = sp_std::vec![
			(NetworkId::Wococo, (Parent, Parachain(1013)).into(), None),
			(NetworkId::Polkadot, (Parent, Parachain(1002)).into(), None),
		];
		// Relay chain currency/balance location (e.g. KsmLocation, DotLocation, ..)
		pub const RelayLocation: MultiLocation = MultiLocation::parent();
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
	pub type TestBridgeXcmSender =
		UnpaidRemoteExporter<BridgeTransfer, ThreadLocalXcmRouter, UniversalLocation>;

	/// No local origins on this chain are allowed to dispatch XCM sends/executions.
	pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

	pub type LocationToAccountId = (
		// Sibling parachain origins convert to AccountId via the `ParaId::into`.
		SiblingParachainConvertsVia<Sibling, AccountId>,
		// Straight up local `AccountId32` origins just alias directly to `AccountId`.
		AccountId32Aliases<RelayNetwork, AccountId>,
	);

	/// Means for transacting the native currency on this chain.
	pub type CurrencyTransactor = CurrencyAdapter<
		// Use this currency:
		Balances,
		// Use this currency when it is a fungible asset matching the given location or name:
		IsConcrete<RelayLocation>,
		// Convert an XCM MultiLocation into a local account id:
		LocationToAccountId,
		// Our chain's account ID type (we can't get away without mentioning it explicitly):
		AccountId,
		// We don't track any teleports of `Balances`.
		(),
	>;

	/// Bridge configuration we use in our tests.
	fn test_bridge_config() -> (NetworkId, BridgeConfig) {
		(
			Wococo,
			BridgeConfig {
				bridge_location: (Parent, Parachain(1013)).into(),
				allowed_target_location: MultiLocation::new(
					2,
					X2(GlobalConsensus(Wococo), Parachain(1000)),
				),
				fee: None,
			},
		)
	}

	/// Benchmarks helper.
	#[cfg(feature = "runtime-benchmarks")]
	pub struct TestBenchmarkHelper;

	#[cfg(feature = "runtime-benchmarks")]
	impl BenchmarkHelper<RuntimeOrigin> for TestBenchmarkHelper {
		fn bridge_config() -> (NetworkId, BridgeConfig) {
			test_bridge_config()
		}

		fn prepare_asset_transfer(
			assets_count: u32,
		) -> (RuntimeOrigin, VersionedMultiAssets, VersionedMultiLocation) {
			// sender account must have enough funds
			let sender_account = account(1);
			let total_deposit = ExistentialDeposit::get() * (1 + assets_count as u64);
			let _ = Balances::deposit_creating(&sender_account, total_deposit);

			// finally - prepare assets and destination
			let assets = VersionedMultiAssets::V3(
				std::iter::repeat(MultiAsset {
					fun: Fungible(ExistentialDeposit::get().into()),
					id: Concrete(RelayLocation::get()),
				})
				.take(assets_count as usize)
				.collect::<Vec<_>>()
				.into(),
			);
			let destination = VersionedMultiLocation::V3(MultiLocation::new(
				2,
				X3(GlobalConsensus(Wococo), Parachain(1000), consensus_account(Wococo, 2)),
			));

			(RuntimeOrigin::signed(sender_account), assets, destination)
		}

		fn prepare_ping() {
			unimplemented!("Not implemented here - not needed");
		}
	}

	parameter_types! {
		pub const TrapCode: u64 = 12345;
	}

	impl Config for TestRuntime {
		type RuntimeEvent = RuntimeEvent;
		type BridgeXcmSender = TestBridgeXcmSender;
		type UniversalLocation = UniversalLocation;
		type WeightInfo = ();
		type AssetTransactor = CurrencyTransactor;
		type AdminOrigin = EnsureRoot<AccountId>;
		type TransferAssetOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
		type TransferPingOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
		type PingMessageBuilder = UnpaidTrapMessageBuilder<TrapCode>;
		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper = TestBenchmarkHelper;
	}

	pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
		sp_tracing::try_init_simple();
		let t = frame_system::GenesisConfig::default().build_storage::<TestRuntime>().unwrap();

		// with 0 block_number events dont work
		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| {
			frame_system::Pallet::<TestRuntime>::set_block_number(1u32.into());
		});

		ext
	}

	fn account(account: u8) -> AccountId32 {
		AccountId32::new([account; 32])
	}

	fn consensus_account(network: NetworkId, account: u8) -> Junction {
		xcm::prelude::AccountId32 {
			network: Some(network),
			id: AccountId32::new([account; 32]).into(),
		}
	}

	#[test]
	fn test_ensure_remote_destination() {
		new_test_ext().execute_with(|| {
			// insert bridge config
			let bridge_network = Wococo;
			let bridge_config = test_bridge_config().1;
			assert_ok!(BridgeTransfer::add_bridge_config(
				RuntimeOrigin::root(),
				bridge_network,
				Box::new(bridge_config.clone()),
			));

			// v2 not supported
			assert_eq!(
				BridgeTransfer::ensure_remote_destination(VersionedMultiLocation::V2(
					xcm::v2::MultiLocation::default()
				)),
				Err(Error::<TestRuntime>::UnsupportedDestination)
			);

			// v3 - "parent: 0" wrong
			assert_eq!(
				BridgeTransfer::ensure_remote_destination(VersionedMultiLocation::V3(
					MultiLocation::new(0, X2(GlobalConsensus(Wococo), Parachain(1000)))
				)),
				Err(Error::<TestRuntime>::UnsupportedDestination)
			);
			// v3 - "parent: 1" wrong
			assert_eq!(
				BridgeTransfer::ensure_remote_destination(VersionedMultiLocation::V3(
					MultiLocation::new(1, X2(GlobalConsensus(Wococo), Parachain(1000)))
				)),
				Err(Error::<TestRuntime>::UnsupportedDestination)
			);

			// v3 - Rococo is not supported
			assert_eq!(
				BridgeTransfer::ensure_remote_destination(VersionedMultiLocation::V3(
					MultiLocation::new(2, X2(GlobalConsensus(Rococo), Parachain(1000)))
				)),
				Err(Error::<TestRuntime>::UnsupportedDestination)
			);

			// v3 - remote_destination is not allowed
			assert_eq!(
				BridgeTransfer::ensure_remote_destination(VersionedMultiLocation::V3(
					MultiLocation::new(2, X2(GlobalConsensus(Wococo), Parachain(1234)))
				)),
				Err(Error::<TestRuntime>::UnsupportedDestination)
			);

			// v3 - ok (allowed)
			assert_eq!(
				BridgeTransfer::ensure_remote_destination(VersionedMultiLocation::V3(
					MultiLocation::new(2, X2(GlobalConsensus(Wococo), Parachain(1000)))
				)),
				Ok((
					bridge_network,
					bridge_config,
					MultiLocation::new(2, X2(GlobalConsensus(Wococo), Parachain(1000)))
				))
			);
		})
	}

	// TODO: add test for pallet_asset not only blances
	#[test]
	fn test_transfer_asset_via_bridge_for_currency_works() {
		new_test_ext().execute_with(|| {
			// initialize some Balances for user_account
			let user_account = account(1);
			let user_account_init_balance = 1000_u64;
			let _ = Balances::deposit_creating(&user_account, user_account_init_balance);
			let user_free_balance = Balances::free_balance(&user_account);
			let balance_to_transfer = 15_u64;
			assert!((user_free_balance - balance_to_transfer) >= ExistentialDeposit::get());
			// TODO: because, sovereign account needs to have ED otherwise reserve fails
			assert!(balance_to_transfer >= ExistentialDeposit::get());

			// insert bridge config
			let bridged_network = Wococo;
			assert_ok!(BridgeTransfer::add_bridge_config(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(test_bridge_config().1),
			));
			let bridge_location = Bridges::<TestRuntime>::get(bridged_network)
				.expect("stored BridgeConfig for bridged_network")
				.bridge_location;

			// checks before
			assert!(ROUTED_MESSAGE.with(|r| r.borrow().is_none()));
			assert_eq!(Balances::free_balance(&user_account), user_account_init_balance);
			let bridge_location_as_sovereign_account =
				SiblingParachainConvertsVia::<Sibling, AccountId>::convert_ref(bridge_location)
					.expect("converted bridge location as accountId");
			assert_eq!(Balances::free_balance(&bridge_location_as_sovereign_account), 0);

			// trigger transfer_asset_via_bridge - should trigger new ROUTED_MESSAGE
			let asset = MultiAsset {
				fun: Fungible(balance_to_transfer.into()),
				id: Concrete(RelayLocation::get()),
			};
			let assets = Box::new(VersionedMultiAssets::V3(asset.into()));

			// destination is account from different consensus
			let destination = Box::new(VersionedMultiLocation::V3(MultiLocation::new(
				2,
				X3(GlobalConsensus(Wococo), Parachain(1000), consensus_account(Wococo, 2)),
			)));

			// trigger asset transfer
			assert_ok!(BridgeTransfer::transfer_asset_via_bridge(
				RuntimeOrigin::signed(account(1)),
				assets,
				destination,
			));

			// check user account decressed
			assert_eq!(
				Balances::free_balance(&user_account),
				user_account_init_balance - balance_to_transfer
			);
			// check reserve account increased
			assert_eq!(Balances::free_balance(&bridge_location_as_sovereign_account), 15);

			// check events
			let events = System::events();
			assert!(!events.is_empty());

			// check reserve asset deposited event
			assert!(System::events().iter().any(|r| matches!(
				r.event,
				RuntimeEvent::BridgeTransfer(Event::ReserveAssetsDeposited { .. })
			)));
			assert!(System::events().iter().any(|r| matches!(
				r.event,
				RuntimeEvent::BridgeTransfer(Event::TransferInitiated { .. })
			)));

			// check fired XCM ExportMessage to bridge-hub
			let fired_xcm =
				ROUTED_MESSAGE.with(|r| r.take().expect("xcm::ExportMessage should be here"));

			if let Some(ExportMessage { xcm, .. }) = fired_xcm.0.iter().find(|instr| {
				matches!(
					instr,
					ExportMessage { network: Wococo, destination: X1(Parachain(1000)), .. }
				)
			}) {
				assert!(xcm.0.iter().any(|instr| matches!(instr, ReserveAssetDeposited(..))));
				assert!(xcm.0.iter().any(|instr| matches!(instr, ClearOrigin)));
				assert!(xcm.0.iter().any(|instr| matches!(instr, DepositAsset { .. })));
			} else {
				assert!(false, "Does not contains [`ExportMessage`], fired_xcm: {:?}", fired_xcm);
			}
		});
	}

	#[test]
	fn test_ping_via_bridge_works() {
		new_test_ext().execute_with(|| {
			// insert bridge config
			let bridged_network = Wococo;
			assert_ok!(BridgeTransfer::add_bridge_config(
				RuntimeOrigin::root(),
				bridged_network,
				Box::new(test_bridge_config().1),
			));

			// checks before
			assert!(ROUTED_MESSAGE.with(|r| r.borrow().is_none()));

			// trigger ping_via_bridge - should trigger new ROUTED_MESSAGE
			// destination is account from different consensus
			let destination = Box::new(VersionedMultiLocation::V3(MultiLocation::new(
				2,
				X3(GlobalConsensus(Wococo), Parachain(1000), consensus_account(Wococo, 2)),
			)));

			// trigger asset transfer
			assert_ok!(BridgeTransfer::ping_via_bridge(
				RuntimeOrigin::signed(account(1)),
				destination,
			));

			// check events
			let events = System::events();
			assert!(!events.is_empty());

			// check TransferInitiated
			assert!(System::events().iter().any(|r| matches!(
				r.event,
				RuntimeEvent::BridgeTransfer(Event::TransferInitiated { .. })
			)));

			// check fired XCM ExportMessage to bridge-hub
			let fired_xcm =
				ROUTED_MESSAGE.with(|r| r.take().expect("xcm::ExportMessage should be here"));

			if let Some(ExportMessage { xcm, .. }) = fired_xcm.0.iter().find(|instr| {
				matches!(
					instr,
					ExportMessage { network: Wococo, destination: X1(Parachain(1000)), .. }
				)
			}) {
				assert!(xcm.0.iter().any(|instr| instr.eq(&Trap(TrapCode::get()))));
			} else {
				assert!(false, "Does not contains [`ExportMessage`], fired_xcm: {:?}", fired_xcm);
			}
		});
	}

	#[test]
	fn test_bridge_config_management_works() {
		let bridged_network = Rococo;
		let bridged_config = Box::new(BridgeConfig {
			bridge_location: (Parent, Parachain(1013)).into(),
			allowed_target_location: MultiLocation::new(
				2,
				X2(GlobalConsensus(bridged_network), Parachain(1000)),
			),
			fee: None,
		});
		let dummy_xcm = Xcm(vec![]);
		let dummy_remote_interior_multilocation = X1(Parachain(1234));

		new_test_ext().execute_with(|| {
			assert_eq!(Bridges::<TestRuntime>::iter().count(), 0);

			// should fail - just root is allowed
			assert_noop!(
				BridgeTransfer::add_bridge_config(
					RuntimeOrigin::signed(account(1)),
					bridged_network,
					bridged_config.clone(),
				),
				DispatchError::BadOrigin
			);

			// should fail - cannot bridged_network should match allowed_target_location
			assert_noop!(
				BridgeTransfer::add_bridge_config(RuntimeOrigin::root(), bridged_network, {
					let remote_network = Westend;
					assert_ne!(bridged_network, remote_network);
					Box::new(test_bridge_config().1)
				}),
				DispatchError::Module(ModuleError {
					index: 52,
					error: [0, 0, 0, 0],
					message: Some("InvalidConfiguration")
				})
			);
			assert_eq!(Bridges::<TestRuntime>::iter().count(), 0);
			assert_eq!(
				BridgeTransfer::exporter_for(
					&bridged_network,
					&dummy_remote_interior_multilocation,
					&dummy_xcm
				),
				None
			);

			// add with root
			assert_ok!(BridgeTransfer::add_bridge_config(
				RuntimeOrigin::root(),
				bridged_network,
				bridged_config.clone(),
			));
			assert_eq!(Bridges::<TestRuntime>::iter().count(), 1);
			assert_eq!(
				Bridges::<TestRuntime>::get(bridged_network),
				Some((*bridged_config.clone()).into())
			);
			assert_eq!(Bridges::<TestRuntime>::get(Wococo), None);
			assert_eq!(
				BridgeTransfer::exporter_for(
					&bridged_network,
					&dummy_remote_interior_multilocation,
					&dummy_xcm
				),
				Some((*bridged_config.clone()).into())
			);
			assert_eq!(
				BridgeTransfer::exporter_for(
					&Wococo,
					&dummy_remote_interior_multilocation,
					&dummy_xcm
				),
				None
			);

			// update fee
			// remove
			assert_ok!(BridgeTransfer::update_bridge_config(
				RuntimeOrigin::root(),
				bridged_network,
				Some((Parent, 200u128).into()),
			));
			assert_eq!(Bridges::<TestRuntime>::iter().count(), 1);
			assert_eq!(
				Bridges::<TestRuntime>::get(bridged_network),
				Some(BridgeConfig {
					bridge_location: bridged_config.bridge_location.clone(),
					allowed_target_location: bridged_config.allowed_target_location.clone(),
					fee: Some((Parent, 200u128).into())
				})
			);
			assert_eq!(
				BridgeTransfer::exporter_for(
					&bridged_network,
					&dummy_remote_interior_multilocation,
					&dummy_xcm
				),
				Some((bridged_config.bridge_location, Some((Parent, 200u128).into())))
			);

			// remove
			assert_ok!(BridgeTransfer::remove_bridge_config(
				RuntimeOrigin::root(),
				bridged_network,
			));
			assert_eq!(Bridges::<TestRuntime>::get(bridged_network), None);
			assert_eq!(Bridges::<TestRuntime>::iter().count(), 0);
		})
	}
}
