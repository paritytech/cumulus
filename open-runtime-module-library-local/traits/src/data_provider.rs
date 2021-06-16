use sp_runtime::DispatchResult;
use sp_std::vec::Vec;

/// Data provider with ability to provide data with no-op, and provide all data.
pub trait DataFeeder<Key, Value, AccountId>: DataProvider<Key, Value> {
	/// Provide a new value for a given key from an operator
	fn feed_value(who: AccountId, key: Key, value: Value) -> DispatchResult;
}

/// A simple trait to provide data
pub trait DataProvider<Key, Value> {
	/// Get data by key
	fn get(key: &Key) -> Option<Value>;
}

/// Extended data provider to provide timestamped data by key with no-op, and
/// all data.
pub trait DataProviderExtended<Key, TimestampedValue> {
	/// Get timestamped value by key
	fn get_no_op(key: &Key) -> Option<TimestampedValue>;
	/// Provide a list of tuples of key and timestamped value
	fn get_all_values() -> Vec<(Key, Option<TimestampedValue>)>;
}

#[allow(dead_code)] // rust cannot detect usage in macro_rules
pub fn median<T: Ord + Clone>(mut items: Vec<T>) -> Option<T> {
	if items.is_empty() {
		return None;
	}

	let mid_index = items.len() / 2;

	// Won't panic as `items` ensured not empty.
	let (_, item, _) = items.select_nth_unstable(mid_index);
	Some(item.clone())
}

#[macro_export]
macro_rules! create_median_value_data_provider {
	($name:ident, $key:ty, $value:ty, $timestamped_value:ty, [$( $provider:ty ),*]) => {
		pub struct $name;
		impl $crate::DataProvider<$key, $value> for $name {
			fn get(key: &$key) -> Option<$value> {
				let mut values = vec![];
				$(
					if let Some(v) = <$provider as $crate::DataProvider<$key, $value>>::get(&key) {
						values.push(v);
					}
				)*
				$crate::data_provider::median(values)
			}
		}
		impl $crate::DataProviderExtended<$key, $timestamped_value> for $name {
			fn get_no_op(key: &$key) -> Option<$timestamped_value> {
				let mut values = vec![];
				$(
					if let Some(v) = <$provider as $crate::DataProviderExtended<$key, $timestamped_value>>::get_no_op(&key) {
						values.push(v);
					}
				)*
				$crate::data_provider::median(values)
			}
			fn get_all_values() -> Vec<($key, Option<$timestamped_value>)> {
				let mut keys = sp_std::collections::btree_set::BTreeSet::new();
				$(
					<$provider as $crate::DataProviderExtended<$key, $timestamped_value>>::get_all_values()
						.into_iter()
						.for_each(|(k, _)| { keys.insert(k); });
				)*
				keys.into_iter().map(|k| (k, Self::get_no_op(&k))).collect()
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use sp_std::cell::RefCell;

	thread_local! {
		static MOCK_PRICE_1: RefCell<Option<u8>> = RefCell::new(None);
		static MOCK_PRICE_2: RefCell<Option<u8>> = RefCell::new(None);
		static MOCK_PRICE_3: RefCell<Option<u8>> = RefCell::new(None);
		static MOCK_PRICE_4: RefCell<Option<u8>> = RefCell::new(None);
	}

	macro_rules! mock_data_provider {
		($provider:ident, $price:ident) => {
			pub struct $provider;
			impl $provider {
				fn set_price(price: Option<u8>) {
					$price.with(|v| *v.borrow_mut() = price)
				}
			}
			impl DataProvider<u8, u8> for $provider {
				fn get(_: &u8) -> Option<u8> {
					$price.with(|v| *v.borrow())
				}
			}
			impl DataProviderExtended<u8, u8> for $provider {
				fn get_no_op(_: &u8) -> Option<u8> {
					$price.with(|v| *v.borrow())
				}
				fn get_all_values() -> Vec<(u8, Option<u8>)> {
					vec![(0, Self::get_no_op(&0))]
				}
			}
		};
	}

	mock_data_provider!(Provider1, MOCK_PRICE_1);
	mock_data_provider!(Provider2, MOCK_PRICE_2);
	mock_data_provider!(Provider3, MOCK_PRICE_3);
	mock_data_provider!(Provider4, MOCK_PRICE_4);

	create_median_value_data_provider!(Providers, u8, u8, u8, [Provider1, Provider2, Provider3, Provider4]);

	#[test]
	fn median_value_data_provider_works() {
		assert_eq!(<Providers as DataProvider<_, _>>::get(&0), None);

		let data = vec![
			(vec![None, None, None, Some(1)], Some(1)),
			(vec![None, None, Some(2), Some(1)], Some(2)),
			(vec![Some(5), Some(2), None, Some(7)], Some(5)),
			(vec![Some(5), Some(13), Some(2), Some(7)], Some(7)),
		];

		for (values, target) in data {
			Provider1::set_price(values[0]);
			Provider2::set_price(values[1]);
			Provider3::set_price(values[2]);
			Provider4::set_price(values[3]);

			assert_eq!(<Providers as DataProvider<_, _>>::get(&0), target);
		}
	}
}
