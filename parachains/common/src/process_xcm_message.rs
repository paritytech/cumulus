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
let origin = origin.into();
log::debug!(target: "bridge-xcm", "XcmExecutor::execute: {:?} {:?}", origin, message);
/*
2023-07-26 10:54:24.030 DEBUG tokio-runtime-worker bridge-xcm: [Parachain] XcmExecutor::execute:
origin = MultiLocation { parents: 0, interior: X1(Parachain(1000)) }
Xcm([
	WithdrawAsset(MultiAssets([MultiAsset { id: Concrete(MultiLocation { parents: 1, interior: Here }), fun: Fungible(1103000000) }])),
	BuyExecution { fees: MultiAsset { id: Concrete(MultiLocation { parents: 1, interior: Here }), fun: Fungible(1103000000) }, weight_limit: Unlimited },
	ExportMessage { network: Polkadot, destination: X1(Parachain(1000)), xcm: Xcm([
		ReserveAssetDeposited(MultiAssets([MultiAsset { id: Concrete(MultiLocation { parents: 2, interior: X1(GlobalConsensus(Kusama)) }), fun: Fungible(1000000000000) }])),
		ClearOrigin,
		BuyExecution { fees: MultiAsset { id: Concrete(MultiLocation { parents: 2, interior: X1(GlobalConsensus(Kusama)) }), fun: Fungible(1000000000000) }, weight_limit: Unlimited },
		DepositAsset { assets: Wild(AllCounted(1)), beneficiary: MultiLocation { parents: 0, interior: X1(AccountId32 { network: None, id: [212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125] }) } },
		SetTopic([100, 180, 26, 216, 71, 30, 4, 149, 48, 116, 216, 88, 16, 53, 156, 79, 158, 197, 121, 147, 208, 1, 77, 253, 72, 253, 105, 124, 170, 65, 19, 144])]) },
		RefundSurplus, DepositAsset { assets: Wild(All), beneficiary: MultiLocation { parents: 1, interior: X1(Parachain(1000)) } }, SetTopic([223, 61, 129, 65, 10, 112, 105, 126, 149, 92, 224, 145, 127, 75, 67, 180, 169, 217, 120, 187, 22, 148, 55, 241, 126, 197, 12, 84, 19, 199, 1, 138])])    

*/
		let pre = XcmExecutor::prepare(message).map_err(|_| ProcessMessageError::Unsupported)?;
		let required = pre.weight_of();
		ensure!(meter.can_accrue(required), ProcessMessageError::Overweight(required));

		let (consumed, result) = match XcmExecutor::execute(origin, pre, id, Weight::zero())
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
