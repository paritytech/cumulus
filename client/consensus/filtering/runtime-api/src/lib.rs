//TODO license
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::vec::Vec;
use parity_scale_codec::Codec;

//TODO this should be generic over an AuthorId type once we start using
// application crypto. For now it is a vec<u8> to be decoded in the runtime.
// This helps keep types concrete while I'm trying to fit the pieces together.
sp_api::decl_runtime_apis! {
    pub trait AuthorFilterAPI<AuthorId: Codec> {
        fn can_author(author: AuthorId, relay_parent: u32) -> bool;
    }
}

/// Idea shelved.
/// it is possible to make the runtime API give more details about why an author is ineligible.
/// Specifically it could distinguish between failing the prechecks and the full checks. But in the
/// who cares (except maybe for debugging). The author can't author, and there's no reason to call
/// both checks.
/// One possible reason is if the full check is considerably more expensive. Anyway, it's shelved
/// for now.
#[allow(dead_code)]
enum AuthorCheckResult {
    /// Author does not even pass the preliminaty checks.
    FailsPreliminaryChecks,
    /// Author passes preliminary checks, but not full checks.
    FailsFullChekcs,
    /// Author is eligible at this slot.
    Eligible,
}
