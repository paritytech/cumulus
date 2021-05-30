/// This file contains the general purpose errors for Konomi chain

use frame_support::dispatch::DispatchError;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

/// An error type that indicates that the some or one of the parameters are/is invalid.
#[derive(Encode, Decode, RuntimeDebug)]
pub struct InvalidParameters {}

impl From<InvalidParameters> for DispatchError {
    fn from(_: InvalidParameters) -> DispatchError {
        DispatchError::Other {
            0: "Invalid Parameters Passed",
        }
    }
}

#[derive(Encode, Decode, RuntimeDebug)]
pub enum CustomError {
    /// Overflow or underflow errors
    FlownError,
    /// Price not ready error
    PriceNotReady,
    /// Pool does not exist
    PoolNotExist,
    /// Some or one of the parameters are/is invalid.
    InvalidParameters,
    /// Inconsistent state error
    InconsistentState,
}

impl From<CustomError> for DispatchError {
    fn from(c: CustomError) -> DispatchError {
        match c {
            CustomError::FlownError => DispatchError::Other { 0: "Underflow/Overflow Error"},
            CustomError::PriceNotReady => DispatchError::Other { 0: "Price is not ready" },
            CustomError::InvalidParameters => DispatchError::Other { 0: "Invalid Parameters Passed" },
            CustomError::InconsistentState => DispatchError::Other { 0: "Inconsistent state" },
            CustomError::PoolNotExist => DispatchError::Other { 0: "Pool does not exist" },
        }
    }
}