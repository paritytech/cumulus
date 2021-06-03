debug:
	cp target/debug/polkadot-collator target/release/polkadot-collator && \
	./target/release/polkadot-collator export-genesis-state --parachain-id 18403 > genesis-state-18403 && \
    ./target/release/polkadot-collator export-genesis-wasm > genesis-wasm-18403 && \
    ./target/release/polkadot-collator export-genesis-state --parachain-id 18401 > genesis-state-18401 && \
    ./target/release/polkadot-collator export-genesis-wasm > genesis-wasm-18401