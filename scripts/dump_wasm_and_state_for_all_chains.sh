#!/bin/bash
# Remark: wasm and state will be identical for different relay-chains
chainspecs=("integritee-rococo-local" \
      "integritee-rococo-local-dev" \
      "integritee-kusama" \
      "shell-rococo-local" \
      "shell-rococo-local-dev" \
      "shell-kusama" \
      )

COLLATOR=./target/release/integritee-collator

$COLLATOR --version
# Print array values in  lines
for spec in ${chainspecs[*]}; do
  echo "dumping spec for $spec"
  $COLLATOR export-genesis-state --chain $spec --parachain-id 2015 > ${spec}.state
  $COLLATOR export-genesis-wasm --chain $spec > ${spec}.wasm
  $COLLATOR build-spec --chain $spec > ${spec}.json
  $COLLATOR build-spec --chain $spec --raw > ${spec}-raw.json
done
