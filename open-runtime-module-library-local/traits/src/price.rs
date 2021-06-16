use crate::DataProvider;
use frame_support::Parameter;
use sp_runtime::traits::{CheckedDiv, MaybeSerializeDeserialize, Member};
use sp_std::marker::PhantomData;

/// A trait to provide relative price for two currencies
pub trait PriceProvider<CurrencyId, Price> {
	fn get_price(base: CurrencyId, quote: CurrencyId) -> Option<Price>;
}

/// A `PriceProvider` implementation based on price data from a `DataProvider`
pub struct DefaultPriceProvider<CurrencyId, Source>(PhantomData<(CurrencyId, Source)>);

impl<CurrencyId, Source, Price> PriceProvider<CurrencyId, Price> for DefaultPriceProvider<CurrencyId, Source>
where
	CurrencyId: Parameter + Member + Copy + MaybeSerializeDeserialize,
	Source: DataProvider<CurrencyId, Price>,
	Price: CheckedDiv,
{
	fn get_price(base_currency_id: CurrencyId, quote_currency_id: CurrencyId) -> Option<Price> {
		let base_price = Source::get(&base_currency_id)?;
		let quote_price = Source::get(&quote_currency_id)?;

		base_price.checked_div(&quote_price)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use sp_runtime::{FixedPointNumber, FixedU128};

	type Price = FixedU128;

	pub struct MockDataProvider;
	impl DataProvider<u32, Price> for MockDataProvider {
		fn get(currency: &u32) -> Option<Price> {
			match currency {
				0 => Some(Price::from_inner(0)),
				1 => Some(Price::from_inner(1)),
				2 => Some(Price::from_inner(2)),
				_ => None,
			}
		}
	}

	type TestPriceProvider = DefaultPriceProvider<u32, MockDataProvider>;

	#[test]
	fn get_price_should_work() {
		assert_eq!(
			TestPriceProvider::get_price(1, 2),
			Some(Price::saturating_from_rational(1, 2))
		);
		assert_eq!(
			TestPriceProvider::get_price(2, 1),
			Some(Price::saturating_from_rational(2, 1))
		);
	}

	#[test]
	fn price_is_none_should_not_panic() {
		assert_eq!(TestPriceProvider::get_price(3, 3), None);
		assert_eq!(TestPriceProvider::get_price(3, 1), None);
		assert_eq!(TestPriceProvider::get_price(1, 3), None);
	}

	#[test]
	fn price_is_zero_should_not_panic() {
		assert_eq!(TestPriceProvider::get_price(0, 0), None);
		assert_eq!(TestPriceProvider::get_price(1, 0), None);
		assert_eq!(TestPriceProvider::get_price(0, 1), Some(Price::from_inner(0)));
	}
}
