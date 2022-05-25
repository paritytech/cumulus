#!/bin/bash

steps=50
repeat=20
category=$1
runtimeName=$2

benhcmarkOutput=./parachains/runtimes/$category/$runtimeName/src/weights
benhcmarkRuntimeName="$runtimeName-dev"

pallets=(
    pallet_assets
	pallet_balances
	pallet_collator_selection
	pallet_multisig
	pallet_proxy
	pallet_session
	pallet_timestamp
	pallet_utility
	pallet_uniques
	cumulus_pallet_xcmp_queue
	frame_system
)

for p in ${pallets[@]}
do
	./artifacts/polkadot-parachain benchmark pallet \
		--chain=$benhcmarkRuntimeName \
		--execution=wasm \
		--wasm-execution=compiled \
		--pallet=$p  \
		--extrinsic='*' \
		--steps=$steps  \
		--repeat=$repeat \
		--json \
        --header=./file_header.txt \
		--output=$benhcmarkOutput
done
