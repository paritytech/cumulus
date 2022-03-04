# Cumulus ☁️

This repository contains both the Cumulus SDK and also specific chains implemented
on top of this SDK.

## Cumulus SDK

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

### Consensus

[`parachain-consensus`](https://github.com/paritytech/cumulus/blob/master/client/consensus/common/src/parachain_consensus.rs) is a
[consensus engine](https://docs.substrate.io/v3/advanced/consensus) for Substrate
that follows a Polkadot
[relay chain](https://wiki.polkadot.network/docs/en/learn-architecture#relay-chain). This will run
a Polkadot node internally, and dictate to the client and synchronization algorithms which chain
to follow,
[finalize](https://wiki.polkadot.network/docs/en/learn-consensus#probabilistic-vs-provable-finality),
and treat as best.

### Collator

A Polkadot [collator](https://wiki.polkadot.network/docs/en/learn-collator) for the parachain is
implemented by the `polkadot-collator` binary.

## Statemint 🪙

This repository also contains the Statemint runtime (as well as the canary runtime Statemine and the
test runtime Westmint).
Statemint is a common good parachain providing an asset store for the Polkadot ecosystem.

### Build & Launch a Node

To run a Statemine or Westmint node (Statemint is not deployed, yet) you will need to compile the
`polkadot-collator` binary:

```bash
cargo build --release --locked -p polkadot-collator
```

Once the executable is built, launch the parachain node via:

```bash
CHAIN=westmint # or statemine
./target/release/polkadot-collator --chain $CHAIN
```

Refer to the [setup instructions below](#local-setup) to run a local network for development.

## Canvas 🧑‍🎨

[![matrix][k1]][k2] [![discord][l1]][l2]

[k1]: https://img.shields.io/badge/matrix-chat-brightgreen.svg?style=flat
[k2]: https://riot.im/app/#/room/#ink:matrix.parity.io
[l1]: https://img.shields.io/discord/722223075629727774?style=flat-square&label=discord
[l2]: https://discord.com/invite/wGUDt2p

This is a node implementation of `Canvas`, a common good parachain for `pallet-contracts`
based wasm smart contracts. Right now this repository only contains the `canvas-kusama` runtime
which we plan to use for both Rococo and Kusama.

If you have any questions, feel free to talk to us on [Element][k2] or on [Discord][l2]
(in the [`ink_smart-contracts`](https://discord.com/channels/722223075629727774/765280480609828864) channel).

## Developing Smart Contracts for Canvas

![Canvas Overview](./docs/canvas-overview.svg)

This node contains Substrate's smart contracts module ‒ the
[`contracts`](https://github.com/paritytech/substrate/tree/master/frame/contracts) pallet.
This `contracts` pallet takes smart contracts as WebAssembly blobs and defines an API
for everything a smart contract needs (storage access, …).
As long as a programming language compiles to WebAssembly and there exists an implementation
of this API in it, you can write a smart contract for this pallet ‒ and thus for Canvas ‒ in
that language.

This is a list of languages you can currently choose from:

* [Parity's ink!](https://github.com/paritytech/ink) for Rust
* [ask!](https://github.com/patractlabs/ask) for Assembly Script
* The [Solang](https://github.com/hyperledger-labs/solang) compiler for Solidity

There are also different user interfaces and command-line tools you can use to deploy
or interact with contracts:

* [polkadot-js](https://polkadot.js.org/apps/)
* [Canvas UI](https://paritytech.github.io/canvas-ui/) (outdated)

If you are looking for a quickstart, we can recommend
[ink!'s Guided Tutorial for Beginners](https://docs.substrate.io/tutorials/v3/ink-workshop/pt1/).

### Build & Launch a Node

To run a Canvas node that connects to Rococo (Kusama and Polkadot parachains are not deployed, yet)
you will need to compile the `polkadot-collator` binary:

```bash
cargo build --release --locked -p polkadot-collator
```

Once the executable is built, launch the parachain node via:

```bash
./target/release/polkadot-collator --chain rocanvas
```

Refer to the [setup instructions below](#local-setup) to run a local network for development.

### Rococo Deployment

We have a live deployment of the Canvas parachain on [Rococo](https://wiki.polkadot.network/docs/build-pdk#rococo-testnet) ‒
a testnet for Polkadot and Kusama parachains.
You can interact with the network through Polkadot JS Apps,
[click here for a direct link to Canvas](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo-canvas-rpc.polkadot.io#/explorer).

The Canvas parachain uses the Rococo relay chain's native token (ROC) instead of having its own token.
Due to this you'll need ROC in order to deploy contracts on Canvas.

As a first step, you should create an account. See [here](https://wiki.polkadot.network/docs/learn-account-generation)
for a detailed guide.

As a second step, you have to get ROC testnet tokens through the [Rococo Faucet](https://wiki.polkadot.network/docs/learn-DOT#getting-rococo-tokens).
This is a chat room in which you need to write:

```bash
!drip YOUR_SS_58_ADDRESS:1002
```

The number `1002` is the parachain id of Canvas on Rococo, by supplying it the faucet will teleport ROC
tokens directly to your account on the parachain.

If everything worked out, the teleported ROC tokens will show up under
[the "Accounts" tab for Canvas](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo-canvas-rpc.polkadot.io#/accounts).

Once you have ROC on Canvas you can deploy a contract as you would normally.
If you're unsure about this, our [guided tutorial](https://docs.substrate.io/tutorials/v3/ink-workshop/pt1/)
will clarify that for you in no time.

## Rococo 👑

[Rococo](https://polkadot.js.org/apps/?rpc=wss://rococo-rpc.polkadot.io) is becoming a [Community Parachain Testbed](https://polkadot.network/blog/rococo-revamp-becoming-a-community-parachain-testbed/) for parachain teams in the Polkadot ecosystem. It supports multiple parachains with the differentiation of long-term connections and recurring short-term connections, to see which parachains are currently connected and how long they will be connected for [see here](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo-rpc.polkadot.io#/parachains).

Rococo is an elaborate style of design and the name describes the painstaking effort that has gone
into this project.

### Build & Launch Rococo Collators

Collators are similar to validators in the relay chain. These nodes build the blocks that will
eventually be included by the relay chain for a parachain.

To run a Rococo collator you will need to compile the following binary:

```bash
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

```bash
./target/release/polkadot-collator --chain $CHAIN --validator
```

### Parachains

* [Statemint](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo-statemint-rpc.polkadot.io#/explorer)
* [Canvas on Rococo](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo-canvas-rpc.polkadot.io#/explorer)
* [RILT](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo.kilt.io#/explorer)

The network uses horizontal message passing (HRMP) to enable communication between parachains and
the relay chain and, in turn, between parachains. This means that every message is sent to the relay
chain, and from the relay chain to its destination parachain.

### Local Setup

Launch a local setup including a Relay Chain and a Parachain.

#### Launch the Relay Chain

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

#### Launch the Parachain

```bash
# Compile
git clone https://github.com/paritytech/cumulus
cargo build --release

# Export genesis state
./target/release/polkadot-collator export-genesis-state > genesis-state

# Export genesis wasm
./target/release/polkadot-collator export-genesis-wasm > genesis-wasm

# Collator1
./target/release/polkadot-collator --collator --alice --force-authoring --tmp --port 40335 --ws-port 9946 -- --execution wasm --chain ../polkadot/rococo-local-cfde.json --port 30335

# Collator2
./target/release/polkadot-collator --collator --bob --force-authoring --tmp --port 40336 --ws-port 9947 -- --execution wasm --chain ../polkadot/rococo-local-cfde.json --port 30336

# Parachain Full Node 1
./target/release/polkadot-collator --tmp --port 40337 --ws-port 9948 -- --execution wasm --chain ../polkadot/rococo-local-cfde.json --port 30337
```

#### Register the parachain

![image](https://user-images.githubusercontent.com/2915325/99548884-1be13580-2987-11eb-9a8b-20be658d34f9.png)

### Containerize

After building `polkadot-collator` with cargo or with Parity CI image as documented in [this chapter](#build--launch-rococo-collators),
the following will allow producing a new docker image where the compiled binary is injected:

```bash
./docker/scripts/build-injected-image.sh
```

Alternatively, you can build an image with a builder pattern:

```bash
docker build --tag $OWNER/$IMAGE_NAME --file ./docker/polkadot-collator_builder.Containerfile .

You may then run your new container:

```bash
docker run --rm -it $OWNER/$IMAGE_NAME --collator --tmp --execution wasm --chain /specs/westmint.json
```
