#!/bin/bash

# Helper script to generate the wasm, state and chain-spec/ -raw.json for a given chain-spec.
#
# Usage: ./scripts/dump_wasm_state_and_spec.sh <chain-spec> <para-id> <collator-binary> <dump-dir>
#
# Example: ./scripts/dump_wasm_state_and_spec.sh shell-kusama-local-dev 2000 collator ./dump_dir
#
# chain-spec is mandatory, the rest is optional.


CHAIN_SPEC=$1
PARA_ID=${2:-2015}
COLLATOR=${3:-./target/release/encointer-collator}
DUMP_DIR=${4:-./chain_dumps}

mkdir -p ${DUMP_DIR}

echo "dumping spec for: $CHAIN_SPEC"
#paraid has been hard-coded in command.rs
#echo "para_id:          ${PARA_ID}"
echo "collator:         ${COLLATOR}"
echo "dump_dir:         ${DUMP_DIR}"
echo ""

$COLLATOR build-spec --chain ${CHAIN_SPEC} >$DUMP_DIR/${CHAIN_SPEC}.json
$COLLATOR build-spec --chain ${CHAIN_SPEC} --raw >$DUMP_DIR/${CHAIN_SPEC}-raw.json

# Overwrite the rust-default value with the $PARA_ID
#sed -i "/\"para_id\": 2015/s/2015/${PARA_ID}/" $DUMP_DIR/${CHAIN_SPEC}.json
#sed -i "/\"para_id\": 2015/s/2015/${PARA_ID}/" $DUMP_DIR/${CHAIN_SPEC}-raw.json

#$COLLATOR export-genesis-state --chain $DUMP_DIR/${CHAIN_SPEC}-raw.json --parachain-id ${PARA_ID} >$DUMP_DIR/${CHAIN_SPEC}.state
$COLLATOR export-genesis-state --chain $DUMP_DIR/${CHAIN_SPEC}-raw.json >$DUMP_DIR/${CHAIN_SPEC}.state
$COLLATOR export-genesis-wasm --chain $DUMP_DIR/${CHAIN_SPEC}-raw.json >$DUMP_DIR/${CHAIN_SPEC}.wasm
