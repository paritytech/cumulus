// Copyright 2020 Parity Technologies (UK) Ltd.
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

//! Implementation of `ProcessMessage` for an `ExecuteXcm` implementation.

use codec::{Decode, FullCodec, MaxEncodedLen};
use cumulus_primitives_core::{AggregateMessageOrigin, ParaId};
use frame_support::{
	ensure,
	traits::{ProcessMessage, ProcessMessageError},
};
use scale_info::TypeInfo;
use sp_io::hashing::blake2_256;
use sp_std::{fmt::Debug, marker::PhantomData};
use sp_weights::{Weight, WeightMeter};
use xcm::prelude::*;

pub mod queue_paused_query {
	use super::*;
	use frame_support::traits::QueuePausedQuery;

	/// Narrow the scope of the `Inner` query from `AggregateMessageOrigin` to `ParaId`.
	///
	/// All non-paraIds will be treated as unpaused.
	pub struct NarrowToSiblings<Inner>(PhantomData<Inner>);

	impl<Inner: QueuePausedQuery<ParaId>> QueuePausedQuery<AggregateMessageOrigin>
		for NarrowToSiblings<Inner>
	{
		fn is_paused(origin: &AggregateMessageOrigin) -> bool {
			match origin {
				AggregateMessageOrigin::Sibling(id) => Inner::is_paused(id),
				_ => false,
			}
		}
	}
}

/// Convert a sibling `ParaId` to an `AggregateMessageOrigin`.
pub struct ParaIdToSibling;
impl sp_runtime::traits::Convert<ParaId, AggregateMessageOrigin> for ParaIdToSibling {
	fn convert(para_id: ParaId) -> AggregateMessageOrigin {
		AggregateMessageOrigin::Sibling(para_id)
	}
}

/// A message processor that delegates execution to an [`ExecuteXcm`].
///
/// FAIL-CI Delete this once <https://github.com/paritytech/polkadot/pull/6271/> merges.
pub struct ProcessXcmMessage<MessageOrigin, XcmExecutor, Call>(
	PhantomData<(MessageOrigin, XcmExecutor, Call)>,
);
impl<
		MessageOrigin: Into<MultiLocation> + FullCodec + MaxEncodedLen + Clone + Eq + PartialEq + TypeInfo + Debug,
		XcmExecutor: ExecuteXcm<Call>,
		Call,
	> ProcessMessage for ProcessXcmMessage<MessageOrigin, XcmExecutor, Call>
{
	type Origin = MessageOrigin;

	/// Process the given message, using no more than the remaining `weight` to do so.
	fn process_message(
		message: &[u8],
		origin: Self::Origin,
		meter: &mut WeightMeter,
		id: &mut [u8; 32],
	) -> Result<bool, ProcessMessageError> {
		// XCM specifically needs Blake2-256
		*id = blake2_256(message);
		let versioned_message = VersionedXcm::<Call>::decode(&mut &message[..])
			.map_err(|_| ProcessMessageError::Corrupt)?;
		let message = Xcm::<Call>::try_from(versioned_message)
			.map_err(|_| ProcessMessageError::Unsupported)?;
		let pre = XcmExecutor::prepare(message).map_err(|_| ProcessMessageError::Unsupported)?;
		let required = pre.weight_of();
		ensure!(meter.can_accrue(required), ProcessMessageError::Overweight(required));

		let (consumed, result) = match XcmExecutor::execute(origin.into(), pre, id, Weight::zero())
		{
			Outcome::Complete(w) => (w, Ok(true)),
			Outcome::Incomplete(w, _) => (w, Ok(false)),
			// In the error-case we assume the worst case and consume all possible weight.
			Outcome::Error(_) => (required, Err(ProcessMessageError::Unsupported)),
		};
		meter.defensive_saturating_accrue(consumed);
		result
	}
}
