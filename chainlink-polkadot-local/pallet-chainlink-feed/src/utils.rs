use frame_support::storage::{with_transaction, TransactionOutcome};
use sp_arithmetic::traits::BaseArithmetic;

/// Execute the supplied function in a new storage transaction.
///
/// All changes to storage performed by the supplied function are discarded if
/// the returned outcome is `Result::Err`.
///
/// Transactions can be nested to any depth. Commits happen to the parent
/// transaction.
// TODO: remove after move to Substrate v3 (once the semantics of #[transactional] work as intended)
pub(crate) fn with_transaction_result<R, E>(f: impl FnOnce() -> Result<R, E>) -> Result<R, E> {
	with_transaction(|| {
		let res = f();
		if res.is_ok() {
			TransactionOutcome::Commit(res)
		} else {
			TransactionOutcome::Rollback(res)
		}
	})
}

/// Determine the median of a slice of values.
///
/// **Warning:** Will panic if passed an empty slice.
pub(crate) fn median<T: Copy + BaseArithmetic>(numbers: &mut [T]) -> T {
	numbers.sort_unstable();

	let mid = numbers.len() / 2;
	if numbers.len() % 2 == 0 {
		numbers[mid - 1].saturating_add(numbers[mid]) / 2.into()
	} else {
		numbers[mid]
	}
}


#[test]
fn median_works() {
	let mut values = vec![4u32, 6, 2, 7];
	assert_eq!(median(&mut values), 5);
	let mut values = vec![4u32, 6, 2, 7, 9];
	assert_eq!(median(&mut values), 6);
}

#[test]
#[should_panic]
fn median_panics_on_empty_slice() {
	let mut empty: Vec<u32> = Vec::new();
	median(&mut empty);
}
