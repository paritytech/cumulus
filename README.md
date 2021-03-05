# Cumulo -- Nimbus ⛈️

Nimbus is a set of tools for building and testing parachain consensus systems on [cumulus](https://github.com/paritytech/cumulus)-based parachains.

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

While Nimbus is primarily a developer toolkit meant to be included in other projects, it is useful
to see a basic node in action. An example node is included in the `rococo-parachains` directory. You
can build it with `cargo build --release` and launch it like any other cumulus parachian.

Rather than reiterate how to start a relay-para network here, I'll simply recommend you use the
excellent [Polkadot Launch](https://github.com/paritytech/polkadot-launch) tool.

TODO, provide a config file in this repo.

To learn more about launching relay-para networks, check out the [cumulus workshop](https://substrate.dev/cumulus-workshop)

## Design Overview

This section gives an overview of the architecture and design decisions in Nimubs. If you want to start
using Nimbus in your project, it is worth reading this.

### Author Inherent

The Author inherent pallet provides an inherent in which block authors can insert their identity into
the runtime. It also includes an inherent data provider for the client-side node to create the appropriate
inherents. This feature alone is useful in many blockchains and can be used for things like block rewards.
This piece can be used alone without any other parts of nimbus if all you desire is authoring information
in the runtime.

The author inherent provides two hooks called `PreliminaryCanAuthor` and `FullCanAuthor`. The
preliminary check will be run by non-authoring nodes before importing a block to quickly rule out some
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

### Author Filters

Many consensus engines will want to restrict which authors can author. Some may have a static set, others
may rotate the set each era, others may elect an always-changing subset of all potential authors. There
is much space for creativity, research, and design, and Nimbus strives to provide a flexible interface
for this creative work. In the majority of cases, you can express all the interesting parts of your
consensus engine simply by implementing the correct filters. The rest of Nimubs will #JustWork for you.

The repository comes with a few example filters already. The examples are:
* Height filter - This filter takes a finite set (eg a staked set) and filters it down to a pseudo-random
subset at each height. The eligible ratio is configurable in the pallet. This is a good learning example
but not secure, because the authors only rotate at each parachain block. Therefore a small colluding
minority of authors could stall the chain.
* Slot filter - Similar to the height filter, but it uses entropy (currently just the block height,
but in the future the relay randomness beacon) to change the elected subset each time the relay chain
authors a block even if the parachain doesn't author. This avoids the stall mentioned previously.
* Aura - The authority round consensus engine is popular in the Substrate ecosystem because it was one
of the first (and simplest!) engines implemented in Substrate. Aura can be expressed in the Nimbus
filter framework and is included as an example filter.

### Author Filter Runtime API

Nimbus makes the design choice to include the final author checking logic in the runtime. This makes
it relatively straight forward (but still not entirely straight forward; read on) for relay chain validators
to check the authorship information. This is in contrast to the existing implementation of Aura and Babe
where the authorship checks are offchain. Because the checks happen in the runtime, authoring nodes would
need to author complete blocks to learn whether they are eligible. This is inefficient, and leads to
confusing logs for users.

Authoring nodes (collators) should be smart enough to predict whether they will be eligible, and not
author at all if they are ineligible. To achieve this, we provide a runtime API that will allow collator
nodes to call into the runtime, and make the minimal calculation necessary to determine whether they will
be eligible. If they aren't they skip the slot saving energy and keeping the logs free of confusing errors.

### Nimbus Consensus
(currently called filtering consensus in the code. TODO change that)

Nimbus consensus is the primary client-side consensus worker. It implements the shiny new `ParachainConsensus`
trait introduced to cumulus in https://github.com/paritytech/cumulus/pull/329 It is not likely that
you will need to change this code directly to implement your engine as it is entirely abstracted over
the filters you use. The consensus engine performs these tasks:

* Slot prediction - it calls the runtime API mentioned previously to determine whether ti is eligible. If not, it returns early.
* Authorship - It calls into a standard Substrate proposer to construct a block (probably including the author inherent).
* Self import - it imports the block that the proposer created (called the pre-block) into the node's local database.
* Sealing - It adds a seal digest to the block - This is what is used by other nodes to verify the authorship information.
**Warning, the seal does not yet conatain a cryptographic signature; it is just mocked. That is one of the next tasks.**

### Verifier and Import Queue

For a parachain node to import a sealed block authored by one of its peers, it needs to remove the post-
runtime seal digest and check it (same in a standalone blockchain). This is the job of the verifier. It
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

The Nimbus tools are intended to be standalone and loosely coupled with Cumulus. That goal is realistic
in the medium term (~ a month or two) but during the initial development, it is easier to work as a fork
of the upstream cumulus repository.

Once the core features are complete, and the line is more clear between what belongs in cumulus vs
nimbus, the repos can be separated.

### Core Features Needed
* Author info in seal, and basic seal checking
* Keystore integration
* Cryptographic block signatures
* Filter pallet refactor -- What is common and what needs re-implemented by each author.

### Additional features for maybe someday
* Share code between verifier and wrapper executive
* Maybe moonbeam's dev service should live with nimbus??
* Aurand as an example of composing filters?
* Two different filter traits?
  * Current one: Propositional logic style (good for unbounded sets)
  * New one: Returns concrete author sets (good for index-based stuff like aura)

## Contributions Welcome

Try it out, open issues, submit PRs, review code. Whether you like to tinker with a running node, or
analyze security from an academic perspective, your contributions are welcome.

I would be happy to support users who want to use nimbus, or want feedback on their consensus engines.
