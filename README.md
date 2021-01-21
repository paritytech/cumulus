# Encointer Parachain:

This is the repository to run encointer as a parachain in the rococo-v1 testnet. It is forked from the Cumulus repo and only adds the encointer-pallets and configuration.

## Launch a local setup including a Relay Chain and a Parachain

### Launch the Relay Chain (Local Rococo Testnet)

```bash
# Compile Polkadot with the real overseer feature
git clone https://github.com/paritytech/polkadot
git checkout rococo-v1
cargo build --release --features=real-overseer

# Generate a raw chain spec
./target/release/polkadot build-spec --chain rococo-local --disable-default-bootnode --raw > rococo-local-cfde-real-overseer.json

# Alice
./target/release/polkadot --chain rococo-local-cfde-real-overseer.json --alice --tmp

# Bob (In a separate terminal)
./target/release/polkadot --chain rococo-local-cfde-real-overseer.json --bob --tmp --port 30334
```

### Launch the Parachain

```bash
# Compile
git clone https://github.com/encointer/encointer-parachain.git
git checkout master
cargo build --release

# Export genesis state
# --parachain-id 200 as an example that can be chosen freely. Make sure to everywhere use the same parachain id
./target/release/encointer-collator export-genesis-state --parachain-id 1862 > genesis-state

# Export genesis wasm
./target/release/encointer-collator export-genesis-wasm > genesis-wasm

# Collator1
./target/release/encointer-collator --collator --tmp --parachain-id 1862 --chain encointer-rococo --port 40335 --ws-port 9946 -- --execution wasm --chain ../polkadot/rococo-local-cfde-real-overseer.json --port 30337 --ws-port 9981

# Collator2 (optional)
./target/release/encointer-collator --collator --tmp --chain encointer-rococo --parachain-id 1862 --port 40336 --ws-port 9947 -- --execution wasm --chain ../polkadot/rococo-local-cfde-real-overseer.json --port 30338 --ws-port 9982

# Parachain Full Node 1
./target/release/encointer-collator --tmp --parachain-id 1862 --port 40337 --ws-port 9948 -- --execution wasm --chain ../polkadot/rococo-local-cfde-real-overseer.json --port 30339
```