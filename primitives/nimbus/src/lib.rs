//TODO license

//! Nimbus Consensus Primitives
//!
//! Primitive types and traits for working with the Nimbus consensus framework.
//! This code can be built to no_std for use in the runtime

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::vec::Vec;
use parity_scale_codec::Codec;
use sp_application_crypto::KeyTypeId;

/// The given account ID is the author of the current block.
pub trait EventHandler<Author> {
	//TODO should we be tking ownership here?
	fn note_author(author: Author);
}

impl<T> EventHandler<T> for () {
	fn note_author(_author: T) {}
}

/// A mechanism for determining the current slot.
/// For now we use u32 as the slot type everywhere. Let's see how long we can get away with that.
pub trait SlotBeacon {
    fn slot() -> u32;
}

/// Trait to determine whether this author is eligible to author in this slot.
/// This is the primary trait your nimbus filter needs to implement.
///
/// This is the proposition-logic variant.
/// That is to say the caller specifies an author an author and the implementation
/// replies whether that author is eligible. This is useful in many cases and is
/// particularly useful when the active set is unbounded.
/// There may be another variant where the caller only supplies a slot and the
/// implementation replies with a complete set of eligible authors.
pub trait CanAuthor<AuthorId> {
	fn can_author(author: &AuthorId, slot: &u32) -> bool;
}
/// Default implementation where anyone can author.
// TODO Promote this is "implementing relay chain consensus in the nimbus framework."
impl<T> CanAuthor<T> for () {
	fn can_author(_: &T, _: &u32) -> bool {
		true
	}
}

/// The KeyTypeId used in the Nimbus consensus framework regardles of wat filters are in place.
/// If this gets well adopted, we could move this definition to sp_core to avoid conflicts.
pub const NIMBUS_KEY_ID: KeyTypeId = KeyTypeId(*b"nmbs");

// The strongly-typed crypto wrappers to be used by Nimbus in the keystore
mod nimbus_crypto {
	use sp_application_crypto::{
		app_crypto,
		sr25519,
	};
	app_crypto!(sr25519, crate::NIMBUS_KEY_ID);
}

//TODO, do I need this? I didn't use it in the keystore-learning example
// sp_application_crypto::with_pair! {
// 	/// A nimbus author keypair.
// 	pub type NimbusPair = nimbus_crypto::Pair;
// }

/// A nimbus author identifier.
pub type NimbusId = nimbus_crypto::Public;

/// A nimbus author signature.
pub type NimbusSignature = nimbus_crypto::Signature;


sp_api::decl_runtime_apis! {
    /// the runtime api used to predict whether an author will be eligible in the given slot
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
