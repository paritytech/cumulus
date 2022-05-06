# Integritee Parachain

This is the repository to run Integritee as a parachain on Kusama and Rococo. It is forked from the [Cumulus](https://github.com/paritytech/cumulus) repository.

## Launch a local setup including a Relay Chain and a Parachain

### Polkadot-launch

Install polkadot-launch according to the instruction in [polkadot-launch](https://github.com/paritytech/polkadot-launch#install)

and use it with

```
polkadot-launch launch-rococo-local-with-integritee.json
```

All polkadot launch scripts can be found in the [polkadot-launch](/polkadot-launch/) folder.

### Manually launch a local Rococo Testnet

Follow the steps provided in https://docs.substrate.io/tutorials/v3/cumulus/start-relay/

But keep the following in mind:
- Our chain has a paraid of 2015 on Rococo and Kusama (see [polkadot-parachains/src/command.rs#44-49](/polkadot-parachains/src/command.rs#44-49))
- For testing on rococo-local use the chain spec `integritee-rococo-local-dev`
- More chain specs can be found in [polkadot-parachains/src/command.rs](/polkadot-parachains/src/command.rs)

## Runtime upgrade
Two runtimes are contained in this repository. First, the shell-runtime, which has been extended compared to the upstream shell-runtime. It has some additional modules including sudo to facilitate a
runtime upgrade with the [sudo upgrade](https://substrate.dev/docs/en/tutorials/forkless-upgrade/sudo-upgrade) method. Second, it runs with the same executor instance as the integritee-runtime, such that an eventual upgrade is simpler to perform, i.e., only the runtime
needs to be upgraded whereas the client can remain the same. Hence, all modules revolving around aura have been included, which provide data the client needs.

### Upgrade procedure
Prepare a local shell network and generate the `integritee-runtime` wasm blob, which contains the upgraded runtime to be executed after the runtime upgrade.
```bash
# launch local setup
node ../polkadot-launch/dist/cli.js polkadot-launch/launch-rococo-local-with-shell.json

# generate wasm blob
 ./target/release/integritee-collator export-genesis-wasm --chain integritee-rococo-local-dev > integritee-rococo-local-dev.wasm
```

After the parachain starts producing blocks, a runtime upgrade can be initiated via the polkadot-js/apps interface.

![image](./docs/sudo-set-code.png)

If successful, a `parachainSystem.validationFunctionStored` event is thrown followed by a `parachainSystem.validationFunctionApplied` event some blocks later. After this procedure, the `Teerex` module should be available in the extrinsics tab in polkadot-js/apps.

### Caveats
* Don't forget to enable file upload if you perform drag and drop for the `genesisHead` and `validationCode`. If it is not enabled, Polkadot-js will interpret the path as a string and won't complain but the registration will fail.
* Don't forget to add the argument `--chain integritee-rococo-local-dev` for the custom chain config. This argument is omitted in the [Cumulus Workshop](https://substrate.dev/cumulus-workshop/).
* The relay chain and the collator need to be about equally recent. This might require frequent rebasing of this repository on the corresponding release branch.

## Benchmark
The current weights have been benchmarked with the following reference hardware:
* Core(TM) i7-10875H
* 32GB of RAM
* NVMe SSD

The benchmarks are run with the following script:

```shell
./scripts/benchmark_all_pallets.sh
```


## More Resources
* Thorough Readme about Rococo and Collators in general in the original [repository](https://github.com/paritytech/cumulus) of this fork.
* Parachains Development in the [Polkadot Wiki](https://wiki.polkadot.network/docs/build-pdk)
* encointer parachain readme: https://github.com/encointer/encointer-parachain
