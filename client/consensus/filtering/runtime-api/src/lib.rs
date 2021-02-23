//TODO license
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::vec::Vec;
use parity_scale_codec::Codec;

//TODO don't pass a vec<u8>, make theAPI generic over a type like the Aura API is.
// That will need to wait until we have a dedicated AuthorityId type.
sp_api::decl_runtime_apis! {
    pub trait AuthorFilterAPI {
        fn can_author(author: Vec<u8>, relay_parent: u32) -> bool;
    }
}
