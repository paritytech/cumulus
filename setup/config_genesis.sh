source target_env

../target/release/polkadot-collator export-genesis-state --parachain-id $PARA_ID > genesis-state-$PARA_ID
../target/release/polkadot-collator export-genesis-wasm > genesis-wasm-$PARA_ID

../target/release/polkadot build-spec --chain $SPEC_PATH --disable-default-bootnode > $TMP_PATH

node addParachainToGenesis.js

../target/release/polkadot build-spec --chain $TMP_PATH --disable-default-bootnode --raw > $OUT_PATH
rm ./$TMP_PATH
rm genesis-state-$PARA_ID
rm genesis-wasm-$PARA_ID