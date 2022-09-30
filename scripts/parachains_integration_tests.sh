#!/usr/bin/env bash

tests=(
    statemine
    statemint
)

rm -R logs &> /dev/null

for t in ${tests[@]}
do
    printf "\n🔍  Running $t tests...\n\n"

    mkdir -p logs/$t

    parachains-integration-tests \
        -m zombienet \
        -c ./parachains/integration-tests/$t/config.toml \
        -cl ./logs/$t/chains.log 2> /dev/null &

    parachains-integration-tests \
        -m test \
        -t ./parachains/integration-tests/$t \
        -tl ./logs/$t/tests.log & tests=$!

    wait $tests

    # trying to stop zombienet gracefully
    kill -s 3 $(ps -ef | awk '/[b]in\/zombienet/{print $2}')
    pkill -TERM polkadot || echo killed
    pkill -TERM parachain || echo killed

    printf "\n🎉 $t integration tests finished! \n\n"
done
