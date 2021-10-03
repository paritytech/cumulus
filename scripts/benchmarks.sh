#!/bin/bash

steps=50
repeat=1
statemineOutput=./polkadot-parachains/statemine/src/weights
statemintOutput=./polkadot-parachains/statemint/src/weights
statemineChain=statemine-dev
statemintChain=statemint-dev
pallets=(
    # pallet_assets
	# pallet_balances
	pallet_collator_selection
	# pallet_multisig
	# pallet_proxy
	# pallet_session
	# pallet_timestamp
	# pallet_utility
)

for p in ${pallets[@]}
do
    RUST_LOG=nacho=debug
	./target/release/polkadot-collator benchmark \
        -ldebug \
		--chain=$statemineChain \
		--execution=native \
		--pallet=$p  \
		--extrinsic='*' \
		--steps=$steps  \
		--repeat=$repeat \
		--raw  \
		--output=$statemineOutput

	# ./target/release/polkadot-collator benchmark \
	# 	--chain=$statemintChain \
	# 	--execution=wasm \
	# 	--wasm-execution=compiled \
	# 	--pallet=$p  \
	# 	--extrinsic='*' \
	# 	--steps=$steps  \
	# 	--repeat=$repeat \
	# 	--raw  \
	# 	--output=$statemintOutput

done
