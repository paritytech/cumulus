#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Triggered(u32),
		TriggeredSigned(T::AccountId, u32),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidMultiLocation,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(Weight::from_ref_time(10_000))]
		pub fn do_something(_origin: OriginFor<T>, something: u32) -> DispatchResult {
			Self::deposit_event(Event::Triggered(something));
			Ok(())
		}

		#[pallet::weight(Weight::from_ref_time(10_000))]
		pub fn do_something_as_signed(origin: OriginFor<T>, something: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::deposit_event(Event::TriggeredSigned(who, something));
			Ok(())
		}
	}
}
