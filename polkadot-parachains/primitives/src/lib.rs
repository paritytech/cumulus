//! Primitives used by the the whole project.

#![cfg_attr(not(feature = "std"), no_std)]

mod errors;
mod konomi;
pub use konomi::*;
pub use errors::*;
