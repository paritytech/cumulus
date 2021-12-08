#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

// pub use polkadot_primitives::v1::{HeadData, ValidationCode}

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;
	use cumulus_pallet_parachain_system as parachain_system;

	#[pallet::config]
	pub trait Config: frame_system::Config + cumulus_pallet_parachain_system::Config {
		type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event {
		MigrationScheduled
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(0)]
		pub fn schedule_migration(
			origin: OriginFor<T>,
			code: Vec<u8>,
			head_data: Vec<u8>,
		) -> DispatchResult {
			ensure_root(origin)?;

			parachain_system::Pallet::<T>::set_code_impl(code)?;
			parachain_system::Pallet::<T>::set_pending_custom_validation_head_data(head_data);

			Self::deposit_event(Event::MigrationScheduled);
			Ok(())
		}
	}
}
