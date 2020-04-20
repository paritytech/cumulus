# parachain-upgrade-pallet

## Purpose

This pallet allows a user to determine when a parachain validation function is legal, and upgrade that validation function, triggering runtime events for both storing and applying the new validation function.

## Dependencies

### Traits

This pallet does not depend on any externally defined traits.

### Pallets

This pallet does not depend on any externally defined pallets.

### Environment

This pallet depends on certain environmental conditions provided by Cumulus. It will not work outside a Cumulus parachain.

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
