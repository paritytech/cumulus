#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// QUESTIONS: Where does the `T` come from?
#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use frame_support::codec::{Encode, Decode};
	use frame_system::{pallet, Origin};
	use frame_support::sp_runtime::traits::{MaybeDisplay, One, AtLeast32BitUnsigned, Zero, Bounded};
	use frame_support::sp_runtime::sp_std::fmt::Debug;
	use frame_support::sp_runtime::print;
	use frame_support::sp_runtime::{FixedU128, FixedPointNumber, FixedPointOperand};
	use frame_support::sp_std::convert::TryInto;
	use traits::{Oracle, MultiAsset};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Balance: Member + Parameter + FixedPointOperand + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;
		type AssetId: Parameter + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T:Config> where {
		/// Some assets were issued. \[asset_id, owner, total_supply\]
		AssetIssued(T::AssetId),
		Issued(T::AssetId, T::AccountId, T::Balance),
		/// Some assets were transferred. \[asset_id, from, to, amount\]
		Transferred(T::AssetId, T::AccountId, T::AccountId, T::Balance),
		/// Some assets were destroyed. \[asset_id, owner, balance\]
		Destroyed(T::AssetId, T::AccountId, T::Balance),
	}

	// The pallet's runtime storage items.
	#[pallet::storage]
	#[pallet::getter(fn next_asset_id)]
	pub(super) type NextAssetId<T: Config> = StorageValue<_, T::AssetId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_asset_total_supply)]
	pub(super) type TotalSupply<T: Config> = StorageMap<_, Twox64Concat, T::AssetId, T::Balance, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_asset_balance)]
	pub(super) type Balances<T: Config> = StorageMap<_, Blake2_128Concat, (T::AssetId, T::AccountId), T::Balance, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn inherent_asset_id)]
	pub(super) type InherentAsset<T: Config> = StorageValue<_, T::AssetId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn price)]
	pub(super) type Price<T: Config> = StorageMap<_, Twox64Concat, T::AssetId, FixedU128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn owner)]
	pub(super) type Owner<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Balance is not zero when destroying.
		NoneZeroBalanceWhenDestroy,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T:Config> Pallet<T> {
		#[pallet::weight(1)]
		pub fn issue_new_asset(origin: OriginFor<T>, price: FixedU128, #[pallet::compact] amount: T::Balance) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;

			let id = <NextAssetId<T>>::get();
			// debug::info!("next asset id is {:?}", id);
			<NextAssetId<T>>::mutate(|id| *id += One::one());
			<Pallet<T>>::_set_price(id, price);

			// TODO: remove this with the actual code, this is for testing only
			<Balances<T>>::insert((id, origin.clone()), amount);

			// debug::info!("amount {:?} issued to user {:?}", amount, origin);
			Self::deposit_event(Event::AssetIssued(id));

			Ok(().into())
		}

		#[pallet::weight(1)]
		pub fn issue(origin: OriginFor<T>, #[pallet::compact] total: T::Balance) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;

			let id = <NextAssetId<T>>::get();
			// debug::info!("next asset id is {:?}", id);
			<NextAssetId<T>>::mutate(|id| *id += One::one());

			<Balances<T>>::insert((id, origin.clone()), total);
			<TotalSupply<T>>::insert(id, total);

			Self::deposit_event(Event::Issued(id, origin, total));

			Ok(().into())
		}

		#[pallet::weight(1)]
		fn destroy(origin: OriginFor<T>, id: T::AssetId) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;
			let balance = <Balances<T>>::take((id, origin.clone()));

			if !balance.is_zero() { return Err(<Error<T>>::NoneZeroBalanceWhenDestroy.into()); }

			<TotalSupply<T>>::mutate(id, |total_supply| *total_supply -= balance);
			Self::deposit_event(Event::Destroyed(id, origin, balance));

			Ok(().into())
		}

		/// What does this function do?
		#[pallet::weight(1)]
		pub fn set_inherent_asset(origin: OriginFor<T>, asset: T::AssetId) -> DispatchResultWithPostInfo {
			//ensure_root(origin)?;
			<InherentAsset<T>>::mutate(|ia| *ia = asset.clone());
			Ok(().into())
		}


		#[pallet::weight(1)]
		pub fn transfer_asset(origin: OriginFor<T>,
							  asset_id: T::AssetId,
							  to_account: T::AccountId,
							  #[pallet::compact] amount: T::Balance
		) -> DispatchResultWithPostInfo {
			let from_account = ensure_signed(origin)?;
			return Self::transfer(from_account, asset_id, to_account, amount);
		}


		#[pallet::weight(1)]
		pub fn set_price(origin: OriginFor<T>, id: T::AssetId, price: FixedU128) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			Self::_set_price(id, price);
			Ok(().into())
		}
	}

	impl<T: Config> Oracle<T::AssetId, FixedU128> for Pallet<T> {
		fn get_rate(asset_id: T::AssetId) -> FixedU128 {
			Self::price(asset_id)
		}
	}

	impl<T: Config> MultiAsset<T::AccountId, T::AssetId, T::Balance> for Pallet<T> {
		fn transfer(from: T::AccountId, id: T::AssetId, to: T::AccountId, amount: T::Balance) -> DispatchResultWithPostInfo {
			Self::transfer(from, id, to, amount)
		}
	}

	impl<T:Config> Pallet<T> {
		/// Issue a new class of fungible assets. There are, and will only ever be, `total`
		/// such assets and they'll all belong to the `origin` initially. It will have an
		/// identifier `AssetId` instance: this will be specified in the `Issued` event.
		/// This will make a increased id asset.
		/// @origin
		/// @total    How much balance of new asset
		fn _issue(account: T::AccountId, total: T::Balance) -> DispatchResultWithPostInfo {
			let id = Self::next_asset_id();
			<NextAssetId<T>>::mutate(|id| *id += One::one());

			<Balances<T>>::insert((id, account.clone()), total);
			<TotalSupply<T>>::insert(id, total);

			Self::deposit_event(Event::Issued(id, account, total));

			Ok(().into())
		}

		pub fn _set_price(id: T::AssetId, price: FixedU128) {
			<Price<T>>::insert(id, price);
		}

		/// Move some assets from one holder to another.
		/// @from_account    The account lost amount of a certain asset balance
		/// @asset_id        The asset id to be transferred
		/// @to_account      The account receive the sent asset balance
		/// @amount          The amount value to be transferred
		pub fn transfer(
			from_account: T::AccountId,
			asset_id: T::AssetId,
			to_account: T::AccountId,
			amount: T::Balance,
		) -> DispatchResultWithPostInfo {
			let origin_account = (asset_id, from_account.clone());
			let origin_balance = <Balances<T>>::get(&origin_account);
			let target = to_account;
			ensure!(!amount.is_zero(), "transfer amount should be non-zero");
			ensure!(origin_balance >= amount,"origin account balance must be greater than or equal to the transfer amount");

			Self::deposit_event(Event::Transferred(
				asset_id,
				from_account,
				target.clone(),
				amount,
			));

			print("before transfer target balance ----> ");
			let b = TryInto::<u128>::try_into(Self::get_asset_balance(&(asset_id.clone(), target.clone())))
				.ok()
				.expect("Balance is u128");
			print(b as u64);
			<Balances<T>>::insert(origin_account, origin_balance - amount);
			<Balances<T>>::mutate((asset_id, target.clone()), |balance| *balance += amount);
			print("after transfer target balance----> ");

			let b = TryInto::<u128>::try_into(Self::get_asset_balance(&(asset_id.clone(), target)))
				.ok()
				.expect("Balance is u128");
			print(b as u64);
			Ok(().into())
		}

		/// Get the asset `id` balance of `who`.
		/// @id    Asset id
		/// @who   Account id
		pub fn balance(id: T::AssetId, who: T::AccountId) -> T::Balance {
			// debug
			print("----> Account Asset Balance");
			let b = TryInto::<u128>::try_into(Self::get_asset_balance(&(id.clone(), who.clone())))
				.ok()
				.expect("Balance is u128");
			print(b as u64);

			<Balances<T>>::get((id, who))
		}

		/// Get the total supply of an asset `id`.
        /// @id    Asset id
		pub fn total_supply(id: T::AssetId) -> T::Balance {
			// debug
			print("----> Asset Total Supply");
			let b = TryInto::<u128>::try_into(Self::get_asset_total_supply(id.clone()))
				.ok()
				.expect("Balance is u128");
			print(b as u64);

			<TotalSupply<T>>::get(id)
		}
	}
}
