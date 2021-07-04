use codec::{Decode, Encode};
use frame_support::{traits::Get, BoundedVec, DefaultNoBound};
use sp_std::convert::TryInto;
#[cfg(feature = "std")]
use sp_std::{fmt, prelude::*};

/// An ordered set backed by `BoundedVec`
#[derive(PartialEq, Eq, Encode, Decode, DefaultNoBound, Clone)]
pub struct OrderedSet<T, S>(pub BoundedVec<T, S>);

impl<T: Ord, S: Get<u32>> OrderedSet<T, S> {
	/// Create a new empty set
	pub fn new() -> Self {
		Self(BoundedVec::default())
	}

	/// Create a set from a `Vec`.
	/// `v` will be sorted and dedup first.
	pub fn from(bv: BoundedVec<T, S>) -> Self {
		let mut v = bv.into_inner();
		v.sort();
		v.dedup();

		Self::from_sorted_set(v.try_into().expect("Did not add any values"))
	}

	/// Create a set from a `Vec`.
	/// Assume `v` is sorted and contain unique elements.
	pub fn from_sorted_set(bv: BoundedVec<T, S>) -> Self {
		Self(bv)
	}

	/// Insert an element.
	/// Return true if insertion happened.
	pub fn insert(&mut self, value: T) -> bool {
		match self.0.binary_search(&value) {
			Ok(_) => false,
			Err(loc) => self.0.try_insert(loc, value).is_ok(),
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
		self.0 = BoundedVec::default();
	}
}

impl<T: Ord, S: Get<u32>> From<BoundedVec<T, S>> for OrderedSet<T, S> {
	fn from(v: BoundedVec<T, S>) -> Self {
		Self::from(v)
	}
}

#[cfg(feature = "std")]
impl<T, S> fmt::Debug for OrderedSet<T, S>
where
	T: fmt::Debug,
	S: Get<u32>,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_tuple("OrderedSet").field(&self.0).finish()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::parameter_types;
	use sp_runtime::RuntimeDebug;

	parameter_types! {
		#[derive(PartialEq, RuntimeDebug)]
		pub const Eight: u32 = 8;
		#[derive(PartialEq, RuntimeDebug)]
		pub const Five: u32 = 5;
	}

	#[test]
	fn from() {
		let v: BoundedVec<i32, Eight> = vec![4, 2, 3, 4, 3, 1].try_into().unwrap();
		let set: OrderedSet<i32, Eight> = v.into();
		assert_eq!(
			set,
			OrderedSet::<i32, Eight>::from(vec![1, 2, 3, 4].try_into().unwrap())
		);
	}

	#[test]
	fn insert() {
		let mut set: OrderedSet<i32, Eight> = OrderedSet::new();
		assert_eq!(set, OrderedSet::<i32, Eight>::from(vec![].try_into().unwrap()));

		assert_eq!(set.insert(1), true);
		assert_eq!(set, OrderedSet::<i32, Eight>::from(vec![1].try_into().unwrap()));

		assert_eq!(set.insert(5), true);
		assert_eq!(set, OrderedSet::<i32, Eight>::from(vec![1, 5].try_into().unwrap()));

		assert_eq!(set.insert(3), true);
		assert_eq!(set, OrderedSet::<i32, Eight>::from(vec![1, 3, 5].try_into().unwrap()));

		assert_eq!(set.insert(3), false);
		assert_eq!(set, OrderedSet::<i32, Eight>::from(vec![1, 3, 5].try_into().unwrap()));
	}

	#[test]
	fn remove() {
		let mut set: OrderedSet<i32, Eight> = OrderedSet::from(vec![1, 2, 3, 4].try_into().unwrap());

		assert_eq!(set.remove(&5), false);
		assert_eq!(
			set,
			OrderedSet::<i32, Eight>::from(vec![1, 2, 3, 4].try_into().unwrap())
		);

		assert_eq!(set.remove(&1), true);
		assert_eq!(set, OrderedSet::<i32, Eight>::from(vec![2, 3, 4].try_into().unwrap()));

		assert_eq!(set.remove(&3), true);
		assert_eq!(set, OrderedSet::<i32, Eight>::from(vec![2, 4].try_into().unwrap()));

		assert_eq!(set.remove(&3), false);
		assert_eq!(set, OrderedSet::<i32, Eight>::from(vec![2, 4].try_into().unwrap()));

		assert_eq!(set.remove(&4), true);
		assert_eq!(set, OrderedSet::<i32, Eight>::from(vec![2].try_into().unwrap()));

		assert_eq!(set.remove(&2), true);
		assert_eq!(set, OrderedSet::<i32, Eight>::from(vec![].try_into().unwrap()));

		assert_eq!(set.remove(&2), false);
		assert_eq!(set, OrderedSet::<i32, Eight>::from(vec![].try_into().unwrap()));
	}

	#[test]
	fn contains() {
		let set: OrderedSet<i32, Eight> = OrderedSet::from(vec![1, 2, 3, 4].try_into().unwrap());

		assert_eq!(set.contains(&5), false);

		assert_eq!(set.contains(&1), true);

		assert_eq!(set.contains(&3), true);
	}

	#[test]
	fn clear() {
		let mut set: OrderedSet<i32, Eight> = OrderedSet::from(vec![1, 2, 3, 4].try_into().unwrap());
		set.clear();
		assert_eq!(set, OrderedSet::new());
	}

	#[test]
	fn exceeding_max_size_should_fail() {
		let mut set: OrderedSet<i32, Five> = OrderedSet::from(vec![1, 2, 3, 4, 5].try_into().unwrap());
		let inserted = set.insert(6);

		assert_eq!(inserted, false)
	}
}
