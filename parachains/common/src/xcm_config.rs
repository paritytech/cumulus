use crate::impls::AccountIdOf;
use core::marker::PhantomData;
use frame_support::{
	log,
	traits::{fungibles::Inspect, tokens::ConversionToAssetBalance, ContainsPair},
	weights::Weight,
};
use sp_runtime::traits::Get;
use xcm::latest::prelude::*;

/// A `ChargeFeeInFungibles` implementation that converts the output of
/// a given WeightToFee implementation an amount charged in
/// a particular assetId from pallet-assets
pub struct AssetFeeAsExistentialDepositMultiplier<
	Runtime,
	WeightToFee,
	BalanceConverter,
	AssetInstance: 'static,
>(PhantomData<(Runtime, WeightToFee, BalanceConverter, AssetInstance)>);
impl<CurrencyBalance, Runtime, WeightToFee, BalanceConverter, AssetInstance>
	cumulus_primitives_utility::ChargeWeightInFungibles<
		AccountIdOf<Runtime>,
		pallet_assets::Pallet<Runtime, AssetInstance>,
	> for AssetFeeAsExistentialDepositMultiplier<Runtime, WeightToFee, BalanceConverter, AssetInstance>
where
	Runtime: pallet_assets::Config<AssetInstance>,
	WeightToFee: frame_support::weights::WeightToFee<Balance = CurrencyBalance>,
	BalanceConverter: ConversionToAssetBalance<
		CurrencyBalance,
		<Runtime as pallet_assets::Config<AssetInstance>>::AssetId,
		<Runtime as pallet_assets::Config<AssetInstance>>::Balance,
	>,
	AccountIdOf<Runtime>:
		From<polkadot_primitives::AccountId> + Into<polkadot_primitives::AccountId>,
{
	fn charge_weight_in_fungibles(
		asset_id: <pallet_assets::Pallet<Runtime, AssetInstance> as Inspect<
			AccountIdOf<Runtime>,
		>>::AssetId,
		weight: Weight,
	) -> Result<
		<pallet_assets::Pallet<Runtime, AssetInstance> as Inspect<AccountIdOf<Runtime>>>::Balance,
		XcmError,
	> {
		let amount = WeightToFee::weight_to_fee(&weight);
		// If the amount gotten is not at least the ED, then make it be the ED of the asset
		// This is to avoid burning assets and decreasing the supply
		let asset_amount = BalanceConverter::to_asset_balance(amount, asset_id)
			.map_err(|_| XcmError::TooExpensive)?;
		Ok(asset_amount)
	}
}

/// Accepts an asset if it is a native asset from a particular `MultiLocation`.
pub struct ConcreteNativeAssetFrom<Location>(PhantomData<Location>);
impl<Location: Get<MultiLocation>> ContainsPair<MultiAsset, MultiLocation>
	for ConcreteNativeAssetFrom<Location>
{
	fn contains(asset: &MultiAsset, origin: &MultiLocation) -> bool {
		log::trace!(target: "xcm::contains",
			"ConcreteNativeAsset asset: {:?}, origin: {:?}, location: {:?}",
			asset, origin, Location::get());
		matches!(asset.id, Concrete(ref id) if id == origin && origin == &Location::get())
	}
}

/// Accepts an asset if it is a native asset from the system (Relay Chain or system parachain).
pub struct ConcreteNativeAssetFromSystem;
impl ContainsPair<MultiAsset, MultiLocation> for ConcreteNativeAssetFromSystem {
	fn contains(asset: &MultiAsset, origin: &MultiLocation) -> bool {
		log::trace!(target: "xcm::contains", "ConcreteNativeAssetFromSystem asset: {:?}, origin: {:?}", asset, origin);
		let parent = MultiLocation::parent();
		let is_system = match origin {
			// The Relay Chain
			MultiLocation { parents: 1, interior: Here } => true,
			// System parachain
			MultiLocation { parents: 1, interior: X1(Parachain(id)) } if *id < 2000 => true,
			// Others
			_ => false,
		};
		matches!(asset.id, Concrete(id) if id == parent && is_system)
	}
}

#[cfg(test)]
mod tests {
	use super::{
		ConcreteNativeAssetFromSystem, ContainsPair, GeneralIndex, Here, MultiAsset, MultiLocation,
		PalletInstance, Parachain, Parent,
	};

	#[test]
	fn native_asset_from_sibling_system_para_works() {
		let expected_asset: MultiAsset = (Parent, 1000000).into();
		let expected_origin: MultiLocation = (Parent, Parachain(1999)).into();

		assert!(ConcreteNativeAssetFromSystem::contains(&expected_asset, &expected_origin));
	}

	#[test]
	fn native_asset_from_sibling_system_para_fails_for_wrong_asset() {
		let unexpected_assets: Vec<MultiAsset> = vec![
			(Here, 1000000).into(),
			((PalletInstance(50), GeneralIndex(1)), 1000000).into(),
			((Parent, Parachain(1000), PalletInstance(50), GeneralIndex(1)), 1000000).into(),
		];
		let expected_origin: MultiLocation = (Parent, Parachain(1000)).into();

		unexpected_assets.iter().for_each(|asset| {
			assert!(!ConcreteNativeAssetFromSystem::contains(asset, &expected_origin));
		});
	}

	#[test]
	fn native_asset_from_sibling_system_para_fails_for_wrong_origin() {
		let expected_asset: MultiAsset = (Parent, 1000000).into();
		let unexpected_origin: MultiLocation = (Parent, Parachain(2000)).into();

		assert!(!ConcreteNativeAssetFromSystem::contains(&expected_asset, &unexpected_origin));
	}
}
