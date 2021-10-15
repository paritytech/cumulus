# Cumulus :cloud:

A set of tools for writing [Substrate](https://substrate.io/)-based
[Polkadot](https://wiki.polkadot.network/en/)
[parachains](https://wiki.polkadot.network/docs/en/learn-parachains). Refer to the included
[overview](docs/overview.md) for architectural details, and the
[Cumulus tutorial](https://docs.substrate.io/tutorials/v3/cumulus/start-relay) for a
guided walk-through of using these tools.

It's easy to write blockchains using Substrate, and the overhead of writing parachains'
distribution, p2p, database, and synchronization layers should be just as low. This project aims to
make it easy to write parachains for Polkadot by leveraging the power of Substrate.

Cumulus clouds are shaped sort of like dots; together they form a system that is intricate,
beautiful and functional.

## Consensus

[`cumulus-consensus`](consensus) is a
[consensus engine](https://docs.substrate.io/v3/advanced/consensus) for Substrate
that follows a Polkadot
[relay chain](https://wiki.polkadot.network/docs/en/learn-architecture#relay-chain). This will run
a Polkadot node internally, and dictate to the client and synchronization algorithms which chain
to follow,
[finalize](https://wiki.polkadot.network/docs/en/learn-consensus#probabilistic-vs-provable-finality),
and treat as best.

## Collator

A Polkadot [collator](https://wiki.polkadot.network/docs/en/learn-collator) for the parachain is
implemented by [`cumulus-collator`](collator).

# Statemint 🪙

This repository also contains the Statemint runtime (as well as the canary runtime Statemine and the
test runtime Westmint).
Statemint is a common good parachain providing an asset store for the Polkadot ecosystem.

## Build & Launch a Node

To run a Statemine or Westmint node (Statemint is not deployed, yet) you will need to compile the
`polkadot-collator` binary:

```sh
cargo build --release --locked -p polkadot-collator
```

Once the executable is built, launch the parachain node via:

```sh
CHAIN=westmint # or statemine
./target/release/polkadot-collator --chain $CHAIN
```

Refer to the [setup instructions below](#local-setup) to run a local network for development.

# Rococo :crown:

[Rococo](https://polkadot.js.org/apps/?rpc=wss://rococo-rpc.polkadot.io) is the testnet for
parachains. It currently runs the parachains
[Tick](https://polkadot.js.org/apps/?rpc=wss://tick-rpc.polkadot.io),
[Trick](https://polkadot.js.org/apps/?rpc=wss://trick-rpc.polkadot.io) and
[Track](https://polkadot.js.org/apps/?rpc=wss://track-rpc.polkadot.io).

Rococo is an elaborate style of design and the name describes the painstaking effort that has gone
into this project. Tick, Trick and Track are the German names for the cartoon ducks known to English
speakers as Huey, Dewey and Louie.

## Build & Launch Rococo Collators

Collators are similar to validators in the relay chain. These nodes build the blocks that will
eventually be included by the relay chain for a parachain.

To run a Rococo collator you will need to compile the following binary:

```
cargo build --release --locked -p polkadot-collator
```

Otherwise you can compile it with
[Parity CI docker image](https://github.com/paritytech/scripts/tree/master/dockerfiles/ci-linux):

```bash
docker run --rm -it -w /shellhere/cumulus \
                    -v $(pwd):/shellhere/cumulus \
                    paritytech/ci-linux:production cargo build --release --locked -p polkadot-collator
sudo chown -R $(id -u):$(id -g) target/
```

If you want to reproduce other steps of CI process you can use the following
[guide](https://github.com/paritytech/scripts#gitlab-ci-for-building-docker-images).

Once the executable is built, launch collators for each parachain (repeat once each for chain
`tick`, `trick`, `track`):

```
./target/release/polkadot-collator --chain $CHAIN --validator
```

## Parachains

The parachains of Rococo all use the same runtime code. The only difference between them is the
parachain ID used for registration with the relay chain:

-   Tick: 100
-   Trick: 110
-   Track: 120

The network uses horizontal message passing (HRMP) to enable communication between parachains and
the relay chain and, in turn, between parachains. This means that every message is sent to the relay
chain, and from the relay chain to its destination parachain.

## Local Setup

Launch a local setup including a Relay Chain and a Parachain.

### Launch the Relay Chain

```bash
# Compile Polkadot with the real overseer feature
git clone https://github.com/paritytech/polkadot
cargo build --release

# Generate a raw chain spec
./target/release/polkadot build-spec --chain rococo-local --disable-default-bootnode --raw > rococo-local-cfde.json

# Alice
./target/release/polkadot --chain rococo-local-cfde.json --alice --tmp

# Bob (In a separate terminal)
./target/release/polkadot --chain rococo-local-cfde.json --bob --tmp --port 30334
```

### Launch the Parachain

```bash
# Compile
git clone https://github.com/paritytech/cumulus
cargo build --release

# Export genesis state
# --parachain-id 200 as an example that can be chosen freely. Make sure to everywhere use the same parachain id
./target/release/polkadot-collator export-genesis-state --parachain-id 200 > genesis-state

# Export genesis wasm
./target/release/polkadot-collator export-genesis-wasm > genesis-wasm

# Collator1
./target/release/polkadot-collator --collator --alice --force-authoring --tmp --parachain-id <parachain_id_u32_type_range> --port 40335 --ws-port 9946 -- --execution wasm --chain ../polkadot/rococo-local-cfde.json --port 30335

# Collator2
./target/release/polkadot-collator --collator --bob --force-authoring --tmp --parachain-id <parachain_id_u32_type_range> --port 40336 --ws-port 9947 -- --execution wasm --chain ../polkadot/rococo-local-cfde.json --port 30336

# Parachain Full Node 1
./target/release/polkadot-collator --tmp --parachain-id <parachain_id_u32_type_range> --port 40337 --ws-port 9948 -- --execution wasm --chain ../polkadot/rococo-local-cfde.json --port 30337
```
### Register the parachain
![image](https://user-images.githubusercontent.com/2915325/99548884-1be13580-2987-11eb-9a8b-20be658d34f9.png)

## Build the docker image

After building `polkadot-collator` with cargo or with Parity docker image as documented in [this chapter](#build--launch-rococo-collators), the following will allow producting a new docker image where the compiled binary is injected:

```
./docker/scripts/build-injected-image.sh
```

You may then start a new contaier:

```
docker run --rm -it $OWNER/$IMAGE_NAME --collator --tmp --parachain-id 1000 --execution wasm --chain /specs/westmint.json
```
