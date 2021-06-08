#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::{MaybeSerializeDeserialize};
use sp_std::fmt::Debug;
use sp_std::marker::PhantomData;
use sp_std::result;
use codec::{FullCodec};
use xcm::v0::{Error as XcmError, MultiAsset, MultiLocation, Result};
use xcm_executor::traits::{MatchesFungible, TransactAsset, Convert};
use xcm_executor::Assets;
use frame_support::sp_runtime::SaturatedConversion;
use pallet_traits::{MultiCurrency};
use frame_support::sp_runtime::traits::{CheckedSub};

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
    Currency: frame_support::traits::Currency<AccountId> + MultiCurrency<AccountId, CurrencyId=CurrencyId>,
    Matcher: MatchesFungible<<Currency as MultiCurrency<AccountId>>::Balance>,
    AccountId: Clone,
    AccountIdConvert: Convert<MultiLocation, AccountId>,
    CurrencyId: FullCodec + Eq + PartialEq + Copy + MaybeSerializeDeserialize + Debug,
    CurrencyIdConvert: Convert<MultiAsset, CurrencyId>,
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
    fn can_check_in(origin: &MultiLocation, what: &MultiAsset) -> Result {
        let currency_id = CurrencyIdConvert::convert_ref(what)
            .map_err(|()| XcmError::FailedToTransactAsset("AccountIdConversionFailed"))?;
        // Check we handle this asset.
        let amount = Matcher::matches_fungible(what)
            .ok_or(XcmError::FailedToTransactAsset("FailedToMatchFungible"))?;
        if let Ok(checked_account) = AccountIdConvert::convert_ref(origin) {
            let new_balance = <Currency as MultiCurrency<AccountId>>::free_balance(currency_id, &checked_account)
                .checked_sub(&amount)
                .ok_or(XcmError::NotWithdrawable)?;
            <Currency as MultiCurrency<AccountId>>::ensure_can_withdraw(currency_id, &checked_account, new_balance)
                .map_err(|_| XcmError::NotWithdrawable)?;
        }
        Ok(())
    }

    fn check_in(origin: &MultiLocation, what: &MultiAsset) {
        if let Some(amount) = Matcher::matches_fungible(what) {
            if let Ok(who) = AccountIdConvert::convert_ref(origin) {
                if let Ok(currency_id) = CurrencyIdConvert::convert_ref(what) {
                    let ok = <Currency as MultiCurrency<AccountId>>::withdraw(currency_id, &who, amount).is_ok();
                    debug_assert!(ok, "`can_check_in` must have returned `true` immediately prior; qed");
                }
            }
        }
    }

    fn check_out(dest: &MultiLocation, what: &MultiAsset) {
        if let Some(amount) = Matcher::matches_fungible(what) {
            if let Ok(who) = AccountIdConvert::convert_ref(dest) {
                if let Ok(currency_id) = CurrencyIdConvert::convert_ref(what) {
                    // TODO: check here
                    Currency::deposit(currency_id, &who, amount).unwrap();
                }
            }
        }
    }

    fn deposit_asset(asset: &MultiAsset, location: &MultiLocation) -> Result {
        match (
            AccountIdConvert::convert_ref(location),
            CurrencyIdConvert::convert_ref(asset),
            Matcher::matches_fungible(&asset),
        ) {
            // known asset
            (Ok(who), Ok(currency_id), Some(amount)) => {
                Currency::deposit(currency_id, &who, amount).map_err(|e| XcmError::FailedToTransactAsset(e.into()))
            }
            // unknown asset
            _ => Err(XcmError::FailedToTransactAsset("UnknownAsset")),
        }
    }

    fn withdraw_asset(asset: &MultiAsset, location: &MultiLocation) -> result::Result<Assets, XcmError> {
        let who = AccountIdConvert::convert(location.clone())
            .map_err(|_| XcmError::FailedToTransactAsset("FailedToMatchFungible"))?;
        let currency_id = CurrencyIdConvert::convert(asset.clone())
            .map_err(|_| XcmError::FailedToTransactAsset("AccountIdConversionFailed"))?;
        let amount = Matcher::matches_fungible(&asset)
            .ok_or(XcmError::FailedToTransactAsset("CurrencyIdConversionFailed"))?
            .saturated_into();
        <Currency as MultiCurrency<AccountId>>::withdraw(currency_id, &who, amount).map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;
        Ok(Assets::from(asset.clone()))
    }
}
