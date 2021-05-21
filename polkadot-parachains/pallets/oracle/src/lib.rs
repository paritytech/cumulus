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
    use frame_support::sp_runtime::{FixedU128, FixedPointNumber};
    use sp_runtime::offchain::{http, Duration};
    use frame_system::offchain::{SubmitTransaction, SigningTypes, SendTransactionTypes};
    use sp_std::vec::Vec;
    use polkadot_parachain_primitives::{CurrencyId, PriceValue};
    use pallet_traits::PriceSetter;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    // TODO: check why CreateSignedTransaction<Call<Self>> is needed for unsigned transactions?
    pub trait Config: frame_system::Config + SendTransactionTypes<Call<Self>> + SigningTypes {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Call: From<Call<Self>>;
        type PriceSetter: PriceSetter<Self>;
        type UnsignedInterval: Get<Self::BlockNumber>;
        /// A configuration for base priority of unsigned transactions.
        type UnsignedPriority: Get<TransactionPriority>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn next_unsigned_at)]
    pub(super) type NextUnsignedAt<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Price of currency is updated
        PriceUpdated(CurrencyId, PriceValue),
    }

    #[pallet::error]
    pub enum Error<T> {
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(block_number: T::BlockNumber) {
            let res = Self::fetch_price_and_send_raw_unsigned(block_number);
            if let Err(e) = res {
                log::error!("Error: {:?}", e);
            }
        }
    }

    #[pallet::call]
    impl<T:Config> Pallet<T> {
        #[pallet::weight(1)]
        pub fn submit_price_unsigned(origin: OriginFor<T>, block_number: T::BlockNumber, prices: Vec<u32>) -> DispatchResultWithPostInfo {
            ensure_none(origin)?;

            // update all the prices
            for (idx, price) in prices.iter().enumerate() {
                let currency_id = CurrencyId::from(idx as u8);
                let price = FixedU128::saturating_from_rational(*price, 100);
                T::PriceSetter::set_price_val(currency_id, price, block_number)?;
                Self::deposit_event(Event::PriceUpdated(currency_id, price));
            }

            // now increment the block number at which we expect next unsigned transaction.
            let current_block = <frame_system::Pallet<T>>::block_number();
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
                log::warn!("Unexpected status code: {}", response.code);
                return Err(http::Error::Unknown);
            }

            let body = response.body().collect::<Vec<u8>>();

            let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
                log::warn!("No UTF8 body");
                http::Error::Unknown
            })?;
            let prices = Self::parse_price(body_str);
            if prices.is_empty() {
                log::warn!("Unable to extract price from the response: {:?}", body_str);
                return Err(http::Error::Unknown);
            }
            log::warn!("Got price: {:?} cents", prices);

            Ok(prices)
        }

        /// Returns `None` when parsing failed or `Some(price in cents)` when parsing is successful.
        fn parse_price(price_str: &str) -> Vec<u32> {
            let components = price_str.split(",");
            let mut prices: Vec<u32> = Vec::new();
            for s in components {
                prices.push(s.parse().unwrap());
            }
            return prices;
        }

        fn validate_transaction_parameters(
            block_number: &T::BlockNumber,
            _new_price: &Vec<u32>,
        ) -> TransactionValidity {
            // Now let's check if the transaction has any chance to succeed.
            let next_unsigned_at = <NextUnsignedAt<T>>::get();
            if &next_unsigned_at > block_number {
                return InvalidTransaction::Stale.into();
            }
            // Let's make sure to reject transactions from the future.
            let current_block = <frame_system::Pallet<T>>::block_number();
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
