# Konomi node

A substrate based node for DeFi innovation.

## Main Processes
### Supply
The user can choose to supply the certain currency to the allowed pool. To calculate the interest, there are many models that one can use. The current model is as follows:

When user supplies certain amount, the interest rate is determined at time of deposit with the following equation:
```
UtilizationRatio = TotalBorrow / TotalSupply
SupplyInterestRate = BorrowInterestRate * UtilizationRatio
```
You can see that the `SupplyInterestRate` is derived from `BorrowInterestRate`. This is because if there are no one paying the interest from borrowing, then there is no interest for supplying. The derivation of `BorrowInterestRate` is mentioned below. The system would reject a transaction if the amount is more than the amount owned by the user. Once all the checks are passed, the protocol would transfer the asset amount from user to the pool.

To calculate the user interest, conceptually, we are performing as follows:
```json
# At the current block, the interest would be
Interest = SupplyInterestRate * TotalUserSupply * (CurrentBlockNumber - LastUpdatedBlockNumber)
LastUpdatedBlockNumber = CurrentBlockNumber
```
Aside from the principle the user has supplied to the pool, user could earn the `Interest` derived above. Please note that, the interest rates are converted to per block.

### Withdraw
To withdraw from the pool, the system would perform several checks to ensure the validity of the attempted transaction.

First of all, the user can only withdraw up to his/her supply + interest.If the asset is one of the account's collateral, the system would ensure liquidation process would not be triggered. Liquidation would be triggered if the total collateral in USD is lower than a certain threshold of the total borrowed in USD. The detailed process is described in Liquidation section.

If all the checks have passed, the asset would be transferred from the pool to user account. And the amount withdraw would deducted from the asset pool.

### Borrow
First of all, if there are no collateral supplied, borrow cannot happen. Compared to supply, user must repay the interest generated from the borrowing. The interest rate is determined at time of deposit with the following equation:
```
UtilizationRatio = TotalBorrow / TotalSupply
BorrowInterestRate = InitialInterestRate + UtilizationRatio * UtilizationFactor
Interest = InterestRate * TotalUserBorrow * (CurrentBlockNumber - LastUpdatedBlockNumber)
```
Here, `InitialInterestRate` is the starting interest rate when there are no borrow from the pool. `BorrowInterestRate` referes to the current time borrow interest rate. `UtilizationFactor` is a constant multiplier for the utilization ratio. The choice of `InitialInterestRate`, `UtilizationFactor` are described in section 3 of https://medium.com/konomi/in-depth-analysis-of-konomi-collateral-and-liquidation-model-e81fb885f8bb. The value of the parameters can be updated during runtime, but only admin is allowed to perform the updates. You can refer to the pallet `FloatingRateLend` for the admin related operations.

One thing to note is that, borrow should not have triggered liquidation. If that happens, the transaction is aborted.

Once all the checks are passed, the protocol would transfer the asset amount from user to the pool.

### Repay
User must repay their borrow principal and the interest.

### Liquidation
Liquidation will be triggered when the total collateral is lower than the total borrowed, specifically in the following equation:
![equations/liquidation_0](equations/liquidation_0.png)

In the above equations, `SafeFactor` is a measurement of how safe the currency is. If the currency is stable, then the value would be closer to 1. Otherwise closer to 0. `ExchangeRate` is the exchange rate of the currency to usd, one can regard this as the price. When the following condition is reached, the account would be marked as `Liquidable`. Arbitrageurs would be able liquidate the account.

![equations/liquidation_1](equations/liquidation_1.png)

The current value of `LiquidationThreshold` is 1.

To avoid liquidation, ensure the `HealthIndex` is always above the `LiquidationThreshold`.

### Arbitrage
Arbitrageurs would be able to supply to the borrowed asset pool of the liquidated account. For every single transaction, the arbitrageurs can only purchase up to `CloseFactor` of the assets. When arbitrageurs purchase from the liquidated account, the amount paid would be deducted from the liquidated account's asset borrowed and would go back to the pool. The equivalent amount of the collateral would be transferred from the liquidated account to the arbitrageurs's account.

## Pallets
- Currencies: A multiple currencies implementation. Currently in develop for cross chain features, a quick demo here: https://www.youtube.com/watch?v=etn5nd7JNPQ
- FloatingRateLend: The floating interest rate lend implemenation
- ChainlinkOracle: Chainlink adaptor for live price

## Local Development

Follow these steps to prepare a local Substrate development environment:

### Build
To build the parachain, use the command: `cargo build --release`.

### Run
#### Relay Chain
Start the relay chain as follows:
```bash
# Compile Polkadot with the real overseer feature
git clone https://github.com/paritytech/polkadot
git fetch
git checkout rococo-v1
cargo build --release

# For the chain spec, please use the chain spec in the konomi cumulus folder: rococo-single-custom.json
# Alice
./target/release/polkadot --chain <path to rococo-single-custom.json> --alice --tmp

# Bob (In a separate terminal)
./target/release/polkadot --chain <path to rococo-single-custom.json> --bob --tmp --port 30334
```

Start the collators
```bash
# Collator1
./target/release/polkadot-collator --collator --tmp --alice --force-authoring --parachain-id 18403 --port 40335 --ws-port 9946 --rpc-methods Unsafe --ws-external --rpc-cors all -- --execution wasm --chain <path to rococo-single-custom.json> --port 30335

# Collator2
./target/release/polkadot-collator --collator --tmp --bob --force-authoring --parachain-id 18403 --port 40336 --ws-port 9947 --rpc-methods Unsafe --ws-external --rpc-cors all -- --execution wasm --chain <path to rococo-single-custom.json> --port 30336

# Parachain Full Node 1
./target/release/polkadot-collator --tmp --parachain-id 18403 --port 40337 --ws-port 9948 --rpc-port 9929 --rpc-methods Unsafe --ws-external --rpc-cors all -- --execution wasm --chain <path to rococo-single-custom.json> --port 30337
```

There is no need to register the parachain on the relay chain as it is already in the chain spec. Alternatively, you can connect to our parachain testnet at: `wss://parachain.konomi.tech/parachain` on polkadot js app.
