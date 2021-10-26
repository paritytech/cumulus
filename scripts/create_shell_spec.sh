#!/usr/bin/env bash

usage() {
    echo Usage:
    echo "$0 <srtool compressed runtime>"
    exit 1
}

set -e

rt_path=$1

binary="./target/release/polkadot-collator"

# build the chain spec we'll manipulate
$binary build-spec --chain shell > shell-spec-plain.json

# convert runtime to hex
cat $rt_path | od -A n -v -t x1 |  tr -d ' \n' > shell-hex.txt

# replace the runtime in the spec with the given runtime
cat shell-spec-plain.json | jq --rawfile code shell-hex.txt '.genesis.runtime.system.code = ("0x" + $code)' > edited-shell-plain.json

# build a raw spec
$binary build-spec --chain edited-shell-plain.json --raw > shell-spec-raw.json

# build genesis data
$binary export-genesis-state --parachain-id=1000 --chain shell-spec-raw.json > shell_genesis

# build genesis wasm
$binary export-genesis-wasm --chain shell-spec-raw.json > shell_wasm
