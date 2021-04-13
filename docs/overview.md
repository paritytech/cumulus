# Cumulus Overview

This document provides high-level documentation for Cumulus. It includes a [Glossary](#Glossary) of domain-specific terms.

### What is Cumulus?
Cumulus is a [Parachain development kit (PDK)](https://wiki.polkadot.network/docs/en/build-pdk). It is a set of tools used to create Polkadot compatible [Parachains](https://substrate.dev/docs/en/knowledgebase/getting-started/glossary#parachain) using the [Substrate development framework](http://substrate.dev/).

### Why would I use Cumulus?
Cumulus allows runtimes built using Substrate [FRAME](https://substrate.dev/docs/en/knowledgebase/runtime/frame) to interface with Parachain validators and to join a network secured by a [Relay Chain](https://substrate.dev/docs/en/knowledgebase/getting-started/glossary#relay-chain). 

This distinction is essential as runtimes created with purely Substrate FRAME are not Parachains and cannot join a Polkadot network. Parachains can implement substrate FRAME, but FRAME is a more flexible set of tools for custom and upgradeable blockchains. Cumulus is a wrapper around a Substrate node that extends an interface to Parachain Validators, opening a direct channel to a network secured by its Relay Chain. 

Cumulus lets developers make Parachains using Substrate. Use cases could include:
- Make a Polkadot-compatible Parachain from genesis.
- Create a sandbox blockchain with Substrate and connect to the Polkadot network when ready.
- Join the Polkadot network at a future point in time.

### How does Cumulus Work?
Cumulus At a high level, Cumulus makes it possible for Substrate nodes to **build** blocks for, and maintain **conesus** with, a Relay Chain.

1. **Consensus** - Cumulus keeps Substrate Nodes in sync with a Relay Chain by importing finalized blocks from Parachain Validators and executing them, thereby extending Parachains canonical with a Network.
2. **Block Building** - Once a Parachain is canonical with a newly finalized block, Cumulus triggers Parachain Collators to build a new unsigned block for Validators to take.

## Registration
The Polkadot protocol requires that Parachains register their state transition functions with Validators as a WASM binary. This executable compiles a Parachain's custom runtime. Cumulus adds the required logic to convert a Substrate runtime into a Parachain runtime with the following code:

```rust
cumulus_pallet_parachain_system::register_validate_block!(Block, Executive);
```

When compiled into a WASM binary and registered with a set of Validator Nodes, the substrate node can join as a Parachain. It does so primarily through a `validate_block` macro call implemented as a Cumulus pallet, which takes the `Block` and `Executive` type. `Executive` is a declared type that dispatches incoming calls to the correct module at runtime; it is an orchestration component—more on `Block` below.

## Consensus
Transactions filter up from Parachains to the Relay Chain. At the very lowest level, Collators author new blocks, some of which are picked up by Parachain Validators. Of the validated set, one is added to the Relay Chain. Parachain nodes must know which of the many blocks authored by Collators was finalized to maintain consensus and then execute the block on their local storage.

By the time a Parachain block filters up to the Relay Chain, it has gone through two compressions. The first compression happens when a Collator authors a Proof of Validity block (PoVBlock) by bundling:

- List of state transitions
- Values in the Parachain’s database that the block modifies
- Hashes of the unaffected points in the Merkle tree

Together, a PoVBlock contains witness data for Validators to apply their registered state transition function (recall the WASM binary) and verify the outcome of a block's state transitions. Similarly, when Validators author blocks for the Relay Chain, they further compress their data into Candidate Receipts.


![Figure 1](/cumulus/docs/images/cumulus-figure-1.png)
**Figure 1:** Flow from Collator block to PoVBlock to Candidate Receipt


Parachains maintain consensus with the Relay Chain by executing the finalized blocks secured on the Relay Chain. But to do this, Parachains must decipher and reconstruct blocks from their compressed versions and change state values. Once a block is added, the Relay Chain signals the finalized Candidate Receipt back to Parachain Validators. Validators then apply the registered state transition function and reverse engineer the PoVBlock that Collators initially compressed. This reconstructed PoVBlock is passed back to Parachain nodes via the Cumulus macro call (recall the `Block` type referenced in [Registration](#Registration)). 

Using the `validate_block` function, implemented as a pallet, Cumulus constructs a partial trie from the PoVBlock witness data and runs through the same series of checks Parachain Validators do in Figure 1. Once state transitions have been validated, Cumulus executes the block by redirecting all host-related storage functions to use values from the partial trie. 


![Figure 2](/cumulus/docs/images/figure 2.png)
**Figure 2:** Finalized Relay block to PoVBlock to Parachain block


When Validators call the `validate_block` function, they pass through the PoVBlock, but they also pass a hash of its on-chain parent header.  If a Parachain node receives a PoVBlock with a parent header it does not recognize, it will not execute the block. Instead, it will wait for other nodes to gossip the canonical state and execute blocks then. Including the parent header of the latest block deters illegitimate branches.

To recap:
1. Collators author blocks for Validators that contain a Proof of Validity of all state transitions within the block.
2. Validators apply their registered state transition function to verify the same result as the Proof.
3. Validators compress the PoVBlock into a Candidate Receipt for the Relay Chain.
4. Once executed, the Relay Chain passes the finalized Candidate Receipt back to Parachain Validators
5. Parachain Validators reconstruct the PoVBlock from the Candidate Receipt and pass it through the `valudate_block` function from Cumulus.
6. Parachains execute the `validate_block` function and execute the block, changing their state to match what is recognized on the Relay Chain.

### Where does Cumulus Come In?
Cumulus lets Substrate-based nodes maintain consensus with a Relay Chain. Importantly, Cumulus exposes an interface to Parachain Validators that lets it interpret and produce blocks that follow the fixed-formatting expected by the Polkadot Network. Furthermore, Cumulus uses these blocks as inputs to execute a crucial Parachain node function: Knowing which block to execute next.

## Block Building
Once a Parachain node is synchronous with its Relay Chain, it can begin the cycle again. Collators will start to bundle and broadcast new blocks to Parachain Validators. These blocks are the same as the PoVBlocks, containing state transitions, Merkle hashes, and state data. Only Collators can proffer new blocks. Light clients and other full nodes finish their process at a consensus.

Cumulus provides a client Collator library to convert Substrate FRAME nodes into Parachain Collators and the additional logic needed to produce new blocks. Production logic is triggered when Collators import PoVBlocks and validate a finalized state change, as described in [Consensus](#Consensus). Data is compressed further and further until one block is finalized on the Relay Chain, and the cycle continues.

## Runtime Upgrades
A core feature of Substrate is upgradeability. Substrate FRAME lets developers make forkless runtime upgrades. In other words, change a state transition function without requiring a coordinated client-side update. But Substrate runtime upgrades are enacted directly. Parachains have to provide notice to their Relay Chain about an upcoming runtime upgrade. 

Parachains have to wait for an arbitrary period of `X` blocks (as defined by the Relay Chain) before applying any runtime updates. The Relay Chain rejects blocks that fall short of this waiting period. Furthermore, after changing a runtime, Parachains must wait for another `Y` blocks before the next runtime upgrade. Cumulus provides all the logic to communicate runtime upgrades to the Relay Chain and follow configured waiting periods.

Recall that Parachain runtimes must register with Parachain Validators in the form of a compiled WASM binary. Updating that runtime upgrades mean compiling a new binary. For Cumulus Parachains, that includes the macro call in [Registration](#Registration). In other words, Cumulus Parachains have to re-register their runtime with Validators.

## Glossary 
**Substrate:** A flexible framework for building modular, efficient, and upgradeable blockchains. Substrate is written in the Rust programming language and maintained by Parity Technologies.

**Framework for Runtime Aggregation of Modularized Entities (FRAME):** A set of modules (called pallets) and support libraries that simplify runtime development.

**Validators:** Secure the Relay Chain by staking $DOT, validating proofs from Collators and participating in consensus with other Validators.

**Collators**: Special nodes that maintain parachains by aggregating parachain transactions into parachain block candidates and producing state transition proofs for validators based on those blocks.

**Relay Chain:** The central chain of Polkadot. All validators of Polkadot are staked on the Relay Chain in DOT and validate for the Relay Chain.

**Cumulus:** A set of tools used to create Polkadot compatible Parachains using the Substrate development framework.

**Candidate Receipt:** Parachain Validators construct a Candidate Receipt from a PoVBlock that gets added to the Relay Chain.
