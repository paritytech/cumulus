#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure, parameter_types, storage,
};
use sp_core::storage::well_known_keys;
use system::ensure_root;

pub type ValidationFunction = Vec<u8>;

parameter_types! {
    // How many blocks must pass between scheduling a validation function
    // upgrade and applying it.
    //
    // We don't have access to T::BlockNumber in this scope, but we know that
    // the BlockNumber type must be convertable from a 32 bit integer, so using
    // a u32 here should be acceptable.
    pub const ValidationUpgradeDelayBlocks: u32 = 1000;
}
/// The pallet's configuration trait.
pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as ParachainUpgrade {
        // we need to store the new validation function for the span between
        // setting it and applying it.
        PendingValidationFunction get(fn new_validation_function): (T::BlockNumber, ValidationFunction);

        // we store the current block number on each block in order to compare to the
        // pending block number
        CurrentBlockNumber get(fn current_block_number): T::BlockNumber;
    }
}

// The pallet's dispatchable functions.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your pallet
        fn deposit_event() = default;

        fn on_initialize(n: T::BlockNumber) {
            CurrentBlockNumber::<T>::put(n);

            if PendingValidationFunction::<T>::exists() {
                let (apply_block, validation_function) = PendingValidationFunction::<T>::take();
                if n >= apply_block {
                    storage::unhashed::put_raw(well_known_keys::CODE, &validation_function);
                    Self::deposit_event(RawEvent::ValidationFunctionApplied(n));
                }
            }
        }

        pub fn upgrade_validation_function(origin, validation_function: ValidationFunction) {
            // TODO: in the future, we can't rely on a superuser existing
            // on-chain who can just wave their hands and make this happen.
            // Instead, this should hook into the democracy pallet and check
            // that a validation function upgrade has been approved; potentially,
            // it should even trigger the validation function upgrade automatically
            // the moment the vote passes.
            ensure_root(origin)?;
            ensure!(!PendingValidationFunction::<T>::exists(), Error::<T>::OverlappingUpgrades);
            ensure!(CurrentBlockNumber::<T>::exists(), Error::<T>::MissingCurrentBlockNumber);

            let apply_block = CurrentBlockNumber::<T>::get() + ValidationUpgradeDelayBlocks::get().into();

            PendingValidationFunction::<T>::put((apply_block, validation_function));
            Self::deposit_event(RawEvent::ValidationFunctionStored(apply_block));
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        BlockNumber = <T as system::Trait>::BlockNumber,
    {
        // The validation function has been scheduled to apply as of the contained block number.
        ValidationFunctionStored(BlockNumber),
        // The validation function was applied as of the contained block number.
        ValidationFunctionApplied(BlockNumber),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Attempt to upgrade validation function while existing upgrade pending
        OverlappingUpgrades,
        /// Current block number was not stored
        MissingCurrentBlockNumber,
    }
}

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
    #[should_panic]
    fn requires_root() {
        new_test_ext().execute_with(|| {
            assert_ok!(TemplateModule::upgrade_validation_function(
                Origin::signed(1),
                Default::default()
            ));
        });
    }
}
