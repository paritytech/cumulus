#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure, parameter_types, storage,
    traits::Get,
};
use sp_core::storage::well_known_keys;
use sp_version::RuntimeVersion;

pub type ValidationFunction = Vec<u8>;
type System<T> = system::Module<T>;

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

    /// Get the chain's current version.
    type Version: Get<RuntimeVersion>;
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as ParachainUpgrade {
        // we need to store the new validation function for the span between
        // setting it and applying it.
        PendingValidationFunction get(fn new_validation_function): (T::BlockNumber, ValidationFunction);
    }
}

// The pallet's dispatchable functions.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your pallet
        fn deposit_event() = default;

        fn on_initialize(n: T::BlockNumber) {
            if let Ok((apply_block, validation_function)) = PendingValidationFunction::<T>::try_get() {
                if n >= apply_block {
                    PendingValidationFunction::<T>::kill();
                    storage::unhashed::put_raw(well_known_keys::CODE, &validation_function);
                    Self::deposit_event(RawEvent::ValidationFunctionApplied(n));
                }
            }
        }

        pub fn set_code(origin, validation_function: ValidationFunction) {
            // TODO: in the future, we can't rely on a superuser existing
            // on-chain who can just wave their hands and make this happen.
            // Instead, this should hook into the democracy pallet and check
            // that a validation function upgrade has been approved; potentially,
            // it should even trigger the validation function upgrade automatically
            // the moment the vote passes.
            System::<T>::can_set_code(origin, &validation_function)?;
            ensure!(!PendingValidationFunction::<T>::exists(), Error::<T>::OverlappingUpgrades);

            let apply_block = System::<T>::block_number() + ValidationUpgradeDelayBlocks::get().into();

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
    }
}

/// tests for this pallet
#[cfg(test)]
mod tests {
    use super::*;

    use codec::Encode;
    use frame_support::{assert_ok, impl_outer_origin, parameter_types, weights::Weight};
    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup, OnInitialize},
        Perbill,
    };
    use system::{EventRecord, InitKind, Phase, RawOrigin};

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
        pub const Version: RuntimeVersion = RuntimeVersion {
            spec_name: sp_version::create_runtime_str!("test"),
            impl_name: sp_version::create_runtime_str!("system-test"),
            authoring_version: 1,
            spec_version: 1,
            impl_version: 1,
            apis: sp_version::create_apis_vec!([]),
        };
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
        type Version = Version;
        type ModuleToIndex = ();
        type AccountData = ();
        type OnNewAccount = ();
        type OnKilledAccount = ();
    }
    impl Trait for Test {
        type Event = Event<Test>;
        type Version = Version;
    }

    type ParachainUpgrade = Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> sp_io::TestExternalities {
        system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap()
            .into()
    }

    struct CallInWasm(Vec<u8>);

    impl sp_core::traits::CallInWasm for CallInWasm {
        fn call_in_wasm(
            &self,
            _wasm_code: &[u8],
            _code_hash: Option<Vec<u8>>,
            _method: &str,
            _call_data: &[u8],
            _ext: &mut dyn sp_externalities::Externalities,
        ) -> Result<Vec<u8>, String> {
            Ok(self.0.clone())
        }
    }

    fn wasm_ext() -> sp_io::TestExternalities {
        let version = RuntimeVersion {
            spec_name: "test".into(),
            spec_version: 2,
            impl_version: 1,
            ..Default::default()
        };
        let call_in_wasm = CallInWasm(version.encode());

        let mut ext = new_test_ext();
        ext.register_extension(sp_core::traits::CallInWasmExt::new(call_in_wasm));
        ext
    }

    #[test]
    fn requires_root() {
        wasm_ext().execute_with(|| {
            ParachainUpgrade::on_initialize(123);
            assert_eq!(
                ParachainUpgrade::set_code(Origin::signed(1), Default::default()),
                Err(sp_runtime::DispatchError::BadOrigin),
            );
        });
    }

    #[test]
    fn requires_root_2() {
        wasm_ext().execute_with(|| {
            ParachainUpgrade::on_initialize(123);
            assert_ok!(ParachainUpgrade::set_code(
                RawOrigin::Root.into(),
                Default::default()
            ));
        });
    }

    #[test]
    fn events() {
        wasm_ext().execute_with(|| {
            System::<Test>::initialize(
                &123,
                &Default::default(),
                &Default::default(),
                &Default::default(),
                InitKind::Full,
            );
            ParachainUpgrade::on_initialize(123);
            assert_ok!(ParachainUpgrade::set_code(
                RawOrigin::Root.into(),
                Default::default()
            ));
            System::<Test>::finalize();
            assert_eq!(
                dbg!(System::<Test>::events()),
                vec![
                    // should be ValidationFunctionStored(1123)
                    EventRecord {
                        phase: Phase::Initialization,
                        event: (),
                        topics: Vec::new(),
                    },
                ],
            );

            System::<Test>::initialize(
                &1234,
                &Default::default(),
                &Default::default(),
                &Default::default(),
                InitKind::Full,
            );
            ParachainUpgrade::on_initialize(1234);
            System::<Test>::finalize();
            assert_eq!(
                dbg!(System::<Test>::events()),
                vec![
                    // should be ValidationFunctionApplied(1234)
                    EventRecord {
                        phase: Phase::Initialization,
                        event: (),
                        topics: Vec::new(),
                    },
                ],
            );
        });
    }

    #[test]
    fn non_overlapping() {
        wasm_ext().execute_with(|| {
            ParachainUpgrade::on_initialize(123);
            assert_ok!(ParachainUpgrade::set_code(
                RawOrigin::Root.into(),
                Default::default()
            ));
            ParachainUpgrade::on_initialize(234);
            assert_eq!(
                ParachainUpgrade::set_code(RawOrigin::Root.into(), Default::default(),),
                Err(Error::<Test>::OverlappingUpgrades.into()),
            )
        })
    }

    #[test]
    fn set_code_with_real_wasm_blob() {
        let executor = substrate_test_runtime_client::new_native_executor();
        let mut ext = new_test_ext();
        ext.register_extension(sp_core::traits::CallInWasmExt::new(executor));
        ext.execute_with(|| {
            ParachainUpgrade::on_initialize(123);
            ParachainUpgrade::set_code(
                RawOrigin::Root.into(),
                substrate_test_runtime_client::runtime::WASM_BINARY.to_vec(),
            )
            .unwrap();
        });
    }
}
