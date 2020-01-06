# How to Use

Simply fork this project and start your runtime development in `lib.rs`.

## Update Your Package Information

In the `Cargo.toml` file of this template, you should update the name and authors of this pallet:

```rust
[package]
name = "<YOUR_PALLET_NAME>"
version = "0.1.0"
authors = ["<YOUR_NAME>"]
edition = "2018"
```

## Updating Your Dependencies

For the time being, Substrate does not have any releases on Cargo, which means there is some magic involved with ensuring that all the dependencies of your pallet and the downstream runtime are the same.

This repository has all Substrate dependencies use:

```
rev = '40a16efefc070faf5a25442bc3ae1d0ea2478eee'
```

> **Note:** Be sure to purge your projects of any `Cargo.lock` files when making changes like this!

## Adding New Dependencies

Substrate FRAME can support any Rust libraries that can compile to Wasm. In general, this means that the libraries you use must compile with `no_std` enabled.

This also means you need to import your dependencies to only use their `std` feature when this pallet uses its `std` dependency.

The common pattern here look like:

1. Import your library with `default-features = false`

    ```TOML
    [dependencies.codec]
    default-features = false
    features = ['derive']
    package = 'parity-scale-codec'
    version = '1.0.0'
    ```

2. Add your library to the `std` feature with its `std` feature enabled

    ```TOML
    std = [
        'codec/std',
        # --snip--
    ]
    ```

We won't teach you the details of using Cargo, but feel free to [become a master](https://doc.rust-lang.org/cargo/).

## Building and Testing

Before you release your pallet, you should check that it can:

1. Build to Native:

    ```
    cargo build --release
    ```

2. Pass your tests:

    ```
    cargo test
    ```

## Update the README

Finally, update the [README](README.md) included with this template with the appropriate information for your pallet.

## Common Issues

Unfortunately keeping things modular can be a little tricky. Here are some common issues you may face.

### Developer Dependencies

When running `cargo build`, everything can compile fine, but when running `cargo test`, you may run into an import error like:

```rust
error[E0432]: unresolved import `sp_core`
  --> src/lib.rs:72:6
   |
72 |     use sp_core::H256;
   |         ^^^^^^^ use of undeclared type or module `sp_core`

error: aborting due to previous error
```

You may need to specify some developer dependency which is needed for your tests:

```
[dev-dependencies.sp-core]
default-features = false
git = 'https://github.com/paritytech/substrate.git'
rev = '40a16efefc070faf5a25442bc3ae1d0ea2478eee'
```

> **Note:** `dev-dependencies` will always use `std`, so you should not set `default-features = false`.


### Using Mismatching Substrate Dependencies

We mentioned above how it is important that your runtime and your module have exactly the same dependency on the Substrate project. If they do not, you will get a lot of `trait bound` errors like this:

```rust
error[E0277]: the trait bound `Runtime: frame_system::Trait` is not satisfied in `Event`
    |
112 | impl system::Trait for Runtime {
    |      ^^^^^^^^^^^^^ within `Event`, the trait `frame_system::Trait` is not implemented for `Runtime`
    |
    = note: required because it appears within the type `Event`

error[E0277]: the trait bound `Runtime: frame_system::Trait` is not satisfied
    |
196 | impl test_module::Trait for Runtime {
    |      ^^^^^^^^^^^^^^^^^^ the trait `frame_system::Trait` is not implemented for `Runtime`
```
