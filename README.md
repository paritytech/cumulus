# parachain-upgrade-pallet

WIP on a pallet to ease parachain upgrades; see https://github.com/paritytech/cumulus/issues/59.

## Purpose

This pallet allows a user to determine when a parachain validation function is legal, and upgrade that validation function, triggering runtime events for both the storing and applying the new validation function.

## Dependencies

### Traits

This pallet does not depend on any externally defined traits.

### Pallets

This pallet depends on the [`balances` pallet](https://github.com/paritytech/substrate/tree/master/frame/balances).

## Installation

### Runtime `Cargo.toml`

To add this pallet to your runtime, simply include the following to your runtime's `Cargo.toml` file:

```TOML
[dependencies.parachain-upgrade-pallet]
default_features = false
git = 'https://github.com/substrate-developer-hub/substrate-pallet-template.git'
```

and update your runtime's `std` feature to include this pallet:

```TOML
std = [
    # --snip--
    'parachain-upgrade-pallet/std',
]
```

### Runtime `lib.rs`

You should implement its trait like so:

```rust
/// Used for test_module
impl parachain_upgrade_pallet::Trait for Runtime {
	type Event = Event;
}
```

and include it in your `construct_runtime!` macro:

```rust
ParachainUpgradePallet: parachain_upgrade_pallet::{Module, Call, Storage, Event<T>},
```

### Genesis Configuration

This template pallet does not have any genesis configuration.

## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```

or by visiting this site: <Add Your Link>
