///! Traits and default implementation for paying transaction fees.
use crate::Config;
use super::*;
use codec::FullCodec;
use frame_support::{
	traits::{Currency, ExistenceRequirement, Get, Imbalance, OnUnbalanced, fungibles::{Balanced, Inspect, CreditOf}, WithdrawReasons},
	unsigned::TransactionValidityError,
};
use pallet_assets::BalanceConversion;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, DispatchInfoOf, MaybeSerializeDeserialize, PostDispatchInfoOf, Saturating, Zero},
	transaction_validity::InvalidTransaction,
};
use sp_std::{fmt::Debug, marker::PhantomData};

pub trait HandleCredit<AccountId, B: Balanced<AccountId>> {
	fn handle_credit(credit: CreditOf<AccountId, B>);
}

/// Handle withdrawing, refunding and depositing of transaction fees.
pub trait OnChargeAssetTransaction<T: Config> {
	/// The underlying integer type in which fees are calculated.
	type Balance: AtLeast32BitUnsigned + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;
	type AssetId: FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default + Eq;
	type LiquidityInfo;

	/// Before the transaction is executed the payment of the transaction fees
	/// need to be secured.
	///
	/// Note: The `fee` already includes the `tip`.
	fn withdraw_fee(
		who: &T::AccountId,
		call: &T::Call,
		dispatch_info: &DispatchInfoOf<T::Call>,
		asset_id: Self::AssetId,
		fee: Self::Balance,
		tip: Self::Balance,
	) -> Result<Self::LiquidityInfo, TransactionValidityError>;

	/// After the transaction was executed the actual fee can be calculated.
	/// This function should refund any overpaid fees and optionally deposit
	/// the corrected amount.
	///
	/// Note: The `fee` already includes the `tip`.
	fn correct_and_deposit_fee(
		who: &T::AccountId,
		dispatch_info: &DispatchInfoOf<T::Call>,
		post_info: &PostDispatchInfoOf<T::Call>,
		corrected_fee: Self::Balance,
		tip: Self::Balance,
		already_withdrawn: Self::LiquidityInfo,
	) -> Result<(), TransactionValidityError>;
}

/// Implements the transaction payment for a module implementing the `Currency`
/// trait (eg. the pallet_balances) using an unbalance handler (implementing
/// `OnUnbalanced`).
///
/// The unbalance handler is given 2 unbalanceds in [`OnUnbalanced::on_unbalanceds`]: fee and
/// then tip.
pub struct FungiblesAdapter<CON, HC>(PhantomData<(CON, HC)>);

/// Default implementation for a Currency and an OnUnbalanced handler.
///
/// The unbalance handler is given 2 unbalanceds in [`OnUnbalanced::on_unbalanceds`]: fee and
/// then tip.
impl<T, CON, HC> OnChargeAssetTransaction<T> for FungiblesAdapter<CON, HC>
where
	T: Config,
	CON: BalanceConversion<BalanceOf<T>, AssetIdOf<T>, AssetBalanceOf<T>>,
	HC: HandleCredit<T::AccountId, T::Fungibles>,
	AssetIdOf<T>: FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default + Eq,
{
	type LiquidityInfo = CreditOf<T::AccountId, T::Fungibles>;
	type Balance = BalanceOf<T>;
	type AssetId = AssetIdOf<T>;

	/// Withdraw the predicted fee from the transaction origin.
	///
	/// Note: The `fee` already includes the `tip`.
	fn withdraw_fee(
		who: &T::AccountId,
		_call: &T::Call,
		_info: &DispatchInfoOf<T::Call>,
		asset_id: Self::AssetId,
		fee: Self::Balance,
		_tip: Self::Balance,
	) -> Result<Self::LiquidityInfo, TransactionValidityError> {
		let converted_fee = CON::to_asset_balance(fee, asset_id)
			.map_err(|_| -> TransactionValidityError { InvalidTransaction::Payment.into() })?;
		let can_withdraw = <T::Fungibles as Inspect<T::AccountId>>::can_withdraw(asset_id.into(), who, converted_fee);
		if !matches!(can_withdraw, WithdrawConsequence::Success) {
			return Err(InvalidTransaction::Payment.into());
		}
		<T::Fungibles as Balanced<T::AccountId>>::withdraw(asset_id.into(), who, converted_fee)
			.map_err(|_| -> TransactionValidityError { InvalidTransaction::Payment.into() })
	}

	/// Hand the fee and the tip over to the `[OnUnbalanced]` implementation.
	/// Since the predicted fee might have been too high, parts of the fee may
	/// be refunded.
	///
	/// Note: The `corrected_fee` already includes the `tip`.
	fn correct_and_deposit_fee(
		who: &T::AccountId,
		_dispatch_info: &DispatchInfoOf<T::Call>,
		_post_info: &PostDispatchInfoOf<T::Call>,
		corrected_fee: Self::Balance,
		_tip: Self::Balance,
		paid: Self::LiquidityInfo,
	) -> Result<(), TransactionValidityError> {
		let converted_fee = CON::to_asset_balance(corrected_fee, paid.asset().into())
			.map_err(|_| -> TransactionValidityError { InvalidTransaction::Payment.into() })?;
		// Calculate how much refund we should return
		let (final_fee, refund) = paid.split(converted_fee);
		// refund to the the account that paid the fees. If this fails, the
		// account might have dropped below the existential balance. In
		// that case we don't refund anything.
		// TODO: what to do in case this errors?
		let _res = <T::Fungibles as Balanced<T::AccountId>>::resolve(who, refund);

		HC::handle_credit(final_fee);
		Ok(())
	}
}
