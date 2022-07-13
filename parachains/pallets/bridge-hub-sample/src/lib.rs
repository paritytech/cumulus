#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::BoundedVec;
use scale_info::TypeInfo;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug)]
pub struct SampleData<BoundedString> {
	a: BoundedString,
	b: u32,
}

#[frame_support::pallet]
pub mod pallet {
	use super::SampleData;
	use crate::BoundedVec;
	use frame_support::{
		dispatch::Weight,
		log,
		pallet_prelude::{Get, Hooks, StorageValue},
		sp_runtime, sp_std,
	};
	use frame_system::pallet_prelude::BlockNumberFor;

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
	}

	////////////////////////////////
	// Custom storage definitions //
	////////////////////////////////

	// Add runtime storage to declare storage items.
	#[pallet::storage]
	#[pallet::getter(fn get_counter)]
	pub type CounterStorage<T: Config> = StorageValue<_, u32>;

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
			sp_runtime::print("on_finalize");
			log::info!("on_finalize: {:?}, rw: {}, ww: {}", n, weight.read, weight.write);
		}

		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			let weight = T::DbWeight::get();
			sp_runtime::print("on_initialize");
			log::info!("on_initialize: {:?}, rw: {}, ww: {}", n, weight.read, weight.write);

			let counter = CounterStorage::<T>::get();
			log::info!("on_initialize actual counter: {:?}", counter);
			match counter {
				Some(counter) => CounterStorage::<T>::put(counter.saturating_add(1)),
				None => CounterStorage::<T>::put(0),
			}

			let sample_data = SampleDataStorage::<T>::get();
			log::info!("on_initialize actual name: {:?}", sample_data);
			match sample_data {
				Some(mut sample_data) => {
					sample_data.b = sample_data.b.saturating_add(2);
					SampleDataStorage::<T>::put(sample_data)
				},
				None => {
					let sample_data = SampleData {
						a: BoundedVec::try_from("abc".as_bytes().to_vec())
							.unwrap_or(BoundedVec::default()),
						b: 5,
					};
					SampleDataStorage::<T>::put(sample_data)
				},
			}

			weight.reads_writes(1, 1)
		}

		fn on_runtime_upgrade() -> Weight {
			let weight = T::DbWeight::get();
			sp_runtime::print("on_runtime_upgrade");
			log::info!("on_runtime_upgrade, rw: {}, ww: {}", weight.read, weight.write);
			0
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		pallet,
		pallet::{CounterStorage, SampleDataStorage},
	};
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
	fn keys() {
		println!("0x{}", to_key(vec!["BridgeHubSample", "CounterStorage"]));
		println!("0x{}", to_key(vec!["BridgeHubSample", "NameStorage"]));
		println!("0x{}", to_key(vec!["BridgeHubSample", "SampleDataStorage"]));
	}

	#[test]
	fn test_decoded_data() {
		let data = decode_storage_value(
			"0x1c000000",
			|| CounterStorage::<Runtime>::storage_value_final_key(),
			|| CounterStorage::<Runtime>::get(),
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

	pub struct StringMaxLength;
	impl Get<u32> for StringMaxLength {
		fn get() -> u32 {
			50
		}
	}

	impl pallet::Config for Runtime {
		type StringMaxLength = StringMaxLength;
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
