//TODO license

//! Nimbus Consensus Primitives
//!
//! TODO rename the crate. It was originally just the runtime api

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::vec::Vec;
use parity_scale_codec::Codec;
use sp_application_crypto::KeyTypeId;

//TODO Maybe move our key type into sp_core if this gets well adopted (to avoid collision)
pub const NIMBUS_KEY_ID: KeyTypeId = KeyTypeId(*b"nmbs");

mod app {
	use sp_application_crypto::{
		app_crypto,
		sr25519,
	};
	app_crypto!(sr25519, crate::NIMBUS_KEY_ID);
}

sp_application_crypto::with_pair! {
	/// A nimbus author keypair.
	pub type NimbusPair = app::Pair;
}

/// A nimbus author identifier.
pub type NimbusId = app::Public;

/// A nimbus author signature.
pub type NimbusSignature = app::Signature;

//TODO this actually does need to be generic over the author id type if you want to use it
// so that the author id is just an account id.
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
