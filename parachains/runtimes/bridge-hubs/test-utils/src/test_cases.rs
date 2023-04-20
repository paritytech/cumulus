// Copyright 2023 Parity Technologies (UK) Ltd.
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

//! Module contains predefined test-case scenarios for `Runtime` with bridging capabilities.

use frame_support::{assert_ok, traits::Get};

// Lets re-use this stuff from assets (later we plan to move it outside of assets as `runtimes/test-utils`)
use asset_test_utils::{AccountIdOf, ExtBuilder, RuntimeHelper, ValidatorIdOf};

// Re-export test_cases from assets
pub use asset_test_utils::{include_teleports_for_native_asset_works, CollatorSessionKeys};

/// Test-case makes sure that `Runtime` can process bridging initialize via governance-like call
pub fn initialize_bridge_by_governance_works<Runtime, XcmConfig, GrandpaPalletInstance>(
	collator_session_key: CollatorSessionKeys<Runtime>,
	runtime_call_encode: Box<
		dyn Fn(pallet_bridge_grandpa::Call<Runtime, GrandpaPalletInstance>) -> Vec<u8>,
	>,
	runtime_para_id: u32,
) where
	Runtime: frame_system::Config
		+ pallet_balances::Config
		+ pallet_session::Config
		+ pallet_xcm::Config
		+ parachain_info::Config
		+ pallet_collator_selection::Config
		+ cumulus_pallet_dmp_queue::Config
		+ cumulus_pallet_parachain_system::Config
		+ pallet_bridge_grandpa::Config<GrandpaPalletInstance>,
	GrandpaPalletInstance: 'static,
	ValidatorIdOf<Runtime>: From<AccountIdOf<Runtime>>,
{
	ExtBuilder::<Runtime>::default()
		.with_collators(collator_session_key.collators())
		.with_session_keys(collator_session_key.session_keys())
		.with_para_id(runtime_para_id.into())
		.with_tracing()
		.build()
		.execute_with(|| {
			// check mode before
			assert_eq!(
			pallet_bridge_grandpa::PalletOperatingMode::<Runtime, GrandpaPalletInstance>::try_get(),
			Err(())
		);

			// encode `initialize` call
			let initialize_call = runtime_call_encode(pallet_bridge_grandpa::Call::<
				Runtime,
				GrandpaPalletInstance,
			>::initialize {
				init_data: test_data::initialiation_data::<Runtime, GrandpaPalletInstance>(12345),
			});

			// overestimate - check `pallet_bridge_grandpa::Pallet::initialize()` call
			let require_weight_at_most =
				<Runtime as frame_system::Config>::DbWeight::get().reads_writes(7, 7);

			// execute XCM with Transacts to initialize bridge as governance does
			// prepare data for xcm::Transact(create)
			assert_ok!(RuntimeHelper::<Runtime>::execute_as_governance(
				initialize_call,
				require_weight_at_most
			)
			.ensure_complete());

			// check mode after
			assert_eq!(
			pallet_bridge_grandpa::PalletOperatingMode::<Runtime, GrandpaPalletInstance>::try_get(),
			Ok(bp_runtime::BasicOperatingMode::Normal)
		);
		})
}

#[macro_export]
macro_rules! include_initialize_bridge_by_governance_works(
	(
		$test_name:tt,
		$runtime:path,
		$xcm_config:path,
		$pallet_bridge_grandpa_instance:path,
		$collator_session_key:expr,
		$runtime_call_encode:expr,
		$runtime_para_id:expr
	) => {
		#[test]
		fn $test_name() {
			$crate::test_cases::initialize_bridge_by_governance_works::<
				$runtime,
				$xcm_config,
				$pallet_bridge_grandpa_instance,
			>(
				$collator_session_key,
				$runtime_call_encode,
				$runtime_para_id
			)
		}
	}
);

// process_export_message_from_system_parachain_works
//
// test_back_preasure_xcmp
//
// dispatch_blob_and_xcm_routing_works_on_bridge_hub_rococo
// dispatch_blob_and_xcm_routing_works_on_bridge_hub_wococo
//
// can_govornance_call_xcm_transact_with_initialize_on_bridge_hub_rococo
// can_govornance_call_xcm_transact_with_initialize_bridge_on_bridge_hub_wococo

mod test_data {

	/// Helper that creates InitializationData mock data, that can be used to initialize bridge GRANDPA pallet
	pub fn initialiation_data<
		Runtime: pallet_bridge_grandpa::Config<GrandpaPalletInstance>,
		GrandpaPalletInstance: 'static,
	>(
		block_number: u32,
	) -> bp_header_chain::InitializationData<
		pallet_bridge_grandpa::BridgedHeader<Runtime, GrandpaPalletInstance>,
	> {
		bp_header_chain::InitializationData {
			header: Box::new(bp_test_utils::test_header(block_number.into())),
			authority_list: Default::default(),
			set_id: 6,
			operating_mode: bp_runtime::BasicOperatingMode::Normal,
		}
	}
}
