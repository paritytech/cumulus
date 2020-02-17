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

use parachain_runtime::{self, opaque::Block, GenesisConfig};

use sc_executor::native_executor_instance;
use sc_network::construct_simple_protocol;
use sc_service::{AbstractService, Configuration};
use sp_consensus::{BlockImport, Environment, Proposer};
use sp_inherents::InherentDataProviders;
use sp_core::crypto::Pair;

use polkadot_collator::build_collator_service;
use polkadot_primitives::parachain::CollatorPair;
use polkadot_service::IsKusama;

use cumulus_collator::CollatorBuilder;

use futures::{future, task::Spawn, FutureExt, TryFutureExt, select, pin_mut};

use log::error;

pub use sc_executor::NativeExecutor;

// Our native executor instance.
native_executor_instance!(
	pub Executor,
	parachain_runtime::api::dispatch,
	parachain_runtime::native_version,
);

construct_simple_protocol! {
	/// Demo protocol attachment for substrate.
	pub struct NodeProtocol where Block = Block { }
}

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
	mut parachain_config: Configuration<GenesisConfig, E>,
	key: Arc<CollatorPair>,
	polkadot_config: polkadot_collator::Configuration,
) -> sc_cli::error::Result<()> {
	sc_cli::run_until_exit(polkadot_config, |polkadot_config| {
		parachain_config.task_executor = polkadot_config.task_executor.clone();

		let (builder, inherent_data_providers) = new_full_start!(parachain_config);
		inherent_data_providers
			.register_provider(sp_timestamp::InherentDataProvider)
			.unwrap();

		let service = builder
			.with_network_protocol(|_| Ok(NodeProtocol::new()))?
			.build()?;

		let proposer_factory = sc_basic_authorship::ProposerFactory {
			client: service.client(),
			transaction_pool: service.transaction_pool(),
		};

		let block_import = service.client();
		let client = service.client();
		let builder = move || {
			CollatorBuilder::new(
				proposer_factory,
				inherent_data_providers,
				block_import,
				crate::PARA_ID,
				client,
			)
		};

/*
		let f = async {
		//let f: Box<dyn future::Future<Output = Result<(), sc_cli::error::Error>>> = async {
			let polkadot_future = polkadot_collator::build_collator(polkadot_config, crate::PARA_ID, key, Box::new(builder)).fuse();
			let parachain_future = service.map(|_| ());

			pin_mut!(polkadot_future, parachain_future);

			select! {
				_ = polkadot_future => {
					return Err(sc_cli::error::Error::Other("boo".into()));
				},
				_ = parachain_future => {},
			}

			Ok(())
		};
*/
		let polkadot_future = polkadot_collator::build_collator(polkadot_config, crate::PARA_ID, key, Box::new(builder));
		let parachain_future = service.map_err(|e| error!("Parachain service error: {:?}", e));
		let f = future::select(polkadot_future, parachain_future).map(|_| Ok(()));

		Ok(f)
	})
}
