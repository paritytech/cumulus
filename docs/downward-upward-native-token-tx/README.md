# Upward/ Downward native-token transfers
Working example for `polkadot-v0.9.13` to transfer the native token from the relay-chain to the parachain
 via the `xcm-pallet` and vice-versa. The message format is very generic, hence the correct extrinsics for those transfers are shown below.

### Preliminaries
The relay chain must have marked encointer as a trusted teleporter, which is not yet the case. This polkadot fork contains
slightly amended westend and rococo runtimes, where encointer is a trusted teleporter.

* Polkadot relay chain: https://github.com/encointer/polkadot/tree/v0.9.13-trusted-encointer

* For debugging it greatly helps to pass `-lxcm=trace` to the collator. a subsequent `less 9944.log | grep xcm`, neatly 
  displays all XCM steps.
  
## Downwards Tx
Send from Alice on the relay chain to Ferdie on the parachain.

![downward-token-tx.png](downward-token-tx.png)

## Upwards Token Tx
Send from Alice on the parachain to Ferdie on the relay chain.

**Note**: The relay chain tracks teleported assets. Hence, we must perform a downward transfer before we do the upward
transfer such that the total token stream in parachain direction remains positive. Otherwise, we get a `NotWithdrawable` 
error upon the upward transfer on the relay chain side. In production system this condition always holds as the 
parachain starts with 0 tokens anyhow.


![upward-token-tx.png](downward-token-tx.png)

# Related Resources
* Shawn Tabrizi's XCM [workshop](https://www.youtube.com/watch?v=5cgq5jOZx9g&list=PLp0_ueXY_enX6S5sogeo7GHD1BXTLj5Xr&index=20) 
  at sub0 and the corresponding [resources](https://www.shawntabrizi.com/xcm-workshop/#/reserve-transfer).
