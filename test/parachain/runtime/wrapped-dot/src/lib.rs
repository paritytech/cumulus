#![cfg_attr(not(feature = "std"), no_std)]

/// A FRAME pallet template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate FRAME, see the example pallet
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs

use frame_support::{
	decl_module,
	decl_storage,
	decl_event,
	decl_error,
	dispatch::{self, Parameter},
	ensure,
	weights::SimpleDispatchInfo,
};
use frame_system::{self as system, ensure_signed};
use codec::Codec;
use sp_runtime::traits::{
	Member,
	AtLeast32Bit,
	MaybeSerializeDeserialize,
	EnsureOrigin,
};
use sp_std::fmt::Debug;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	/// The wrapped dot balance of an account on this parachain
	type Balance: Parameter + Member + AtLeast32Bit + Codec + Default + Copy +
		MaybeSerializeDeserialize + Debug;
	/// Origin from which DOTs may be transferred into the parachain
	type TransferInOrigin: EnsureOrigin<Self::Origin>;
}

// This pallet's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as WrappedDot {
		Balances get(balance_of): map hasher(blake2_128_concat) T::AccountId => T::Balance;
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T>
		where
			AccountId = <T as system::Trait>::AccountId,
			Balance = <T as Trait>::Balance,
	{
		/// DOTs were transferred within the parachain
		InternalTransfer(AccountId, AccountId, Balance),
		/// DOTs were transferred out to the relay chain
		TransferOut(AccountId, AccountId, Balance),
		/// DOTs were transferred in from the relay chain
		TransferIn(AccountId, Balance),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// The user trying to make a transfer does not have enough funds
		InsufficientFunds,
		// Assuming for now that entire issuance fits in balance type, so not checking for overflow
		// /// Integer overflow occurred
		// Overflow,
	}
}

// The pallet's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;

		/// Transfer wrapped DOT tokens within the parachain
		#[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
		pub fn transfer(origin, dest: T::AccountId, value: T::Balance) -> dispatch::DispatchResult {
			// Check it was signed and get the signer.
			let source = ensure_signed(origin)?;

			// Ensure the source account has enough funds
			let source_balance = Balances::<T>::get(&source);
			ensure!(source_balance > value, Error::<T>::InsufficientFunds);

			// Update the balances
			Balances::<T>::insert(&source, source_balance - value);
			Balances::<T>::insert(&dest, Balances::<T>::get(&dest) + value);

			Self::deposit_event(RawEvent::InternalTransfer(source, dest, value));
			Ok(())
		}

		/// Simulate a transfer of DOTs in from the relay chain
		#[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
		pub fn transfer_in(origin, dest: T::AccountId, value: T::Balance) -> dispatch::DispatchResult {
			// Ensure that the method is called only from the correct origin.
			T::ExternalOrigin::ensure_origin(origin)?;

			// Mint new tokens in the parachain
			Balances::<T>::insert(&dest, Balances::<T>::get(&dest) + value);

			Self::deposit_event(RawEvent::TransferIn(dest, value));
			Ok(())
		}

		/// Simulate a transfer of DOTs out to the relay chain
		#[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
		pub fn transferout(origin, relay_dest: T::AccountId, value: T::Balance) -> dispatch::DispatchResult {
			// Check it was signed and get the signer.
			let source = ensure_signed(origin)?;

			// Ensure the source account has enough funds
			let source_balance = Balances::<T>::get(&source);
			ensure!(source_balance > value, Error::<T>::InsufficientFunds);

			// Update source balance
			Balances::<T>::insert(&source, source_balance - value);

			//TODO upward message to relay chain

			Self::deposit_event(RawEvent::TransferOut(source, relay_dest, value));
			Ok(())
		}
	}
}
