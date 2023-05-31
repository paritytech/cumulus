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

use cumulus_primitives_core::{AggregateMessageOrigin, ParaId};
use frame_support::traits::{ProcessMessage, ProcessMessageError};

use sp_std::marker::PhantomData;
use sp_weights::WeightMeter;

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
		id: &mut [u8; 32],
	) -> Result<bool, ProcessMessageError> {
		P::process_message(message, AggregateMessageOrigin::Sibling(origin), meter, id)
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
		id: &mut [u8; 32],
	) -> Result<bool, ProcessMessageError> {
		use AggregateMessageOrigin::*;
		match origin {
			// DMP and local messages can be directly forwarded to the XCM executor since there is no flow control.
			o @ Parent | o @ Loopback => XcmProcessor::process_message(message, o, meter, id),
			// XCMoHRMP need to be tunneled back through the XCMP queue pallet to respect the suspension logic.
			Sibling(para) => XcmpQueue::process_message(message, para, meter, id),
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
