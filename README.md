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
# --parachain-id 1862 as an example that can be chosen freely. Make sure to everywhere use the same parachain id
./target/release/encointer-collator export-genesis-state --chain encointer-rococo --parachain-id 1862 > genesis-state

# Export genesis wasm
./target/release/encointer-collator export-genesis-wasm --chain encointer-rococo > genesis-wasm

# Collator
./target/release/encointer-collator --collator --tmp --parachain-id 1862 --chain encointer-rococo --port 40335 --ws-port 9946 -- --execution wasm --chain ../polkadot/rococo-local-cfde-real-overseer.json --port 30337 --ws-port 9981
```

### Register the Parachain
Go to [polkadot.js.org/apps/](https://polkadot.js.org/apps/) and register the parachain via the paraSudoWrapper. After registering, the collator should start producing blocks when the next era starts.

**Note:** Change the parachain-id to 1862 when registering the parachain and do not forget to enable file upload if you perform drag and drop. If it is not enabled, Polkadot-js will interpred the path as a string and won't complain. But the registration will fail.

![image](https://user-images.githubusercontent.com/2915325/99548884-1be13580-2987-11eb-9a8b-20be658d34f9.png)

