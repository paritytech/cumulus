// Copyright 2021 Parity Technologies (UK) Ltd.
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

//! Pallet that allows block authors to include their identity in a block via an inherent.
//! Currently the author does not _prove_ their identity, just states it. So it should not be used,
//! for things like equivocation slashing that require authenticated authorship information.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	traits::FindAuthor,
};
use parity_scale_codec::{Decode, Encode};
use sp_inherents::{InherentIdentifier, IsFatalError};
use sp_runtime::{
	ConsensusEngineId, DigestItem, RuntimeString, RuntimeAppPublic,
};
use log::debug;
use nimbus_primitives::{AccountLookup, CanAuthor, NIMBUS_ENGINE_ID, SlotBeacon, EventHandler, INHERENT_IDENTIFIER};

mod exec;
pub use exec::BlockExecutor;
use sp_std::marker::PhantomData;

/// A SlotBeacon that starts a new slot based on this chain's block height.
///TODO there is also (aparently) a BlockNumberProvider trait. Maybe make this a blanket implementation for that?
/// I wonder when that trait is used though. I'm not going to over-engineer this yet.
pub struct HeightBeacon<R>(PhantomData<R>);

impl<R: frame_system::Config> SlotBeacon for HeightBeacon<R> {
	fn slot() -> u32 {
		use core::convert::TryInto;
		frame_system::Pallet::<R>::block_number().try_into().map_err(|_|()).expect("block number should fit into u32 or else nimbus won't work.")
	}
}

/// A SlotBeacon that starts a new slot based on the relay chain's block height.
/// This can only be used when cumulus's parachain system pallet is present.
pub struct RelayChainBeacon<R>(PhantomData<R>);

// TODO this is the only place we depend on parachain system. This impl should live in a different crate.
impl<R: cumulus_pallet_parachain_system::Config> SlotBeacon for RelayChainBeacon<R> {
	fn slot() -> u32 {
		cumulus_pallet_parachain_system::Pallet::<R>::validation_data()
			.expect("validation data was set in parachain system inherent")
			.relay_parent_number
	}
}

///TODO
/// A SlotBeacon that starts a new slot based on the timestamp. Like the one used in sc-consensus-aura et al.
pub struct IntervalBeacon;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use super::*;

	/// The Author Inherent pallet. The core of the nimbus consensus framework's runtime presence.
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		// This is copied from Aura. I wonder if I really need all those trait bounds. For now I'll leave them.
		// TODO could I remove this type entirely and just always use NimbusId? Why didn't Aura do that?
		/// The identifier type for an authority.
		type AuthorId: Member + Parameter + RuntimeAppPublic + Default + MaybeSerializeDeserialize;

		/// A type to convert between AuthorId and AccountId. This is useful when you want to associate
		/// Block authoring behavior with an AccoutId for rewards or slashing. If you do not need to
		/// hold an AccountID responsible for authoring use `()` which acts as an identity mapping.
		type AccountLookup: AccountLookup<Self::AuthorId, Self::AccountId>;

		/// Other pallets that want to be informed about block authorship
		type EventHandler: EventHandler<Self::AccountId>;

		/// The final word on whether the reported author can author at this height.
		/// This will be used when executing the inherent. This check is often stricter than the
		/// Preliminary check, because it can use more data.
		/// If the pallet that implements this trait depends on an inherent, that inherent **must**
		/// be included before this one.
		type CanAuthor: CanAuthor<Self::AccountId>;

		/// Some way of determining the current slot for purposes of verifying the author's eligibility
		type SlotBeacon: SlotBeacon;
	}

	// If the AccountId type supports it, then this pallet can be BoundToRuntimeAppPublic
	impl<T> sp_runtime::BoundToRuntimeAppPublic for Pallet<T>
	where
		T: Config,
		T::AuthorId: RuntimeAppPublic,
	{
		type Public = T::AuthorId;
	}
	#[pallet::error]
	pub enum Error<T> {
		/// Author already set in block.
		AuthorAlreadySet,
		/// No AccountId was found to be associated with this author
		NoAccountId,
		/// The author in the inherent is not an eligible author.
		CannotBeAuthor,
	}


	/// Author of current block.
	#[pallet::storage]
	pub type Author<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_: T::BlockNumber) -> Weight {
			<Author<T>>::kill();
			0
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Inherent to set the author of a block
		#[pallet::weight((0, DispatchClass::Mandatory))]
		fn set_author(origin: OriginFor<T>, author: T::AuthorId) -> DispatchResult {

			ensure_none(origin)?;

			ensure!(<Author<T>>::get().is_none(), Error::<T>::AuthorAlreadySet);
			debug!(target: "author-inherent", "Author was not already set");

			let slot = T::SlotBeacon::slot();
			debug!(target: "author-inherent", "Slot is {:?}", slot);

			let account = T::AccountLookup::lookup_account(&author).ok_or(
				Error::<T>::NoAccountId
			)?;

			ensure!(T::CanAuthor::can_author(&account, &slot), Error::<T>::CannotBeAuthor);

			// Update storage
			Author::<T>::put(&account);

			// Add a consensus digest so the client-side worker can verify the block is signed by the right person.
			frame_system::Pallet::<T>::deposit_log(DigestItem::<T::Hash>::Consensus(
				NIMBUS_ENGINE_ID,
				author.encode(),
			));

			// Notify any other pallets that are listening (eg rewards) about the author
			T::EventHandler::note_author(account);

			Ok(())
		}
	}

	#[pallet::inherent]
	impl<T:Config> ProvideInherent for Pallet<T> {
		type Call = Call<T>;
		type Error = InherentError;
		const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

		fn is_inherent_required(_: &InherentData) -> Result<Option<Self::Error>, Self::Error> {
			// Return Ok(Some(_)) unconditionally because this inherent is required in every block
			// If it is not found, throw an AuthorInherentRequired error.
			Ok(Some(InherentError::Other(
				sp_runtime::RuntimeString::Borrowed("AuthorInherentRequired"),
			)))
		}

		fn create_inherent(data: &InherentData) -> Option<Self::Call> {
			let author_raw = data
				.get_data::<T::AuthorId>(&INHERENT_IDENTIFIER);

			debug!("In create_inherent (runtime side). data is");
			debug!("{:?}", author_raw);

			let author = author_raw
				.expect("Gets and decodes authorship inherent data")?;

			Some(Call::set_author(author))
		}

		fn is_inherent(call: &Self::Call) -> bool {
			matches!(call, Call::set_author(_))
		}
	}

	impl<T: Config> FindAuthor<T::AccountId> for Pallet<T> {
		fn find_author<'a, I>(_digests: I) -> Option<T::AccountId>
		where
			I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
		{
			// We don't use the digests at all.
			// This will only return the correct author _after_ the authorship inherent is processed.
			<Author<T>>::get()
		}
	}

	/// To learn whether a given AuthorId can author, you call the author-inherent directly.
	/// It will do the mapping lookup.
	impl<T: Config> CanAuthor<T::AuthorId> for Pallet<T> {
		fn can_author(author: &T::AuthorId, slot: &u32) -> bool {
			let account = match T::AccountLookup::lookup_account(&author) {
				Some(account) => account,
				// Authors whose account lookups fail will not be eligible
				None => {
					return false;
				},
			};

			T::CanAuthor::can_author(&account, slot)
		}
	}
}

#[derive(Encode)]
#[cfg_attr(feature = "std", derive(Debug, Decode))]
pub enum InherentError {
	Other(RuntimeString),
}

impl IsFatalError for InherentError {
	fn is_fatal_error(&self) -> bool {
		match *self {
			InherentError::Other(_) => true,
		}
	}
}

impl InherentError {
	/// Try to create an instance ouf of the given identifier and data.
	#[cfg(feature = "std")]
	pub fn try_from(id: &InherentIdentifier, data: &[u8]) -> Option<Self> {
		if id == &INHERENT_IDENTIFIER {
			<InherentError as parity_scale_codec::Decode>::decode(&mut &data[..]).ok()
		} else {
			None
		}
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	use crate as author_inherent;

	use frame_support::{
		assert_noop, assert_ok, parameter_types,
		traits::{OnFinalize, OnInitialize},
	};
	use sp_core::H256;
	use sp_io::TestExternalities;
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentityLookup},
	};

	pub fn new_test_ext() -> TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
		TestExternalities::new(t)
	}

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
	type Block = frame_system::mocking::MockBlock<Test>;

	// Configure a mock runtime to test the pallet.
	frame_support::construct_runtime!(
		pub enum Test where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
			AuthorInherent: author_inherent::{Pallet, Call, Storage, Inherent},
		}
	);

	parameter_types! {
		pub const BlockHashCount: u64 = 250;
	}
	impl frame_system::Config for Test {
		type BaseCallFilter = ();
		type BlockWeights = ();
		type BlockLength = ();
		type DbWeight = ();
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Call = Call;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type Version = ();
		type PalletInfo = PalletInfo;
		type AccountData = ();
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
		type SS58Prefix = ();
		type OnSetCode = ();
	}
	impl Config for Test {
		type AuthorId = u64;
		type EventHandler = ();
		type CanAuthor = ();
	}

	pub fn roll_to(n: u64) {
		while System::block_number() < n {
			System::on_finalize(System::block_number());
			System::set_block_number(System::block_number() + 1);
			System::on_initialize(System::block_number());
			AuthorInherent::on_initialize(System::block_number());
		}
	}

	#[test]
	fn set_author_works() {
		new_test_ext().execute_with(|| {
			assert_ok!(AuthorInherent::set_author(Origin::none(), 1));
			roll_to(1);
			assert_ok!(AuthorInherent::set_author(Origin::none(), 1));
			roll_to(2);
		});
	}

	#[test]
	fn must_be_inherent() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				AuthorInherent::set_author(Origin::signed(1), 1),
				sp_runtime::DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn double_author_fails() {
		new_test_ext().execute_with(|| {
			assert_ok!(AuthorInherent::set_author(Origin::none(), 1));
			assert_noop!(
				AuthorInherent::set_author(Origin::none(), 1),
				Error::<Test>::AuthorAlreadySet
			);
		});
	}
}
