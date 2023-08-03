// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of Parity Bridges Common.

// Parity Bridges Common is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Bridges Common is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Bridges Common.  If not, see <http://www.gnu.org/licenses/>.

//! Pallet that may be used instead of `SovereignPaidRemoteExporter` in the XCM router
//! configuration. The main thing that the pallet offers is the dynamic message fee,
//! that is computed based on the bridge queues state. It starts exponentially increasing
//! if the queue between this chain and the sibling/child bridge hub is congested.
//!
//! All other bridge hub queues offer some backpressure mechanisms. So if at least one
//! of all queues is congested, it will eventually lead to the growth of the queue at
//! this chain.

#![cfg_attr(not(feature = "std"), no_std)]

use bp_xcm_bridge_hub_router::{BridgeState, LocalXcmChannel};
use codec::Encode;
use frame_support::traits::Get;
use sp_core::H256;
use sp_runtime::{traits::One, FixedPointNumber, FixedU128, Saturating};
use xcm::prelude::*;
use xcm_builder::{ExporterFor, SovereignPaidRemoteExporter};

pub use pallet::*;
pub use weights::WeightInfo;

pub mod benchmarking;
pub mod weights;

mod mock;

/// The factor that is used to increase current message fee factor when bridge experiencing
/// some lags.
const EXPONENTIAL_FEE_BASE: FixedU128 = FixedU128::from_rational(105, 100); // 1.05
/// The factor that is used to increase current message fee factor for every sent kilobyte.
const MESSAGE_SIZE_FEE_BASE: FixedU128 = FixedU128::from_rational(1, 1000); // 0.001

/// Maximal size of the XCM message that may be sent over bridge.
///
/// This should be less than the maximal size, allowed by the messages pallet, because
/// the message itself is wrapped in other structs and is double encoded.
pub const HARD_MESSAGE_SIZE_LIMIT: u32 = 32 * 1024;

/// The target that will be used when publishing logs related to this pallet.
///
/// This doesn't match the pattern used by other bridge pallets (`runtime::bridge-*`). But this
/// pallet has significant differences with those pallets. The main one is that is intended to
/// be deployed at sending chains. Other bridge pallets are likely to be deployed at the separate
/// bridge hub parachain.
pub const LOG_TARGET: &str = "xcm::bridge-hub-router";

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// Benchmarks results from runtime we're plugged into.
		type WeightInfo: WeightInfo;

		/// Universal location of this runtime.
		type UniversalLocation: Get<InteriorMultiLocation>;
		/// Relative location of the sibling bridge hub.
		type SiblingBridgeHubLocation: Get<MultiLocation>;
		/// The bridged network that this config is for.
		type BridgedNetworkId: Get<NetworkId>;

		/// Origin of the sibling bridge hub that is allowed to report bridge status.
		type BridgeHubOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// Actual message sender (`HRMP` or `DMP`) to the sibling bridge hub location.
		type ToBridgeHubSender: SendXcm;
		/// Underlying channel with the sibling bridge hub. It must match the channel, used
		/// by the `Self::ToBridgeHubSender`.
		type WithBridgeHubChannel: LocalXcmChannel;

		/// Base bridge fee that is paid for every outbound message.
		type BaseFee: Get<u128>;
		/// Additional fee that is paid for every byte of the outbound message.
		type ByteFee: Get<u128>;
		/// Asset that is used to paid bridge fee.
		type FeeAsset: Get<AssetId>;
	}

	#[pallet::pallet]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
		fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
			Bridge::<T, I>::mutate(|bridge| {
				// TODO: make sure that `WithBridgeHubChannel::is_congested` returns true if either
				// of XCM channels (outbound/inbound) is suspended. Because if outbound is suspended
				// that is definitely congestion. If inbound is suspended, then we are not able to
				// receive the "report_bridge_status" signal (that maybe sent by the bridge hub).

				// if the channel with sibling/child bridge hub is suspended, we don't change
				// anything
				if T::WithBridgeHubChannel::is_congested() {
					return T::WeightInfo::on_initialize_when_congested()
				}

				// if bridge has reported congestion, we don't change anything
				if bridge.is_congested {
					return T::WeightInfo::on_initialize_when_congested()
				}

				// if fee factor is already minimal, we don't change anything
				if bridge.delivery_fee_factor == FixedU128::one() {
					return T::WeightInfo::on_initialize_when_congested()
				}

				let previous_factor = bridge.delivery_fee_factor;
				bridge.delivery_fee_factor =
					FixedU128::one().max(bridge.delivery_fee_factor / EXPONENTIAL_FEE_BASE);
				log::info!(
					target: LOG_TARGET,
					"Bridge queue is uncongested. Decreased fee factor from {} to {}",
					previous_factor,
					bridge.delivery_fee_factor,
				);

				T::WeightInfo::on_initialize_when_non_congested()
			})
		}
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Notification about congested bridge queue.
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::zero())] // TODO
		pub fn report_bridge_status(
			origin: OriginFor<T>,
			// this argument is not currently used, but to ease future migration, we'll keep it
			// here
			_bridge_id: H256,
			is_congested: bool,
		) -> DispatchResult {
			let _ = T::BridgeHubOrigin::ensure_origin(origin)?;
			Bridge::<T, I>::mutate(|bridge| {
				bridge.is_congested = is_congested;
			});
			Ok(())
		}
	}

	/// Bridge that we are using.
	///
	/// **bridges-v1** assumptions: all outbound messages through this router are using single lane
	/// and to single remote consensus. If there is some other remote consensus that uses the same
	/// bridge hub, the separate pallet instance shall be used, In v2 we'll have all required
	/// primitives (lane-id aka bridge-id, derived from XCM locations) to support multiple  bridges
	/// by the same pallet instance.
	#[pallet::storage]
	#[pallet::getter(fn bridge)]
	pub type Bridge<T: Config<I>, I: 'static = ()> = StorageValue<_, BridgeState, ValueQuery>;

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Called when new message is sent (queued to local outbound XCM queue) over the bridge.
		pub(crate) fn on_message_sent_to_bridge(message_size: u32) {
			let _ = Bridge::<T, I>::try_mutate(|bridge| {
				let is_channel_with_bridge_hub_congested = T::WithBridgeHubChannel::is_congested();
				let is_bridge_congested = bridge.is_congested;

				// if outbound queue is not congested AND bridge has not reported congestion, do
				// nothing
				if !is_channel_with_bridge_hub_congested && !is_bridge_congested {
					return Err(())
				}

				// ok - we need to increase the fee factor, let's do that
				let message_size_factor = FixedU128::from_u32(message_size.saturating_div(1024))
					.saturating_mul(MESSAGE_SIZE_FEE_BASE);
				let total_factor = EXPONENTIAL_FEE_BASE.saturating_add(message_size_factor);
				let previous_factor = bridge.delivery_fee_factor;
				bridge.delivery_fee_factor =
					bridge.delivery_fee_factor.saturating_mul(total_factor);

				log::info!(
					target: LOG_TARGET,
					"Bridge queue is congested. Increased fee factor from {} to {}",
					previous_factor,
					bridge.delivery_fee_factor,
				);

				Ok(())
			});
		}
	}
}

/// We'll be using `SovereignPaidRemoteExporter` to send remote messages over the sibling/child
/// bridge hub.
type ViaBridgeHubExporter<T, I> = SovereignPaidRemoteExporter<
	Pallet<T, I>,
	<T as Config<I>>::ToBridgeHubSender,
	<T as Config<I>>::UniversalLocation,
>;

// This pallet acts as the `ExporterFor` for the `SovereignPaidRemoteExporter` to compute
// message fee using fee factor.
impl<T: Config<I>, I: 'static> ExporterFor for Pallet<T, I> {
	fn exporter_for(
		network: &NetworkId,
		remote_location: &InteriorMultiLocation,
		message: &Xcm<()>,
	) -> Option<(MultiLocation, Option<MultiAsset>)> {
		// ensure that the message is sent to the expected bridged network
		if *network != T::BridgedNetworkId::get() {
			return None
		}

		// compute fee amount. Keep in mind that this is only the bridge fee. The fee for sending
		// message from this chain to child/sibling bridge hub is determined by the
		// `Config::ToBridgeHubSender`
		let message_size = message.encoded_size();
		let message_fee = (message_size as u128).saturating_mul(T::ByteFee::get());
		let fee_sum = T::BaseFee::get().saturating_add(message_fee);
		let fee_factor = Self::bridge().delivery_fee_factor;
		let fee = fee_factor.saturating_mul_int(fee_sum);

		log::info!(
			target: LOG_TARGET,
			"Going to send message to {:?} ({} bytes) over bridge. Computed bridge fee {} using fee factor {}",
			(network, remote_location),
			fee,
			fee_factor,
			message_size,
		);

		Some((T::SiblingBridgeHubLocation::get(), Some((T::FeeAsset::get(), fee).into())))
	}
}

// This pallet acts as the `SendXcm` to the sibling/child bridge hub instead of regular
// XCMP/DMP transport. This allows injecting dynamic message fees into XCM programs that
// are going to the bridged network.
impl<T: Config<I>, I: 'static> SendXcm for Pallet<T, I> {
	type Ticket = (u32, <T::ToBridgeHubSender as SendXcm>::Ticket);

	fn validate(
		dest: &mut Option<MultiLocation>,
		xcm: &mut Option<Xcm<()>>,
	) -> SendResult<Self::Ticket> {
		// we won't have an access to `dest` and `xcm` in the `delvier` method, so precompute
		// everything required here
		let message_size = xcm
			.as_ref()
			.map(|xcm| xcm.encoded_size() as _)
			.ok_or(SendError::MissingArgument)?;

		// bridge doesn't support oversized/overweight messages now. So it is better to drop such
		// messages here than at the bridge hub. Let's check the message size.
		if message_size > HARD_MESSAGE_SIZE_LIMIT {
			return Err(SendError::ExceedsMaxMessageSize)
		}

		// just use exporter to validate destination and insert instructions to pay message fee
		// at the sibling/child bridge hub
		//
		// the cost will include both cost of: (1) to-sibling bridg hub delivery (returned by
		// the `Config::ToBridgeHubSender`) and (2) to-bridged bridge hub delivery (returned by
		// `Self::exporter_for`)
		ViaBridgeHubExporter::<T, I>::validate(dest, xcm)
			.map(|(ticket, cost)| ((message_size, ticket), cost))
	}

	fn deliver(ticket: Self::Ticket) -> Result<XcmHash, SendError> {
		// use router to enqueue message to the sibling/child bridge hub. This also should handle
		// payment for passing through this queue.
		let (message_size, ticket) = ticket;
		let xcm_hash = ViaBridgeHubExporter::<T, I>::deliver(ticket)?;

		// increase delivery fee factor if required
		Self::on_message_sent_to_bridge(message_size);

		Ok(xcm_hash)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use mock::*;

	use frame_support::traits::Hooks;

	fn congested_bridge(delivery_fee_factor: FixedU128) -> BridgeState {
		BridgeState { is_congested: true, delivery_fee_factor }
	}

	fn uncongested_bridge(delivery_fee_factor: FixedU128) -> BridgeState {
		BridgeState { is_congested: false, delivery_fee_factor }
	}

	#[test]
	fn initial_fee_factor_is_one() {
		run_test(|| {
			assert_eq!(Bridge::<TestRuntime, ()>::get(), uncongested_bridge(FixedU128::one()),);
		})
	}

	#[test]
	fn fee_factor_is_not_decreased_from_on_initialize_when_xcm_channel_is_congested() {
		run_test(|| {
			Bridge::<TestRuntime, ()>::put(uncongested_bridge(FixedU128::from_rational(125, 100)));
			TestWithBridgeHubChannel::make_congested();

			// it should not decrease, because xcm channel is congested
			let old_bridge = XcmBridgeHubRouter::bridge();
			XcmBridgeHubRouter::on_initialize(One::one());
			assert_eq!(XcmBridgeHubRouter::bridge(), old_bridge);
		})
	}

	#[test]
	fn fee_factor_is_not_decreased_from_on_initialize_when_bridge_has_reported_congestion() {
		run_test(|| {
			Bridge::<TestRuntime, ()>::put(congested_bridge(FixedU128::from_rational(125, 100)));

			// it should not decrease, because bridge congested
			let old_bridge = XcmBridgeHubRouter::bridge();
			XcmBridgeHubRouter::on_initialize(One::one());
			assert_eq!(XcmBridgeHubRouter::bridge(), old_bridge);
		})
	}

	#[test]
	fn fee_factor_is_decreased_from_on_initialize_when_xcm_channel_is_uncongested() {
		run_test(|| {
			Bridge::<TestRuntime, ()>::put(uncongested_bridge(FixedU128::from_rational(125, 100)));

			// it shold eventually decreased to one
			while XcmBridgeHubRouter::bridge().delivery_fee_factor > FixedU128::one() {
				XcmBridgeHubRouter::on_initialize(One::one());
			}

			// verify that it doesn't decreases anymore
			XcmBridgeHubRouter::on_initialize(One::one());
			assert_eq!(XcmBridgeHubRouter::bridge(), uncongested_bridge(FixedU128::one()));
		})
	}

	#[test]
	fn not_applicable_if_destination_is_within_other_network() {
		run_test(|| {
			assert_eq!(
				send_xcm::<XcmBridgeHubRouter>(
					MultiLocation::new(2, X2(GlobalConsensus(Rococo), Parachain(1000))),
					vec![].into(),
				),
				Err(SendError::NotApplicable),
			);
		});
	}

	#[test]
	fn exceeds_max_message_size_if_size_is_above_hard_limit() {
		run_test(|| {
			assert_eq!(
				send_xcm::<XcmBridgeHubRouter>(
					MultiLocation::new(2, X2(GlobalConsensus(Rococo), Parachain(1000))),
					vec![ClearOrigin; HARD_MESSAGE_SIZE_LIMIT as usize].into(),
				),
				Err(SendError::ExceedsMaxMessageSize),
			);
		});
	}

	#[test]
	fn returns_proper_delivery_price() {
		run_test(|| {
			let dest = MultiLocation::new(2, X1(GlobalConsensus(BridgedNetworkId::get())));
			let xcm: Xcm<()> = vec![ClearOrigin].into();
			let msg_size = xcm.encoded_size();

			// initially the base fee is used: `BASE_FEE + BYTE_FEE * msg_size + HRMP_FEE`
			let expected_fee = BASE_FEE + BYTE_FEE * (msg_size as u128) + HRMP_FEE;
			assert_eq!(
				XcmBridgeHubRouter::validate(&mut Some(dest), &mut Some(xcm.clone()))
					.unwrap()
					.1
					.get(0),
				Some(&(BridgeFeeAsset::get(), expected_fee).into()),
			);

			// but when factor is larger than one, it increases the fee, so it becomes:
			// `(BASE_FEE + BYTE_FEE * msg_size) * F + HRMP_FEE`
			let factor = FixedU128::from_rational(125, 100);
			Bridge::<TestRuntime, ()>::put(uncongested_bridge(factor));
			let expected_fee =
				(FixedU128::saturating_from_integer(BASE_FEE + BYTE_FEE * (msg_size as u128)) *
					factor)
					.into_inner() / FixedU128::DIV +
					HRMP_FEE;
			assert_eq!(
				XcmBridgeHubRouter::validate(&mut Some(dest), &mut Some(xcm)).unwrap().1.get(0),
				Some(&(BridgeFeeAsset::get(), expected_fee).into()),
			);
		});
	}

	#[test]
	fn sent_message_doesnt_increase_factor_if_xcm_channel_is_uncongested() {
		run_test(|| {
			let old_bridge = XcmBridgeHubRouter::bridge();
			assert_eq!(
				send_xcm::<XcmBridgeHubRouter>(
					MultiLocation::new(
						2,
						X2(GlobalConsensus(BridgedNetworkId::get()), Parachain(1000))
					),
					vec![ClearOrigin].into(),
				)
				.map(drop),
				Ok(()),
			);

			assert!(TestToBridgeHubSender::is_message_sent());
			assert_eq!(old_bridge, XcmBridgeHubRouter::bridge());
		});
	}

	#[test]
	fn sent_message_increases_factor_if_xcm_channel_is_congested() {
		run_test(|| {
			TestWithBridgeHubChannel::make_congested();

			let old_bridge = XcmBridgeHubRouter::bridge();
			assert_eq!(
				send_xcm::<XcmBridgeHubRouter>(
					MultiLocation::new(
						2,
						X2(GlobalConsensus(BridgedNetworkId::get()), Parachain(1000))
					),
					vec![ClearOrigin].into(),
				)
				.map(drop),
				Ok(()),
			);

			assert!(TestToBridgeHubSender::is_message_sent());
			assert!(
				old_bridge.delivery_fee_factor < XcmBridgeHubRouter::bridge().delivery_fee_factor
			);
		});
	}

	#[test]
	fn sent_message_increases_factor_if_bridge_has_reported_congestion() {
		run_test(|| {
			Bridge::<TestRuntime, ()>::put(congested_bridge(FixedU128::one()));

			let old_bridge = XcmBridgeHubRouter::bridge();
			assert_eq!(
				send_xcm::<XcmBridgeHubRouter>(
					MultiLocation::new(
						2,
						X2(GlobalConsensus(BridgedNetworkId::get()), Parachain(1000))
					),
					vec![ClearOrigin].into(),
				)
				.map(drop),
				Ok(()),
			);

			assert!(TestToBridgeHubSender::is_message_sent());
			assert!(
				old_bridge.delivery_fee_factor < XcmBridgeHubRouter::bridge().delivery_fee_factor
			);
		});
	}
}
