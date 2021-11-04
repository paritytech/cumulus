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
# ...so that we can verify the checksum
sha256sum --check chainspec-checksum

cp shell-head-data shell-head-data-copy
# generate the checksum for the freshly generated head data
sha256sum shell-head-data-copy > head-data-checksum
# overwrite the head data with the repo version...
cp polkadot-parachains/res/shell-statemint-head-data shell-head-data-copy
# ...so that we can verify the checksum
sha256sum --check head-data-checksum
