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

# replace the runtime in the spec with the given runtime and set some values to production
cat shell-spec-plain.json | jq --rawfile code shell-hex.txt '.genesis.runtime.system.code = ("0x" + $code)' \
    | jq '.name = "Shell"' \
    | jq '.id = "shell"' \
    | jq '.chainType = "Live"' \
    | jq '.bootNodes = ["/ip4/34.65.213.18/tcp/30334/p2p/12D3KooWF63ZxKtZMYs5247WQA8fcTiGJb2osXykc31cmjwNLwem", "/ip4/34.65.104.62/tcp/30334/p2p/12D3KooWBJTqpLCKVBEbH4jvn7DvxDhSnmFFM7e8rohQqTGfmLSK", "/ip4/34.65.35.228/tcp/30334/p2p/12D3KooWF5jLarW11PF1yVncAKnk9rNGEa9WiZ7hMkEpT4HPpkLp", "/ip4/34.65.251.121/tcp/30334/p2p/12D3KooWDj3YNpKp2hW8g5zasxk54PtxHS1fN4AZUPu3hUwitPnh"]' \
    | jq '.relay_chain = "polkadot"' \
    > edited-shell-plain.json

# build a raw spec
$binary build-spec --chain edited-shell-plain.json --raw > shell-spec-raw.json

# build genesis data
$binary export-genesis-state --parachain-id=1000 --chain shell-spec-raw.json > shell-head-data

# build genesis wasm
$binary export-genesis-wasm --chain shell-spec-raw.json > shell-wasm
