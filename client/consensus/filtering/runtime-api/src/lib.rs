//TODO license
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::vec::Vec;
use parity_scale_codec::Codec;

sp_api::decl_runtime_apis! {
    pub trait AuthorFilterAPI<AccountId: Codec> {
        fn can_author(author: AccountId, relay_parent: u32) -> bool;
    }
}
