#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::dispatch::{DispatchResultWithPostInfo, DispatchResult};
use polkadot_parachain_primitives::{PriceValue, Price, CurrencyId};
use frame_system::Config;
use frame_support::pallet_prelude::{MaybeSerializeDeserialize, Member};
use frame_support::Parameter;
use frame_support::sp_runtime::traits::AtLeast32BitUnsigned;
use frame_support::sp_runtime::FixedPointOperand;

/// A trait to provide the price for a currency
pub trait PriceProvider<T> where T: Config {
	fn price(currency_id: CurrencyId) -> Price<T>;
}

/// A trait to set the price for a currency
pub trait PriceSetter<T> where T: Config {
	/// Set the price of the currency by the currency id
	fn set_price_val(currency_id: CurrencyId, price: PriceValue, block_number: T::BlockNumber) -> DispatchResultWithPostInfo;

	/// Set the price of the currency by the currency id
	fn set_price(currency_id: CurrencyId, price: Price<T>) -> DispatchResultWithPostInfo;
}

/// Abstraction over a fungible multi-currency system.
pub trait MultiCurrency<AccountId> {
	type Balance: Parameter + Member + FixedPointOperand + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;

	// Public immutables
	/// Existential deposit of `currency_id`.
	fn minimum_balance(currency_id: CurrencyId) -> Self::Balance;

	/// Total issuance of the `currency_id`.
	fn total_issuance(currency_id: CurrencyId) -> Self::Balance;

	/// The combined balance of `who` under `currency_id`.
	fn total_balance(currency_id: CurrencyId, who: &AccountId) -> Self::Balance;

	/// The free balance of `who` under `currency_id`.
	fn free_balance(currency_id: CurrencyId, who: &AccountId) -> Self::Balance;

	/// A dry-run of `withdraw`. Returns `Ok` iff the account is able to make a
	/// withdrawal of the given amount.
	fn ensure_can_withdraw(currency_id: CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult;

	// Public mutables
	/// Transfer some amount from one account to another.
	fn transfer(
		currency_id: CurrencyId,
		from: &AccountId,
		to: &AccountId,
		amount: Self::Balance,
	) -> DispatchResult;

	/// Add `amount` to the balance of `who` under `currency_id` and increase
	/// total issuance.
	fn deposit(currency_id: CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult;

	/// Remove `amount` from the balance of `who` under `currency_id` and reduce
	/// total issuance.
	fn withdraw(currency_id: CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult;

	/// Same result as `slash(currency_id, who, value)` (but without the
	/// side-effects) assuming there are no balance changes in the meantime and
	/// only the reserved balance is not taken into account.
	fn can_slash(currency_id: CurrencyId, who: &AccountId, value: Self::Balance) -> bool;

	/// Deduct the balance of `who` by up to `amount`.
	///
	/// As much funds up to `amount` will be deducted as possible.  If this is
	/// less than `amount`,then a non-zero value will be returned.
	fn slash(currency_id: CurrencyId, who: &AccountId, amount: Self::Balance) -> Self::Balance;
}

/// A trait for querying a value by a key.
pub trait GetByKey<Key, Value> {
	/// Return the value.
	fn get(k: &Key) -> Value;
}

/// Handler for account which has dust, need to burn or recycle it
pub trait OnDust<AccountId, CurrencyId, Balance> {
	fn on_dust(who: &AccountId, currency_id: CurrencyId, amount: Balance);
}

impl<AccountId, CurrencyId, Balance> OnDust<AccountId, CurrencyId, Balance> for () {
	fn on_dust(_: &AccountId, _: CurrencyId, _: Balance) {}
}