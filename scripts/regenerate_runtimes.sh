#!/bin/bash

cd tools/runtime-codegen
cargo run --bin runtime-codegen -- --from-node-url "wss://rococo-bridge-hub-rpc.polkadot.io:443" > ../../relays/client-bridge-hub-rococo/src/codegen_runtime.rs
cargo run --bin runtime-codegen -- --from-node-url "http://localhost:20433" > ../../relays/client-rialto-parachain/src/codegen_runtime.rs
cd -
cargo fmt --all