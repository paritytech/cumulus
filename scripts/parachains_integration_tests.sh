#!/usr/bin/env bash

tests=(
    assets/statemine
    assets/statemint
    collectives/collectives-polkadot
)

rm -R logs &> /dev/null

for t in ${tests[@]}
do
    printf "\n🔍  Running $t tests...\n\n"

    mkdir -p logs/$t

    DEBUG=zombie::metrics parachains-integration-tests \
        -m zombienet \
        -c ./parachains/integration-tests/e2e/$t/config.toml \
        -cl ./logs/$t/chains.log 2> /dev/null &

    parachains-integration-tests \
        -m test \
        -t ./parachains/integration-tests/e2e/$t \
        -tl ./logs/$t/tests.log & tests=$!

    wait $tests

    pkill -f polkadot
    pkill -f parachain

    printf "\n🎉 $t integration tests finished! \n\n"
done
