use codec::{Decode, EncodeLike, FullCodec, FullEncode};
use frame_support::{
	storage::{
		generator::{StorageDoubleMap as StorageDoubleMapT, StorageMap as StorageMapT},
		unhashed, StorageDoubleMap, StorageMap,
	},
	ReversibleStorageHasher,
};
use sp_std::prelude::*;

/// Utility to iterate through items in a storage map.
/// Forks from substrate, expose previous_key field
pub struct StorageMapIterator<K, V, Hasher> {
	prefix: Vec<u8>,
	pub previous_key: Vec<u8>,
	drain: bool,
	_phantom: sp_std::marker::PhantomData<(K, V, Hasher)>,
}

/// Forks from substrate
impl<K: Decode + Sized, V: Decode + Sized, Hasher: ReversibleStorageHasher> Iterator
	for StorageMapIterator<K, V, Hasher>
{
	type Item = (K, V);

	fn next(&mut self) -> Option<(K, V)> {
		loop {
			let maybe_next = sp_io::storage::next_key(&self.previous_key).filter(|n| n.starts_with(&self.prefix));
			break match maybe_next {
				Some(next) => {
					self.previous_key = next;
					match unhashed::get::<V>(&self.previous_key) {
						Some(value) => {
							if self.drain {
								unhashed::kill(&self.previous_key)
							}
							let mut key_material = Hasher::reverse(&self.previous_key[self.prefix.len()..]);
							match K::decode(&mut key_material) {
								Ok(key) => Some((key, value)),
								Err(_) => continue,
							}
						}
						None => continue,
					}
				}
				None => None,
			};
		}
	}
}

/// Shim for StorageMapIterator, add more features
pub struct StorageMapIteratorShim<K, V, H> {
	pub storage_map_iterator: StorageMapIterator<K, V, H>,
	pub remain_iterator_count: Option<u32>,
	pub finished: bool,
}

impl<K: Decode + Sized, V: Decode + Sized, H: ReversibleStorageHasher> Iterator for StorageMapIteratorShim<K, V, H> {
	type Item = <StorageMapIterator<K, V, H> as Iterator>::Item;

	fn next(&mut self) -> Option<Self::Item> {
		// check accumulated iteration count
		if let Some(remain_iterator_count) = self.remain_iterator_count {
			if remain_iterator_count == 0 {
				return None;
			} else {
				self.remain_iterator_count = Some(remain_iterator_count - 1);
			}
		}

		self.storage_map_iterator.next().or_else(|| {
			// mark this map iterator has already finished
			self.finished = true;
			None
		})
	}
}

/// A strongly-typed map in storage whose keys and values can be iterated over.
pub trait IterableStorageMapExtended<K: FullEncode, V: FullCodec>: StorageMap<K, V> {
	/// The type that iterates over all `(key, value)`.
	type Iterator: Iterator<Item = (K, V)>;

	/// Enumerate all elements in the map in no particular order. If you alter
	/// the map while doing this, you'll get undefined results.
	fn iter(max_iterations: Option<u32>, start_key: Option<Vec<u8>>) -> Self::Iterator;

	/// Remove all elements from the map and iterate through them in no
	/// particular order. If you add elements to the map while doing this,
	/// you'll get undefined results.
	fn drain(max_iterations: Option<u32>, start_key: Option<Vec<u8>>) -> Self::Iterator;
}

impl<K: FullCodec, V: FullCodec, G: StorageMapT<K, V>> IterableStorageMapExtended<K, V> for G
where
	G::Hasher: ReversibleStorageHasher,
{
	type Iterator = StorageMapIteratorShim<K, V, G::Hasher>;

	/// Enumerate all elements in the map.
	fn iter(max_iterations: Option<u32>, start_key: Option<Vec<u8>>) -> Self::Iterator {
		let prefix = G::prefix_hash();
		let previous_key = start_key
			.filter(|k| k.starts_with(&prefix))
			.unwrap_or_else(|| prefix.clone());
		let storage_map_iterator = StorageMapIterator {
			prefix,
			previous_key,
			drain: false,
			_phantom: Default::default(),
		};

		StorageMapIteratorShim {
			storage_map_iterator,
			remain_iterator_count: max_iterations,
			finished: false,
		}
	}

	/// Enumerate all elements in the map.
	fn drain(max_iterations: Option<u32>, start_key: Option<Vec<u8>>) -> Self::Iterator {
		let prefix = G::prefix_hash();

		let previous_key = start_key
			.filter(|k| k.starts_with(&prefix))
			.unwrap_or_else(|| prefix.clone());
		let storage_map_iterator = StorageMapIterator {
			prefix,
			previous_key,
			drain: true,
			_phantom: Default::default(),
		};

		StorageMapIteratorShim {
			storage_map_iterator,
			remain_iterator_count: max_iterations,
			finished: false,
		}
	}
}

/// Iterate over a prefix and decode raw_key and raw_value into `T`.
/// Forks from substrate, expose previous_key field
pub struct MapIterator<T> {
	prefix: Vec<u8>,
	pub previous_key: Vec<u8>,
	/// If true then value are removed while iterating
	drain: bool,
	/// Function that take `(raw_key_without_prefix, raw_value)` and decode `T`.
	/// `raw_key_without_prefix` is the raw storage key without the prefix
	/// iterated on.
	closure: fn(&[u8], &[u8]) -> Result<T, codec::Error>,
}

/// Forks from substrate
impl<T> Iterator for MapIterator<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			let maybe_next = sp_io::storage::next_key(&self.previous_key).filter(|n| n.starts_with(&self.prefix));
			break match maybe_next {
				Some(next) => {
					self.previous_key = next;
					let raw_value = match unhashed::get_raw(&self.previous_key) {
						Some(raw_value) => raw_value,
						None => {
							frame_support::print("ERROR: next_key returned a key with no value in MapIterator");
							continue;
						}
					};
					if self.drain {
						unhashed::kill(&self.previous_key)
					}
					let raw_key_without_prefix = &self.previous_key[self.prefix.len()..];
					let item = match (self.closure)(raw_key_without_prefix, &raw_value[..]) {
						Ok(item) => item,
						Err(_e) => {
							frame_support::print("ERROR: (key, value) failed to decode in MapIterator");
							continue;
						}
					};

					Some(item)
				}
				None => None,
			};
		}
	}
}

/// Shim for MapIterator, add more features
pub struct MapIteratorShim<T> {
	pub map_iterator: MapIterator<T>,
	pub remain_iterator_count: Option<u32>,
	pub finished: bool,
}

impl<T> Iterator for MapIteratorShim<T> {
	type Item = <MapIterator<T> as Iterator>::Item;

	fn next(&mut self) -> Option<Self::Item> {
		// check accumulated iteration count
		if let Some(remain_iterator_count) = self.remain_iterator_count {
			if remain_iterator_count == 0 {
				return None;
			} else {
				self.remain_iterator_count = Some(remain_iterator_count - 1);
			}
		}

		self.map_iterator.next().or_else(|| {
			// mark this map iterator has already finished
			self.finished = true;
			None
		})
	}
}

/// A strongly-typed map in storage whose keys and values can be iterated over.
pub trait IterableStorageDoubleMapExtended<K1: FullCodec, K2: FullCodec, V: FullCodec>:
	StorageDoubleMap<K1, K2, V>
{
	/// The type that iterates over all `(key2, value)`.
	type PrefixIterator: Iterator<Item = (K2, V)>;

	/// The type that iterates over all `(key1, key2, value)`.
	type Iterator: Iterator<Item = (K1, K2, V)>;

	/// Enumerate all elements in the map with first key `k1` in no particular
	/// order. If you add or remove values whose first key is `k1` to the map
	/// while doing this, you'll get undefined results.
	fn iter_prefix(
		k1: impl EncodeLike<K1>,
		max_iterations: Option<u32>,
		start_key: Option<Vec<u8>>,
	) -> Self::PrefixIterator;

	/// Remove all elements from the map with first key `k1` and iterate through
	/// them in no particular order. If you add elements with first key `k1` to
	/// the map while doing this, you'll get undefined results.
	fn drain_prefix(
		k1: impl EncodeLike<K1>,
		max_iterations: Option<u32>,
		start_key: Option<Vec<u8>>,
	) -> Self::PrefixIterator;

	/// Enumerate all elements in the map in no particular order. If you add or
	/// remove values to the map while doing this, you'll get undefined results.
	fn iter(max_iterations: Option<u32>, start_key: Option<Vec<u8>>) -> Self::Iterator;

	/// Remove all elements from the map and iterate through them in no
	/// particular order. If you add elements to the map while doing this,
	/// you'll get undefined results.
	fn drain(max_iterations: Option<u32>, start_key: Option<Vec<u8>>) -> Self::Iterator;
}

impl<K1: FullCodec, K2: FullCodec, V: FullCodec, G: StorageDoubleMapT<K1, K2, V>>
	IterableStorageDoubleMapExtended<K1, K2, V> for G
where
	G::Hasher1: ReversibleStorageHasher,
	G::Hasher2: ReversibleStorageHasher,
{
	type PrefixIterator = MapIteratorShim<(K2, V)>;
	type Iterator = MapIteratorShim<(K1, K2, V)>;

	fn iter_prefix(
		k1: impl EncodeLike<K1>,
		max_iterations: Option<u32>,
		start_key: Option<Vec<u8>>,
	) -> Self::PrefixIterator {
		let prefix = G::storage_double_map_final_key1(k1);
		let previous_key = start_key
			.filter(|k| k.starts_with(&prefix))
			.unwrap_or_else(|| prefix.clone());

		let map_iterator = MapIterator {
			prefix,
			previous_key,
			drain: false,
			closure: |raw_key_without_prefix, mut raw_value| {
				let mut key_material = G::Hasher2::reverse(raw_key_without_prefix);
				Ok((K2::decode(&mut key_material)?, V::decode(&mut raw_value)?))
			},
		};

		MapIteratorShim {
			map_iterator,
			remain_iterator_count: max_iterations,
			finished: false,
		}
	}

	fn drain_prefix(
		k1: impl EncodeLike<K1>,
		max_iterations: Option<u32>,
		start_key: Option<Vec<u8>>,
	) -> Self::PrefixIterator {
		let mut shim = Self::iter_prefix(k1, max_iterations, start_key);
		shim.map_iterator.drain = true;
		shim
	}

	fn iter(max_iterations: Option<u32>, start_key: Option<Vec<u8>>) -> Self::Iterator {
		let prefix = G::prefix_hash();
		let previous_key = start_key
			.filter(|k| k.starts_with(&prefix))
			.unwrap_or_else(|| prefix.clone());

		let map_iterator = MapIterator {
			prefix,
			previous_key,
			drain: false,
			closure: |raw_key_without_prefix, mut raw_value| {
				let mut k1_k2_material = G::Hasher1::reverse(raw_key_without_prefix);
				let k1 = K1::decode(&mut k1_k2_material)?;
				let mut k2_material = G::Hasher2::reverse(k1_k2_material);
				let k2 = K2::decode(&mut k2_material)?;
				Ok((k1, k2, V::decode(&mut raw_value)?))
			},
		};

		MapIteratorShim {
			map_iterator,
			remain_iterator_count: max_iterations,
			finished: false,
		}
	}

	fn drain(max_iterations: Option<u32>, start_key: Option<Vec<u8>>) -> Self::Iterator {
		let mut shim = Self::iter(max_iterations, start_key);
		shim.map_iterator.drain = true;
		shim
	}
}
