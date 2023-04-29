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

/// Transform a `ProcessMessage` implementation to assume that the origin is a sibling chain.
///
/// TODO move this to Substrate into a `TransformProcessMessageOrigin` adapter.
pub struct ProcessFromSibling<P>(core::marker::PhantomData<P>);
impl<P: ProcessMessage<Origin = AggregateMessageOrigin>> ProcessMessage for ProcessFromSibling<P> {
	type Origin = ParaId;

	fn process_message(
		message: &[u8],
		origin: Self::Origin,
		meter: &mut WeightMeter,
	) -> Result<bool, ProcessMessageError> {
		P::process_message(message, AggregateMessageOrigin::Sibling(origin), meter)
	}
}

/// Splits queued messages up between DMP and XCMoHRMP dispatch.
pub struct SplitMessages<XcmProcessor, XcmpQueue>(PhantomData<(XcmProcessor, XcmpQueue)>);
impl<XcmProcessor, XcmpQueue> ProcessMessage for SplitMessages<XcmProcessor, XcmpQueue>
where
	XcmProcessor: ProcessMessage<Origin = AggregateMessageOrigin>,
	XcmpQueue: ProcessMessage<Origin = ParaId>,
{
	type Origin = AggregateMessageOrigin;

	fn process_message(
		message: &[u8],
		origin: Self::Origin,
		meter: &mut WeightMeter,
	) -> Result<bool, ProcessMessageError> {
		use AggregateMessageOrigin::*;
		match origin {
			// DMP and local messages can be directly forwarded to the XCM executor since there is no flow control.
			o @ Parent | o @ Loopback => XcmProcessor::process_message(message, o, meter),
			// XCMoHRMP need to be tunneled back through the XCMP queue pallet to respect the suspension logic.
			Sibling(para) => XcmpQueue::process_message(message, para, meter),
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
	) -> Result<bool, ProcessMessageError> {
		let hash = blake2_256(message);
		let versioned_message = VersionedXcm::<Call>::decode(&mut &message[..])
			.map_err(|_| ProcessMessageError::Corrupt)?;
		let message = Xcm::<Call>::try_from(versioned_message)
			.map_err(|_| ProcessMessageError::Unsupported)?;
		let pre = XcmExecutor::prepare(message).map_err(|_| ProcessMessageError::Unsupported)?;
		let required = pre.weight_of();
		ensure!(meter.can_accrue(required), ProcessMessageError::Overweight(required));

		let (consumed, result) =
			match XcmExecutor::execute(origin.into(), pre, hash, Weight::zero()) {
				Outcome::Complete(w) => (w, Ok(true)),
				Outcome::Incomplete(w, _) => (w, Ok(false)),
				// In the error-case we assume the worst case and consume all possible weight.
				Outcome::Error(_) => (required, Err(ProcessMessageError::Unsupported)),
			};
		meter.defensive_saturating_accrue(consumed);
		result
	}
}
