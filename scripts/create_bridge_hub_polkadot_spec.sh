#!/usr/bin/env bash

usage() {
    echo Usage:
    echo "$1 <srtool compressed runtime path>"
    echo "$2 <para_id>"
    echo "e.g.: ./scripts/create_bridge_hub_polkadot_spec.sh ./target/release/wbuild/bridge-hub-polkadot-runtime/bridge_hub_polkadot_runtime.compact.compressed.wasm 1002"
    exit 1
}

if [ -z "$1" ]; then
    usage
fi

if [ -z "$2" ]; then
    usage
fi

set -e

rt_path=$1
para_id=$2

echo "Generating chain spec for runtime: $rt_path and para_id: $para_id"

binary="./target/release/polkadot-parachain"

# build the chain spec we'll manipulate
$binary build-spec --chain bridge-hub-polkadot-dev > chain-spec-plain.json

# convert runtime to hex
cat $rt_path | od -A n -v -t x1 |  tr -d ' \n' > rt-hex.txt

# replace the runtime in the spec with the given runtime and set some values to production
# TODO: missing .bootNodes
# TODO: missing .genesis.runtime.collatorSelection.invulnerables
# TODO: missing .genesis.runtime.session.keys
cat chain-spec-plain.json | jq --rawfile code rt-hex.txt '.genesis.runtime.system.code = ("0x" + $code)' \
    | jq '.name = "Polkadot BridgeHub"' \
    | jq '.id = "bridge-hub-polkadot"' \
    | jq '.chainType = "Live"' \
    | jq '.bootNodes = [

                       ]' \
    | jq '.relay_chain = "polkadot"' \
    | jq --argjson para_id $para_id '.para_id = $para_id' \
    | jq --argjson para_id $para_id '.genesis.runtime.parachainInfo.parachainId = $para_id' \
    | jq '.genesis.runtime.balances.balances = []' \
    | jq '.genesis.runtime.collatorSelection.invulnerables = [
                                                                "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                                                                "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"
                                                             ]' \
    | jq '.genesis.runtime.session.keys = [
                                              [
                                                "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                                                "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                                                {
                                                  "aura": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
                                                }
                                              ],
                                              [
                                                "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
                                                "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
                                                {
                                                  "aura": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"
                                                }
                                              ]
                                          ]' \
    > edited-chain-spec-plain.json

# build a raw spec
$binary build-spec --chain edited-chain-spec-plain.json --raw > chain-spec-raw.json
cp edited-chain-spec-plain.json bridge-hub-polkadot-spec.json
cp chain-spec-raw.json ./parachains/chain-specs/bridge-hub-polkadot.json
cp chain-spec-raw.json bridge-hub-polkadot-spec-raw.json

# build genesis data
$binary export-genesis-state --chain chain-spec-raw.json > bridge-hub-polkadot-genesis-head-data

# build genesis wasm
$binary export-genesis-wasm --chain chain-spec-raw.json > bridge-hub-polkadot-wasm

# cleanup
rm -f rt-hex.txt
rm -f chain-spec-plain.json
rm -f chain-spec-raw.json
rm -f edited-chain-spec-plain.json
