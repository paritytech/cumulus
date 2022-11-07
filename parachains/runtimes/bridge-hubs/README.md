# Bridge-hubs Parachain

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

### Prepare/Build/Deploy
```
# Prepare empty directory for testing
mkdir -p ~/local_bridge_testing/bin
mkdir -p ~/local_bridge_testing/logs


# 1. Build polkadot binary
git clone https://github.com/paritytech/polkadot.git
cd polkadot
cargo build --release
cp target/release/polkadot ~/local_bridge_testing/bin/polkadot

# 2. Build cumulus polkadot-parachain binary
cd <cumulus-git-repo-dir>
cargo build --release --locked -p polkadot-parachain@0.9.230
cp target/release/polkadot-parachain ~/local_bridge_testing/bin/polkadot-parachain

# 3. Build substrate-relay binary
git clone https://github.com/paritytech/parity-bridges-common.git
cd parity-bridges-common
cargo build -p substrate-relay
cp target/release/substrate-relay ~/local_bridge_testing/bin/substrate-relay
```

### Run chains (Rococo + BridgeHub, Wococo + BridgeHub) from `./scripts/bridges_rococo_wococo.sh`

```
./scripts/bridges_rococo_wococo.sh stop

./scripts/bridges_rococo_wococo.sh start-rococo
# TODO: check log and activate parachain manually

./scripts/bridges_rococo_wococo.sh start-wococo
# TODO: check log and activate parachain manually
```

### Run relayers (Rococo, Wococo)

**Accounts of BridgeHub parachains:**
- `Bob` is pallet owner of all bridge pallets
- `Alice` is `Sudo`

**1. Init bridges**

Need to wait for parachain activation, then run:

```
./scripts/bridges_rococo_wococo.sh init-ro-wo
./scripts/bridges_rococo_wococo.sh init-wo-ro
```

or

```
# Rococo -> Wococo
RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
	~/local_bridge_testing/bin/substrate-relay init-bridge rococo-to-bridge-hub-wococo \
	--source-host localhost \
	--source-port 48943 \
	--target-host localhost \
	--target-port 8945 \
	--target-signer //Bob

# Wococo -> Rococo
RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
	~/local_bridge_testing/bin/substrate-relay init-bridge wococo-to-bridge-hub-rococo \
	--source-host localhost \
	--source-port 48945 \
	--target-host localhost \
	--target-port 8943 \
	--target-signer //Bob
```

**2. Relay (Grandpa relay-chain) headers**

**source-host/source-port** - WS-port of collator's inner RelayChain validator
**target-host/target-port** - WS-port of BridgeHub collator

```
# Rococo -> Wococo
RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
	~/local_bridge_testing/bin/substrate-relay relay-headers rococo-to-bridge-hub-wococo \
	--source-host localhost \
	--source-port 48943 \
	--target-host localhost \
	--target-port 8945 \
	--target-signer //Bob \
	--target-transactions-mortality=4

# Wococo -> Rococo
RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
	~/local_bridge_testing/bin/substrate-relay relay-headers wococo-to-bridge-hub-rococo \
	--source-host localhost \
	--source-port 48945 \
	--target-host localhost \
	--target-port 8943 \
	--target-signer //Bob \
	--target-transactions-mortality=4
```

**Check parachain collators:**
- Rococo parachain:
  - https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A8943#/chainstate
  - Pallet: **bridgeWococoGrandpa**
  - Keys: **bestFinalized()**
- Wococo parachain:
	- https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A8945#/chainstate
	- Pallet: **bridgeRococoGrandpa**
	- Keys: **bestFinalized()**

**3. Relay (BridgeHub parachain) headers**

```
# Rococo -> Wococo
RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
	~/local_bridge_testing/bin/substrate-relay relay-parachains bridge-hub-rococo-to-bridge-hub-wococo \
	--source-host localhost \
	--source-port 48943 \
	--target-host localhost \
	--target-port 8945 \
	--target-signer //Bob \
	--target-transactions-mortality=4

# Wococo -> Rococo
RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
	~/local_bridge_testing/bin/substrate-relay relay-parachains bridge-hub-wococo-to-bridge-hub-rococo \
	--source-host localhost \
	--source-port 48945 \
	--target-host localhost \
	--target-port 8943 \
	--target-signer //Bob \
	--target-transactions-mortality=4
```

**Check parachain collators:**
- Rococo parachain:
	- https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A8943#/chainstate
	- Pallet: **bridgeWococoParachain**
	- Keys: **bestParaHeads()**
- Wococo parachain:
	- https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A8945#/chainstate
	- Pallet: **bridgeRococoParachain**
	- Keys: **bestParaHeads()**

**4. Relay (XCM) messages**

TODO:

---

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

----

###### TODO: fix zombienet ports as bridges_rococo_wococo.sh, because networks colide and interfere by default, because of autodiscovery on localhost

###### Run chains (Rococo + BridgeHub, Wococo + BridgeHub) with Zombienet

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
