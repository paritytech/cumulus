#!/bin/bash

steps=50
repeat=20
statemineOutput=./polkadot-parachains/statemine-runtime/src/weights
statemintOutput=./polkadot-parachains/statemint-runtime/src/weights
statemineChain=statemine-dev
statemintChain=statemint-dev
pallets=(
    pallet_session_benchmarking
)

for p in ${pallets[@]}
do
	./target/release/polkadot-collator benchmark \
		--chain=$statemineChain \
		--execution=wasm \
		--wasm-execution=compiled \
		--pallet=$p  \
		--extrinsic='*' \
		--steps=$steps  \
		--repeat=$repeat \
		--raw  \
		--output=$statemineOutput

	./target/release/polkadot-collator benchmark \
		--chain=$statemintChain \
		--execution=wasm \
		--wasm-execution=compiled \
		--pallet=$p  \
		--extrinsic='*' \
		--steps=$steps  \
		--repeat=$repeat \
		--raw  \
		--output=$statemintOutput

done
