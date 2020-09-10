// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Cumulus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_event, decl_error, decl_module, dispatch::Dispatchable,
	traits::{Currency, ExistenceRequirement, WithdrawReason},
	Parameter, parameter_types
};
use frame_system::{RawOrigin, ensure_signed};
use sp_runtime::{RuntimeDebug, traits::CheckedConversion};
use sp_std::convert::TryFrom;

use codec::{Encode, Decode};
use cumulus_primitives::{
	xcm::{v0::{Xcm, XcmError, XcmResult, SendXcm, ExecuteXcm}, VersionedXcm}, ParaId
};
use polkadot_parachain::primitives::AccountIdConversion;
use frame_support::sp_std::result;
use polkadot_parachain::xcm::{
	VersionedMultiAsset, VersionedMultiLocation,
	v0::{MultiOrigin, MultiAsset, MultiLocation, Junction, Ai}
};
use frame_support::traits::Get;
use sp_runtime::app_crypto::sp_core::crypto::UncheckedFrom;
use polkadot_parachain::xcm::v0::AssetInstance;
use sp_std::collections::btree_map::{BTreeMap, Entry};
use sp_std::collections::btree_set::BTreeSet;

/// Origin for the parachains module.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub enum Origin {
	/// It comes from the relay-chain.
	RelayChain,
	/// It comes from a parachain.
	Parachain(ParaId),
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub trait DepositAsset {
	fn deposit_asset(what: &MultiAsset, who: &MultiLocation) -> XcmResult;
}
impl<X: DepositAsset, Y: DepositAsset> DepositAsset for (X, Y) {
	fn deposit_asset(what: &MultiAsset, who: &MultiLocation) -> XcmResult {
		X::deposit_asset(what, who).or_else(|| Y::deposit_asset(what, who))
	}
}

pub trait MatchesFungible<Balance> {
	fn matches_fungible(a: &MultiAsset) -> Option<Balance>;
}
pub struct IsConcrete<T>;
impl<T: Get<MultiLocation>, B: CheckedFrom<u128>> MatchesFungible<B> for IsConcrete<T> {
	fn matches_fungible(a: &MultiAsset) -> Option<B> {
		match a {
			MultiAsset::ConcreteFungible { id, amount } if id == T::get() =>
				amount.checked_into(),
			_ => false,
		}
	}
}
pub struct IsAbstract<T>;
impl<T: Get<&'static [u8]>, B: CheckedFrom<u128>> MatchesFungible<B> for IsAbstract<T> {
	fn matches_fungible(a: &MultiAsset) -> Option<B> {
		match a {
			MultiAsset::AbstractFungible { id, amount } if &id[..] == T::get() =>
				amount.checked_into(),
			_ => false,
		}
	}
}
impl<B: Balance, X: MatchesFungible<B>, Y: MatchesFungible<B>> MatchesFungible<B> for (X, Y) {
	fn matches_fungible(a: &MultiAsset) -> Option<B> {
		X::matches_fungible(a).or_else(|| Y::matches_fungible(a))
	}
}

pub trait PunnFromLocation<T> {
	fn punn_from_location(m: &MultiLocation) -> Option<T>;
}

pub struct AccountId32Punner<AccountId>;
impl<AccountId: UncheckedFrom<[u8; 32]>> PunnFromLocation<AccountId> for AccountId32Punner<AccountId> {
	fn punn_from_location(m: &MultiLocation) -> Option<AccountId> {
		match m {
			MultiLocation::X1(Junction::AccountId32 { ref id, .. }) =>
				Some(AccountId::unchecked_from(id.clone())),
			_ => None,
		}
	}
}

pub struct CurrencyAdapter<Currency, Matcher, AccountIdConverter, AccountId>;
impl<
	Matcher: MatchesAsset,
	AccountIdConverter: PunnFromLocation<AccountId>,
	Currency: Currency<AccountId>,
	AccountId,	// can't get away without it since Currency is generic over it.
> DepositAsset for CurrencyAdapter<Currency, Matcher, AccountIdConverter, AccountId> {
	fn deposit_asset(what: &MultiAsset, who: &MultiLocation) -> XcmResult {
		// Check we handle this asset.
		let amount = Matcher::matches_asset(&what).ok_or(())?;
		let who = AccountIdConverter::punn_from_location(who)?;
		Currency::deposit_creating(&who, amount).map_err(|_| ())?;
		Ok(())
	}
}

parameter_types! {
	const DotLocation: MultiLocation = MultiLocation::X1(Junction::Parent);
	const DotName: &'static [u8] = &b"DOT"[..];
	const MyLocation: MultiLocation = MultiLocation::Null;
	const MyName: &'static [u8] = &b"ABC"[..];
}
/*
type MyDepositAsset = (
	// Convert a Currency impl into a DepositAsset
	CurrencyAdapter<
		// Use this currency:
		balances_pallet::Module::<T, Instance1>,
		// Use this currency when it is a fungible asset matching the given location or name:
		(IsConcrete<DotLocation>, IsAbstract<DotName>),
		// Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
		AccountId32Punner<T::AccountId>,
		// Our chain's account ID type (we can't get away without mentioning it explicitly):
		T::AccountId,
	>,
	CurrencyAdapter<
		balances_pallet::Module::<T, DefaultInstance>,
		(IsConcrete<MyLocation>, IsAbstract<MyName>),
		AccountId32Punner<T::AccountId>,
		T::AccountId,
	>,
);
*/

//	TODO: multiasset_pallet::Module::<T>,

/// Configuration trait of this pallet.
pub trait Trait: frame_system::Trait {
	/// Event type used by the runtime.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// The outer origin type.
	type Origin: From<Origin>
	+ From<frame_system::RawOrigin<Self::AccountId>>
	+ Into<result::Result<Origin, <Self as Trait>::Origin>>;

	/// The outer call dispatch type.
	type Call: Parameter + Dispatchable<Origin=<Self as Trait>::Origin> + From<Call<Self>>;

	type XcmExecutive: ExecuteXcm;

	type Currency: Currency;
}

decl_event! {
	pub enum Event<T> where
		AccountId = <T as frame_system::Trait>::AccountId
	{
		/// Transferred tokens to the account on the relay chain.
		TransferredToRelayChain(VersionedMultiLocation, VersionedMultiAsset),
		/// Soem assets have been received.
		ReceivedAssets(AccountId, VersionedMultiAsset),
		/// Transferred tokens to the account from the given parachain account.
		TransferredToParachainViaReserve(ParaId, VersionedMultiLocation, VersionedMultiAsset),
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// A version of a data format is unsupported.
		UnsupportedVersion,
		/// Asset given was invalid or unsupported.
		BadAsset,
		/// Location given was invalid or unsupported.
		BadLocation,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: <T as frame_system::Trait>::Origin {
		fn deposit_event() = default;

		// TODO: Handle fee payment in terms of weight.
		#[weight = 10]
		fn execute(origin, xcm: VersionedXcm) {
			// TODO: acceptable origins are Signed (which results in an XCM origin of
			//   MultiLocation::AccountId32 and Parachain which corresponds to
			//   MultiOrigin::Parachain).
			let xcm_origin = origin.into();

			T::ExecuteXcm::execute_xcm(xcm_origin, xcm);
		}
/*
		/// Transfer some `asset` from the Parachain to the given `dest`.
		// TODO: Remove
		#[weight = 10]
		fn transfer(origin, dest: VersionedMultiLocation, versioned_asset: VersionedMultiAsset) {
			let who = ensure_signed(origin)?;

			// TODO: all this should be removed and refactored into the `execute` function.

			let asset = MultiAsset::try_from(versioned_asset.clone())
				.map_err(|_| Error::<T>::UnsupportedVersion)?;
			let dest = MultiLocation::try_from(dest).map_err(|_| Error::<T>::UnsupportedVersion)?;

			let (asset, amount) = match asset {
				// The only asset whose reserve we recognise for now is native tokens from the
				// relay chain, identified as the singular asset of the Relay-chain. From our
				// context (i.e. a parachain) , this is `Parent`. From the Relay-chain's context,
				// it is `Null`.
				MultiAsset::ConcreteFungible { id: MultiLocation::X1(Junction::Parent), amount } =>
					(MultiAsset::ConcreteFungible { id: MultiLocation::Null, amount }, amount),
				_ => Err(Error::<T>::BadAsset)?,	// Asset not recognised.
			};

			let amount: BalanceOf::<T> = amount.checked_into().ok_or(Error::<T>::BadAsset)?;

			match dest {
				MultiLocation::X3(Junction::Parent, Junction::Parachain{ id: dest_id }, dest_loc) => {
					// Reserve transfer using the Relay-chain.
					let _ = T::Currency::withdraw(
						&who,
						amount,
						WithdrawReason::Transfer.into(),
						ExistenceRequirement::AllowDeath,
					)?;

					let dest_loc = MultiLocation::from(dest_loc);
					let msg = Xcm::ReserveAssetTransfer {
						asset,
						dest: Junction::Parachain { id: dest_id }.into(),
						effect: Ai::DepositAsset { asset: MultiAsset::Wild, dest: dest_loc.clone() },
					};
					// TODO: Check that this will work prior to withdraw.
					let _ = T::UmpSender::send_upward(msg.into());

					Self::deposit_event(Event::<T>::TransferredToParachainViaReserve(
						dest_id.into(),
						dest_loc.into(),
						versioned_asset,
					));
				}
				MultiLocation::X2(Junction::Parent, dest_loc) => {
					// Direct withdraw/deposit on the Relay-chain
					let _ = T::Currency::withdraw(
						&who,
						amount,
						WithdrawReason::Transfer.into(),
						ExistenceRequirement::AllowDeath,
					)?;

					let dest_loc = MultiLocation::from(dest_loc);
					let msg = Xcm::WithdrawAsset {
						asset,
						effect: Ai::DepositAsset { asset: MultiAsset::Wild, dest: dest_loc.clone() },
					};
					let _ = T::UmpSender::send_upward(msg.into());

					Self::deposit_event(Event::<T>::TransferredToRelayChain(
						dest_loc.into(),
						versioned_asset,
					));
				}
				_ => Err(Error::<T>::BadLocation)?,	// Invalid location.
			}
		}
		*/
	}
}

enum AssetId {
	Concrete(MultiLocation),
	Abstract(Vec<u8>),
}

#[derive(Default, Clone)]
struct Assets {
	pub fungible: BTreeMap<AssetId, u128>,
	pub non_fungible: BTreeSet<(AssetId, AssetInstance)>,
}

impl From<Vec<MultiAsset>> for Assets {
	fn from(assets: Vec<MultiAsset>) -> Assets {
		let mut result = Self::default();
		for asset in assets.into_iter() {
			result.subsume(asset)
		}
		result
	}
}
impl Assets {
	/// Modify `self` to include `MultiAsset`, saturating if necessary.
	pub fn saturating_subsume(&mut self, asset: MultiAsset) {
		match asset {
			MultiAsset::ConcreteFungible { id, amount } => {
				self.fungible
					.entry(AssetId::Concrete(id))
					.and_modify(|e| *e = e.saturating_add(amount))
					.or_insert(amount);
			}
			MultiAsset::AbstractFungible { id, amount } => {
				self.fungible
					.entry(AssetId::Abstract(id))
					.and_modify(|e| *e = e.saturating_add(amount))
					.or_insert(amount);
			}
			MultiAsset::ConcreteNonFungible { class, instance} => {
				self.non_fungible.insert((AssetId::Concrete(class), instance));
			}
			MultiAsset::AbstractNonFungible { class, instance} => {
				self.non_fungible.insert((AssetId::Abstract(class), instance));
			}
			MultiAsset::Each(ref assets) => {
				for asset in assets.into_iter() {
					self.saturating_subsume(asset.clone())
				}
			}
			_ => (),
		}
	}

	pub fn saturating_subsume_fungible(&mut self, id: AssetId, amount: u128) {
		self.fungible
			.entry(id)
			.and_modify(|e| *e = e.saturating_add(amount))
			.or_insert(amount);
	}

	pub fn saturating_subsume_non_fungible(&mut self, class: AssetId, instance: AssetInstance) {
		self.non_fungible.insert((class, instance));
	}

	/// Take all possible assets up to `assets` from `self`, mutating `self` and returning the
	/// assets taken.
	///
	/// Wildcards work.
	pub fn saturating_take(&mut self, assets: Vec<MultiAsset>) -> Assets {
		let mut result = Assets::default();
		for asset in assets.into_iter() {
			match asset {
				MultiAsset::None => (),
				MultiAsset::All => return self.swapped(Assets::default()),
				x @ MultiAsset::ConcreteFungible {..} | MultiAsset::AbstractFungible {..} => {
					let (id, amount) = match x {
						MultiAsset::ConcreteFungible { id, amount } => (AssetId::Concrete(id), amount),
						MultiAsset::AbstractFungible { id, amount } => (AssetId::Abstract(id), amount),
						_ => unreachable!(),
					};
					// remove the maxmimum possible up to id/amount from self, add the removed onto
					// result
					self.fungible.entry(id.clone())
						.and_modify(|e| if *e >= amount {
							result.saturating_subsume_fungible(id, amount);
							*e = *e - amount;
						} else {
							result.saturating_subsume_fungible(id, *e);
							*e = 0
						});
				}
				x @ MultiAsset::ConcreteNonFungible {..} | MultiAsset::AbstractNonFungible {..} => {
					let (class, instance) = match x {
						MultiAsset::ConcreteNonFungible { class, instance } => (AssetId::Concrete(class), instance),
						MultiAsset::AbstractNonFungible { class, instance } => (AssetId::Abstract(class), instance),
						_ => unreachable!(),
					};
					// remove the maxmimum possible up to id/amount from self, add the removed onto
					// result
					if let Some(entry) = self.non_fungible.take(&(class, instance)) {
						self.non_fungible.insert(entry);
					}
				}
				// TODO: implement partial wildcards.
				MultiAsset::AllFungible
				| MultiAsset::AllNonFungible
				| MultiAsset::AllAbstractFungible { id }
				| MultiAsset::AllAbstractNonFungible { class }
				| MultiAsset::AllConcreteFungible { id }
				| MultiAsset::AllConcreteNonFungible { class } => (),
			}
		}
		result
	}

	pub fn swapped(&mut self, mut with: Assets) -> Self {
		sp_std::mem::swap(&mut *self, &mut with);
		with
	}
}

pub trait XcmExecutorConfig {
	/// The outer origin type.
	type Origin: From<Origin>
	+ From<frame_system::RawOrigin<Self::AccountId>>
	+ Into<result::Result<Origin, <Self as Trait>::Origin>>;

	/// The outer call dispatch type.
	type Call: Parameter + Dispatchable<Origin=<Self as Trait>::Origin> + From<Call<Self>>;

	type XcmSender: SendXcm;

	/// How to deposit an asset.
	type AssetDepositor: DepositAsset;

	// TODO: How to withdraw an asset.
}

pub struct XcmExecutor<Config>;

impl<Config: XcmExecutorConfig> XcmExecutor<Config> {
	fn execute_effects(origin: &Origin, holding: &mut Assets, effect: Ai) -> XcmResult {
		match effect {
			Ai::DepositAsset { assets, dest } => {
				let deposited = holding.saturating_take(assets);
				for (id, amount) in deposited.fungible.into_iter() {
					let asset = match id {
						AssetId::Concrete(id) => MultiAsset::ConcreteFungible { id, amount },
						AssetId::Abstract(id) => MultiAsset::AbstractFungible { id, amount },
					};
					Config::AssetDepositor::deposit_asset(&asset, &dest)?;
				}
				for (id, instance) in deposited.non_fungible {
					let asset = match id {
						AssetId::Concrete(class) => MultiAsset::ConcreteNonFungible { class, instance },
						AssetId::Abstract(class) => MultiAsset::AbstractNonFungible { class, instance },
					};
					Config::AssetDepositor::deposit_asset(&asset, &dest)?;
				}
			},
			_ => Err(()),
		}
		Ok(())
	}
}

impl<Config: XcmExecutorConfig> ExecuteXcm for XcmExecutor<Config> {
	fn execute_xcm(origin: MultiLocation, msg: VersionedXcm) -> XcmResult {
		let (mut holding, effects) = match (origin, Xcm::try_from(msg)) {
			(origin, Ok(Xcm::ForwardedFromParachain { id, inner })) => {
				let new_origin = origin.pushed_with(Junction::Parachain{id}).map_err(|_| ())?;
				Self::execute_xcm(new_origin, *inner)
			}
			(_origin, Ok(Xcm::WithdrawAsset { assets, effects })) => {
				// TODO: Take as much of `assets` from the origin account (on-chain) and place in holding.

				// TODO: This will require either a new config trait `AssetWithdrawer`, or to
				//   introduce withdraw facilities into `AssetDepositor` (and renaming accordingly).
				let holding = Assets::from(assets); // << just a stub.

				(holding, effects)
			}
			(origin, Ok(Xcm::ReserveAssetCredit { assets, effects })) => {
				// TODO: check whether we trust origin to be our reserve location for this asset via
				//   config trait.
				if assets.len() == 1 &&
					matches!(&assets[0], MultiAsset::ConcreteFungible { ref id, .. } if id == &origin)
				{
					// We only trust the origin to send us assets that they identify as their
					// sovereign assets.
					(Assets::from(assets), effects)
				} else {
					Err(())?
				}
			}
			(_origin, Ok(Xcm::TeleportAsset { assets, effects })) => {
				// TODO: check whether we trust origin to teleport this asset to us via config trait.
				Err(())?	// << we don't trust any chains, for now.
			}
			(origin, Ok(Xcm::Transact { origin_type, call })) => {
				// We assume that the Relay-chain is allowed to use transact on this parachain.

				// TODO: Weight fees should be paid.

				// TODO: allow this to be configurable in the trait.
				// TODO: allow the trait to issue filters for the relay-chain

				if let Ok(message_call) = <T as Trait>::Call::decode(&mut &call[..]) {
					let origin: <T as Trait>::Origin = match origin_type {
						// TODO: Allow sovereign accounts to be configured via the trait.
						MultiOrigin::SovereignAccount => {
							match origin {
								// Relay-chain doesn't yet have a sovereign account on the parachain.
								MultiLocation::X1(Junction::Parent) => Err(())?,
								MultiLocation::X2(Junction::Parent, Junction::Parachain{ id }) =>
									RawOrigin::Signed(id.into_account()).into(),
								_ => Err(())?,
							}
						}
						// We assume we are a parachain.
						//
						// TODO: Use the config trait to convert the multilocation into an origin.
						MultiOrigin::Native => match origin {
							MultiLocation::X1(Junction::Parent) => Origin::RelayChain.into(),
							MultiLocation::X2(Junction::Parent, Junction::Parachain{id}) =>
								Origin::Parachain(id.into()).into(),
							_ => Err(())?,
						},
						MultiOrigin::Superuser => match origin {
							MultiLocation::X1(Junction::Parent) =>
								// We assume that the relay-chain is allowed to execute with superuser
								// privileges if it wants.
								// TODO: allow this to be configurable in the trait.
								RawOrigin::Root.into(),
							MultiLocation::X2(Junction::Parent, Junction::Parachain{id}) =>
								// We assume that parachains are not allowed to execute with
								// superuser privileges.
								// TODO: allow this to be configurable in the trait.
								Err(())?,
							_ => Err(())?,
						}
					};
					let _ok = message_call.dispatch(origin).is_ok();
					// Not much to do with the result as it is. It's up to the parachain to ensure that the
					// message makes sense.
					return Ok(());
				}
			}
			_ => Err(())?,	// Unhandled XCM message.
		};

		// TODO: stuff that should happen after holding is populated but before effects,
		//   including depositing fees for effects from holding account.

		for effect in effects.into_iter() {
			let _ = Self::execute_effects(&origin, &mut holding, effect)?;
		}

		// TODO: stuff that should happen after effects including refunding unused fees.

		Ok(())
	}
}
