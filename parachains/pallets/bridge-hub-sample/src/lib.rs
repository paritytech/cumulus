#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::PalletError;
use scale_info::TypeInfo;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug)]
pub struct SampleData<BoundedString> {
	a: BoundedString,
	b: u32,
}

#[derive(Encode, Decode, PalletError, TypeInfo, Debug, PartialEq)]
pub struct InvalidParameterDetails {
	max: u8,
	actual: u8,
}

#[frame_support::pallet]
pub mod pallet {
	use super::SampleData;
	use crate::InvalidParameterDetails;
	use frame_support::{
		dispatch::{DispatchErrorWithPostInfo, PostDispatchInfo, Weight},
		ensure, log,
		pallet_prelude::{DispatchResultWithPostInfo, Get, Hooks, IsType, StorageValue},
		sp_std,
		weights::Pays,
		BoundedVec,
	};
	use frame_system::{
		ensure_signed,
		pallet_prelude::{BlockNumberFor, OriginFor},
	};

	// Declare the pallet type
	// This is a placeholder to implement traits and methods.
	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(sp_std::marker::PhantomData<T>);

	// Add the runtime configuration trait
	// All types and constants go here.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		#[pallet::constant]
		type StringMaxLength: Get<u32>;

		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SetNameCalled(u32),
		CallsPerBlock(u32),
	}

	#[pallet::error]
	#[derive(PartialEq)]
	pub enum Error<T> {
		InvalidParameter(InvalidParameterDetails),
	}

	////////////////////////////////
	// Custom storage definitions //
	////////////////////////////////

	// Add runtime storage to declare storage items.
	#[pallet::storage]
	#[pallet::getter(fn get_counter_per_block)]
	pub type CounterPerBlockStorage<T: Config> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn get_total_counter)]
	pub type TotalCounterStorage<T: Config> = StorageValue<_, u32>;

	// Add runtime storage to declare storage items.
	#[pallet::storage]
	#[pallet::getter(fn get_name)]
	pub type NameStorage<T: Config> = StorageValue<_, BoundedVec<u8, T::StringMaxLength>>;

	#[pallet::storage]
	#[pallet::getter(fn get_sample_data)]
	pub type SampleDataStorage<T: Config> =
		StorageValue<_, SampleData<BoundedVec<u8, T::StringMaxLength>>>;

	//////////////////
	// Custom hooks //
	//////////////////

	//  Add hooks to define some logic that should be executed
	//  in a specific context, for example on_initialize.
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: BlockNumberFor<T>) {
			let weight = T::DbWeight::get();
			log::info!("on_finalize: {:?}, rw: {}, ww: {}", n, weight.read, weight.write);

			if let Some(new) = Self::recalculate_total_count() {
				Self::deposit_event(Event::CallsPerBlock(new));
			}
		}

		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			let weight = T::DbWeight::get();
			log::info!("on_initialize: {:?}, rw: {}, ww: {}", n, weight.read, weight.write);

			Self::reset_counter_per_block()
		}

		fn on_runtime_upgrade() -> Weight {
			let weight = T::DbWeight::get();
			log::info!("on_runtime_upgrade, rw: {}, ww: {}", weight.read, weight.write);
			0
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight((500_000, Pays::Yes))]
		pub fn set_name(
			origin: OriginFor<T>,
			name: BoundedVec<u8, T::StringMaxLength>,
		) -> DispatchResultWithPostInfo {
			use frame_support::traits::OriginTrait;
			log::info!("set_name called: origin.caller: {:?}, name: {:?}", origin.caller(), name);
			let _ = ensure_signed(origin)?;

			let name_len = name.len();

			// handle error
			ensure!(
				name_len < 10,
				DispatchErrorWithPostInfo {
					post_info: PostDispatchInfo {
						actual_weight: Some(5_000_000),
						pays_fee: Pays::Yes
					},
					error: Error::<T>::InvalidParameter(InvalidParameterDetails {
						max: 9,
						actual: name_len as u8
					})
					.into()
				}
			);

			if name_len > 8 {
				return Err(DispatchErrorWithPostInfo {
					post_info: PostDispatchInfo { actual_weight: None, pays_fee: Pays::No },
					error: Error::<T>::InvalidParameter(InvalidParameterDetails {
						max: 8,
						actual: name_len as u8,
					})
					.into(),
				})
			}

			// Store to storage
			let _ = NameStorage::<T>::put(name);
			let actual_count = Self::increment_counter_per_block();

			// Emit event
			Self::deposit_event(Event::SetNameCalled(actual_count));

			// Override weights
			if name_len < 5 {
				Ok(Pays::No.into())
			} else {
				Ok((Some(400_000), Pays::Yes).into())
			}
		}
	}

	impl<T: Config> Pallet<T> {
		fn increment_counter_per_block() -> u32 {
			let value = CounterPerBlockStorage::<T>::get()
				.map(|value| value.saturating_add(1))
				.unwrap_or(1);
			CounterPerBlockStorage::<T>::put(value);
			value
		}

		fn reset_counter_per_block() -> Weight {
			CounterPerBlockStorage::<T>::put(0);
			T::DbWeight::get().writes(1)
		}

		// Return diff, if total count is changed
		fn recalculate_total_count() -> Option<u32> {
			match Self::get_counter_per_block() {
				Some(count) if count > 0 =>
					TotalCounterStorage::<T>::mutate(|total_counter| match total_counter {
						Some(total_counter) => {
							*total_counter = total_counter.saturating_add(count);
							Some(count)
						},
						None => {
							*total_counter = Some(count);
							Some(count)
						},
					}),
				Some(_) => None,
				None => None,
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		pallet,
		pallet::{CounterPerBlockStorage, SampleDataStorage},
		InvalidParameterDetails,
	};
	use codec::Decode;
	use frame_support::{
		pallet_prelude::{ConstU32, Get},
		sp_io::TestExternalities,
		sp_runtime,
		storage::{generator::StorageValue, unhashed},
		StorageHasher, Twox128,
	};

	fn to_key(keys: Vec<&'static str>) -> String {
		hex::encode(keys.iter().map(|k| Twox128::hash(k.as_bytes())).collect::<Vec<_>>().concat())
	}

	#[test]
	fn test_keys() {
		println!("0x{}", to_key(vec!["BridgeHubSample", "CounterStorage"]));
		println!("0x{}", to_key(vec!["BridgeHubSample", "NameStorage"]));
		println!("0x{}", to_key(vec!["BridgeHubSample", "SampleDataStorage"]));
	}

	#[test]
	fn test_decoded_data() {
		let data = decode_storage_value(
			"0x1c000000",
			|| CounterPerBlockStorage::<Runtime>::storage_value_final_key(),
			|| CounterPerBlockStorage::<Runtime>::get(),
		);
		println!("counter: {:?}", data);

		let data = decode_storage_value(
			"0x0c6162633f000000",
			|| SampleDataStorage::<Runtime>::storage_value_final_key(),
			|| SampleDataStorage::<Runtime>::get(),
		);
		println!("sample_data: {:?}", data);
	}

	fn decode_storage_value<D>(
		hex_string_from_polkadot_js: &str,
		final_key: fn() -> [u8; 32],
		get: fn() -> D,
	) -> D {
		let raw_value = hex::decode(hex_string_from_polkadot_js.replace("0x", "")).expect("error");

		TestExternalities::default().execute_with(|| {
			// insert raw data
			let key = final_key();
			unhashed::put_raw(&key, &raw_value);

			// get decoded data back
			get()
		})
	}

	#[test]
	fn test_decode_error() {
		let decode_error = |hex_string_from_polkadot_js: &str| -> pallet::Error<Runtime> {
			let data = hex::decode(hex_string_from_polkadot_js.replace("0x", "")).expect("error");
			Decode::decode(&mut data.as_slice()).expect("error")
		};

		let dump_error = |error: &pallet::Error<Runtime>| match error {
			pallet::Error::<Runtime>::InvalidParameter(details) =>
				println!("InvalidParameter -> {:?}", details),
			pallet::Error::<Runtime>::__Ignore(a, b) => println!("__Ignore({:?}, {:?})", a, b),
		};

		let error = decode_error("0x00090b00");
		dump_error(&error);
		assert_eq!(
			pallet::Error::<Runtime>::InvalidParameter(InvalidParameterDetails {
				max: 9,
				actual: 11
			}),
			error
		);

		let error = decode_error("0x00080900");
		dump_error(&error);
		assert_eq!(
			pallet::Error::<Runtime>::InvalidParameter(InvalidParameterDetails {
				max: 8,
				actual: 9
			}),
			error
		);
	}

	pub struct StringMaxLength;
	impl Get<u32> for StringMaxLength {
		fn get() -> u32 {
			50
		}
	}

	impl pallet::Config for Runtime {
		type StringMaxLength = StringMaxLength;
		type Event = Event;
	}

	impl frame_system::Config for Runtime {
		type BaseCallFilter = frame_support::traits::Everything;
		type BlockWeights = ();
		type BlockLength = ();
		type Origin = Origin;
		type Call = Call;
		type Index = u64;
		type BlockNumber = u32;
		type Hash = sp_runtime::testing::H256;
		type Hashing = sp_runtime::traits::BlakeTwo256;
		type AccountId = u64;
		type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = Event;
		type BlockHashCount = ConstU32<250>;
		type DbWeight = ();
		type Version = ();
		type PalletInfo = PalletInfo;
		type AccountData = ();
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
		type SS58Prefix = ();
		type OnSetCode = ();
		type MaxConsumers = ConstU32<16>;
	}

	pub type Header = sp_runtime::generic::Header<u32, sp_runtime::traits::BlakeTwo256>;
	pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
	pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<u32, Call, (), ()>;

	frame_support::construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
		{
			// Exclude part `Storage` in order not to check its metadata in tests.
			System: frame_system exclude_parts { Storage },
			Example: pallet,
		}
	);
}
