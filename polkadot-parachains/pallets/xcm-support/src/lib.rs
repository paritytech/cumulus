#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::{MaybeSerializeDeserialize};
use sp_std::fmt::Debug;
use sp_std::marker::PhantomData;
use sp_std::result;
use codec::{FullCodec};
use xcm::v0::{Error as XcmError, MultiAsset, MultiLocation, Result};
use xcm_executor::traits::{MatchesFungible, TransactAsset};
use xcm_executor::Assets;
use frame_support::sp_runtime::SaturatedConversion;

use pallet_traits::MultiCurrency;
use frame_support::sp_runtime::traits::{Convert};

/// The `TransactAsset` implementation, to handle `MultiAsset` deposit/withdraw.
///
/// If the asset is known, deposit/withdraw will be handled by `MultiCurrency`,
/// else by `UnknownAsset` if unknown.
pub struct XCMCurrencyAdapter<
    Currency,
    Matcher,
    AccountId,
    AccountIdConvert,
    CurrencyId,
    CurrencyIdConvert,
>(
    PhantomData<(
        Currency,
        Matcher,
        AccountId,
        AccountIdConvert,
        CurrencyId,
        CurrencyIdConvert,
    )>,
);

impl<
    Currency: MultiCurrency<AccountId, CurrencyId = CurrencyId>,
    Matcher: MatchesFungible<<Currency as MultiCurrency<AccountId>>::Balance>,
    AccountId: Clone,
    AccountIdConvert: Convert<MultiLocation, Option<AccountId>>,
    CurrencyId: FullCodec + Eq + PartialEq + Copy + MaybeSerializeDeserialize + Debug,
    CurrencyIdConvert: Convert<MultiAsset, Option<CurrencyId>>,
> TransactAsset
for XCMCurrencyAdapter<
    Currency,
    Matcher,
    AccountId,
    AccountIdConvert,
    CurrencyId,
    CurrencyIdConvert,
>
{
    // fn can_check_in(_origin: &MultiLocation, what: &MultiAsset) -> Result {
    //     let currency_id = CurrencyIdConvert::convert(what.clone())
    //         .ok_or(XcmError::FailedToTransactAsset("AccountIdConversionFailed"))?;
    //     // Check we handle this asset.
    //     let amount: Currency::Balance = Matcher::matches_fungible(what)
    //         .ok_or(XcmError::FailedToTransactAsset("FailedToMatchFungible"))?;
    //     let who = AccountIdConvert::convert(location.clone())
    //         .ok_or(XcmError::FailedToTransactAsset("FailedToMatchFungible"))?;
    //     if let Some(checked_account) = CheckedAccount::get() {
    //         let new_balance = Currency::free_balance(currency_id, &checked_account)
    //             .checked_sub(&amount)
    //             .ok_or(XcmError::NotWithdrawable)?;
    //         Currency::ensure_can_withdraw(currency_id, &checked_account, amount)
    //             .map_err(|_| XcmError::NotWithdrawable)?;
    //     }
    //     Ok(())
    // }
    //
    // fn check_in(_origin: &MultiLocation, what: &MultiAsset) {
    //     if let Some(amount) = Matcher::matches_fungible(what) {
    //         if let Some(who) = AccountIdConvert::convert(location.clone()) {
    //             if let Some(currency_id) = CurrencyIdConvert::convert(what.clone()) {
    //                 let ok = Currency::withdraw(currency_id, &checked_account, amount).is_ok();
    //                 debug_assert!(ok, "`can_check_in` must have returned `true` immediately prior; qed");
    //             }
    //         }
    //     }
    // }
    //
    // fn check_out(_dest: &MultiLocation, what: &MultiAsset) {
    //     if let Some(amount) = Matcher::matches_fungible(what) {
    //         let who = AccountIdConvert::convert(location.clone())
    //             .ok_or(XcmError::FailedToTransactAsset("FailedToMatchFungible"))?;
    //         let currency_id = CurrencyIdConvert::convert(what.clone())
    //             .ok_or(XcmError::FailedToTransactAsset("AccountIdConversionFailed"))?;
    //         Currency::deposit(currency_id, who, amount);
    //     }
    // }

    fn deposit_asset(asset: &MultiAsset, location: &MultiLocation) -> Result {
        match (
            AccountIdConvert::convert(location.clone()),
            CurrencyIdConvert::convert(asset.clone()),
            Matcher::matches_fungible(&asset),
        ) {
            // known asset
            (Some(who), Some(currency_id), Some(amount)) => {
                Currency::deposit(currency_id, &who, amount).map_err(|e| XcmError::FailedToTransactAsset(e.into()))
            }
            // unknown asset
            _ => Err(XcmError::FailedToTransactAsset("UnknownAsset")),
        }
    }

    fn withdraw_asset(asset: &MultiAsset, location: &MultiLocation) -> result::Result<Assets, XcmError> {
        let who = AccountIdConvert::convert(location.clone())
            .ok_or(XcmError::FailedToTransactAsset("FailedToMatchFungible"))?;
        let currency_id = CurrencyIdConvert::convert(asset.clone())
            .ok_or(XcmError::FailedToTransactAsset("AccountIdConversionFailed"))?;
        let amount = Matcher::matches_fungible(&asset)
            .ok_or(XcmError::FailedToTransactAsset("CurrencyIdConversionFailed"))?
            .saturated_into();
        Currency::withdraw(currency_id, &who, amount).map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;
        Ok(Assets::from(asset.clone()))
    }
}
