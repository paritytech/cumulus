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

# 1. Install zombienet
Go to: https://github.com/paritytech/zombienet/releases
Copy the apropriate binary (zombienet-linux) from the latest release to ~/local_bridge_testing/bin

# 2. Build polkadot binary
git clone https://github.com/paritytech/polkadot.git
cd polkadot
cargo build --release
cp target/release/polkadot ~/local_bridge_testing/bin/polkadot

# 3. Build cumulus polkadot-parachain binary
cd <cumulus-git-repo-dir>
cargo build --release --locked -p polkadot-parachain@0.9.300
cp target/release/polkadot-parachain ~/local_bridge_testing/bin/polkadot-parachain

# 4. Build substrate-relay binary
git clone https://github.com/paritytech/parity-bridges-common.git
cd parity-bridges-common
cargo build --release -p substrate-relay
cp target/release/substrate-relay ~/local_bridge_testing/bin/substrate-relay
```

### Run chains (Rococo + BridgeHub, Wococo + BridgeHub) with zombienet

```
# Rococo + BridgeHubRococo + Rockmine (mirroring Kusama)
POLKADOT_BINARY_PATH=~/local_bridge_testing/bin/polkadot \
POLKADOT_PARACHAIN_BINARY_PATH=~/local_bridge_testing/bin/polkadot-parachain \
POLKADOT_PARACHAIN_BINARY_PATH_FOR_ROCKMINE=~/local_bridge_testing/bin/polkadot-parachain \
	~/local_bridge_testing/bin/zombienet-linux --provider native spawn ./zombienet/bridge-hubs/bridge_hub_rococo_local_network.toml
```

```
# Wococo + BridgeHubWococo + Wockmint (mirroring Polkadot)
POLKADOT_BINARY_PATH=~/local_bridge_testing/bin/polkadot \
POLKADOT_PARACHAIN_BINARY_PATH=~/local_bridge_testing/bin/polkadot-parachain \
POLKADOT_PARACHAIN_BINARY_PATH_FOR_WOCKMINT=~/local_bridge_testing/bin/polkadot-parachain \
	~/local_bridge_testing/bin/zombienet-linux --provider native spawn ./zombienet/bridge-hubs/bridge_hub_wococo_local_network.toml
```

### Run relayers (Rococo, Wococo)

**Accounts of BridgeHub parachains:**
- `Bob` is pallet owner of all bridge pallets
- `Alice` is `Sudo`

**1. Init bridges**

Need to wait for parachain activation (start producing blocks), then run:

```
# Rococo -> Wococo
RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
	~/local_bridge_testing/bin/substrate-relay init-bridge rococo-to-bridge-hub-wococo \
	--source-host localhost \
	--source-port 9942 \
	--target-host localhost \
	--target-port 8945 \
	--target-signer //Bob

# Wococo -> Rococo
RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
	~/local_bridge_testing/bin/substrate-relay init-bridge wococo-to-bridge-hub-rococo \
	--source-host localhost \
	--source-port 9945 \
	--target-host localhost \
	--target-port 8943 \
	--target-signer //Bob
```

**2. Relay relay-chain headers, parachain headers and messages**

```
RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
    ~/local_bridge_testing/bin/substrate-relay relay-headers-and-messages bridge-hub-rococo-bridge-hub-wococo \
    --rococo-host localhost \
    --rococo-port 9942 \
    --bridge-hub-rococo-host localhost \
    --bridge-hub-rococo-port 8943 \
    --bridge-hub-rococo-signer //Charlie \
    --wococo-headers-to-bridge-hub-rococo-signer //Bob \
    --wococo-parachains-to-bridge-hub-rococo-signer //Bob \
    --bridge-hub-rococo-transactions-mortality 4 \
    --wococo-host localhost \
    --wococo-port 9945 \
    --bridge-hub-wococo-host localhost \
    --bridge-hub-wococo-port 8945 \
    --bridge-hub-wococo-signer //Charlie \
    --rococo-headers-to-bridge-hub-wococo-signer //Bob \
    --rococo-parachains-to-bridge-hub-wococo-signer //Bob \
    --bridge-hub-wococo-transactions-mortality 4 \
    --lane 00000001 \
    --lane 00000002
```

**Check relay-chain headers relaying:**
- Rococo parachain:
	- https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A8943#/chainstate
	- Pallet: **bridgeWococoGrandpa**
	- Keys: **bestFinalized()**
- Wococo parachain:
	- https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A8945#/chainstate
	- Pallet: **bridgeRococoGrandpa**
	- Keys: **bestFinalized()**

**Check parachain headers relaying:**
- Rococo parachain:
	- https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A8943#/chainstate
	- Pallet: **bridgeWococoParachain**
	- Keys: **bestParaHeads()**
- Wococo parachain:
	- https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A8945#/chainstate
	- Pallet: **bridgeRococoParachain**
	- Keys: **bestParaHeads()**

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
