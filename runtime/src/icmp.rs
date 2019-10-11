// Copyright 2019 Parity Technologies (UK) Ltd.
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

//! The main entry point for messaging I/O of the runtime.
//!
//! This module includes:
//! - logic for creating inherent extrinsics on the client-side from a set of incoming messages;
//! - API to allow runtimes to configure the how messages are handled;
//! - API to allow runtimes to deposit outgoing messages.

use codec::Codec;
use support::{decl_module, decl_storage, decl_event};
use system::ensure_none;
use runtime_primitives::traits::Dispatchable;
use polkadot_primitives::parachain::{Chain, Id as ParaId};

/// Means of handling a bunch of "messages" (opaque blobs of data) coming in from other chains.
pub trait HandleMessages {
	/// Messages have arrived: do something with them. The default implementation just forwards
	/// each message to be handled by `handle_message`.
	fn handle_messages(messages: &[(Chain, Vec<u8>)]) {
		for (from, ref data) in messages.iter() {
			Self::handle_message(*from, data)
		}
	}

	/// Handle an individual message of `_data` from endpoint `_from`. The default implementation
	/// simply drops the message.
	fn handle_message(_from: Chain, _data: &[u8]) {}
}

/// Empty tuple drops all messages.
impl HandleMessages for () {}

/// An origin for this module.
#[derive(PartialEq, Eq, Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum Origin {
	/// It comes from a parachain.
	Parachain(ParaId),

	/// It comes from the Relay chain.
	Relay,
}

/// A message handler which treats each message as a `Call` and dispatches them as per `Call`s
/// with a corresponding `Origin`.
pub struct DispatchCall<Origin, Call>(::rstd::marker::PhantomData<(Origin, Call)>);

impl<
	Call: Codec + Dispatchable
> HandleMessages for DispatchCall<Origin, Call> where Call::Origin: From<Origin> {
	fn handle_message(from: Chain, mut data: &[u8]) {
		if let Ok(call) = Call::decode(&mut data) {
			let origin: Call::Origin = match from {
				Chain::Parachain(id) => Origin::Parachain(id),
				Chain::Relay => Origin::Relay,
			}.into();
			// we disregard the result for now, much like transactions. If we eventually get some
			// economic disincentive to spam the chain, then we could place events down here.
			let _ = call.dispatch(origin);
		}
	}
}

/// The module's configuration trait.
pub trait Trait: system::Trait {
	/// The type which is used to handle incoming messages.
	type OnIncoming: HandleMessages;

	/// The outer origin type.
	type Origin: From<Origin> + From<system::RawOrigin<Self::AccountId>>;

	/// The overarching event type.
	type Event: From<Event> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: <T as system::Trait>::Origin {
		fn deposit_event() = default;

		/// Provide any incoming messages from external ICMP chains (i.e. parachains or the relay
		/// chain) for this block to execute.
		fn note_incoming(origin, messages: Vec<(Chain, Vec<u8>)>) {
			ensure_none(origin)?;

			T::OnIncoming::handle_messages(&messages);
		}
	}
}

decl_event!(
	pub enum Event {
		// Just a dummy event.
		Dummy,
	}
);

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use primitives::H256;
	use support::{impl_outer_origin, assert_ok, parameter_types};
	use sr_primitives::{
		traits::{BlakeTwo256, IdentityLookup}, testing::Header, weights::Weight, Perbill,
	};

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	}
	impl system::Trait for Test {
		type Origin = Origin;
		type Call = ();
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type WeightMultiplierUpdate = ();
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
	}
	impl Trait for Test {
		type Event = ();
	}
	type TemplateModule = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities {
		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[test]
	fn it_works_for_default_value() {
		new_test_ext().execute_with(|| {
			// Just a dummy test for the dummy funtion `do_something`
			// calling the `do_something` function with a value 42
			assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
			// asserting that the stored value is equal to what we stored
			assert_eq!(TemplateModule::something(), Some(42));
		});
	}
}
