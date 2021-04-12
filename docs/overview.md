# Cumulus Overview

This document provides high-level documentation for Cumulus. A Glossary of terms is included.

### What is Cumulus?
Cumulus is a [Parachain development kit (PDK)](https://wiki.polkadot.network/docs/en/build-pdk) that provides a set of tools to create Polkadot compatible [Parachains](https://substrate.dev/docs/en/knowledgebase/getting-started/glossary#parachain) using the [Substrate development framework](http://substrate.dev/).

### Why would I use Cumulus?
Cumulus allows runtimes created using Substrate [FRAME](https://substrate.dev/docs/en/knowledgebase/runtime/frame) to interface with Parachain valditors and join a network secured by a [Relay Chain](https://substrate.dev/docs/en/knowledgebase/getting-started/glossary#relay-chain). 

This distinction is important as runtimes created with purely Substrate FRAME are not Parachains and cannot join a Polkadot network. Parachains can be an implementation of Substrate FRAME, but FRAME is a more flexible set of tools for building modular, efficient, and upgradeable blockchains. Cumulus is a wrapper around Substrate nodes that extends an interface to Parachain Validators, opening a direct channel to network secured by Relay Chains. 

Cumulus lets developers make Parachains using Substrate. Use cases could include:
- Make a Polkadot-compatible Parachain from genesis.
- Create a sandbox blockchain with Substrate and connect to the Polkadot network when ready for release.
- Join the Polkadot network at a future point in time.

### How does Cumulus Work?
Cumulus At a high level, Cumulus operates in two stages

1. **Consensus** - Cumulus keeps Substrate Nodes in sync with a Relay Chain by importing finalized blocks from Parachain Validators and executing them, thereby extending Parachains to be canonical with a Network. This triggers the next step.
2. **Block Building** - Once a Parachain is canonical with a newly finalized block, Cumulus triggers Parachain Collators to generate a new unsigned block for Validators to take.

## Registration
The Polkadot protocol requires that Parachains to register their state transition functions with Validators in the form of WASM binary. This executable contains a custom Parachain runtime. To make a Substrate runtime into a Parachain runtime, additional functionality must be added. Cumulus makes that easy by adding one line of code to the Substrate runtime:

```rust
cumulus_pallet_parachain_system::register_validate_block!(Block, Executive);
```
Now, when compiled into a WASM binary and registered with a set of Validator Nodes, the substrate node can join as a Parachain. It does so primarily through a `validate_block` function implemeneted as a Cumulus pallet, which takes the `Block` and `Executive` type through this single macro call. `Executive` is a declared type that dispatches incoming calls to the correct module at runtime; it is an orchestration component.

## Consensus
Transactions from a Parachain filter up through the relay chain. At first, Collators author new blocks, some of which Parachain Validtors will select. Of these validated blocks, one is signed and added to the Relay Chain. To maintain consensus, Parachain nodes must know which of the many blocks authored by collators was finalized and then execute it on their local storage.

By the time a Parachain block filters up to the Relay Chain, it has gone through multiple compressions. The first compression happens when a Collator authors a Proof of Validity block (PoVBlock) by bundling:

- List of state transitions
- Values in the parachain’s database that the block modifies
- Hashes of the unaffected points in the Merkle tree

Together, a PoVBlock contains the necessary ingredients for Validators to apply their registered state transition function (recall the WASM binary) and verify the outcome of a block's state transitions. Similarly, when Validators author blocks for the Relay Chain, they further compress their data into Candidate Receipts.

**[Figure 1: Collator to Validator to Relay Chain]**

Parachains maintain consensus with the Relay Chain by executing the finalized blocks secured on the Relay Chain. But to do this, Parachains must decipher and reconstruct blocks from their compressed versions. Once a block has been added, the Relay Chain signals the finalized Candidate Receipt to Parachain Validators. Validators then apply the registered state transition function to reverse engineer the PoVBlock that was originally compressed as part of the Candidate Receipt. This reconstructed PoVBlock is passed back to Parachain nodes via the Cumulus macro call (recall the `Block` type referenced in [Registration](#Registration)). 

Using the `validate_block` function, implemented as a pallet, Cumulus constructs a partial trie from the PoVBlock witness data. All storage related host functions are then redirected to use the partial trie that shows where state transitions occured. The `validate_block` function runs through a series of checks to validate whether state transitions included in the PoVBlock generate the same state that Cumulus received from Validators. If so, Cumulus will execute the block and stay canonical with the Relay Chain.

**[Figure 2: Relay Chain to Validator to Parachain Node]**

When Validators call the `validate_block` function, they pass through the PoVBlock, but they also pass the parent header of the Parachain block stored on the Relay Chain. Passing the parent header of the latest blocks ensures that Parachains do not execute blocks that do not share the same parent block and thereby creating illegitimate branches. If a Parachain node receives a PoVBlock with a parent header it does not recognize, it will not execute the block. Instead, it will wait for other nodes to gossip the canonical state and execute blocks then.

To recap:
1. Collators author blocks for Validators that contain a Proof of Validity of all state transitions within the block.
2. Validators apply their registered state transition function to verify the same outcome as the Proof.
3. Validators compress the PoVBlock into a Candidate Receipt for the Relay Chain.
4. Once executed, the Relay Chain passes the finalized Candidate Receipt back to Parachain Validators
5. Parachain Validators reconstruct the PoVBlock from the Candidate Receipt and use it to call a node's `validate_block` function.
6. Parachains execute the `validate_block` function and execute the block, changing their state to match what is recognized on the Relay Chain

### Where does Cumulus Come In?
Cumulus lets Substrate-based nodes maintain consensus with a Relay Chain. First and foremost, Cumulus exposes an interface to Parachain Validators that lets it interpret and produce blocks that follow the fixed-formatting expected by the Polkadot Network. Furthermore, Cumulus uses these blocks as inputs to execute an important Parachain node function: Knowing which block to execute next.

## Block Building
Once a Parachain node is synchronous with its Relay Chain, it can begin to bundle and broadcast new blocks to be validated. These blocks are the same as the PoVBlocks, containing state transitions, Merkle hashes, and state data, except these messages are being sent _to_ Validators, not _from_ them. However, only a small set of "Collators" are able to produce PoVBlocks. Light-clients and full-nodes created using Cumulus can only maintain consensus, they cannot proffer new blocks to Validators.

Cumulus provides a client Collator library to convert Substrate FRAME nodes into Parachain Collators and the additional logic needed to produce new blocks as per protocol instructions (See under [_II. Block Perparation_](https://polkadot.network/the-path-of-a-parachain-block/)). Production logic is triggered when Collators import PoVBlocks and validate a finalized state change, as described in [Consensus](#Consensus). Once again, data is compressed further and further until one block is finalized on the Relay Chain, and the cycle repeats.
