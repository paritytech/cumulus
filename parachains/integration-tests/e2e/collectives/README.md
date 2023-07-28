E2E tests concerning Polkadot Governance and the Collectives Parachain. The tests run by the Parachain Integration Tests [tool](https://github.com/paritytech/parachains-integration-tests/).

## Requirements
The tests require some changes to the regular production runtime builds:

RelayChain runtime:
1. Alice has SUDO
2. Public Referenda `StakingAdmin`, `FellowshipAdmin` tracks settings:
``` yaml
prepare_period: 5 Block,
decision_period: 1 Block,
confirm_period: 1 Block,
min_enactment_period: 1 Block,
```
Collectives runtime:
1. Fellowship Referenda `Fellows` track settings:
``` yaml
prepare_period: 5 Block,
decision_period: 1 Block,
confirm_period: 1 Block,
min_enactment_period: 1 Block,
```
