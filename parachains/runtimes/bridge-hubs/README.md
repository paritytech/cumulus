# Bride-hubs Parachain

Implementation of _BridgeHub_, a blockchain to support message passing between Substrate based chains like Polkadot and Kusama networks.

_BridgeHub_ allows users to:

- Passing arbitrary messages between different Substrate chains (Polkadot <-> Kusama).
-- Message passing is

Every _BridgeHub_ is meant to be **_common good parachain_** with main responsibilities:
- sync finality proofs between relay chains
- sync finality proofs between BridgeHub parachains
- pass (XCM) messages between different BridgeHub parachains

![](./docs/bridge-hub-parachain-design.jpg "Basic deployment setup")

## How to test locally Rococo <-> Wococo

### Build/Deploy
```
# Prepare empty directory for testing
mkdir -p ~/local_bridge_testing/bin
mkdir -p ~/local_bridge_testing/logs


# 1. Build polkadot binary
TODO: description

# 2. Build cumulus polkadot-parachain binary
cd <cumulus-git-repo-dir>
cargo build --release --locked -p polkadot-parachain@0.9.230
cp target/release/polkadot-parachain ~/local_bridge_testing/bin/polkadot-parachain

# 3. Build substrate-relay binary
git clone  https://github.com/paritytech/parity-bridges-common.git
cd parity-bridges-common
cargo build -p substrate-relay
rm ~/local_bridge_testing/bin/substrate-relay
cp target/release/substrate-relay ~/local_bridge_testing/bin/substrate-relay
```

### Run chains (Rococo + BridgeHub, Wococo + BridgeHub) with Zombienet

```
# Rococo
POLKADOT_BINARY_PATH=~/local_bridge_testing/bin/polkadot \
	POLKADOT_PARACHAIN_BINARY_PATH=~/local_bridge_testing/bin/polkadot-parachain \
	~/local_bridge_testing/bin/zombienet-linux --provider native spawn ./zombienet_tests/0004-run_bridge_hubs_rococo.toml

# Wococo
POLKADOT_BINARY_PATH=~/local_bridge_testing/bin/polkadot \
	POLKADOT_PARACHAIN_BINARY_PATH=~/local_bridge_testing/bin/polkadot-parachain \
	~/local_bridge_testing/bin/zombienet-linux --provider native spawn ./zombienet_tests/0004-run_bridge_hubs_wococo.toml
```

### Run chains (Rococo + BridgeHub, Wococo + BridgeHub) from `cmd`

#### Run relay chains (Rococo, Wococo)
```
<build polkadot repo at first and copy binary to ~/local_bridge_testing/bin/ >

# Rococo
~/local_bridge_testing/bin/polkadot build-spec --chain rococo-local --disable-default-bootnode --raw > ~/local_bridge_testing/rococo-local-cfde.json
~/local_bridge_testing/bin/polkadot --chain ~/local_bridge_testing/rococo-local-cfde.json --alice --tmp --port 30332 --rpc-port 9932 --ws-port 9942
~/local_bridge_testing/bin/polkadot --chain ~/local_bridge_testing/rococo-local-cfde.json --bob --tmp --port 30333 --rpc-port 9933 --ws-port 9943
~/local_bridge_testing/bin/polkadot --chain ~/local_bridge_testing/rococo-local-cfde.json --charlie --tmp --port 30334 --rpc-port 9934 --ws-port 9944

# Wococo
~/local_bridge_testing/bin/polkadot build-spec --chain wococo-local --disable-default-bootnode --raw > ~/local_bridge_testing/wococo-local-cfde.json
~/local_bridge_testing/bin/polkadot --chain ~/local_bridge_testing/wococo-local-cfde.json --alice --tmp --port 30335 --rpc-port 9935 --ws-port 9945
~/local_bridge_testing/bin/polkadot --chain ~/local_bridge_testing/wococo-local-cfde.json --bob --tmp --port 30336 --rpc-port 9936 --ws-port 9946
~/local_bridge_testing/bin/polkadot --chain ~/local_bridge_testing/wococo-local-cfde.json --charlie --tmp --port 30337 --rpc-port 9937 --ws-port 9947
~/local_bridge_testing/bin/polkadot --chain ~/local_bridge_testing/wococo-local-cfde.json --dave --tmp --port 30338 --rpc-port 9938 --ws-port 9948
```
We need at least 5 nodes together (validator + collator) to finalize blocks.

#### Run BridgeHub parachains (Rococo, Wococo)

**1. Generate spec + genesis + wasm (paraId=1013)**
```
# Rococo
rm ~/local_bridge_testing/bridge-hub-rococo-local-raw.json
~/local_bridge_testing/bin/polkadot-parachain build-spec --chain bridge-hub-rococo-local --raw --disable-default-bootnode > ~/local_bridge_testing/bridge-hub-rococo-local-raw.json
~/local_bridge_testing/bin/polkadot-parachain export-genesis-state --chain ~/local_bridge_testing/bridge-hub-rococo-local-raw.json > ~/local_bridge_testing/bridge-hub-rococo-local-genesis
~/local_bridge_testing/bin/polkadot-parachain export-genesis-wasm --chain ~/local_bridge_testing/bridge-hub-rococo-local-raw.json > ~/local_bridge_testing/bridge-hub-rococo-local-genesis-wasm

# Wococo
rm ~/local_bridge_testing/bridge-hub-wococo-local-raw.json
~/local_bridge_testing/bin/polkadot-parachain build-spec --chain bridge-hub-wococo-local --raw --disable-default-bootnode > ~/local_bridge_testing/bridge-hub-wococo-local-raw.json
~/local_bridge_testing/bin/polkadot-parachain export-genesis-state --chain ~/local_bridge_testing/bridge-hub-wococo-local-raw.json > ~/local_bridge_testing/bridge-hub-wococo-local-genesis
~/local_bridge_testing/bin/polkadot-parachain export-genesis-wasm --chain ~/local_bridge_testing/bridge-hub-wococo-local-raw.json > ~/local_bridge_testing/bridge-hub-wococo-local-genesis-wasm
```

**2. Run collators (Rococo, Wococo)**
```
# Rococo
~/local_bridge_testing/bin/polkadot-parachain --collator --alice --force-authoring --tmp --port 40333 --rpc-port 8933 --ws-port 8943 --chain ~/local_bridge_testing/bridge-hub-rococo-local-raw.json -- --execution wasm --chain ~/local_bridge_testing/rococo-local-cfde.json --port 41333 --rpc-port 48933 --ws-port 48943
~/local_bridge_testing/bin/polkadot-parachain --collator --bob --force-authoring --tmp --port 40334 --rpc-port 8934 --ws-port 8944 --chain ~/local_bridge_testing/bridge-hub-rococo-local-raw.json -- --execution wasm --chain ~/local_bridge_testing/rococo-local-cfde.json --port 41334 -rpc-port 48934 --ws-port 48944

# Wococo
~/local_bridge_testing/bin/polkadot-parachain --collator --alice --force-authoring --tmp --port 40335 --rpc-port 8935 --ws-port 8945 --chain ~/local_bridge_testing/bridge-hub-wococo-local-raw.json -- --execution wasm --chain ~/local_bridge_testing/wococo-local-cfde.json --port 41335 --rpc-port 48935 --ws-port 48945
~/local_bridge_testing/bin/polkadot-parachain --collator --bob --force-authoring --tmp --port 40336 --rpc-port 8936 --ws-port 8946 --chain ~/local_bridge_testing/bridge-hub-wococo-local-raw.json -- --execution wasm --chain ~/local_bridge_testing/wococo-local-cfde.json --port 41336 --rpc-port 48936 --ws-port 48946
```

**3. Activate parachains (Rococo, Wococo) (paraId=1013)**
```
# Rococo
https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9942#/explorer

# Wococo
https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9945#/explorer
```

**4. After parachain activation, we should see new blocks in collator's node**
```
https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/parachains
```

### Run relayers (Rococo, Wococo)

**Accounts of BridgeHub parachains:**
- `Bob` is pallet owner of all bridge pallets
- `Alice` is `Sudo`

**1. Init bridges**
```
# Rococo -> Wococo
RUST_LOG=runtime=trace,rpc=trace,runtime::bridge=trace \
	 ~/local_bridge_testing/bin/substrate-relay init-bridge bridge-hub-rococo-to-bridge-hub-wococo \
	--source-host localhost \
	--source-port 48943 \
	--target-host localhost \
	--target-port 8945 \
	--target-signer //Bob

# Wococo -> Rococo
RUST_LOG=runtime=trace,rpc=trace,runtime::bridge=trace \
	 ~/local_bridge_testing/bin/substrate-relay init-bridge bridge-hub-wococo-to-bridge-hub-rococo \
	--source-host localhost \
	--source-port 48945 \
	--target-host localhost \
	--target-port 8943 \
	--target-signer //Bob
```

**2. Relay headers**

TODO:

**2. Relay (Grandpa relay-chain) headers**

TODO:

**3. Relay (BridgeHub parachain) headers**

TODO:

**4. Relay (XCM) messages**

TODO:

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
