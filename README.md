# Cumulo -- Nimbus ⛈️

Nimbus is a framework for building slot-based parachain consensus systems on [cumulus](https://github.com/paritytech/cumulus)-based parachains.

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
to see a basic network in action. An example node is included in the `rococo-parachains` directory. You
can build it with `cargo build --release` and launch it like any other cumulus parachian.

Rather than reiterate how to start a relay-para network here, I'll simply recommend you use the
excellent [Polkadot Launch](https://github.com/paritytech/polkadot-launch) tool. This repo was tested with version 1.4.1.
A [lauch config file](./nimbus-launch-config.json) is provided.

```bash
# Install polkadot launch (I used v1.4.1)
npm i -g polkadot-launch

# Build polkadot (I used 127eb17a; check Cargo.lock to be sure)
cd polkadot
git checkout rococo-v1
cargo build --release
cd ..

# Build Nimbus example node (written against 2d13f4ad)
cd cumulus
git checkout nimbus
cargo build --release

<<<<<<< HEAD
# Launch the multi-chain
polkdot-launch ./nimbus-launch-config.json
```

To learn more about launching relay-para networks, check out the [cumulus workshop](https://substrate.dev/cumulus-workshop).

## Design Overview

This section gives an overview of the architecture and design decisions in Nimubs. If you want to start
using Nimbus in your project, it is worth reading this.

### Author Inherent

The Author inherent pallet allows block authors to insert their identity into
the runtime. It also includes an inherent data provider for the client-side node to create the appropriate
inherents. This feature alone is useful in many blockchains and can be used for things like block rewards.
This piece can be used alone without any other parts of nimbus if all you desire is authoring information
in the runtime.

The author inherent provides two validation hooks called `PreliminaryCanAuthor` and `FullCanAuthor`. The
preliminary check will be run before importing a block to quickly rule out some
invalid authors. This check should be cheap and should not depend on other runtime state. The full
check will be called during the inherent execution and is the final say for whether an author is eligible.
The preliminary check may rule out authors, but does not guarantee eligibility. Said another way, it
may give false positives, but never false negatives. These two hooks are both optional. If you don't
want to restrict authorship at all, you can just use `()`.

As a concrete example, in a Proof of Stake system the preliminary check may make sure that the claimed
author is staked at all, and if not, they are obviously ineligible. Then the full check will determine
whether the author is eligible at this particular slot. It is possible to be staked without being eligible
at this slot. Because the slot-eligibility depends on other runtime state, that check must be deferred.

Finally, the pallet copies the authorship information into a consensus digest that will stick around
in the block header. This digest can be used by UIs to display the author, and also by the consensus
engine to verify the block authorship.

**Security Notice** If you use the author inhernet alone, the authorship information is not proven,
rather it is a claim by the block author. It is entirely possible that a dishonest node may insert
someone else's identity via the author inherent. In some cases (famously bitcoin), this is all that's
desired. When used in this mode it is more common to use the term "beneficiary" than "author". If you
want your authorship information to be proven, you will need to use more parts of Nimbus; read on.

**Feedback Requested** I've occasionally heard and wondered myself whether this would work better as a
pre-runtime digest than an inherent. I'm open to that change. I build it as an inherent because I don't
clearly understand the distinction between the two, and I'm more familiar with inherents.

### Author Filters

A primary job of a consensus engine is deciding who can author each block. Some may have a static set, others
may rotate the set each era, others may elect an always-changing subset of all potential authors. There
is much space for creativity, research, and design, and Nimbus strives to provide a flexible interface
for this creative work. In the majority of cases, you can express all the interesting parts of your
consensus engine simply by implementing the correct filters. The rest of Nimubs will #JustWork for you.

The repository comes with a few example filters already, and additional examples are welcome. The examples are:
* Height filter - This filter takes a finite set (eg a staked set) and filters it down to a pseudo-random
subset at each height. The eligible ratio is configurable in the pallet. This is a good learning example
but not secure, because the authors only rotate at each parachain block. Therefore a small colluding
minority of authors could stall the chain.
* Slot filter - Similar to the height filter, but it uses entropy (currently just the block height,
but in the future the relay randomness beacon) to change the elected subset each time the relay chain
authors a block even if the parachain doesn't author. This avoids the stall mentioned previously.
* Aura - The authority round consensus engine is popular in the Substrate ecosystem because it was one
of the first (and simplest!) engines implemented in Substrate. Aura can be expressed in the Nimbus
filter framework and is included as an example filter. If you are considering using aura, that crate
has good documentation on how it differs from `sc-consensus-aura`.

### Author Filter Runtime API

Nimbus makes the design choice to include the author checking logic in the runtime. This makes
it relatively straight forward (but still not entirely straight forward; at least until 
https://github.com/paritytech/polkadot/issues/2888 is resolved) for relay chain validators
to check the authorship information. This is in contrast to the existing implementations of Aura
and Babe where the authorship checks are offchain.

While moving the check in-runtime simplifies interfacing with validators, it makes in impossible
for authoring nodes to predict whether they will be eligible without calling into the runtime.
To achieve this, we provide a runtime API that makes the minimal calculation necessary to determine
 whether a specified author  will be eligible at the specified slot.

### Nimbus Consensus Worker

Nimbus consensus is the primary client-side consensus worker. It implements the shiny new `ParachainConsensus`
trait introduced to cumulus in https://github.com/paritytech/cumulus/pull/329. It is not likely that
you will need to change this code directly to implement your engine as it is entirely abstracted over
the filters you use. The consensus engine performs these tasks:

* Slot prediction - it calls the runtime API mentioned previously to determine whether ti is eligible. If not, it returns early.
* Authorship - It calls into a standard Substrate proposer to construct a block (probably including the author inherent).
* Self import - it imports the block that the proposer created (called the pre-block) into the node's local database.
* Sealing - It adds a seal digest to the block - This is what is used by other nodes to verify the authorship information.

**Warning, the seal does not yet contain a cryptographic signature; it is just mocked. That is one of the next tasks.**

### Verifier and Import Queue

For a parachain node to import a sealed block authored by one of its peers, it needs to remove the post-
runtime seal digest and check its signature (same in a standalone blockchain). This is the job of the verifier. It
will remove the nimbus seal and compare it to the nimbus consensus digest from the runtime. If that process fails,
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
=======
```
cargo build --release -p polkadot-collator
```

Once the executable is built, launch collators for each parachain (repeat once each for chain
`tick`, `trick`, `track`):

```
./target/release/polkadot-collator --chain $CHAIN --validator
>>>>>>> polkadot-v0.9.1
```

You can see that the validators use the exact same executive that the parachain nodes do. Now that
we have sealed blocks, that must change. The validators need to strip and verify the seal, and re-execute
the pre-block just like the parachain nodes did. And without access to an offchain verifier, they must
do this all in the runtime. For that purpose, we provide and alternate executive which wraps the normal
FRAME executive. The wrapper strips and checks the seal, just like the verifier did, and then and then
passes the pre-block to the inner FRAME executive for re-execution.

## Write Your Own Consensus Logic

If you have an idea for a new slot-based parachain consensus algorithm, Nimbus is a quick way to get
it working! The fastest way to start hacking is to fork this repo and customize the template node.

If you'd rather dive in than read one more sentence, then **start hacking in the `author-slot-filter`
pallet.**

In most cases, you can use all the off-the-shelf components and simply write your filters. It is also
possible to compose existing filters to build more complex logic from smaller pieces. (composition
interface is subject to change though.)

## Authoring and Import Diagrams

TODO, these should help explain the various processes and how they are coordinated.

The gist: One node authors the block, then it is processed in three different ways.

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

### Core Features Needed
* Cryptographic block signatures and checking
* Update the CanAuthor trait to take a slot number, and make a seperate trait for a slot beacon

<<<<<<< HEAD
### Additional features for maybe someday
* Share code between verifier and wrapper executive
* Client-side worker for standalone (non para) blockchain
* Aurand as an example of composing filters
* Second filter trait for exhaustive sets (As opposed to current propositional approach)

## Contributions Welcome

Try it out, open issues, submit PRs, review code. Whether you like to tinker with a running node, or
analyze security from an academic perspective, your contributions are welcome.

I would be happy to support users who want to use nimbus, or want feedback on their consensus engines.
=======
# Export genesis state
# --parachain-id 200 as an example that can be chosen freely. Make sure to everywhere use the same parachain id
./target/release/polkadot-collator export-genesis-state --parachain-id 200 > genesis-state

# Export genesis wasm
./target/release/polkadot-collator export-genesis-wasm > genesis-wasm

# Collator1
./target/release/polkadot-collator --collator --tmp --parachain-id <parachain_id_u32_type_range> --port 40335 --ws-port 9946 -- --execution wasm --chain ../polkadot/rococo-local-cfde.json --port 30335

# Collator2
./target/release/polkadot-collator --collator --tmp --parachain-id <parachain_id_u32_type_range> --port 40336 --ws-port 9947 -- --execution wasm --chain ../polkadot/rococo-local-cfde.json --port 30336

# Parachain Full Node 1
./target/release/polkadot-collator --tmp --parachain-id <parachain_id_u32_type_range> --port 40337 --ws-port 9948 -- --execution wasm --chain ../polkadot/rococo-local-cfde.json --port 30337
```
### Register the parachain
![image](https://user-images.githubusercontent.com/2915325/99548884-1be13580-2987-11eb-9a8b-20be658d34f9.png)
>>>>>>> polkadot-v0.9.1
