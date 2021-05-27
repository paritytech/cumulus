#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::dispatch::{DispatchResultWithPostInfo, DispatchResult};
use polkadot_parachain_primitives::{PriceValue, Price, CurrencyId};
use frame_system::Config;
use frame_support::pallet_prelude::{MaybeSerializeDeserialize};
use frame_support::sp_runtime::traits::AtLeast32BitUnsigned;
use sp_std::fmt::Debug;
use codec::{FullCodec};

/// A trait to provide the price for a currency
pub trait PriceProvider<T> where T: Config {
	type CurrencyId;
	fn price(currency_id: Self::CurrencyId) -> Price<T>;
}

/// A trait to set the price for a currency
pub trait PriceSetter<T> where T: Config {
	/// Set the price of the currency by the currency id
	fn set_price_val(currency_id: CurrencyId, price: PriceValue, block_number: T::BlockNumber) -> DispatchResultWithPostInfo;

	/// Set the price of the currency by the currency id
	fn set_price(currency_id: CurrencyId, price: Price<T>) -> DispatchResultWithPostInfo;
}

/// Abstraction over a fungible (single) currency system.
pub trait BasicCurrency<AccountId> {
	/// The balance of an account.
	type Balance: AtLeast32BitUnsigned + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;

	// Public immutables

	/// Existential deposit.
	fn minimum_balance() -> Self::Balance;

	/// The total amount of issuance.
	fn total_issuance() -> Self::Balance;

	/// The combined balance of `who`.
	fn total_balance(who: &AccountId) -> Self::Balance;

	/// The free balance of `who`.
	fn free_balance(who: &AccountId) -> Self::Balance;

	/// A dry-run of `withdraw`. Returns `Ok` iff the account is able to make a
    /// withdrawal of the given amount.
	fn ensure_can_withdraw(who: &AccountId, amount: Self::Balance) -> DispatchResult;

	// Public mutables

	/// Transfer some amount from one account to another.
	fn transfer(from: &AccountId, to: &AccountId, amount: Self::Balance) -> DispatchResult;

	/// Add `amount` to the balance of `who` and increase total issuance.
	fn deposit(who: &AccountId, amount: Self::Balance) -> DispatchResult;

	/// Remove `amount` from the balance of `who` and reduce total issuance.
	fn withdraw(who: &AccountId, amount: Self::Balance) -> DispatchResult;

	/// Same result as `slash(who, value)` (but without the side-effects)
    /// assuming there are no balance changes in the meantime and only the
    /// reserved balance is not taken into account.
	fn can_slash(who: &AccountId, value: Self::Balance) -> bool;

	/// Deduct the balance of `who` by up to `amount`.
    ///
    /// As much funds up to `amount` will be deducted as possible. If this is
    /// less than `amount`,then a non-zero value will be returned.
	fn slash(who: &AccountId, amount: Self::Balance) -> Self::Balance;
}

/// Abstraction over a fungible multi-currency system.
pub trait MultiCurrency<AccountId> {
	/// The currency identifier.
	type CurrencyId: FullCodec + Eq + PartialEq + Copy + MaybeSerializeDeserialize + Debug;

	/// The balance of an account.
	type Balance: AtLeast32BitUnsigned + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;

	// Public immutables

	/// Existential deposit of `currency_id`.
	fn minimum_balance(currency_id: Self::CurrencyId) -> Self::Balance;

	/// The total amount of issuance of `currency_id`.
	fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance;

	// The combined balance of `who` under `currency_id`.
	fn total_balance(currency_id: Self::CurrencyId, who: &AccountId) -> Self::Balance;

	// The free balance of `who` under `currency_id`.
	fn free_balance(currency_id: Self::CurrencyId, who: &AccountId) -> Self::Balance;

	/// A dry-run of `withdraw`. Returns `Ok` iff the account is able to make a
	/// withdrawal of the given amount.
	fn ensure_can_withdraw(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult;

	// Public mutables

	/// Transfer some amount from one account to another.
	fn transfer(
		currency_id: Self::CurrencyId,
		from: &AccountId,
		to: &AccountId,
		amount: Self::Balance,
	) -> DispatchResult;

	/// Add `amount` to the balance of `who` under `currency_id` and increase
	/// total issuance.
	fn deposit(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult;

	/// Remove `amount` from the balance of `who` under `currency_id` and reduce
	/// total issuance.
	fn withdraw(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult;

	/// Same result as `slash(currency_id, who, value)` (but without the
	/// side-effects) assuming there are no balance changes in the meantime and
	/// only the reserved balance is not taken into account.
	fn can_slash(currency_id: Self::CurrencyId, who: &AccountId, value: Self::Balance) -> bool;

	/// Deduct the balance of `who` by up to `amount`.
	///
	/// As much funds up to `amount` will be deducted as possible.  If this is
	/// less than `amount`,then a non-zero value will be returned.
	fn slash(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::Balance) -> Self::Balance;
}
