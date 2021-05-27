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

/// An error type that indicates that the some or one of the parameters are/is invalid.
#[derive(Encode, Decode, RuntimeDebug)]
pub struct Overflown {}

impl From<Overflown> for DispatchError {
    fn from(_: Overflown) -> DispatchError {
        DispatchError::Other {
            0: "Overflow",
        }
    }
}

/// An error type that indicates that the some or one of the parameters are/is invalid.
#[derive(Encode, Decode, RuntimeDebug)]
pub struct Underflow {}

impl From<Underflow> for DispatchError {
    fn from(_: Underflow) -> DispatchError {
        DispatchError::Other {
            0: "Underflow",
        }
    }
}


/// An error type that indicates that the some or one of the parameters are/is invalid.
#[derive(Encode, Decode, RuntimeDebug)]
pub struct FlowError {}

impl From<FlowError> for DispatchError {
    fn from(_: FlowError) -> DispatchError {
        DispatchError::Other {
            0: "Underflow/Overflow Error",
        }
    }
}