# Bridge-hubs Parachain

Implementation of _BridgeHub_, a blockchain to support message passing between Substrate based chains like Polkadot and Kusama networks.

_BridgeHub_ allows users to:

- Passing arbitrary messages between different Substrate chains (Polkadot <-> Kusama).
  -- Message passing is

_BridgeHub_ is meant to be **_system parachain_** with main responsibilities:
- sync finality proofs between relay chains
- sync finality proofs between BridgeHub parachains
- pass (XCM) messages between different BridgeHub parachains

![](./docs/bridge-hub-parachain-design.jpg "Basic deployment setup")

## How to try locally
```
cd <base-cumulus-repo-directory>
cargo build --release -p polkadot-parachain@0.9.320

# script expect to have pre-built polkadot binary on the path: ../polkadot/target/release/polkadot
# if using `kusama-local` / `polkadot-local`, build polkadot with `--features fast-runtime`

# BridgeHubRococo
zombienet-linux --provider native spawn ./zombienet/examples/bridge_hub_rococo_local_network.toml

# or

# BridgeHubKusama
zombienet-linux --provider native spawn ./zombienet/examples/bridge_hub_kusama_local_network.toml

or

# BridgeHubPolkadot
zombienet-linux --provider native spawn ./zombienet/examples/bridge_hub_polkadot_local_network.toml
```

----
## Git subtree `./bridges`

Add Bridges repo as a local remote and synchronize it with latest `master` from bridges repo:

### How to update `bridges` subtree
```
cd <cumulus-git-repo-dir>
# this will update new git branches from bridges repo
# there could be unresolved conflicts, but dont worry,
# lots of them are caused because of removed unneeded files with patch step :)
# so before solving conflicts just run patch
./scripts/bridges_update_subtree.sh fetch
# this will remove unneeded files and checks if subtree modules compiles
./scripts/bridges_update_subtree.sh patch
# if there are conflicts, this could help, removes locally deleted files at least
# (but you can also do this manually)
./scripts/bridges_update_subtree.sh merge
# when conflicts resolved, you can check build again - should pass
# also important: this updates global Cargo.lock
./scripts/bridges_update_subtree.sh patch
````
We use `--squash` to avoid adding individual commits and rather squashing them
all into one.
Now we use `master` branch, but in future, it could change to some release branch/tag.

### How was first time initialized (does not need anymore)
```
cd <cumulus-git-repo-dir>
git remote add -f bridges git@github.com:paritytech/parity-bridges-common.git
# (ran just only first time, when subtree was initialized)
git subtree add --prefix=bridges bridges master --squash
# remove unnecessery files
./scripts/bridges_update_subtree.sh patch
./scripts/bridges_update_subtree.sh merge
git commit --amend
```
