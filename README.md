# Cumulus :cloud:

Cumulus is a [Parachain development kit (PDK)](https://wiki.polkadot.network/docs/en/build-pdk). It is a set of tools used to create Polkadot compatible [Parachains](https://substrate.dev/docs/en/knowledgebase/getting-started/glossary#parachain) using the [Substrate development framework](http://substrate.dev/).

You can learn more about Cumulus by reading the [Overview](docs/overview.md)

_Cumulus clouds are shaped sort of like dots; together they form a system that is intricate, beautiful, and functional._

# Getting Started

This tutorial will guide you through the steps needed to launch and connect a Parachain to a Relay Chain using Cumulus.

## Step 1: Launch a Relay Chain
Use Polkadot to launch a Relay Chain node. This tutorial uses [Rococo](https://polkadot.network/introducing-rococo-polkadots-parachain-testnet/), a public testnet specifically for Parachains. 

```bash
# launch a relay chain Node
git clone https://github.com/paritytech/polkadot.git
cd polkadot
git fetch
git checkout rococo-v1
cargo build --release --features=real-overseer

# generate a raw rococo chain-spec
./target/release/polkadot build-spec \
   --chain rococo-local \
   --disable-default-bootnode \
   --raw > rococo-local-cfde.json

# start Alice's node
./target/release/polkadot --chain rococo-local-cfde.json --alice

# start Bob's node (in a separate terminal and port)
./target/release/polkadot --chain rococo-local-cfde.json --bob --port 30334

```

## Step 2: Launch A Parachain
Use Cumulus to launch a Parachain node.

```bash
# clone the cumulus repository
git clone https://github.com/paritytech/cumulus
cd cumulus
git fetch
git checkout rococo-v1
cargo build --release -p rococo-collator

# export genesis state
# --parachain-id 200 is an arbitrary number chosen freely. use the same parachain id everywhere
./target/release/rococo-collator export-genesis-state --parachain-id 200 > genesis-state

# export genesis wasm
./target/release/rococo-collator export-genesis-wasm > genesis-wasm

# launch collator node
./target/release/rococo-collator \
   --collator \
   --tmp \
   --parachain-id 200 \
   --port 40335 \
   --ws-port 9946 \
   -- \
   --execution wasm \
   --chain ../polkadot/rococo-local-cfde.json \
   --port 30335

# launch full node
./target/release/rococo-collator \
   --tmp \
   --parachain-id 200 \
   --port 40337 \
   --ws-port 9948 \
   -- \
   --execution wasm \
   --chain ../polkadot/rococo-local-cfde.json 
   --port 30337
```

## Step 3: Register with the Relay Chain
Use Cumulus to register the Parachain. This is normally done in the Polkadot network with [auctions](https://wiki.polkadot.network/docs/en/learn-auction), but tutorial uses Sudo.

A transaction registering the Parachain can be submitted from `Apps > Sudo > parasSudoWrapper > sudoScheduleParaInitialize` with parameters generated in Step 2:

- id: 200
- genesisHead: genesis-state
- validationCode: genesis-wasm
- parachain: Yes


![image](https://user-images.githubusercontent.com/2915325/99548884-1be13580-2987-11eb-9a8b-20be658d34f9.png)

# Further Resources
- [Cumulus Workshop](https://substrate.dev/cumulus-workshop/#/en/)
- [Architectural Overview](docs/overview.md)
- [Substrate Developer Hub](https://substrate.dev/en/)
