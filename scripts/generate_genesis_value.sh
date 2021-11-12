#!/usr/bin/env bash

usage() {
    echo Usage:
    echo "$0 <chain-id>"
    exit 1
}

set -e

chain_id=$1
work_dir="polkadot-parachains/res"

[ -z "$chain_id" ] && usage

pushd scripts/generate_genesis_values
yarn
popd

node scripts/generate_genesis_values $work_dir/$chain_id.json $work_dir/${chain_id}_values.json

pushd scripts/scale_encode_genesis
yarn
popd
node scripts/scale_encode_genesis $work_dir/${chain_id}_values.json $work_dir/${chain_id}_values.scale
