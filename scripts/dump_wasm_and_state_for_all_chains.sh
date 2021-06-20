#!/bin/bash
# Remark: wasm and state will be identical for different relay-chains
chainspecs=("integritee-rococo-local" \
      "integritee-rococo-local-dev" \
      "integritee-kusama" \
      "shell-rococo-local" \
      "shell-rococo-local-dev" \
      "shell-kusama" \
      )

# Print array values in  lines
for spec in ${chainspecs[*]}; do
  echo "dumping spec for $spec"
  ./target/release/integritee-collator export-genesis-state --chain $spec --parachain-id 2015 > ${spec}.state
  ./target/release/integritee-collator export-genesis-wasm --chain $spec > ${spec}.wasm
  ./target/release/integritee-collator build-spec --chain $spec > ${spec}.json
done
