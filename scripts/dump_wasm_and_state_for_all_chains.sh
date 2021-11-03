#!/bin/bash
# Remark: wasm and state will be identical for different relay-chains

# Helper script to generate the wasm, state and chain-spec/ -raw.json for a all chain-specs.
#
# Usage: ./scripts/dump_wasm_and_state_for_all_chains.sh <para-id> <collator-binary> <dump-dir>
#
# Example: ./scripts/dump_wasm_state_and_spec.sh 2000 ./collator ./dump_dir
#
# All arguments are optional.

PARA_ID=${1:-2015}
COLLATOR=${2:-./target/release/encointer-collator}
DUMP_DIR=${3:-./chain_dumps}

mkdir -p ${DUMP_DIR}

chainspecs=("encointer-rococo" \
      "encointer-rococo-local" \

      "launch-rococo" \
      "launch-rococo-local" \

      "launch-kusama" \
      "launch-kusama-fresh" \
      "launch-kusama-local" \
      "launch-kusama-local-dev" \

      "launch-westend" \
      "launch-westend-fresh" \
      "launch-westend-local" \
      "launch-westend-local-dev" \

      "sybil-dummy-rococo" \
      "sybil-dummy-rococo-local"
      )

$COLLATOR --version
# Print array values in  lines
for spec in ${chainspecs[*]}; do
  ./scripts/dump_wasm_state_and_spec.sh ${spec} ${PARA_ID} ${COLLATOR} ${DUMP_DIR}
done
