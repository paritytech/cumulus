#!/usr/bin/env bash

# this script runs the cumulus-test-parachain-collator after fetching
# appropriate bootnode IDs
#
# this is _not_ a general-purpose script; it is closely tied to the
# root docker-compose.yml

set -e -o pipefail

ctpc="/usr/bin/cumulus-test-parachain-collator"

if [ ! -x "$ctpc" ]; then
    echo "FATAL: $ctpc does not exist or is not executable"
    exit 1
fi

# name the variable with the incoming args so it isn't overwritten later by function calls
args=( "$@" )

alice="172.28.1.1:9933"
bob="172.28.1.2:9935"

get_id () {
    node="$1"
    /wait-for-it.sh "$node" -t 10 -s -- \
        curl -sS \
            -H 'Content-Type: application/json' \
            --data '{"id":1,"jsonrpc":"2.0","method":"system_networkState"}' \
            "$node" |\
    jq -r '.result.peerId'
}

bootnode () {
    node="$1"
    ip=$(cut -d: -f1 <<< "$node")
    port=$(cut -d: -f2 <<< "$node")
    id=$(get_id "$node")
    echo "/ip4/$ip/tcp/$port/p2p/$id"
}

args+=( "--bootnodes" "$(bootnode "$alice")" "--bootnodes" "$(bootnode "$bob")" )

set -x
"$ctpc" "${args[@]}"
