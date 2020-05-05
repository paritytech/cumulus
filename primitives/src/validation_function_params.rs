//! Validation Function Parameters govern the ability of parachains to upgrade their validation functions.

use codec::{Decode, Encode};
use polkadot_parachain::primitives::{RelayChainBlockNumber, ValidationParams};
use polkadot_primitives::parachain::{GlobalValidationSchedule, LocalValidationData};

/// Validation Function Parameters
///
/// This struct is the subset of [`ValidationParams`](polkadot_parachain::ValidationParams)
/// which is of interest when upgrading parachain validation functions.
#[derive(PartialEq, Eq, Encode, Decode, Clone, Copy, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ValidationFunctionParams {
	/// The maximum code size permitted, in bytes.
	pub max_code_size: u32,
	/// The current relay-chain block number.
	pub relay_chain_height: RelayChainBlockNumber,
	/// Whether a code upgrade is allowed or not, and at which height the upgrade
	/// would be applied after, if so. The parachain logic should apply any upgrade
	/// issued in this block after the first block
	/// with `relay_chain_height` at least this value, if `Some`. if `None`, issue
	/// no upgrade.
	pub code_upgrade_allowed: Option<RelayChainBlockNumber>,
}

impl From<&ValidationParams> for ValidationFunctionParams {
	fn from(vp: &ValidationParams) -> Self {
		ValidationFunctionParams {
			max_code_size: vp.max_code_size,
			relay_chain_height: vp.relay_chain_height,
			code_upgrade_allowed: vp.code_upgrade_allowed,
		}
	}
}

impl From<(GlobalValidationSchedule, LocalValidationData)> for ValidationFunctionParams {
	fn from(t: (GlobalValidationSchedule, LocalValidationData)) -> Self {
		let (global_validation, local_validation) = t;
		ValidationFunctionParams {
			max_code_size: global_validation.max_code_size,
			relay_chain_height: global_validation.block_number,
			code_upgrade_allowed: local_validation.code_upgrade_allowed,
		}
	}
}

/// A trait which is called when the validation function parameters are set
#[impl_trait_for_tuples::impl_for_tuples(30)]
pub trait OnValidationFunctionParams {
	fn on_validation_function_params(vfp: ValidationFunctionParams);
}
