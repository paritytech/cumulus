use cumulus_runtime::ValidationFunctionParams;
use polkadot_collator::PolkadotClient;
use polkadot_parachain::primitives::Id as ParaID;
use polkadot_primitives::{Block, Hash, parachain::ParachainHost};
use polkadot_validation::{pipeline::validation_params};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::Error;

pub trait ValidationFunctionParamsExtractor {
	fn get_vfp(&self, relay_parent: Hash, para_id: ParaID) -> ValidationFunctionParams;
}

impl<B, E, R> ValidationFunctionParamsExtractor for PolkadotClient<B, E, R>
where
	Self: ProvideRuntimeApi<Block>,
	<Self as ProvideRuntimeApi<Block>>::Api: ParachainHost<Block, Error = Error>,
{
	fn get_vfp(&self, relay_parent: Hash, para_id: ParaID) -> ValidationFunctionParams {
		let (lvd, gvs, _) = validation_params(self, relay_parent, para_id).expect("can get validation function params");

		ValidationFunctionParams {
			max_code_size: gvs.max_code_size,
			relay_chain_height: gvs.block_number,
			code_upgrade_allowed: lvd.code_upgrade_allowed,
		}
	}
}
