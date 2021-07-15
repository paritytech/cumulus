# Cumulo -- Nimbus ⛈️

Nimbus is a framework for building parachain consensus systems on [cumulus](https://github.com/paritytech/cumulus)-based parachains.

Given the regular six-second pulse-like nature of the relay chain, it is natural to think about slot-
based consensus algorithms for parachains. The parachain network is responsible for liveness and
decetralization and the relay chain is responsible for finality. There is a rich design space for such
algorithms, yet some tasks are common to all (or most) of them. These common tasks include:

* Signing and signature checking blocks
* Injecting authorship information into the parachain
* Block authorship and import accounting
* Filtering a large (potentially unbounded) set of potential authors to a smaller (but still potentially unbounded) set.
* Detecting when it is your turn to author an skipping other slots

Nimbus aims to provide standard implementations for the logistical parts of such consensus engines,
along with helpful traits for implementing the parts that researchers and developers want to customize.

## Try the Demo

While Nimbus is primarily a development framework meant to be included in other projects, it is useful
to see a basic network in action. An example network is included in the `polkadot-parachains` example collator. You
can build it with `cargo build --release` and launch it like any other cumulus parachian.
Make sure to specify `--chain nimbus`.

Rather than reiterate how to start a relay-para network here, I'll simply recommend you use the
excellent [Polkadot Launch](https://github.com/paritytech/polkadot-launch) tool. This repo was tested with version 1.4.1.
A [lauch config file](./nimbus-launch-config.json) is provided.

```bash
# Install polkadot launch (I used v1.4.1)
npm i -g polkadot-launch

# Build polkadot (I used 82aa404c; check Cargo.lock to be sure)
cd polkadot
cargo build --release
cd ..

# Build Polkadot-parachains example collator
cd cumulus
git checkout nimbus
cargo build --release

# Launch the multi-chain
polkdot-launch ./nimbus-launch-config.json
```

To learn more about launching relay-para networks, check out the [cumulus workshop](https://substrate.dev/cumulus-workshop).

## Design Overview

If you want to start using Nimbus in your project, it is worth reading this.

At its core nimbus is a consensus engine that considers blocks valid if and only if they inject the author's public identity into the runtime, _and_ seal the block with a signature
by the author's private key.

Compared to most consensus engines, this is _very_ permissive -- anyone who can create a signature can author valid blocks. In order to build more useful and familiar consensus engine on this foundation, nimbus provides a framework for creating filters to further restrict the set of eligible authors. These filters live inside the runtime.

Being general in the consensus layer and deferring most checks to the runtime is the key
to nimbus's re-usability as a framework. And is the reason that *writing a consensus engine is as easy as writing a pallet* when you use nimbus.

### Author Inherent

The Author inherent pallet allows block authors to insert their identity into
the runtime. This feature alone is useful in many blockchains and can be used for things like block rewards.

The author inherent provides a validation hook called `CanAuthor`. This check will be called during the inherent execution and is the main entry point to nimbus's author filters.
If you don't want to restrict authorship at all, you can just use `()`.

As a concrete example, in a simple Proof of Stake system this check will determine
whether the author is staked. In a more realistic PoS system the `CanAuthor` check might
first make sure the author is staked, and then make sure they are eligible in _this slot_ according to round robin rules.

Finally, the pallet copies the authorship information into a consensus digest that will stick around
in the block header. This digest can be used by UIs to display the author, and also by the consensus
engine to verify the block authorship.

**PreRuntimeDigest**
I believe the design should be changed slightly to use a preruntime digest rather than an inherent for a few reasons:

* The data wouldn't be duplicated between an inherent and a digest.
* Nimbus client-side worker would support non-frame runtimes.
* That's how sc-consensus-aura does it.

### Author Filters

A primary job of a consensus engine is deciding who can author each block. Some may have a static set, others
may rotate the set each era, others may elect an always-changing subset of all potential authors. There
is much space for creativity, research, and design, and Nimbus strives to provide a flexible interface
for this creative work. You can express all the interesting parts of your
consensus engine simply by creating filters that implement the `CanAuthor` trait. The rest of Nimubs will #JustWork for you.

This repository comes with a few example filters already, and additional examples are welcome. The examples are:
* PseudoRandom FixedSized Subset - This filter takes a finite set (eg a staked set) and filters it down to a pseudo-random
subset at each height. The eligible ratio is configurable in the pallet. This is a good learning example.
* Aura - The authority round consensus engine is popular in the Substrate ecosystem because it was one
of the first (and simplest!) engines implemented in Substrate. Aura can be expressed in the Nimbus
filter framework and is included as an example filter. If you are considering using aura, that crate
has good documentation on how it differs from `sc-consensus-aura`.
* (Planned) FixedSizedSubset - The author submits a VRF output that has to be below a threshold to be able to author.
* (Planed) Filter Combinator - A filter that wraps two other filters. It uses one in even slots and the other in odd slots.

### Author Filter Runtime API

Nimbus makes the design choice to include the author checking logic in the runtime. This is in contrast to the existing implementations of Aura and Babe where the authorship checks are offchain.

While moving the check in-runtime, provides a lot of flexibility, and simplifies interfacing with relay-chain validators, it makes it impossible
for authoring nodes to predict whether they will be eligible without calling into the runtime.
To achieve this, we provide a runtime API that makes the minimal calculation necessary to determine
whether a specified author will be eligible at the specified slot.

### Nimbus Consensus Worker

Nimbus consensus is the primary client-side consensus worker. It implements the `ParachainConsensus`
trait introduced to cumulus in https://github.com/paritytech/cumulus/pull/329. It is not likely that
you will need to change this code directly to implement your engine as it is entirely abstracted over
the filters you use. The consensus engine performs these tasks:

* Slot prediction - it calls the runtime API mentioned previously to determine whether ti is eligible. If not, it returns early.
* Authorship - It calls into a standard Substrate proposer to construct a block (probably including the author inherent).
* Self import - it imports the block that the proposer created (called the pre-block) into the node's local database.
* Sealing - It adds a seal digest to the block - This is what is used by other nodes to verify the authorship information.

### Verifier and Import Queue

For a parachain node to import a sealed block authored by one of its peers, it needs to first check that the signature is valid by the author that was injected into the runtime. This is the job of the verifier. It
will remove the nimbus seal and check it against the nimbus consensus digest from the runtime. If that process fails,
the block is immediately thrown away before the expensive execution even begins. If it succeeds, then
the pre-block (the part that's left after the seal is stripped) is passed into the
[import pipeline](https://substrate.dev/docs/en/knowledgebase/advanced/block-import) for processing
and execution. Finally, the locally produced result is compared to the result received across the network.

### Custom Block Executor

We've already discussed how parachain nodes (both the one that authors a block, and also its peers)
import blocks. In a standalone blockchain, that's the end of the story. But for a parachain, we also
need our relay chain validators to re-execute and validate the parachain block. Validators do this in
a unique way, and entirely in wasm. Providing the `validate_block` function that the validators use
is the job of the `register_validate_block!` macro from Cumulus.

Typically a cumulus runtime invokes that macro like this:
```rust
cumulus_pallet_parachain_system::register_validate_block!(Runtime, Executive);
```

You can see that the validators use the exact same executive that the parachain nodes do. Now that
we have sealed blocks, that must change. The validators need to strip and verify the seal, and re-execute
the pre-block just like the parachain nodes did. And without access to an offchain verifier, they must
do this all in the runtime. For that purpose, we provide and alternate executive which wraps the normal
FRAME executive. The wrapper strips and checks the seal, just like the verifier did, and then passes the pre-block to the inner FRAME executive for re-execution.

## Write Your Own Consensus Logic

If you have an idea for a new slot-based parachain consensus algorithm, Nimbus is a quick way to get
it working! The fastest way to start hacking is to fork this repo and customize the template node.

If you'd rather dive in than read one more sentence, then **start hacking in the `author-slot-filter`
pallet.**

In most cases, you can use all the off-the-shelf components and simply write your filters. It is also
possible to compose existing filters to build more complex logic from smaller pieces. 

## Authoring and Import Diagrams

One node authors the block, then it is processed in three different ways.

|                     | Author | Parachain Peer | Relay Validator |
| ------------------- | ------ | -------------- | --------- |
| Predict Eligibility |    ✅   |    ❌          |    ❌      |
| Author Block        |    ✅   |    ❌          |    ❌      |
| Runs Verifier       |    ❌   |    ✅          |    ❌      |
| Import Pipeline     |    ✅   |    ✅          |    ❌      |
| Custom Pre exec     |    ❌   |    ❌          |    ✅      |
| Normal FRAME exec   |    ✅   |    ✅          |    ✅      |

## Roadmap

The Nimbus framework is intended to be loosely coupled with Cumulus. It remains to be
seen whether it should live with Cumulus or in its own repository.

### Next tasks
* Proper trait for interacting with digests
* More example filters
* Share code between verifier and wrapper executive
* Client-side worker for standalone (non para) blockchain
* Aurand as an example of composing filters
* Second filter trait for exhaustive sets (As opposed to current propositional approach)

## Contributions Welcome

Try it out, open issues, submit PRs, review code. Whether you like to tinker with a running node, or
analyze security from an academic perspective, your contributions are welcome.

I am happy to support users who want to use nimbus, or want feedback on their consensus engines.
