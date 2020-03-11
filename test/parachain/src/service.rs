// Copyright 2019 Parity Technologies (UK) Ltd.
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

use std::sync::Arc;

use parachain_runtime::{self, GenesisConfig};

use sc_executor::native_executor_instance;
use sc_service::{AbstractService, Configuration};
use sc_finality_grandpa::{FinalityProofProvider as GrandpaFinalityProofProvider, StorageAndProofProvider};

use polkadot_primitives::parachain::CollatorPair;

use cumulus_collator::CollatorBuilder;

use futures::FutureExt;

pub use sc_executor::NativeExecutor;

// Our native executor instance.
native_executor_instance!(
	pub Executor,
	parachain_runtime::api::dispatch,
	parachain_runtime::native_version,
);

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
macro_rules! new_full_start {
	($config:expr) => {{
		let inherent_data_providers = sp_inherents::InherentDataProviders::new();

		let builder = sc_service::ServiceBuilder::new_full::<
			parachain_runtime::opaque::Block,
			parachain_runtime::RuntimeApi,
			crate::service::Executor,
		>($config)?
		.with_select_chain(|_config, backend| Ok(sc_client::LongestChain::new(backend.clone())))?
		.with_transaction_pool(|config, client, _| {
			let pool_api = Arc::new(sc_transaction_pool::FullChainApi::new(client.clone()));
			let pool = sc_transaction_pool::BasicPool::new(config, pool_api);
			Ok(pool)
		})?
		.with_import_queue(|_config, client, _, _| {
			let import_queue = cumulus_consensus::import_queue::import_queue(
				client.clone(),
				client,
				inherent_data_providers.clone(),
			)?;

			Ok(import_queue)
		})?;

		(builder, inherent_data_providers)
		}};
}

/// Run a collator node with the given parachain `Configuration` and relaychain `Configuration`
///
/// This function blocks until done.
pub fn run_collator<E: sc_service::ChainSpecExtension>(
	parachain_config: Configuration<GenesisConfig, E>,
	key: Arc<CollatorPair>,
	mut polkadot_config: polkadot_collator::Configuration,
) -> sc_cli::Result<()> {
	sc_cli::run_service_until_exit(parachain_config, move |parachain_config| {
		polkadot_config.task_executor = parachain_config.task_executor.clone();

		let (builder, inherent_data_providers) = new_full_start!(parachain_config);
		inherent_data_providers
			.register_provider(sp_timestamp::InherentDataProvider)
			.unwrap();

		let service = builder
			.with_finality_proof_provider(|client, backend| {
				// GenesisAuthoritySetProvider is implemented for StorageAndProofProvider
				let provider = client as Arc<dyn StorageAndProofProvider<_, _>>;
				Ok(Arc::new(GrandpaFinalityProofProvider::new(backend, provider)) as _)
			})?
			.build()?;

		let proposer_factory = sc_basic_authorship::ProposerFactory::new(
			service.client(),
			service.transaction_pool(),
		);

		let block_import = service.client();
		let client = service.client();
		let builder = CollatorBuilder::new(
			proposer_factory,
			inherent_data_providers,
			block_import,
			crate::PARA_ID,
			client,
		);

		let polkadot_future = polkadot_collator::start_collator(
			builder,
			crate::PARA_ID,
			key,
			polkadot_config,
		).map(|_| ());
		service.spawn_essential_task("polkadot", polkadot_future);

		Ok(service)
	})
}
