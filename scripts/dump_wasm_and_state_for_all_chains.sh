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
COLLATOR=${2:-./target/release/integritee-collator}
DUMP_DIR=${3:-./chain_dumps}

mkdir -p ${DUMP_DIR}

chainspecs=("integritee-rococo-local" \
      "integritee-rococo-local-dev" \
      "integritee-rococo" \
      "integritee-kusama-local" \
      "integritee-kusama-local-dev" \
      "integritee-kusama" \
      "integritee-polkadot-local" \
      "integritee-polkadot-local-dev" \
      "integritee-polkadot" \
      "shell-rococo-local" \
      "shell-rococo-local-dev" \
      "shell-rococo" \
      "shell-kusama-local" \
      "shell-kusama-local-dev" \
      "shell-kusama" \
      "shell-polkadot-local" \
      "shell-polkadot-local-dev" \
      "shell-polkadot" \

      )

$COLLATOR --version
# Print array values in  lines
for spec in ${chainspecs[*]}; do
  ./scripts/dump_wasm_state_and_spec.sh ${spec} ${PARA_ID} ${COLLATOR} ${DUMP_DIR}
done
