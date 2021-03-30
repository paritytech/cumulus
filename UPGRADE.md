# upgrade rococo collator

in this repo:
```
git fetch upstream
git rebase upstream/rococo-v1
```

In case of conflicting Cargo.lock, make sure to use:
```
git checkout upstream/rococo-v1 Cargo.lock && git add Cargo.lock
```
then
```
cargo update
```

In case this fails: find out the latest commits for polkadot substrate and cumulus `rococo-v1` branches

cumulus: paritytech/cumulus@24b1ee6bd1d96f255889f167e59ef9c9399a6305
polkadot: paritytech/polkadot@2f7b975015d5c3f50199cda82b9b84e38726d001

substrate: 
    here you MUST use the last master commet, not the one on the rococo-v1 branch
    paritytech/substrate@a94749cb5321cbc43403ead66a1c915236720f8d

after successful rebase, fix dependencies:
```
cargo update -p sp-std --precise a94749cb5321cbc43403ead66a1c915236720f8d
cargo update -p xcm --precise 2f7b975015d5c3f50199cda82b9b84e38726d001
```
cargo update -p xcm --precise c257eaffe1bfd9f79d0fc78d8309cc072c12fd76

## upgrade validators

1. stop current validators
2. purge chain
3. 

rotate session keys

```
curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "author_rotateKeys", "params":[]}' http://localhost:9933
```


