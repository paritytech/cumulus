# Cumulus Overview

This document provides high-level documentation for Cumulus.

### What is Cumulus?
Cumulus is a [Parachain developer kit (PDK)](https://wiki.polkadot.network/docs/en/build-pdk) that provides a set of tools to create Polkadot compatible [Parachains](https://substrate.dev/docs/en/knowledgebase/getting-started/glossary#parachain) using the [Substrate development framework](http://substrate.dev/).

### Why would I use Cumulus?
Cumulus aims to bridge the gap between Substrate blockchains and Parachains, making it easy for developers to build Parachains using Substrate.

This distinction is important as not all Substrate blockhains are inherently compatible with the Polkadot [Relay Chain](https://substrate.dev/docs/en/knowledgebase/getting-started/glossary#relay-chain). Substrate serves a broader purpose beyond Polkadot by providing a flexible framework for building modular, efficient, and upgradeable blockchains.

### How does Cumulus Work?
Cumulus provides interfaces and extensions to convert a Substrate [FRAME](https://substrate.dev/docs/en/knowledgebase/getting-started/glossary#frame) runtime into a Parachain runtime.

## Runtime

The Substrate client contains a modular runtime component that consists of various modules called "[Pallets](https://substrate.dev/docs/en/knowledgebase/getting-started/glossary#pallet)." A sampling of these pallets include various consensus mechanisms, voting, staking, and [many more](https://substrate.dev/docs/en/knowledgebase/runtime/frame#prebuilt-pallets).

Combining different pallets within the Substrate runtime allows developers to modify the underlying logic that defines how each newly processed block changes the overall state of the blockchain. To register with the Polkadot Relay Chain, each Parachain must supply an external `validate_block` API function that Parachain validators can call.

Cumulus makes this process as easy as adding the following code to the bottom of your runtime:
```rust
cumulus_pallet_parachain_system::register_validate_block!(Block, Executive);
```

## Block Building
Once registered on the Polkadot network, the Parachain can now submit block candidates to the Relay Chain. Authority nodes, known as "[Collators](https://wiki.polkadot.network/docs/en/learn-collator)," are responsible for collating and executing transactions to create block candidates. Cumulus provides the block production logic that notifies each Collator of the Parachain to build a Parachain block. 

Along with the block candidate, Collators also present a cryptographic state transition proof to the Relay Chain. These "[Proof of Validity Blocks](https://wiki.polkadot.network/docs/en/glossary#proof-of-validity)" along with the Parachain registry (recall the runtime macro call) allows validators to validate the state transition presented in the block candidates.

When the Parachain validator calls the `validate_block` function, the function will pass the PoVBlock and the parent geader of the Parachain (read as: registry). Using the PoVBlock's proof, Cumulus can reconstruct a partial trie. This partial trie is used as storage while executing the block. After the setup is done, Cumulus calls the `execute_block` function with the transactions and the header stored in the PoVBlock. On success, the new Parachain header is returned as part of the `validate_block` result, indicating the Relay Chain has successfully recognized the block candidate.
