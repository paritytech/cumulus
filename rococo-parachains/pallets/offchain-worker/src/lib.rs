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

use sp_std::vec::Vec;

// QUESTIONS: Where does the `T` come from?
#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use frame_support::codec::{Encode, Decode};
    use frame_system::pallet;
    use frame_support::sp_runtime::print;
    use frame_support::sp_runtime::traits::{MaybeDisplay, AtLeast32BitUnsigned};
    use frame_support::sp_runtime::sp_std::fmt::Debug;
    use frame_support::sp_runtime::{FixedU128, FixedPointNumber};
    use sp_runtime::offchain::{http, Duration};
    use frame_system::offchain::{SubmitTransaction, CreateSignedTransaction, SigningTypes, SendTransactionTypes};
    use sp_std::vec::Vec;
    use pallet_assets as assets;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    // TODO: check why CreateSignedTransaction<Call<Self>> is needed for unsigned transactions?
    pub trait Config: frame_system::Config + assets::Config + SendTransactionTypes<Call<Self>> + SigningTypes {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Call: From<Call<Self>>;
        // Traituration parameters
        type UnsignedInterval: Get<Self::BlockNumber>;
        /// A configuration for base priority of unsigned transactions.
        type UnsignedPriority: Get<TransactionPriority>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // The pallet's runtime storage items.
    // https://substrate.dev/docs/en/knowledgebase/runtime/storage
    #[pallet::storage]
    #[pallet::getter(fn next_unsigned_at)]
    pub(super) type NextUnsignedAt<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        NewPrice(T::AssetId, FixedU128),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        CryptoAlreadyExists,
        CryptoDoesNotExist,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(block_number: T::BlockNumber) {
            // debug::info!("Hello World from offchain workers!");
            // debug::info!("Current block: {:?}", block_number);

            print("hello world from offchain worker");
            let res = Self::fetch_price_and_send_raw_unsigned(block_number);
            if let Err(e) = res {
                // debug::error!("Error: {:?}", e);
                print(e);
            }
        }
    }

    #[pallet::call]
    impl<T:Config> Pallet<T> {
        #[pallet::weight(1)]
        pub fn submit_price_unsigned(origin: OriginFor<T>, _block_number: T::BlockNumber, prices: Vec<u32>) -> DispatchResultWithPostInfo {
            // This ensures that the function can only be called via unsigned transaction.
            ensure_none(origin)?;
            // Add the price to the on-chain list, but mark it as coming from an empty address.

            // debug::info!("received prices from caller, ready to update prices");
            // update all the prices
            for (idx, price) in prices.iter().enumerate() {

                print("price for index");
                print(idx);
                print(price);

                // TODO: check u8 here, seems different setup has diff value
                // TODO: Check on the build pipeline, should be u32 or u64.
                let asset_id = T::AssetId::from(idx as u8);

                let price = FixedU128::saturating_from_rational(*price, 100);
                <assets::Module<T>>::_set_price(asset_id, price);
                Self::deposit_event(Event::NewPrice(asset_id, price));
            }

            // now increment the block number at which we expect next unsigned transaction.
            let current_block = <frame_system::Module<T>>::block_number();
            <NextUnsignedAt<T>>::put(current_block + T::UnsignedInterval::get());

            Ok(().into())
        }
    }

    impl<T:Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        fn validate_unsigned(
            _source: TransactionSource,
            call: &Self::Call,
        ) -> TransactionValidity {
            // Firstly let's check that we call the right function.
            if let Call::submit_price_unsigned(block_number, new_price) = call {
                Self::validate_transaction_parameters(block_number, new_price)
            } else {
                InvalidTransaction::Call.into()
            }
        }
    }

    impl<T:Config> Pallet<T> {
        /// A helper function to fetch the price and send a raw unsigned transaction.
        fn fetch_price_and_send_raw_unsigned(block_number: T::BlockNumber) -> DispatchResultWithPostInfo {
            let next_unsigned_at = <NextUnsignedAt<T>>::get();
            if next_unsigned_at > block_number {
                // debug::// debug!("Too early to send unsigned transaction");
                return Ok(().into())
            }

            let prices = Self::fetch_price().map_err(|_| "Failed to fetch price")?;

            let call = Call::submit_price_unsigned(block_number, prices);

            SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
                .map_err(|()| "Unable to submit unsigned transaction.")?;

            Ok(().into())
        }

        /// Fetch current price and return the result in cents.
        fn fetch_price() -> Result<Vec<u32>, http::Error> {
            let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));
            let request = http::Request::get(
                "http://node-helper:8080/assets/prices"
            );
            let pending = request
                .deadline(deadline)
                .send()
                .map_err(|_| http::Error::IoError)?;

            let response = pending.try_wait(deadline)
                .map_err(|_| http::Error::DeadlineReached)??;
            if response.code != 200 {
                // debug::warn!("Unexpected status code: {}", response.code);
                return Err(http::Error::Unknown);
            }

            let body = response.body().collect::<Vec<u8>>();

            let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
                // debug::warn!("No UTF8 body");
                http::Error::Unknown
            })?;
            // debug::warn!("BODY: {}", body_str);
            let prices = Self::parse_price(body_str);
            if prices.is_empty() {
                // debug::warn!("Unable to extract price from the response: {:?}", body_str);
                return Err(http::Error::Unknown);
            }
            // debug::warn!("Got price: {:?} cents", prices);

            Ok(prices)
        }

        /// Returns `None` when parsing failed or `Some(price in cents)` when parsing is successful.
        fn parse_price(price_str: &str) -> Vec<u32> {
            let components = price_str.split(",");
            let mut prices: Vec<u32> = Vec::new();
            for s in components {
                prices.push(s.parse().unwrap());
            }
            // debug::info!("prices are {:?}", prices);
            return prices;
        }

        fn validate_transaction_parameters(
            block_number: &T::BlockNumber,
            new_price: &Vec<u32>,
        ) -> TransactionValidity {
            // Now let's check if the transaction has any chance to succeed.
            let next_unsigned_at = <NextUnsignedAt<T>>::get();
            if &next_unsigned_at > block_number {
                return InvalidTransaction::Stale.into();
            }
            // Let's make sure to reject transactions from the future.
            let current_block = <frame_system::Module<T>>::block_number();
            if &current_block < block_number {
                return InvalidTransaction::Future.into();
            }

            ValidTransaction::with_tag_prefix("OffchainWorker")
                .priority(T::UnsignedPriority::get())
                .and_provides(next_unsigned_at)
                .longevity(5)
                .propagate(true)
                .build()
        }
    }
}
