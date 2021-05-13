// Copyright 2021 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus. If not, see <http://www.gnu.org/licenses/>.

//! Utility module to validate cumulus-runtime timestamps against the
//! relay chain slot.

use cumulus_primitives_core::{OnValidationData, Slot};
use frame_support::traits::Get;

use crate::{Config, PersistedValidationData, RelaySlot};

// Temporary global that holds the state as populated by the hooks `OnTimestampSet` or
// `OnValidationData`.
#[derive(Default)]
pub(crate) struct TimestampValidationParams {
	relay_chain_slot: Option<Slot>,
	timestamp_slot: Option<Slot>,
}

// Stores the [`TimestampValidationParams`] that are being passed to `validate_block`.
//
// This value will only be set when a parachain validator validates a given `PoV`.
environmental::environmental!(TIMESTAMP_VALIDATION_PARAMS: TimestampValidationParams);

/// Set the [`TimestampValidationParams`] for the local context and execute the given closure in
/// this context.
#[cfg(not(feature = "std"))]
pub(crate) fn run_with_timestamp_validation_params<R>(
	f: impl FnOnce() -> R,
) -> R {
	let mut params = Default::default();
	TIMESTAMP_VALIDATION_PARAMS::using(&mut params, f)
}

/// Utility to validate the timestamps set from a cumulus-enabled runtime against
/// the relay chain slot number. To enable this validation the runtime hooks for
/// `OnTimestampSet` and `OnValidationData` should be set to this struct using the
/// appropriate slot duration of the relay chain.
pub struct ValidateTimestampAgainstRelayChainSlot<
	T: pallet_timestamp::Config,
	RelaySlotDuration: Get<T::Moment>,
>(sp_std::marker::PhantomData<(T, RelaySlotDuration)>);

impl<T, RelaySlotDuration> OnValidationData
	for ValidateTimestampAgainstRelayChainSlot<T, RelaySlotDuration>
where
	T: Config + pallet_timestamp::Config,
	RelaySlotDuration: Get<T::Moment>,
{
	fn on_validation_data(_data: &PersistedValidationData) {
		TIMESTAMP_VALIDATION_PARAMS::with(|p| {
			let relay_chain_slot = match RelaySlot::<T>::get() {
				Some(slot) => slot,
				_ => {
					// this should be unreachable as the relay slot should always be populated after
					// we have processed the validation data.
					return;
				}
			};

			if let Some(timestamp_slot) = p.timestamp_slot {
				assert_eq!(
					timestamp_slot, relay_chain_slot,
					"Timestamp slot must match `CurrentSlot`"
				);
			} else {
				p.relay_chain_slot = Some(relay_chain_slot);
			}
		});
	}
}

impl<T, RelaySlotDuration> frame_support::traits::OnTimestampSet<T::Moment>
	for ValidateTimestampAgainstRelayChainSlot<T, RelaySlotDuration>
where
	T: pallet_timestamp::Config,
	RelaySlotDuration: Get<T::Moment>,
{
	fn on_timestamp_set(moment: T::Moment) {
		use sp_runtime::SaturatedConversion;

		TIMESTAMP_VALIDATION_PARAMS::with(|p| {
			let timestamp_slot = moment / RelaySlotDuration::get();
			let timestamp_slot = Slot::from(timestamp_slot.saturated_into::<u64>());

			if let Some(relay_chain_slot) = p.relay_chain_slot {
				assert_eq!(
					timestamp_slot, relay_chain_slot,
					"Timestamp slot must match `CurrentSlot`"
				);
			} else {
				p.timestamp_slot = Some(timestamp_slot);
			}
		});
	}
}
