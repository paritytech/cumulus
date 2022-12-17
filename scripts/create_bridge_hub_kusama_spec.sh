#!/usr/bin/env bash

usage() {
    echo Usage:
    echo "$1 <srtool compressed runtime path>"
    echo "$2 <para_id>"
    echo "e.g.: ./scripts/create_bridge_hub_kusama_spec.sh ./target/release/wbuild/bridge-hub-kusama-runtime/bridge_hub_kusama_runtime.compact.compressed.wasm 1003"
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
$binary build-spec --chain bridge-hub-kusama-dev > chain-spec-plain.json

# convert runtime to hex
cat $rt_path | od -A n -v -t x1 |  tr -d ' \n' > rt-hex.txt

# replace the runtime in the spec with the given runtime and set some values to production
cat chain-spec-plain.json | jq --rawfile code rt-hex.txt '.genesis.runtime.system.code = ("0x" + $code)' \
    | jq '.name = "Kusama BridgeHub"' \
    | jq '.id = "bridge-hub-kusama"' \
    | jq '.chainType = "Live"' \
    | jq '.bootNodes = []' \
    | jq '.relay_chain = "kusama"' \
    | jq --argjson para_id $para_id '.para_id = $para_id' \
    | jq --argjson para_id $para_id '.genesis.runtime.parachainInfo.parachainId = $para_id' \
    | jq '.genesis.runtime.balances.balances = []' \
    | jq '.genesis.runtime.collatorSelection.invulnerables = [
                                                                "DQkekNBt8g6D7bPUEqhgfujADxzzfivr1qQZJkeGzAqnEzF",
                                                                "HbUc5qrLtKAZvasioiTSf1CunaN2SyEwvfsgMuYQjXA5sfk",
                                                                "JEe4NcVyuWFEwZe4WLfRtynDswyKgvLS8H8r4Wo9d3t61g1",
                                                                "FAe4DGhQHKTm35n5MgBFNBZvyEJcm7QAwgnVNQU8KXP2ixn"
                                                             ]' \
    | jq '.genesis.runtime.session.keys = [
                                            [
                                                "DQkekNBt8g6D7bPUEqhgfujADxzzfivr1qQZJkeGzAqnEzF",
                                                "DQkekNBt8g6D7bPUEqhgfujADxzzfivr1qQZJkeGzAqnEzF",
                                                    {
                                                        "aura": "DQkekNBt8g6D7bPUEqhgfujADxzzfivr1qQZJkeGzAqnEzF"
                                                    }
                                            ],
                                            [
                                                "HbUc5qrLtKAZvasioiTSf1CunaN2SyEwvfsgMuYQjXA5sfk",
                                                "HbUc5qrLtKAZvasioiTSf1CunaN2SyEwvfsgMuYQjXA5sfk",
                                                    {
                                                        "aura": "HbUc5qrLtKAZvasioiTSf1CunaN2SyEwvfsgMuYQjXA5sfk"
                                                    }
                                            ],
                                            [
                                                "JEe4NcVyuWFEwZe4WLfRtynDswyKgvLS8H8r4Wo9d3t61g1",
                                                "JEe4NcVyuWFEwZe4WLfRtynDswyKgvLS8H8r4Wo9d3t61g1",
                                                    {
                                                        "aura": "JEe4NcVyuWFEwZe4WLfRtynDswyKgvLS8H8r4Wo9d3t61g1"
                                                    }
                                            ],
                                            [
                                                "FAe4DGhQHKTm35n5MgBFNBZvyEJcm7QAwgnVNQU8KXP2ixn",
                                                "FAe4DGhQHKTm35n5MgBFNBZvyEJcm7QAwgnVNQU8KXP2ixn",
                                                    {
                                                        "aura": "FAe4DGhQHKTm35n5MgBFNBZvyEJcm7QAwgnVNQU8KXP2ixn"
                                                    }
                                            ]
                                          ]' \
    > edited-chain-spec-plain.json

# build a raw spec
$binary build-spec --chain edited-chain-spec-plain.json --raw > chain-spec-raw.json
cp chain-spec-raw.json ./parachains/chain-specs/bridge-hub-kusama.json

# build genesis data
$binary export-genesis-state --chain chain-spec-raw.json > bridge-hub-kusama-genesis-head-data

# build genesis wasm
$binary export-genesis-wasm --chain chain-spec-raw.json > bridge-hub-kusama-wasm
