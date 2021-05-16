/// This file contains the floating-rate-pool related errors for Konomi chain
/// This it to make the errors importable by other modules
///
use frame_support::dispatch::DispatchError;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

/// An error type that indicates that the floating-rate-pool is not enabled.
#[derive(Encode, Decode, RuntimeDebug)]
pub struct PoolNotEnabled {}

impl From<PoolNotEnabled> for DispatchError {
    fn from(_: PoolNotEnabled) -> DispatchError {
        DispatchError::Module {
            index: 15,
            error: 1,
            message: "Pool Is Not Enabled".into(),
        }
    }
}

/// An error type that indicates that the floating-rate-pool is not enabled.
#[derive(Encode, Decode, RuntimeDebug)]
pub struct PoolPriceNotReady {}

impl From<PoolPriceNotReady> for DispatchError {
    fn from(_: PoolPriceNotReady) -> DispatchError {
        DispatchError::Module {
            index: 15,
            error: 4,
            message: "Pool Price Not Ready".into(),
        }
    }
}


/// An error type that indicates that the floating-rate-pool is not enabled.
#[derive(Encode, Decode, RuntimeDebug)]
pub struct PoolAlreadyExists {}

impl From<PoolAlreadyExists> for DispatchError {
    fn from(_: PoolAlreadyExists) -> DispatchError {
        DispatchError::Module {
            index: 15,
            error: 2,
            message: "Pool Already Exists".into(),
        }
    }
}