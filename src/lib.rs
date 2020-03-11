#![cfg_attr(not(feature = "std"), no_std)]

/// A pallet template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on FRAME pallets, see the example.
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs
use frame_support::{decl_event, decl_module, decl_storage};
use pallet_balances::Trait as BalanceTrait;
use system::ensure_signed;

pub type ValidationFunction = Vec<u8>;

/// The pallet's configuration trait.
pub trait Trait: system::Trait + BalanceTrait {
    // TODO: Add other types and constants required configure this pallet.

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as TemplateModule {
        // we need to store the new validation function for the span between
        // setting it and applying it.
        NewValidationFunction get(fn new_validation_function): Option<ValidationFunction>;
    }
}

// The pallet's dispatchable functions.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your pallet
        fn deposit_event() = default;

        pub fn upgrade_validation_function(origin, validation_function: ValidationFunction) {
            let who = ensure_signed(origin)?;

            NewValidationFunction::put(validation_function);

            // unimplemented!("dispatch this to a real function in polkadot");

            Self::deposit_event(RawEvent::ValidationFunctionStored(who));
        }
    }
}

impl<T: Trait + BalanceTrait> Module<T> {
    /// Returns `Some(balance)` with the upgrade fee when an upgrade would be legal.
    /// Returns `None` when an upgrade would be illegal.
    pub fn can_upgrade_validation_function(_len: usize) -> Option<<T as BalanceTrait>::Balance> {
        // unimplemented!("dispatch this to a real function in polkadot")
        None
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        // Note when the validation function is updated.
        ValidationFunctionStored(AccountId),

        // Where do we dispatch this event?
        ValidationFunctionApplied,
    }
);

/// tests for this pallet
#[cfg(test)]
mod tests {
    use super::*;

    use frame_support::{assert_ok, impl_outer_origin, parameter_types, weights::Weight};
    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        Perbill,
    };

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the pallet, we construct most of a mock runtime. This means
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
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
        type ModuleToIndex = ();
    }
    impl Trait for Test {
        type Event = ();
    }
    impl BalanceTrait for Test {
        type Balance = u64;
        type TransferPayment = ();
        type Event = ();
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type DustRemoval = ();
        type ExistentialDeposit = ();
        type TransferFee = ();
        type CreationFee = ();
    }
    type TemplateModule = Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> sp_io::TestExternalities {
        system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap()
            .into()
    }

    #[test]
    fn it_works_for_default_value() {
        new_test_ext().execute_with(|| {
            assert_eq!(TemplateModule::can_upgrade_validation_function(0), None);
            assert_ok!(TemplateModule::upgrade_validation_function(
                Origin::signed(1),
                Default::default()
            ));
        });
    }
}
