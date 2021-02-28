//TODO license
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::vec::Vec;
use parity_scale_codec::Codec;

//TODO this should be generic over an AuthorId type once we start using
// application crypto. For now it is a vec<u8> to be decoded in the runtime.
// This helps keep types concrete while I'm trying to fit the pieces together.
sp_api::decl_runtime_apis! {
    pub trait AuthorFilterAPI {
        fn can_author(author: Vec<u8>, relay_parent: u32) -> bool;
    }
}

//TODO maybe return a Result<bool, AuthorCheckError>
enum AuthorCheckError {
    /// Not in the active author set. (eg. not staked)
    AuthorNotActive,
    /// In the active set, but not eligible at this slot
    NotEligibletThisSlot,
    /// Thebytes passed in didn't decode properly.
    /// This won't be necessary after we're using application crypto.
    DecodingError,
}
