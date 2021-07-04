source target_env

../target/release/polkadot-collator export-genesis-state --parachain-id $PARA_ID > ${HEAD_PATH}
../target/release/polkadot-collator export-genesis-wasm > ${WASM_PATH}

../target/release/polkadot build-spec --chain $SPEC_PATH --disable-default-bootnode > $TMP_PATH

node addParachainToGenesis.js

../target/release/polkadot build-spec --chain $TMP_PATH --disable-default-bootnode --raw > $OUT_PATH
rm ./$TMP_PATH
rm ${HEAD_PATH}
rm ${WASM_PATH}
