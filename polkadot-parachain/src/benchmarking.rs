// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Cumulus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

use std::{marker::PhantomData, sync::Arc, time::Duration};

use codec::Encode;
use sc_client_api::BlockBackend;
use sp_core::{Pair, H256};
use sp_inherents::{InherentData, InherentDataProvider};
use sp_keyring::Sr25519Keyring;
use sp_runtime::{generic, traits::SignedExtension, OpaqueExtrinsic, SaturatedConversion};

use cumulus_primitives_parachain_inherent::MockValidationDataInherentDataProvider;
use parachains_common::BlockNumber;

use crate::service::ParachainClient;

/// Trait for providing the signed extra for a given runtime.
trait SignedExtraProvider<SignedExtra: SignedExtension> {
	fn get_extra(period: u64, best_block: BlockNumber, nonce: u32) -> SignedExtra;
	fn get_additional_signed(genesis_hash: H256, best_hash: H256) -> SignedExtra::AdditionalSigned;
}

/// Generates the signed extra for the given runtime.
pub struct SignedExtraFor<SignedExtra>(PhantomData<SignedExtra>);

impl SignedExtraProvider<asset_hub_kusama_runtime::SignedExtra>
	for SignedExtraFor<asset_hub_kusama_runtime::SignedExtra>
{
	fn get_extra(
		period: u64,
		best_block: BlockNumber,
		nonce: u32,
	) -> asset_hub_kusama_runtime::SignedExtra {
		(
			frame_system::CheckNonZeroSender::<asset_hub_kusama_runtime::Runtime>::new(),
			frame_system::CheckSpecVersion::<asset_hub_kusama_runtime::Runtime>::new(),
			frame_system::CheckTxVersion::<asset_hub_kusama_runtime::Runtime>::new(),
			frame_system::CheckGenesis::<asset_hub_kusama_runtime::Runtime>::new(),
			frame_system::CheckEra::<asset_hub_kusama_runtime::Runtime>::from(generic::Era::mortal(
				period,
				best_block.saturated_into(),
			)),
			frame_system::CheckNonce::<asset_hub_kusama_runtime::Runtime>::from(nonce),
			frame_system::CheckWeight::<asset_hub_kusama_runtime::Runtime>::new(),
			pallet_asset_tx_payment::ChargeAssetTxPayment::<asset_hub_kusama_runtime::Runtime>::from(0, None),
		)
	}

	fn get_additional_signed(
		genesis_hash: H256,
		best_hash: H256,
	) -> <asset_hub_kusama_runtime::SignedExtra as SignedExtension>::AdditionalSigned {
		(
			(),
			asset_hub_kusama_runtime::VERSION.spec_version,
			asset_hub_kusama_runtime::VERSION.transaction_version,
			genesis_hash,
			best_hash,
			(),
			(),
			(),
		)
	}
}

impl SignedExtraProvider<asset_hub_westend_runtime::SignedExtra>
	for SignedExtraFor<asset_hub_westend_runtime::SignedExtra>
{
	fn get_extra(
		period: u64,
		best_block: BlockNumber,
		nonce: u32,
	) -> asset_hub_westend_runtime::SignedExtra {
		(
			frame_system::CheckNonZeroSender::<asset_hub_westend_runtime::Runtime>::new(),
			frame_system::CheckSpecVersion::<asset_hub_westend_runtime::Runtime>::new(),
			frame_system::CheckTxVersion::<asset_hub_westend_runtime::Runtime>::new(),
			frame_system::CheckGenesis::<asset_hub_westend_runtime::Runtime>::new(),
			frame_system::CheckEra::<asset_hub_westend_runtime::Runtime>::from(
				generic::Era::mortal(period, best_block.saturated_into()),
			),
			frame_system::CheckNonce::<asset_hub_westend_runtime::Runtime>::from(nonce),
			frame_system::CheckWeight::<asset_hub_westend_runtime::Runtime>::new(),
			pallet_asset_conversion_tx_payment::ChargeAssetTxPayment::<
				asset_hub_westend_runtime::Runtime,
			>::from(0, None),
		)
	}

	fn get_additional_signed(
		genesis_hash: H256,
		best_hash: H256,
	) -> <asset_hub_westend_runtime::SignedExtra as SignedExtension>::AdditionalSigned {
		(
			(),
			asset_hub_westend_runtime::VERSION.spec_version,
			asset_hub_westend_runtime::VERSION.transaction_version,
			genesis_hash,
			best_hash,
			(),
			(),
			(),
		)
	}
}

impl SignedExtraProvider<asset_hub_polkadot_runtime::SignedExtra>
	for SignedExtraFor<asset_hub_polkadot_runtime::SignedExtra>
{
	fn get_extra(
		period: u64,
		best_block: BlockNumber,
		nonce: u32,
	) -> asset_hub_polkadot_runtime::SignedExtra {
		(
			frame_system::CheckNonZeroSender::<asset_hub_polkadot_runtime::Runtime>::new(),
			frame_system::CheckSpecVersion::<asset_hub_polkadot_runtime::Runtime>::new(),
			frame_system::CheckTxVersion::<asset_hub_polkadot_runtime::Runtime>::new(),
			frame_system::CheckGenesis::<asset_hub_polkadot_runtime::Runtime>::new(),
			frame_system::CheckEra::<asset_hub_polkadot_runtime::Runtime>::from(
				generic::Era::mortal(period, best_block.saturated_into()),
			),
			frame_system::CheckNonce::<asset_hub_polkadot_runtime::Runtime>::from(nonce),
			frame_system::CheckWeight::<asset_hub_polkadot_runtime::Runtime>::new(),
			pallet_asset_tx_payment::ChargeAssetTxPayment::<asset_hub_polkadot_runtime::Runtime>::from(0, None),
		)
	}

	fn get_additional_signed(
		genesis_hash: H256,
		best_hash: H256,
	) -> <asset_hub_polkadot_runtime::SignedExtra as SignedExtension>::AdditionalSigned {
		(
			(),
			asset_hub_polkadot_runtime::VERSION.spec_version,
			asset_hub_polkadot_runtime::VERSION.transaction_version,
			genesis_hash,
			best_hash,
			(),
			(),
			(),
		)
	}
}

impl SignedExtraProvider<collectives_polkadot_runtime::SignedExtra>
	for SignedExtraFor<collectives_polkadot_runtime::SignedExtra>
{
	fn get_extra(
		period: u64,
		best_block: BlockNumber,
		nonce: u32,
	) -> collectives_polkadot_runtime::SignedExtra {
		(
			frame_system::CheckNonZeroSender::<collectives_polkadot_runtime::Runtime>::new(),
			frame_system::CheckSpecVersion::<collectives_polkadot_runtime::Runtime>::new(),
			frame_system::CheckTxVersion::<collectives_polkadot_runtime::Runtime>::new(),
			frame_system::CheckGenesis::<collectives_polkadot_runtime::Runtime>::new(),
			frame_system::CheckEra::<collectives_polkadot_runtime::Runtime>::from(
				generic::Era::mortal(period, best_block.saturated_into()),
			),
			frame_system::CheckNonce::<collectives_polkadot_runtime::Runtime>::from(nonce),
			frame_system::CheckWeight::<collectives_polkadot_runtime::Runtime>::new(),
		)
	}

	fn get_additional_signed(
		genesis_hash: H256,
		best_hash: H256,
	) -> <collectives_polkadot_runtime::SignedExtra as SignedExtension>::AdditionalSigned {
		(
			(),
			collectives_polkadot_runtime::VERSION.spec_version,
			collectives_polkadot_runtime::VERSION.transaction_version,
			genesis_hash,
			best_hash,
			(),
			(),
		)
	}
}

impl SignedExtraProvider<bridge_hub_polkadot_runtime::SignedExtra>
	for SignedExtraFor<bridge_hub_polkadot_runtime::SignedExtra>
{
	fn get_extra(
		period: u64,
		best_block: BlockNumber,
		nonce: u32,
	) -> bridge_hub_polkadot_runtime::SignedExtra {
		(
			frame_system::CheckNonZeroSender::<bridge_hub_polkadot_runtime::Runtime>::new(),
			frame_system::CheckSpecVersion::<bridge_hub_polkadot_runtime::Runtime>::new(),
			frame_system::CheckTxVersion::<bridge_hub_polkadot_runtime::Runtime>::new(),
			frame_system::CheckGenesis::<bridge_hub_polkadot_runtime::Runtime>::new(),
			frame_system::CheckEra::<bridge_hub_polkadot_runtime::Runtime>::from(
				generic::Era::mortal(period, best_block.saturated_into()),
			),
			frame_system::CheckNonce::<bridge_hub_polkadot_runtime::Runtime>::from(nonce),
			frame_system::CheckWeight::<bridge_hub_polkadot_runtime::Runtime>::new(),
			pallet_transaction_payment::ChargeTransactionPayment::<
				bridge_hub_polkadot_runtime::Runtime,
			>::from(0),
		)
	}

	fn get_additional_signed(
		genesis_hash: H256,
		best_hash: H256,
	) -> <bridge_hub_polkadot_runtime::SignedExtra as SignedExtension>::AdditionalSigned {
		(
			(),
			bridge_hub_polkadot_runtime::VERSION.spec_version,
			bridge_hub_polkadot_runtime::VERSION.transaction_version,
			genesis_hash,
			best_hash,
			(),
			(),
			(),
		)
	}
}

impl SignedExtraProvider<bridge_hub_kusama_runtime::SignedExtra>
	for SignedExtraFor<bridge_hub_kusama_runtime::SignedExtra>
{
	fn get_extra(
		period: u64,
		best_block: BlockNumber,
		nonce: u32,
	) -> bridge_hub_kusama_runtime::SignedExtra {
		(
			frame_system::CheckNonZeroSender::<bridge_hub_kusama_runtime::Runtime>::new(),
			frame_system::CheckSpecVersion::<bridge_hub_kusama_runtime::Runtime>::new(),
			frame_system::CheckTxVersion::<bridge_hub_kusama_runtime::Runtime>::new(),
			frame_system::CheckGenesis::<bridge_hub_kusama_runtime::Runtime>::new(),
			frame_system::CheckEra::<bridge_hub_kusama_runtime::Runtime>::from(
				generic::Era::mortal(period, best_block.saturated_into()),
			),
			frame_system::CheckNonce::<bridge_hub_kusama_runtime::Runtime>::from(nonce),
			frame_system::CheckWeight::<bridge_hub_kusama_runtime::Runtime>::new(),
			pallet_transaction_payment::ChargeTransactionPayment::<
				bridge_hub_kusama_runtime::Runtime,
			>::from(0),
		)
	}

	fn get_additional_signed(
		genesis_hash: H256,
		best_hash: H256,
	) -> <bridge_hub_kusama_runtime::SignedExtra as SignedExtension>::AdditionalSigned {
		(
			(),
			bridge_hub_kusama_runtime::VERSION.spec_version,
			bridge_hub_kusama_runtime::VERSION.transaction_version,
			genesis_hash,
			best_hash,
			(),
			(),
			(),
		)
	}
}

impl SignedExtraProvider<bridge_hub_rococo_runtime::SignedExtra>
	for SignedExtraFor<bridge_hub_rococo_runtime::SignedExtra>
{
	fn get_extra(
		period: u64,
		best_block: BlockNumber,
		nonce: u32,
	) -> bridge_hub_rococo_runtime::SignedExtra {
		(
			frame_system::CheckNonZeroSender::<bridge_hub_rococo_runtime::Runtime>::new(),
			frame_system::CheckSpecVersion::<bridge_hub_rococo_runtime::Runtime>::new(),
			frame_system::CheckTxVersion::<bridge_hub_rococo_runtime::Runtime>::new(),
			frame_system::CheckGenesis::<bridge_hub_rococo_runtime::Runtime>::new(),
			frame_system::CheckEra::<bridge_hub_rococo_runtime::Runtime>::from(
				generic::Era::mortal(period, best_block.saturated_into()),
			),
			frame_system::CheckNonce::<bridge_hub_rococo_runtime::Runtime>::from(nonce),
			frame_system::CheckWeight::<bridge_hub_rococo_runtime::Runtime>::new(),
			pallet_transaction_payment::ChargeTransactionPayment::<bridge_hub_rococo_runtime::Runtime>::from(0),
			bridge_hub_rococo_runtime::BridgeRejectObsoleteHeadersAndMessages {},
			(
				bridge_hub_rococo_runtime::bridge_hub_wococo_config::BridgeRefundBridgeHubRococoMessages::default(),
				bridge_hub_rococo_runtime::bridge_hub_rococo_config::BridgeRefundBridgeHubWococoMessages::default(),
			),
		)
	}

	fn get_additional_signed(
		genesis_hash: H256,
		best_hash: H256,
	) -> <bridge_hub_rococo_runtime::SignedExtra as SignedExtension>::AdditionalSigned {
		(
			(),
			bridge_hub_rococo_runtime::VERSION.spec_version,
			bridge_hub_rococo_runtime::VERSION.transaction_version,
			genesis_hash,
			best_hash,
			(),
			(),
			(),
			(),
			((), ()),
		)
	}
}

/// Generates `System::Remark` extrinsics for the benchmarks.
///
/// Note: Should only be used for benchmarking.
pub struct RemarkBuilder<RuntimeApi, SignedExtra, RuntimeCall, Extra, SystemConfig> {
	client: Arc<ParachainClient<RuntimeApi>>,
	_phantom: PhantomData<(SignedExtra, RuntimeCall, Extra, SystemConfig)>,
}

impl<RuntimeApi, SignedExtra, RuntimeCall, Extra, SystemConfig>
	RemarkBuilder<RuntimeApi, SignedExtra, RuntimeCall, Extra, SystemConfig>
{
	/// Creates a new [`Self`] from the given client.
	pub fn new(client: Arc<ParachainClient<RuntimeApi>>) -> Self {
		Self { client, _phantom: PhantomData }
	}
}

impl<RuntimeApi, SignedExtra, RuntimeCall, Extra, SystemConfig>
	frame_benchmarking_cli::ExtrinsicBuilder
	for RemarkBuilder<RuntimeApi, SignedExtra, RuntimeCall, Extra, SystemConfig>
where
	SystemConfig: frame_system::Config,
	SignedExtra: Clone,
	RuntimeCall: From<frame_system::Call<SystemConfig>> + Clone + Encode,
	SignedExtra: SignedExtension,
	Extra: SignedExtraProvider<SignedExtra>,
{
	fn pallet(&self) -> &str {
		"system"
	}

	fn extrinsic(&self) -> &str {
		"remark"
	}

	fn build(&self, nonce: u32) -> Result<OpaqueExtrinsic, &'static str> {
		let period = polkadot_runtime_common::BlockHashCount::get()
			.checked_next_power_of_two()
			.map(|c| c / 2)
			.unwrap_or(2) as u64;
		let best_block = self.client.chain_info().best_number;
		let extra = Extra::get_extra(period, best_block, nonce);

		let call: RuntimeCall = frame_system::Call::remark { remark: vec![] }.into();
		let genesis_hash = self.client.block_hash(0).ok().flatten().expect("Genesis block exists");
		let best_hash = self.client.chain_info().best_hash;
		let payload = generic::SignedPayload::<RuntimeCall, SignedExtra>::from_raw(
			call.clone().into(),
			extra.clone(),
			Extra::get_additional_signed(genesis_hash, best_hash),
		);

		let sender = Sr25519Keyring::Bob.pair();
		let signature = payload.using_encoded(|x| sender.sign(x));
		let extrinsic = generic::UncheckedExtrinsic::new_signed(
			call,
			sp_runtime::AccountId32::from(sender.public()),
			parachains_common::Signature::Sr25519(signature),
			extra,
		);

		Ok(extrinsic.into())
	}
}

/// Generates inherent data for the `benchmark overhead` command.
pub fn inherent_benchmark_data() -> sc_cli::Result<InherentData> {
	let mut inherent_data = InherentData::new();

	let timestamp = sp_timestamp::InherentDataProvider::new(Duration::ZERO.into());
	futures::executor::block_on(timestamp.provide_inherent_data(&mut inherent_data))
		.map_err(|e| format!("creating inherent data: {e:?}"))?;

	let parachain_inherent = MockValidationDataInherentDataProvider {
		current_para_block: 1,
		relay_offset: 0,
		relay_blocks_per_para_block: 1,
		para_blocks_per_relay_epoch: 0,
		relay_randomness_config: (),
		xcm_config: Default::default(),
		raw_downward_messages: Default::default(),
		raw_horizontal_messages: Default::default(),
	};

	futures::executor::block_on(parachain_inherent.provide_inherent_data(&mut inherent_data))
		.map_err(|e| format!("creating inherent data: {e:?}"))?;

	Ok(inherent_data)
}
