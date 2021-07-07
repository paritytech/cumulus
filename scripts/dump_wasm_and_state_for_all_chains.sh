#!/bin/bash
# Remark: wasm and state will be identical for different relay-chains
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
