# Chainlink Price Feed Module

This pallet provides Chainlink price feed functionality to Substrate-based chains that include it.

It aims to mostly port the [FluxAggregator](https://github.com/smartcontractkit/chainlink/blob/dabada25f5dd7bbc49a76ed1d172a83083cdd8f0/evm-contracts/src/v0.6/FluxAggregator.sol)
Solidity contract but changes the design in the following ways:
+ **Storage Pruning:** It is usually not advisable to keep all oracle state around, so we add storage pruning logic
  to the pallet. (See the `prune` extrinsic.)
+ **View Functions:** Substrate does not have an exact equivalent of Solidity view functions. We will instead make the
  client side smarter to read and interpret storage to surface the same information.
+ **Contracts --> Feeds:** Vanilla Substrate does not have contracts. We will thus replace the deployment of a contract with
  the creation of per-feed metadata and a storage prefix.
  - Oracle metadata (admin and withdrawable funds) will be tracked globally (for the whole pallet) per oracle.
+ **On-Chain Interaction:** In Substrate runtimes, pallets interact with other pallets differently from how users interact
  with the pallet. We thus define two interfaces for interaction with the pallet:
  - Rust traits for pallets (`FeedOracle` as well as `FeedInterface` and `MutableFeedInterface`)
  - Extrinsics for users of the chain
+ **Token (Separation of Concerns):** Most of the interaction with the LINK token will happen via a dedicated token pallet
  and thus the token-based interaction with the feed pallet will be minimal.
  (Token changes are done via the configured type implementing the `Currency` trait.)
  - We do introduce a `Debt` storage item to track how much the pallet owes to oracles in cases where the reserves are insufficient.

## Integration into Chain
Include the pallet in your runtime and configure it.
The following snippet shows an example:

```Rust
parameter_types! {
    // Used to generate the fund account that pools oracle payments.
	pub const FeedModule: ModuleId = ModuleId(*b"linkfeed");
    // The minimum amount of tokens to keep in reserve for oracle payment.
	pub const MinimumReserve: Balance = ExistentialDeposit::get() * 1000;
    // Maximum length of the feed description.
	pub const StringLimit: u32 = 30;
    // Maximum number of oracles per feed.
	pub const OracleCountLimit: u32 = 25;
    // Maximum number of feeds.
	pub const FeedLimit: FeedId = 100;
    // Minimum amount of rounds to keep when pruning.
	pub const PruningWindow: RoundId = 15;
}

impl pallet_chainlink_feed::Trait for Runtime {
	type Event = Event;
	type FeedId = FeedId;
	type Value = Value;
    // A module that provides currency functionality to manage
    // oracle rewards. Balances in this example.
	type Currency = Balances;
	type ModuleId = FeedModule;
	type MinimumReserve = MinimumReserve;
	type StringLimit = StringLimit;
	type OracleCountLimit = OracleCountLimit;
	type FeedLimit = FeedLimit;
	type PruningWindow = PruningWindow;
    // Implementation of the WeightInfo trait for your runtime.
    // Default weights available in the pallet but not recommended for production.
	type WeightInfo = ChainlinkWeightInfo;
}
```

## Usage in a Pallet
You need to inject the pallet into the consuming pallet in a similar way to how the feed pallet
depends on a pallet implementing the `Currency` trait.
Define an associated `Oracle` type implementing the `FeedOracle` trait.

```Rust
pub trait Trait: frame_system::Trait {
    // -- snip --
    type Oracle: FeedOracle<Self>;
}
```

You can then access a feed by calling the `feed` and `feed_mut` functions in your pallet code:
```Rust
let feed = T::Oracle::feed(0.into()).ok_or(Error::<T>::FeedMissing)?;
let RoundData { answer, .. } = feed.latest_data();
do_something_with_answer(answer);
```

## Architecture

### Storage
Most storage items in the pallet are scoped under a feed identified by an id.
The storage layout follows the following structure.

Scoped to a feed:
```
FeedId => FeedConfig
(FeedId, RoundId) => Round
(FeedId, RoundId) => RoundDetails
(FeedId, requester: AccountId) => Requester
(FeedId, oracle_acc: AccountId) => OracleStatus
```
Associated with an account:
```
oracle_acc: AccountId => OracleMeta
feed_creator: AccountId => ()
```
Pallet-global values:
```
PalletAdmin
PendingPalletAdmin
Debt
FeedCounter
```

### Interaction
Access to a feed is done via the `Feed` type which automatically syncs changes to storage on drop
(if the `should_sync` flag is set). This makes it harder to forgot to update the storage with changes
but means that care should be taken with scoping the variable. (E.g. the feed needs to be initialized
*within* the closure passed to `with_transaction_result` in order for the auto-sync writes to be
covered by the transactional write.)
