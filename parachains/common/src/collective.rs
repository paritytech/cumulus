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

use frame_support::{
	dispatch::{DispatchError, DispatchResultWithPostInfo},
	weights::Weight,
};
use pallet_collective::{Config, Pallet, ProposalIndex};
use sp_std::boxed::Box;

pub trait ProposalProvider<AccountId, Hash, Proposal>:
	ProposalPropose<AccountId, Proposal>
	+ ProposalVote<AccountId, Hash>
	+ ProposalDisapprove<Hash>
	+ ProposalClose<Hash>
	+ ProposalOfHash<Hash, Proposal>
{
}

impl<T: Config<I>, I: 'static> ProposalProvider<T::AccountId, T::Hash, T::Proposal>
	for Pallet<T, I>
{
}

pub trait ProposalPropose<AccountId, Proposal> {
	fn do_propose_proposed(
		who: AccountId,
		threshold: u32,
		proposal: Box<Proposal>,
		length_bound: u32,
	) -> Result<(u32, u32), DispatchError>;
}

impl<T: Config<I>, I: 'static> ProposalPropose<T::AccountId, T::Proposal> for Pallet<T, I> {
	fn do_propose_proposed(
		who: T::AccountId,
		threshold: u32,
		proposal: Box<T::Proposal>,
		length_bound: u32,
	) -> Result<(u32, u32), DispatchError> {
		Self::do_propose_proposed(who, threshold, proposal, length_bound)
	}
}

pub trait ProposalVote<AccountId, Hash> {
	fn do_vote(
		who: AccountId,
		proposal: Hash,
		index: ProposalIndex,
		approve: bool,
	) -> Result<bool, DispatchError>;
}

impl<T: Config<I>, I: 'static> ProposalVote<T::AccountId, T::Hash> for Pallet<T, I> {
	fn do_vote(
		who: T::AccountId,
		proposal: T::Hash,
		index: ProposalIndex,
		approve: bool,
	) -> Result<bool, DispatchError> {
		Self::do_vote(who, proposal, index, approve)
	}
}

pub trait ProposalDisapprove<Hash> {
	fn do_disapprove_proposal(proposal_hash: Hash) -> u32;
}

impl<T: Config<I>, I: 'static> ProposalDisapprove<T::Hash> for Pallet<T, I> {
	fn do_disapprove_proposal(proposal_hash: T::Hash) -> u32 {
		Self::do_disapprove_proposal(proposal_hash)
	}
}

pub trait ProposalClose<Hash> {
	fn do_close(
		proposal_hash: Hash,
		index: ProposalIndex,
		proposal_weight_bound: Weight,
		length_bound: u32,
	) -> DispatchResultWithPostInfo;
}

impl<T: Config<I>, I: 'static> ProposalClose<T::Hash> for Pallet<T, I> {
	fn do_close(
		proposal_hash: T::Hash,
		index: ProposalIndex,
		proposal_weight_bound: Weight,
		length_bound: u32,
	) -> DispatchResultWithPostInfo {
		Self::do_close(proposal_hash, index, proposal_weight_bound, length_bound)
	}
}

pub trait ProposalOfHash<Hash, Proposal> {
	fn proposal_of(proposal_hash: Hash) -> Option<Proposal>;
}

impl<T: Config<I>, I: 'static> ProposalOfHash<T::Hash, T::Proposal> for Pallet<T, I> {
	fn proposal_of(proposal_hash: T::Hash) -> Option<T::Proposal> {
		Self::proposal_of(proposal_hash)
	}
}
