#!/usr/bin/env bash

# Example for `compute` and `storage` at 50% and 5120 `trash_data_count`
# ./scripts/create_glutton_spec.sh ./target/release/polkadot-parachain rococo 1300 G7Z5mTmTQsjEGBVqVGDZyR9m7RoHNZJk6JeykyfKQ3vmBiR 500000000 5500000000 5120
usage() {
    echo Usage:
    echo "$0 <binary path> <relay chain> <parachain id> <sudo key> <compute> <storage> <trash_data_count>"
    exit 1
}

set -e

binary_path=$1
relay_chain=$2
id="glutton_$2_$3"
protocol_id="glutton_$2_$3"
para_id=$3
sudo=$4
compute=$5
storage=$6
trash_data_count=$7

[ -z "$binary_path" ] && usage
[ -z "$relay_chain" ] && usage
[ -z "$id" ] && usage
[ -z "$protocol_id" ] && usage
[ -z "$para_id" ] && usage
[ -z "$sudo" ] && usage
[ -z "$compute" ] && usage
[ -z "$storage" ] && usage
[ -z "$trash_data_count" ] && usage

# build the chain spec we'll manipulate
$binary_path build-spec --disable-default-bootnode --chain "glutton-kusama-genesis-$para_id" > "plain-glutton-$para_id-$relay_chain-spec.json"

# replace the runtime in the spec with the given runtime and set some values to production
cat "plain-glutton-$para_id-$relay_chain-spec.json" \
    | jq --arg id $id '.id = $id' \
    | jq --arg id $id '.protocolId = $id' \
    | jq --arg relay_chain $relay_chain '.relay_chain = $relay_chain' \
    | jq --argjson para_id $para_id '.para_id = $para_id' \
    | jq --arg sudo $sudo '.genesis.runtime.sudo.key = $sudo' \
    | jq --argjson para_id $para_id '.genesis.runtime.parachainInfo.parachainId = $para_id' \
    | jq --argjson compute $compute '.genesis.runtime.glutton.compute = $compute' \
    | jq --argjson storage $storage '.genesis.runtime.glutton.storage = $storage' \
    | jq --argjson trash_data_count $trash_data_count '.genesis.runtime.glutton.trashDataCount = $trash_data_count' \
    > glutton-$para_id-$relay_chain-spec.json

# build a raw spec
$binary_path build-spec --disable-default-bootnode --chain "glutton-$para_id-$relay_chain-spec.json" --raw > "glutton-$para_id-$relay_chain-raw-spec.json"

# build genesis data
$binary_path export-genesis-state --chain "glutton-$para_id-$relay_chain-raw-spec.json" > "glutton-$para_id-$relay_chain-head-data"

# build genesis wasm
$binary_path export-genesis-wasm --chain "glutton-$para_id-$relay_chain-raw-spec.json" > "glutton-$para_id-$relay_chain-validation-code"

rm "plain-glutton-$para_id-$relay_chain-spec.json"
