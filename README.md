# substrate-pallet-template

This is a template for a Substrate pallet which lives as its own crate so it can be imported into multiple other runtimes. It is based on the ["template" pallet](https://github.com/paritytech/substrate/tree/master/bin/node-template/runtime/src/template.rs) which is included with the [Substrate node template](https://github.com/paritytech/substrate/tree/master/bin/node-template).

Check out the [HOWTO](HOWTO.md) to learn how to use this for your own runtime module.

This README should act as a general template for distributing your module to others.

## Purpose

This pallet acts as a template for building other pallets.

It currently allows a user to put a `u32` value into storage, which triggers a runtime event.

## Dependencies

### Traits

This pallet does not depend on any externally defined traits.

### Pallets

This pallet does not depend on any other FRAME pallet or externally developed modules.

## Installation

### Runtime `Cargo.toml`

To add this pallet to your runtime, simply include the following to your runtime's `Cargo.toml` file:

```TOML
[dependencies.substrate-pallet-template]
default_features = false
git = 'https://github.com/substrate-developer-hub/substrate-pallet-template.git'
```

and update your runtime's `std` feature to include this pallet:

```TOML
std = [
    # --snip--
    'example_pallet/std',
]
```

### Runtime `lib.rs`

You should implement it's trait like so:

```rust
/// Used for test_module
impl example_pallet::Trait for Runtime {
	type Event = Event;
}
```

and include it in your `construct_runtime!` macro:

```rust
ExamplePallet: substrate_pallet_template::{Module, Call, Storage, Event<T>},
```

### Genesis Configuration

This template pallet does not have any genesis configuration.

## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```

or by visiting this site: <Add Your Link>
