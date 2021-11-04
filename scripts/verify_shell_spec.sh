#!/usr/bin/env bash

usage() {
    echo Usage:
    echo "$0 <generated shell spec>"
    exit 1
}

set -e

spec_path=$1

# sort the json files by key to make sure they are the same when hashed
jq --sort-keys . polkadot-parachains/res/shell-statemint.json > sorted.json
# generate the checksum
sha256sum sorted.json > chainspec-checksum

# overwrite the sorted chainspec json...
jq --sort-keys . $spec_path > sorted.json
# ... so that we can verify the checksum
sha256sum --check chainspec-checksum
