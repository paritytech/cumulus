use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

/// An ordered set backed by `Vec`
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(RuntimeDebug, PartialEq, Eq, Encode, Decode, Default, Clone)]
pub struct OrderedSet<T>(pub Vec<T>);

impl<T: Ord> OrderedSet<T> {
	/// Create a new empty set
	pub fn new() -> Self {
		Self(Vec::new())
	}

	/// Create a set from a `Vec`.
	/// `v` will be sorted and dedup first.
	pub fn from(mut v: Vec<T>) -> Self {
		v.sort();
		v.dedup();
		Self::from_sorted_set(v)
	}

	/// Create a set from a `Vec`.
	/// Assume `v` is sorted and contain unique elements.
	pub fn from_sorted_set(v: Vec<T>) -> Self {
		Self(v)
	}

	/// Insert an element.
	/// Return true if insertion happened.
	pub fn insert(&mut self, value: T) -> bool {
		match self.0.binary_search(&value) {
			Ok(_) => false,
			Err(loc) => {
				self.0.insert(loc, value);
				true
			}
		}
	}

	/// Remove an element.
	/// Return true if removal happened.
	pub fn remove(&mut self, value: &T) -> bool {
		match self.0.binary_search(&value) {
			Ok(loc) => {
				self.0.remove(loc);
				true
			}
			Err(_) => false,
		}
	}

	/// Return if the set contains `value`
	pub fn contains(&self, value: &T) -> bool {
		self.0.binary_search(&value).is_ok()
	}

	/// Clear the set
	pub fn clear(&mut self) {
		self.0.clear();
	}
}

impl<T: Ord> From<Vec<T>> for OrderedSet<T> {
	fn from(v: Vec<T>) -> Self {
		Self::from(v)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn from() {
		let v = vec![4, 2, 3, 4, 3, 1];
		let set: OrderedSet<i32> = v.into();
		assert_eq!(set, OrderedSet::from(vec![1, 2, 3, 4]));
	}

	#[test]
	fn insert() {
		let mut set: OrderedSet<i32> = OrderedSet::new();
		assert_eq!(set, OrderedSet::from(vec![]));

		assert_eq!(set.insert(1), true);
		assert_eq!(set, OrderedSet::from(vec![1]));

		assert_eq!(set.insert(5), true);
		assert_eq!(set, OrderedSet::from(vec![1, 5]));

		assert_eq!(set.insert(3), true);
		assert_eq!(set, OrderedSet::from(vec![1, 3, 5]));

		assert_eq!(set.insert(3), false);
		assert_eq!(set, OrderedSet::from(vec![1, 3, 5]));
	}

	#[test]
	fn remove() {
		let mut set: OrderedSet<i32> = OrderedSet::from(vec![1, 2, 3, 4]);

		assert_eq!(set.remove(&5), false);
		assert_eq!(set, OrderedSet::from(vec![1, 2, 3, 4]));

		assert_eq!(set.remove(&1), true);
		assert_eq!(set, OrderedSet::from(vec![2, 3, 4]));

		assert_eq!(set.remove(&3), true);
		assert_eq!(set, OrderedSet::from(vec![2, 4]));

		assert_eq!(set.remove(&3), false);
		assert_eq!(set, OrderedSet::from(vec![2, 4]));

		assert_eq!(set.remove(&4), true);
		assert_eq!(set, OrderedSet::from(vec![2]));

		assert_eq!(set.remove(&2), true);
		assert_eq!(set, OrderedSet::from(vec![]));

		assert_eq!(set.remove(&2), false);
		assert_eq!(set, OrderedSet::from(vec![]));
	}

	#[test]
	fn contains() {
		let set: OrderedSet<i32> = OrderedSet::from(vec![1, 2, 3, 4]);

		assert_eq!(set.contains(&5), false);

		assert_eq!(set.contains(&1), true);

		assert_eq!(set.contains(&3), true);
	}

	#[test]
	fn clear() {
		let mut set: OrderedSet<i32> = OrderedSet::from(vec![1, 2, 3, 4]);
		set.clear();
		assert_eq!(set, OrderedSet::new());
	}
}
