// Copyright (C) 2021 Parity Technologies (UK) Ltd.
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

// TODO add docs after general approach is accepted

use crate::collective::ProposalProvider as CollectiveProposalProvider;
use frame_support::{
	dispatch::{DispatchError, DispatchResultWithPostInfo},
	weights::Weight,
};
use pallet_alliance::{Config, ProposalIndex, ProposalProvider};
use sp_std::marker::PhantomData;
use sp_std::boxed::Box;

/// Adapter from {collective::ProposalProvider} to {pallet_alliance::ProposalProvider}.
pub struct CollectiveAdapter<C, T, I = ()>(PhantomData<(C, T, I)>);

impl<C, T, I> ProposalProvider<T::AccountId, T::Hash, T::Proposal>
	for CollectiveAdapter<C, T, I>
where
	C: CollectiveProposalProvider<T::AccountId, T::Hash, T::Proposal>,
	T: Config<I>,
	I: 'static,
{
	fn propose_proposal(
		who: T::AccountId,
		threshold: u32,
		proposal: Box<T::Proposal>,
		length_bound: u32,
	) -> Result<(u32, u32), DispatchError> {
		C::do_propose_proposed(who, threshold, proposal, length_bound)
	}

	fn vote_proposal(
		who: T::AccountId,
		proposal: T::Hash,
		index: ProposalIndex,
		approve: bool,
	) -> Result<bool, DispatchError> {
		C::do_vote(who, proposal, index, approve)
	}

	fn veto_proposal(proposal_hash: T::Hash) -> u32 {
		C::do_disapprove_proposal(proposal_hash)
	}

	fn close_proposal(
		proposal_hash: T::Hash,
		proposal_index: ProposalIndex,
		proposal_weight_bound: Weight,
		length_bound: u32,
	) -> DispatchResultWithPostInfo {
		C::do_close(proposal_hash, proposal_index, proposal_weight_bound, length_bound)
	}

	fn proposal_of(proposal_hash: T::Hash) -> Option<T::Proposal> {
		C::proposal_of(proposal_hash)
	}
}
