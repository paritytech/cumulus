# This creates and extended weight file that contains:
# * `WeightInfo` trait declaration
# * `WeightInfo` implementation for an `IntegriteeRuntimeWeight` struct
# * `WeightInfo` implementation for `()` used in testing.
#
# This output file is intended to be copied to the `pallet_teerex` repository and used there.

COLLATOR=./target/release/integritee-collator

$COLLATOR \
  benchmark \
  --chain=integritee-rococo-local-dev \
  --steps=50 \
  --repeat=20 \
  --pallet=pallet_teerex \
  --extrinsic="*" \
  --execution=wasm \
  --wasm-execution=compiled \
  --heap-pages=4096 \
  --output=./pallet_teerex_weights.rs \
  --template=./scripts/frame-weight-template-full-info.hbs

