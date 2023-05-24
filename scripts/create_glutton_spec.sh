#!/usr/bin/env bash

#  G7Z5mTmTQsjEGBVqVGDZyR9m7RoHNZJk6JeykyfKQ3vmBiR
usage() {
    echo Usage:
    echo "$0 <binary path> <relay chain> <parachain id> <sudo key>"
    exit 1
}

set -e

binary_path=$1
relay_chain=$2
id="glutton_$2_$3"
protocol_id="glutton_$2_$3"
para_id=$3
sudo=$4

[ -z "$binary_path" ] && usage
[ -z "$relay_chain" ] && usage
[ -z "$id" ] && usage
[ -z "$protocol_id" ] && usage
[ -z "$para_id" ] && usage
[ -z "$sudo" ] && usage

# binary="./target/release/polkadot-parachain"

# build the chain spec we'll manipulate
$binary_path build-spec --disable-default-bootnode --chain "glutton-kusama-genesis-$para_id" > "plain-glutton-$para_id-$relay_chain-spec.json"

# convert runtime to hex
# cat $runtime_path | od -A n -v -t x1 |  tr -d ' \n' > seedling-hex.txt

# replace the runtime in the spec with the given runtime and set some values to production
cat "plain-glutton-$para_id-$relay_chain-spec.json" \
    | jq --arg id $id '.id = $id' \
    | jq --arg id $id '.protocolId = $id' \
    | jq --arg relay_chain $relay_chain '.relay_chain = $relay_chain' \
    | jq --argjson para_id $para_id '.para_id = $para_id' \
    | jq --arg sudo $sudo '.genesis.runtime.sudo.key = $sudo' \
    | jq --argjson para_id $para_id '.genesis.runtime.parachainInfo.parachainId = $para_id' \
    > glutton-$para_id-$relay_chain-spec.json

# build a raw spec
$binary_path build-spec --disable-default-bootnode --chain "glutton-$para_id-$relay_chain-spec.json" --raw > "glutton-$para_id-$relay_chain-raw-spec.json"

# build genesis data
$binary_path export-genesis-state --chain "glutton-$para_id-$relay_chain-raw-spec.json" > "glutton-$para_id-$relay_chain-head-data"

# build genesis wasm
$binary_path export-genesis-wasm --chain "glutton-$para_id-$relay_chain-raw-spec.json" > "glutton-$para_id-$relay_chain-validation-code"
