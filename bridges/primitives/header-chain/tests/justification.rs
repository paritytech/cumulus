// Copyright 2020-2021 Parity Technologies (UK) Ltd.
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

//! Tests for Grandpa Justification code.

use bp_header_chain::justification::{
	optimize_justification, required_justification_precommits, verify_justification, Error,
};
use bp_test_utils::*;

type TestHeader = sp_runtime::testing::Header;

#[test]
fn valid_justification_accepted() {
	let authorities = vec![(ALICE, 1), (BOB, 1), (CHARLIE, 1)];
	let params = JustificationGeneratorParams {
		header: test_header(1),
		round: TEST_GRANDPA_ROUND,
		set_id: TEST_GRANDPA_SET_ID,
		authorities: authorities.clone(),
		ancestors: 7,
		forks: 3,
	};

	let justification = make_justification_for_header::<TestHeader>(params.clone());
	assert_eq!(
		verify_justification::<TestHeader>(
			header_id::<TestHeader>(1),
			TEST_GRANDPA_SET_ID,
			&voter_set(),
			&justification,
		),
		Ok(()),
	);

	assert_eq!(justification.commit.precommits.len(), authorities.len());
	assert_eq!(justification.votes_ancestries.len(), params.ancestors as usize);
}

#[test]
fn valid_justification_accepted_with_single_fork() {
	let params = JustificationGeneratorParams {
		header: test_header(1),
		round: TEST_GRANDPA_ROUND,
		set_id: TEST_GRANDPA_SET_ID,
		authorities: vec![(ALICE, 1), (BOB, 1), (CHARLIE, 1)],
		ancestors: 5,
		forks: 1,
	};

	assert_eq!(
		verify_justification::<TestHeader>(
			header_id::<TestHeader>(1),
			TEST_GRANDPA_SET_ID,
			&voter_set(),
			&make_justification_for_header::<TestHeader>(params)
		),
		Ok(()),
	);
}

#[test]
fn valid_justification_accepted_with_arbitrary_number_of_authorities() {
	use finality_grandpa::voter_set::VoterSet;
	use sp_finality_grandpa::AuthorityId;

	let n = 15;
	let required_signatures = required_justification_precommits(n as _);
	let authorities = accounts(n).iter().map(|k| (*k, 1)).collect::<Vec<_>>();

	let params = JustificationGeneratorParams {
		header: test_header(1),
		round: TEST_GRANDPA_ROUND,
		set_id: TEST_GRANDPA_SET_ID,
		authorities: authorities.clone().into_iter().take(required_signatures as _).collect(),
		ancestors: n.into(),
		forks: required_signatures,
	};

	let authorities = authorities
		.iter()
		.map(|(id, w)| (AuthorityId::from(*id), *w))
		.collect::<Vec<(AuthorityId, _)>>();
	let voter_set = VoterSet::new(authorities).unwrap();

	assert_eq!(
		verify_justification::<TestHeader>(
			header_id::<TestHeader>(1),
			TEST_GRANDPA_SET_ID,
			&voter_set,
			&make_justification_for_header::<TestHeader>(params)
		),
		Ok(()),
	);
}

#[test]
fn justification_with_invalid_target_rejected() {
	assert_eq!(
		verify_justification::<TestHeader>(
			header_id::<TestHeader>(2),
			TEST_GRANDPA_SET_ID,
			&voter_set(),
			&make_default_justification::<TestHeader>(&test_header(1)),
		),
		Err(Error::InvalidJustificationTarget),
	);
}

#[test]
fn justification_with_invalid_commit_rejected() {
	let mut justification = make_default_justification::<TestHeader>(&test_header(1));
	justification.commit.precommits.clear();

	assert_eq!(
		verify_justification::<TestHeader>(
			header_id::<TestHeader>(1),
			TEST_GRANDPA_SET_ID,
			&voter_set(),
			&justification,
		),
		Err(Error::ExtraHeadersInVotesAncestries),
	);
}

#[test]
fn justification_with_invalid_authority_signature_rejected() {
	let mut justification = make_default_justification::<TestHeader>(&test_header(1));
	justification.commit.precommits[0].signature =
		sp_core::crypto::UncheckedFrom::unchecked_from([1u8; 64]);

	assert_eq!(
		verify_justification::<TestHeader>(
			header_id::<TestHeader>(1),
			TEST_GRANDPA_SET_ID,
			&voter_set(),
			&justification,
		),
		Err(Error::InvalidAuthoritySignature),
	);
}

#[test]
fn justification_with_invalid_precommit_ancestry() {
	let mut justification = make_default_justification::<TestHeader>(&test_header(1));
	justification.votes_ancestries.push(test_header(10));

	assert_eq!(
		verify_justification::<TestHeader>(
			header_id::<TestHeader>(1),
			TEST_GRANDPA_SET_ID,
			&voter_set(),
			&justification,
		),
		Err(Error::ExtraHeadersInVotesAncestries),
	);
}

#[test]
fn justification_is_invalid_if_we_dont_meet_threshold() {
	// Need at least three authorities to sign off or else the voter set threshold can't be reached
	let authorities = vec![(ALICE, 1), (BOB, 1)];

	let params = JustificationGeneratorParams {
		header: test_header(1),
		round: TEST_GRANDPA_ROUND,
		set_id: TEST_GRANDPA_SET_ID,
		authorities: authorities.clone(),
		ancestors: 2 * authorities.len() as u32,
		forks: 2,
	};

	assert_eq!(
		verify_justification::<TestHeader>(
			header_id::<TestHeader>(1),
			TEST_GRANDPA_SET_ID,
			&voter_set(),
			&make_justification_for_header::<TestHeader>(params)
		),
		Err(Error::TooLowCumulativeWeight),
	);
}

#[test]
fn optimizer_does_noting_with_minimal_justification() {
	let justification = make_default_justification::<TestHeader>(&test_header(1));

	let num_precommits_before = justification.commit.precommits.len();
	let justification = optimize_justification::<TestHeader>(
		header_id::<TestHeader>(1),
		TEST_GRANDPA_SET_ID,
		&voter_set(),
		justification,
	)
	.unwrap();
	let num_precommits_after = justification.commit.precommits.len();

	assert_eq!(num_precommits_before, num_precommits_after);
}

#[test]
fn unknown_authority_votes_are_removed_by_optimizer() {
	let mut justification = make_default_justification::<TestHeader>(&test_header(1));
	justification.commit.precommits.push(signed_precommit::<TestHeader>(
		&bp_test_utils::Account(42),
		header_id::<TestHeader>(1),
		justification.round,
		TEST_GRANDPA_SET_ID,
	));

	let num_precommits_before = justification.commit.precommits.len();
	let justification = optimize_justification::<TestHeader>(
		header_id::<TestHeader>(1),
		TEST_GRANDPA_SET_ID,
		&voter_set(),
		justification,
	)
	.unwrap();
	let num_precommits_after = justification.commit.precommits.len();

	assert_eq!(num_precommits_before - 1, num_precommits_after);
}

#[test]
fn duplicate_authority_votes_are_removed_by_optimizer() {
	let mut justification = make_default_justification::<TestHeader>(&test_header(1));
	justification
		.commit
		.precommits
		.push(justification.commit.precommits.first().cloned().unwrap());

	let num_precommits_before = justification.commit.precommits.len();
	let justification = optimize_justification::<TestHeader>(
		header_id::<TestHeader>(1),
		TEST_GRANDPA_SET_ID,
		&voter_set(),
		justification,
	)
	.unwrap();
	let num_precommits_after = justification.commit.precommits.len();

	assert_eq!(num_precommits_before - 1, num_precommits_after);
}

#[test]
fn redundant_authority_votes_are_removed_by_optimizer() {
	let mut justification = make_default_justification::<TestHeader>(&test_header(1));
	justification.commit.precommits.push(signed_precommit::<TestHeader>(
		&EVE,
		header_id::<TestHeader>(1),
		justification.round,
		TEST_GRANDPA_SET_ID,
	));

	let num_precommits_before = justification.commit.precommits.len();
	let justification = optimize_justification::<TestHeader>(
		header_id::<TestHeader>(1),
		TEST_GRANDPA_SET_ID,
		&voter_set(),
		justification,
	)
	.unwrap();
	let num_precommits_after = justification.commit.precommits.len();

	assert_eq!(num_precommits_before - 1, num_precommits_after);
}
