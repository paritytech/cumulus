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

use bp_xcm_bridge_hub_router::LocalXcmChannel;
use codec::Encode;
use frame_support::traits::Get;
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
pub const LOG_TARGET: &str = "runtime::bridge-hub-router";

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

		/// Actual message sender (XCMP/DMP) to the sibling bridge hub location.
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
			// if XCM queue is still congested, we don't change anything
			if T::WithBridgeHubChannel::is_congested() {
				return T::WeightInfo::on_initialize_when_congested()
			}

			DeliveryFeeFactor::<T, I>::mutate(|f| {
				let previous_factor = *f;
				*f = InitialFactor::get().max(*f / EXPONENTIAL_FEE_BASE);
				if previous_factor != *f {
					log::info!(
						target: LOG_TARGET,
						"Bridge queue is uncongested. Decreased fee factor from {} to {}",
						previous_factor,
						f,
					);

					T::WeightInfo::on_initialize_when_non_congested()
				} else {
					// we have not actually updated the `DeliveryFeeFactor`, so we may deduct
					// single db write from maximal weight
					T::WeightInfo::on_initialize_when_non_congested()
						.saturating_sub(T::DbWeight::get().writes(1))
				}
			})
		}
	}

	/// Initialization value for the delivery fee factor.
	#[pallet::type_value]
	pub fn InitialFactor() -> FixedU128 {
		FixedU128::one()
	}

	/// The number to multiply the base delivery fee by.
	#[pallet::storage]
	#[pallet::getter(fn delivery_fee_factor)]
	pub type DeliveryFeeFactor<T: Config<I>, I: 'static = ()> =
		StorageValue<_, FixedU128, ValueQuery, InitialFactor>;

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Called when new message is sent (queued to local outbound XCM queue) over the bridge.
		pub(crate) fn on_message_sent_to_bridge(message_size: u32) {
			// if outbound queue is not congested, do nothing
			if !T::WithBridgeHubChannel::is_congested() {
				return
			}

			// ok - we need to increase the fee factor, let's do that
			let message_size_factor = FixedU128::from_u32(message_size.saturating_div(1024))
				.saturating_mul(MESSAGE_SIZE_FEE_BASE);
			let total_factor = EXPONENTIAL_FEE_BASE.saturating_add(message_size_factor);
			DeliveryFeeFactor::<T, I>::mutate(|f| {
				let previous_factor = *f;
				*f = f.saturating_mul(total_factor);
				log::info!(
					target: LOG_TARGET,
					"Bridge queue is congested. Increased fee factor from {} to {}",
					previous_factor,
					f,
				);
				*f
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
		_remote_location: &InteriorMultiLocation,
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
		let fee_factor = Self::delivery_fee_factor();
		let fee = fee_factor.saturating_mul_int(fee_sum);

		log::info!(
			target: LOG_TARGET,
			"Going to send message ({} bytes) over bridge. Computed bridge fee {} using fee factor {}",
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
		let xcm_hash = T::ToBridgeHubSender::deliver(ticket)?;

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

	#[test]
	fn initial_fee_factor_is_one() {
		run_test(|| {
			assert_eq!(DeliveryFeeFactor::<TestRuntime, ()>::get(), FixedU128::one());
		})
	}

	#[test]
	fn fee_factor_is_not_decreased_from_on_initialize_when_queue_is_congested() {
		run_test(|| {
			DeliveryFeeFactor::<TestRuntime, ()>::put(FixedU128::from_rational(125, 100));
			TestWithBridgeHubChannel::make_congested();

			// it should not decrease, because queue is congested
			let old_delivery_fee_factor = XcmBridgeHubRouter::delivery_fee_factor();
			XcmBridgeHubRouter::on_initialize(One::one());
			assert_eq!(XcmBridgeHubRouter::delivery_fee_factor(), old_delivery_fee_factor);
		})
	}

	#[test]
	fn fee_factor_is_decreased_from_on_initialize_when_queue_is_uncongested() {
		run_test(|| {
			DeliveryFeeFactor::<TestRuntime, ()>::put(FixedU128::from_rational(125, 100));

			// it shold eventually decreased to one
			while XcmBridgeHubRouter::delivery_fee_factor() > FixedU128::one() {
				XcmBridgeHubRouter::on_initialize(One::one());
			}

			// verify that it doesn't decreases anymore
			XcmBridgeHubRouter::on_initialize(One::one());
			assert_eq!(XcmBridgeHubRouter::delivery_fee_factor(), FixedU128::one());
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
			DeliveryFeeFactor::<TestRuntime, ()>::put(factor);
			let expected_fee =
				(FixedU128::saturating_from_integer(BASE_FEE + BYTE_FEE * (msg_size as u128)) *
					factor)
					.into_inner() / FixedU128::DIV +
					HRMP_FEE;
			assert_eq!(
				XcmBridgeHubRouter::validate(&mut Some(dest), &mut Some(xcm.clone()))
					.unwrap()
					.1
					.get(0),
				Some(&(BridgeFeeAsset::get(), expected_fee).into()),
			);
		});
	}

	#[test]
	fn sent_message_doesnt_increase_factor_if_queue_is_uncongested() {
		run_test(|| {
			let old_delivery_fee_factor = XcmBridgeHubRouter::delivery_fee_factor();
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
			assert_eq!(old_delivery_fee_factor, XcmBridgeHubRouter::delivery_fee_factor());
		});
	}

	#[test]
	fn sent_message_increases_factor_if_queue_is_congested() {
		run_test(|| {
			TestWithBridgeHubChannel::make_congested();

			let old_delivery_fee_factor = XcmBridgeHubRouter::delivery_fee_factor();
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
			assert!(old_delivery_fee_factor < XcmBridgeHubRouter::delivery_fee_factor());
		});
	}
}
