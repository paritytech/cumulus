#!/bin/bash

function ensure_binaries() {
    if [[ ! -f ~/local_bridge_testing/bin/polkadot ]]; then
        echo "  Required polkadot binary '~/local_bridge_testing/bin/polkadot' does not exist!"
        echo "  You need to build it and copy to this location!"
        echo "  Please, check ./parachains/runtimes/bridge-hubs/README.md (Prepare/Build/Deploy)"
        exit 1
    fi
    if [[ ! -f ~/local_bridge_testing/bin/polkadot-parachain ]]; then
        echo "  Required polkadot-parachain binary '~/local_bridge_testing/bin/polkadot-parachain' does not exist!"
        echo "  You need to build it and copy to this location!"
        echo "  Please, check ./parachains/runtimes/bridge-hubs/README.md (Prepare/Build/Deploy)"
        exit 1
    fi
    if [[ ! -f ~/local_bridge_testing/bin/substrate-relay ]]; then
        echo "  Required substrate-relay binary '~/local_bridge_testing/bin/substrate-relay' does not exist!"
        echo "  You need to build it and copy to this location!"
        echo "  Please, check ./parachains/runtimes/bridge-hubs/README.md (Prepare/Build/Deploy)"
        exit 1
    fi
}

function register_parachain() {
    local PORT=$1
    local PARA_ID=$2
    local GENESIS_FILE=$3
    local GENESIS_WASM_FILE=$4

    echo ""
    echo ""
    echo "  Please, register parachain: https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A${PORT}#/sudo"
    echo "  parasSudoWrapper.sudoScheduleParaInitialize:"
    echo "      ParaId: $PARA_ID"
    echo "      Genesis (file): $GENESIS_FILE"
    echo "      Genesis wasm (file): $GENESIS_WASM_FILE"
    echo "      Parachain: Yes"
    echo ""
    echo ""

    # TODO: find the way to do it automatically
}

function show_node_log_file() {
    local NAME=$1
    local WS_PORT=$2
    local FILE_PATH=$3

    echo ""
    echo ""
    echo "  Node(${NAME}): ws-port: ${WS_PORT}"
    echo "  Logs: ${FILE_PATH}"
    echo ""
    echo ""
}

function check_parachain_collator() {
    local PORT=$1
    local PALLET=$2

    echo ""
    echo ""
    echo "  Once relayers work, check parachain: https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A${PORT}#/chainstate"
    echo "  Pallet: ${PALLET}"
    echo "  Keys:"
    echo "      bestFinalized()"
    echo ""
    echo ""
}

ensure_binaries

case "$1" in
  start-rococo)
    # TODO: change to generate: ./bin/polkadot key generate-node-key
    VALIDATOR_KEY_ALICE=(12D3KooWRD6kEQuKDn7BCsMfW71U8AEFDYTc7N2dFQthRsjSR8J5 aa5dc9a97f82f9af38d1f16fbc9c72e915577c3ca654f7d33601645ca5b9a8a5)
    VALIDATOR_KEY_BOB=(12D3KooWELR4gpc8unmH8WiauK133v397KoMfkkjESWJrzg7Y3Pq 556a36beaeeb95225eb84ae2633bd8135e5ace22f03291853654f07bc5da20ae)
    VALIDATOR_KEY_CHARLIE=(12D3KooWRatYNHCRpyzhjPqfSPHYsUTi1zX4452kxSWg8ek99Vun 56e6593340685021d27509c0f48836c1dcb84c5aeba730ada68f54a9ae242e67)
    COLLATOR_KEY_ALICE=(12D3KooWAU51GKCfPBfhaKK8gr6XmHNVXd4e96BYL9v2QkWtDrAm 15642c2e078ddfaacd1e68afcc8bab6dd8085b72b3038b799ddfb46bec18696c)
    COLLATOR_KEY_BOB=(12D3KooWPg8SEatSi9HvdPJVYBNxeNAvyPTo3Rd4WPdzLRcM8gaS ac651cd6b9a2c7768bca92369ecd4807b54059c33775257e6ba5f72ae91a136d)

    # Start Rococo relay
    ~/local_bridge_testing/bin/polkadot build-spec --chain rococo-local --disable-default-bootnode --raw > ~/local_bridge_testing/rococo-local-cfde.json
    ~/local_bridge_testing/bin/polkadot --chain ~/local_bridge_testing/rococo-local-cfde.json --alice --tmp --port 30332 --rpc-port 9932 --ws-port 9942 --no-mdns --node-key ${VALIDATOR_KEY_ALICE[1]} --bootnodes=/ip4/127.0.0.1/tcp/30333/p2p/${VALIDATOR_KEY_BOB[0]} &> ~/local_bridge_testing/logs/rococo_relay_alice.log &
    ~/local_bridge_testing/bin/polkadot --chain ~/local_bridge_testing/rococo-local-cfde.json --bob --tmp --port 30333 --rpc-port 9933 --ws-port 9943 --no-mdns --node-key ${VALIDATOR_KEY_BOB[1]} --bootnodes=/ip4/127.0.0.1/tcp/30332/p2p/${VALIDATOR_KEY_ALICE[0]} &> ~/local_bridge_testing/logs/rococo_relay_bob.log &
    ~/local_bridge_testing/bin/polkadot --chain ~/local_bridge_testing/rococo-local-cfde.json --charlie --tmp --port 30334 --rpc-port 9934 --ws-port 9944 --no-mdns ${VALIDATOR_KEY_CHARLIE[1]} --bootnodes=/ip4/127.0.0.1/tcp/30332/p2p/${VALIDATOR_KEY_ALICE[0]} &> ~/local_bridge_testing/logs/rococo_relay_charlie.log &
    sleep 2

    # Prepare Rococo parachain
    rm ~/local_bridge_testing/bridge-hub-rococo-local-raw.json
    ~/local_bridge_testing/bin/polkadot-parachain build-spec --chain bridge-hub-rococo-local --raw --disable-default-bootnode > ~/local_bridge_testing/bridge-hub-rococo-local-raw.json
    ~/local_bridge_testing/bin/polkadot-parachain export-genesis-state --chain ~/local_bridge_testing/bridge-hub-rococo-local-raw.json > ~/local_bridge_testing/bridge-hub-rococo-local-genesis
    ~/local_bridge_testing/bin/polkadot-parachain export-genesis-wasm --chain ~/local_bridge_testing/bridge-hub-rococo-local-raw.json > ~/local_bridge_testing/bridge-hub-rococo-local-genesis-wasm

    # Rococo
    ~/local_bridge_testing/bin/polkadot-parachain --chain ~/local_bridge_testing/bridge-hub-rococo-local-raw.json --collator --alice --force-authoring --tmp --port 40333 --rpc-port 8933 --ws-port 8943 --no-mdns --node-key ${COLLATOR_KEY_ALICE[1]} --bootnodes=/ip4/127.0.0.1/tcp/40334/p2p/${COLLATOR_KEY_BOB[0]} -- --execution wasm --chain ~/local_bridge_testing/rococo-local-cfde.json --port 41333 --rpc-port 48933 --ws-port 48943 --no-mdns --node-key ${COLLATOR_KEY_ALICE[1]} --bootnodes=/ip4/127.0.0.1/tcp/30332/p2p/${VALIDATOR_KEY_ALICE[0]} &> ~/local_bridge_testing/logs/rococo_para_alice.log &
    show_node_log_file alice 8943 ~/local_bridge_testing/logs/rococo_para_alice.log
    ~/local_bridge_testing/bin/polkadot-parachain --chain ~/local_bridge_testing/bridge-hub-rococo-local-raw.json --collator --bob --force-authoring --tmp --port 40334 --rpc-port 8934 --ws-port 8944 --no-mdns --node-key ${COLLATOR_KEY_BOB[1]} --bootnodes=/ip4/127.0.0.1/tcp/40333/p2p/${COLLATOR_KEY_ALICE[0]} -- --execution wasm --chain ~/local_bridge_testing/rococo-local-cfde.json --port 41334 --rpc-port 48934 --ws-port 48944 --no-mdns --node-key ${COLLATOR_KEY_BOB[1]} --bootnodes=/ip4/127.0.0.1/tcp/30333/p2p/${VALIDATOR_KEY_BOB[0]} &> ~/local_bridge_testing/logs/rococo_para_bob.log &
    show_node_log_file bob 8944 ~/local_bridge_testing/logs/rococo_para_bob.log

    register_parachain 9942 1013 ~/local_bridge_testing/bridge-hub-rococo-local-genesis ~/local_bridge_testing/bridge-hub-rococo-local-genesis-wasm
    check_parachain_collator 8943 bridgeWococoGrandpa
    ;;
  start-wococo)
    # TODO: change to generate: ./bin/polkadot key generate-node-key
    VALIDATOR_KEY_ALICE=(12D3KooWKB4SqNJAttmDHXEKtETLPr8Nixso8gUNzNKDS9b6HtYj 8c87694d3a3b3a0e201caceaae095aac66a76ba37545c439f7e0d986670da8e6)
    VALIDATOR_KEY_BOB=(12D3KooWJq6xkfyV3LFxoNgsCskAwLv6VfLhL3cjHfW4UxDNwroF 9ebcd7371b427d63c087762fe3dc5a1d4b01875cb8611c263feda21364d4a329)
    VALIDATOR_KEY_CHARLIE=(12D3KooW9yQQXz6xUJY2YgizKTFLS5fPaPi4KznQdMHEW4Hzt3NL 8bb1c50421a73082b8275ead6e3f150a774a0f8d6ad4a786df5d5b7688115cb1)
    VALIDATOR_KEY_DAVE=(12D3KooWJP1R7XxqEuSryXSKYRVz9wdMRvd5LSqjRpdK4AvjnB12 e160d8b0ef4d66e02abf6663417b024f27f6cb2f826b746f3d267c58a32e9c05)
    COLLATOR_KEY_ALICE=(12D3KooWNsQbEdW1eyC6TXAJCXxZ8qzkCJPWFqhWjco1SYYdaKPU 93ac8257255961b3d3ec0038747f0f9c7ddaa5dd352297daa352e098285965b7)
    COLLATOR_KEY_BOB=(12D3KooWQ9yxdcf7kavoaKNWmt92tkCGQ8C4sq2JNBgCjvGtvvFg 7e396f01a8a74ecf2010ab2d57ca9fc6c5d673914d97ad6a066fe724e60c3412)

    # Start Wococo relay
    ~/local_bridge_testing/bin/polkadot build-spec --chain wococo-local --disable-default-bootnode --raw > ~/local_bridge_testing/wococo-local-cfde.json
    ~/local_bridge_testing/bin/polkadot --chain ~/local_bridge_testing/wococo-local-cfde.json --alice --tmp --port 30335 --rpc-port 9935 --ws-port 9945 --no-mdns --node-key ${VALIDATOR_KEY_ALICE[1]} --bootnodes=/ip4/127.0.0.1/tcp/30336/p2p/${VALIDATOR_KEY_BOB[0]} &> ~/local_bridge_testing/logs/wococo_relay_alice.log &
    ~/local_bridge_testing/bin/polkadot --chain ~/local_bridge_testing/wococo-local-cfde.json --bob --tmp --port 30336 --rpc-port 9936 --ws-port 9946 --no-mdns --node-key ${VALIDATOR_KEY_BOB[1]} --bootnodes=/ip4/127.0.0.1/tcp/30335/p2p/${VALIDATOR_KEY_ALICE[0]} &> ~/local_bridge_testing/logs/wococo_relay_bob.log &
    ~/local_bridge_testing/bin/polkadot --chain ~/local_bridge_testing/wococo-local-cfde.json --charlie --tmp --port 30337 --rpc-port 9937 --ws-port 9947 --no-mdns --node-key ${VALIDATOR_KEY_CHARLIE[1]} --bootnodes=/ip4/127.0.0.1/tcp/30335/p2p/${VALIDATOR_KEY_ALICE[0]} &> ~/local_bridge_testing/logs/wococo_relay_charlie.log &
    ~/local_bridge_testing/bin/polkadot --chain ~/local_bridge_testing/wococo-local-cfde.json --dave --tmp --port 30338 --rpc-port 9938 --ws-port 9948 --no-mdns --node-key ${VALIDATOR_KEY_DAVE[1]} --bootnodes=/ip4/127.0.0.1/tcp/30336/p2p/${VALIDATOR_KEY_BOB[0]} &> ~/local_bridge_testing/logs/wococo_relay_dave.log &
    sleep 2

    # Prepare Wococo parachain
    rm ~/local_bridge_testing/bridge-hub-wococo-local-raw.json
    ~/local_bridge_testing/bin/polkadot-parachain build-spec --chain bridge-hub-wococo-local --raw --disable-default-bootnode > ~/local_bridge_testing/bridge-hub-wococo-local-raw.json
    ~/local_bridge_testing/bin/polkadot-parachain export-genesis-state --chain ~/local_bridge_testing/bridge-hub-wococo-local-raw.json > ~/local_bridge_testing/bridge-hub-wococo-local-genesis
    ~/local_bridge_testing/bin/polkadot-parachain export-genesis-wasm --chain ~/local_bridge_testing/bridge-hub-wococo-local-raw.json > ~/local_bridge_testing/bridge-hub-wococo-local-genesis-wasm

    # Wococo
    ~/local_bridge_testing/bin/polkadot-parachain --chain ~/local_bridge_testing/bridge-hub-wococo-local-raw.json --collator --alice --force-authoring --tmp --port 40335 --rpc-port 8935 --ws-port 8945 --no-mdns --node-key ${COLLATOR_KEY_ALICE[1]} --bootnodes=/ip4/127.0.0.1/tcp/40336/p2p/${COLLATOR_KEY_BOB[0]} -- --execution wasm --chain ~/local_bridge_testing/wococo-local-cfde.json --port 41335 --rpc-port 48935 --ws-port 48945 --no-mdns --node-key ${COLLATOR_KEY_ALICE[1]} --bootnodes=/ip4/127.0.0.1/tcp/30335/p2p/${VALIDATOR_KEY_ALICE[0]} &> ~/local_bridge_testing/logs/wococo_para_alice.log &
    show_node_log_file alice 8945 ~/local_bridge_testing/logs/wococo_para_alice.log
    ~/local_bridge_testing/bin/polkadot-parachain --chain ~/local_bridge_testing/bridge-hub-wococo-local-raw.json --collator --bob --force-authoring --tmp --port 40336 --rpc-port 8936 --ws-port 8946 --no-mdns --node-key ${COLLATOR_KEY_BOB[1]} --bootnodes=/ip4/127.0.0.1/tcp/40335/p2p/${COLLATOR_KEY_ALICE[0]} -- --execution wasm --chain ~/local_bridge_testing/wococo-local-cfde.json --port 41336 --rpc-port 48936 --ws-port 48946 --no-mdns --node-key ${COLLATOR_KEY_BOB[1]} --bootnodes=/ip4/127.0.0.1/tcp/30336/p2p/${VALIDATOR_KEY_BOB[0]} &> ~/local_bridge_testing/logs/wococo_para_bob.log &
    show_node_log_file bob 8946 ~/local_bridge_testing/logs/wococo_para_bob.log

    register_parachain 9945 1013 ~/local_bridge_testing/bridge-hub-wococo-local-genesis ~/local_bridge_testing/bridge-hub-wococo-local-genesis-wasm
    check_parachain_collator 8945 bridgeRococoGrandpa
    ;;
  init-ro-wo)
    # Init bridge Rococo->Wococo
    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        ~/local_bridge_testing/bin/substrate-relay init-bridge rococo-to-bridge-hub-wococo \
	--source-host localhost \
	--source-port 48943 \
	--target-host localhost \
	--target-port 8945 \
	--target-signer //Bob
    ;;
  init-wo-ro)
    # Init bridge Wococo->Rococo
    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        ~/local_bridge_testing/bin/substrate-relay init-bridge wococo-to-bridge-hub-rococo \
        --source-host localhost \
        --source-port 48945 \
        --target-host localhost \
        --target-port 8943 \
        --target-signer //Bob
    ;;
  stop)
    pkill -f polkadot
    pkill -f parachain
    ;;
  *) echo "A command is require. Supported commands: start-rococo, start-wococo, init-ro-wo, init-wo-ro, stop"; exit 1;;
esac
