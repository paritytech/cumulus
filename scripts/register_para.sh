#!/usr/bin/env bash

set -e -o pipefail

# this is straightforward: just send the sudo'd tx to the alice node

polkadot-js-api \
    --ws ws://172.28.1.1:9944 \
    --sudo \
    --seed "//Alice" \
    tx.registrar.registerPara \
        100 \
        '{"scheduling":"Always"}' \
        @/runtime/cumulus_test_parachain_runtime.compact.wasm \
        @/genesis/genesis-state
