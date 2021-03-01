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

use frame_executive::ExecuteBlock;
use sp_application_crypto::RuntimeAppPublic;
use sp_consensus_aura::digests::CompatibleDigestItem;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// The configuration trait.
	#[pallet::config]
	pub trait Config: pallet_aura::Config + frame_system::Config {}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

pub struct BlockExecutor<T, I>(sp_std::marker::PhantomData<(T, I)>);

impl<Block, T, I> ExecuteBlock<Block> for BlockExecutor<T, I>
where
	Block: BlockT,
	T: Config,
	I: ExecuteBlock<Block>,
{
	fn execute_block(block: Block) -> Block::Header {
		let (mut header, extrinsics) = block.deconstruct();

		let post_hash = header.hash();

		let mut seal_and_index = None;
		header
			.digest()
			.logs()
			.iter()
			.enumerate()
			.for_each(
				|(i, s)| {
					let seal =
						CompatibleDigestItem::<<T::AuthorityId as RuntimeAppPublic>::Signature>::as_aura_seal(s);
					match (seal, seal_and_index.is_some()) {
					(Some(_), true) => panic!("Found multiple AuRa seals"),
					(None, _) => (),
					(Some(s), false) => {
						seal_and_index = Some((s, i));
					}
					}},
			);

		let (seal, index) = seal_and_index.expect("Could not find an AuRa seal!");

		// Remove the digest before continue the processing
		header.digest_mut().logs.remove(index);

		let mut new_header = I::execute_block(Block::new(header, extrinsics));

		new_header
			.digest_mut()
			.logs
			.insert(
				index,
				CompatibleDigestItem::<<T::AuthorityId as RuntimeAppPublic>::Signature>::aura_seal(seal),
			);

		assert_eq!(
			post_hash,
			new_header.hash(),
			"New header with AuRa seal doesn't match the expected hash!",
		);

		new_header
	}
}
