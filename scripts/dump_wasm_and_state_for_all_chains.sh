#!/bin/bash
# Remark: wasm and state will be identical for different relay-chains

# Helper script to generate the wasm, state and chain-spec/ -raw.json for a all chain-specs.
#
# Usage: ./scripts/dump_wasm_and_state_for_all_chains.sh <para-id> <collator-binary> <dump-dir>
#
# Example: ./scripts/dump_wasm_state_and_spec.sh 2000 ./collator ./dump_dir
#
# All arguments are optional.

COLLATOR=${2:-./target/release/integritee-collator}
DUMP_DIR=${3:-./chain_dumps}

mkdir -p ${DUMP_DIR}

chainspecs=("integritee-rococo-local" \
      "integritee-rococo-local-dev" \
      "integritee-rococo" \
      "integritee-westend-local" \
      "integritee-westend-local-dev" \
      "integritee-westend" \
      "integritee-kusama-local" \
      "integritee-kusama-local-dev" \
      "integritee-kusama" \
      "integritee-polkadot-local" \
      "integritee-polkadot-local-dev" \
      "integritee-polkadot" \
      "shell-rococo-local" \
      "shell-rococo-local-dev" \
      "shell-westend-local" \
      "shell-westend-local-dev" \
      "shell-kusama-local" \
      "shell-kusama-local-dev" \
      "shell-polkadot-local" \
      "shell-polkadot-local-dev" \
      )

$COLLATOR --version
# Print array values in  lines
for spec in ${chainspecs[*]}; do
  ./scripts/dump_wasm_state_and_spec.sh ${spec} ${COLLATOR} ${DUMP_DIR}
done
