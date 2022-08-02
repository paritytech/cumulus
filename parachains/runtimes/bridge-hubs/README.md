# Bride-hubs Parachain

Implementation of _BridgeHub_, a blockchain to support message passing between Substrate based chains like Polkadot and Kusama networks.

_BridgeHub_ allows users to:

- Passing arbitrary messages between different Substrate chains (Polkadot <-> Kusama).
-- Message passing is

Every _BridgeHub_ is meant to be **_common good parachain_** with main responsibilities:
- sync finality proofs between relay chains
- sync finality proofs between BridgeHub parachains
- pass (XCM) messages between different BridgeHub parachains

![](/home/bparity/parity/cumulus/cumulus/parachains/runtimes/bridge-hubs/docs/bridge-hub-parachain-design.jpg "Basic deployment setup")

## How to test locally Rococo <-> Wococo

### Deploy
```
cd <cumulus-git-repo-dir>
cargo build --release --locked -p polkadot-parachain@0.9.220

mkdir -p ~/local_bridge_testing/bin

rm ~/local_bridge_testing/bin/polkadot-parachain
cp target/release/polkadot-parachain ~/local_bridge_testing/bin/polkadot-parachain
ls -lrt ~/local_bridge_testing/bin/polkadot-parachain
```

### Run relay chain (Rococo)
```
<build polkadot repo at first and copy binary to ~/local_bridge_testing/bin/ >

~/local_bridge_testing/bin/polkadot build-spec --chain rococo-local --disable-default-bootnode --raw > ~/local_bridge_testing/rococo-local-cfde.json
~/local_bridge_testing/bin/polkadot --chain ~/local_bridge_testing/rococo-local-cfde.json --alice --tmp
~/local_bridge_testing/bin/polkadot --chain ~/local_bridge_testing/rococo-local-cfde.json --bob --tmp --port 30334
```

### Run Rococo BridgeHub parachain
#### Generate spec + genesis + wasm (paraId=1013)
```
rm ~/local_bridge_testing/bridge-hub-rococo-local-raw.json
~/local_bridge_testing/bin/polkadot-parachain build-spec --chain bridge-hub-rococo-local --raw --disable-default-bootnode > ~/local_bridge_testing/bridge-hub-rococo-local-raw.json
~/local_bridge_testing/bin/polkadot-parachain export-genesis-state --chain ~/local_bridge_testing/bridge-hub-rococo-local-raw.json > ~/local_bridge_testing/bridge-hub-rococo-local-genesis
~/local_bridge_testing/bin/polkadot-parachain export-genesis-wasm --chain ~/local_bridge_testing/bridge-hub-rococo-local-raw.json > ~/local_bridge_testing/bridge-hub-rococo-local-genesis-wasm
```

#### Run collators
```
~/local_bridge_testing/bin/polkadot-parachain --collator --alice --force-authoring --tmp --port 40335 --ws-port 9946 --chain ~/local_bridge_testing/bridge-hub-rococo-local-raw.json -- --execution wasm --chain ~/local_bridge_testing/rococo-local-cfde.json --port 30335
~/local_bridge_testing/bin/polkadot-parachain --collator --bob --force-authoring --tmp --port 40336 --ws-port 9947 --chain ~/local_bridge_testing/bridge-hub-rococo-local-raw.json -- --execution wasm --chain ~/local_bridge_testing/rococo-local-cfde.json --port 30336
```

#### Run parachain node
```
~/local_bridge_testing/bin/polkadot-parachain --tmp --port 40337 --ws-port 9948 --chain ~/local_bridge_testing/bridge-hub-rococo-local-raw.json -- --execution wasm --chain ~/local_bridge_testing/rococo-local-cfde.json --port 30337
```

#### Activate parachain (paraId=1013)
```
https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer
```

#### After parachain activation, we should see new blocks in collator's node
```
https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/parachains
```


## Git subtree `./bridges`

Add Bridges repo as a local remote and synchronize it with latest `master` from bridges repo:
```
git remote add -f bridges git@github.com:paritytech/parity-bridges-common.git
# (ran just only first time, when subtree was initialized)
# git subtree add --prefix=bridges bridges master --squash

# Synchro bridges repo
git fetch bridges --prune
git subtree pull --prefix=bridges bridges master --squash
````
We use `--squash` to avoid adding individual commits and rather squashing them
all into one.

Now we use `master` branch, but in future, it could change to some release branch/tag.

Original `./bridges/Cargo.toml` was renamed to `./bridges/Cargo.toml_removed_for_bridges_subtree_feature` to avoid confusion for `Cargo` having multiple workspaces.
