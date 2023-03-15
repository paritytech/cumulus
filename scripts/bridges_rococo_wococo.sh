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
}

function ensure_relayer() {
    if [[ ! -f ~/local_bridge_testing/bin/substrate-relay ]]; then
        echo "  Required substrate-relay binary '~/local_bridge_testing/bin/substrate-relay' does not exist!"
        echo "  You need to build it and copy to this location!"
        echo "  Please, check ./parachains/runtimes/bridge-hubs/README.md (Prepare/Build/Deploy)"
        exit 1
    fi
}

function ensure_polkadot_js_api() {
    if ! which polkadot-js-api &> /dev/null; then
        echo ''
        echo 'Required command `polkadot-js-api` not in PATH, please, install, e.g.:'
        echo "npm install -g @polkadot/api-cli@beta"
        echo "      or"
        echo "yarn global add @polkadot/api-cli"
        echo ''
        exit 1
    fi
    if ! which jq &> /dev/null; then
        echo ''
        echo 'Required command `jq` not in PATH, please, install, e.g.:'
        echo "apt install -y jq"
        echo ''
        exit 1
    fi
    generate_hex_encoded_call_data "check" "--"
    local retVal=$?
    if [ $retVal -ne 0 ]; then
        echo ""
        echo ""
        echo "-------------------"
        echo "Installing (nodejs) sub module: ./scripts/generate_hex_encoded_call"
        pushd ./scripts/generate_hex_encoded_call
        npm install
        popd
    fi
}

function generate_hex_encoded_call_data() {
    local type=$1
    local endpoint=$2
    local output=$3
    shift
    shift
    shift
    echo "Input params: $@"

    node ./scripts/generate_hex_encoded_call "$type" "$endpoint" "$output" "$@"
    local retVal=$?

    if [ $type != "check" ]; then
        local hex_encoded_data=$(cat $output)
        echo "Generated hex-encoded bytes to file '$output': $hex_encoded_data"
    fi

    return $retVal
}

STATEMINE_ACCOUNT_SEED_FOR_LOCAL="//Alice"
ROCKMINE2_ACCOUNT_SEED_FOR_ROCOCO="scatter feed race company oxygen trip extra elbow slot bundle auto canoe"

function send_xcm_transact_remark_from_statemine() {
    local url=$1
    local seed=$2
    local bridge_hub_para_id=$3
    local target_network=$4
    local target_network_para_id=$5
    local target_network_para_endpoint=$6
    echo "  calling send_xcm_transact_remark_from_statemine:"
    echo "      url: ${url}"
    echo "      seed: ${seed}"
    echo "      bridge_hub_para_id: ${bridge_hub_para_id}"
    echo "      target_network: ${target_network}"
    echo "      target_network_para_id: ${target_network_para_id}"
    echo "      target_network_para_endpoint: ${target_network_para_endpoint}"
    echo "      params:"

    # generate data for Transact on the other side (Westmint)
    local tmp_file=$(mktemp)
    generate_hex_encoded_call_data "remark-with-event" "${target_network_para_endpoint}" "${tmp_file}"
    local hex_encoded_data=$(cat $tmp_file)

    local dest=$(jq --null-input \
                    --arg bridge_hub_para_id "$bridge_hub_para_id" \
                    '{ "V3": { "parents": 1, "interior": { "X1": { "Parachain": $bridge_hub_para_id } } } }')

    local message=$(jq --null-input \
                       --arg target_network "$target_network" \
                       --arg target_network_para_id "$target_network_para_id" \
                       --argjson hex_encoded_data $hex_encoded_data \
                       '
                       {
                          "V3": [
                            {
                              "ExportMessage": {
                                "network": $target_network,
                                "destination": {
                                  "X1": {
                                    "Parachain": $target_network_para_id
                                  }
                                },
                                "xcm": [
                                  {
                                    "Transact": {
                                      "origin_kind": "SovereignAccount",
                                      "require_weight_at_most": {
                                        "ref_time": 1000000000,
                                        "proof_size": 0,
                                      },
                                      "call": {
                                        "encoded": $hex_encoded_data
                                      }
                                    }
                                  }
                                ]
                              }
                            }
                          ]
                        }
                        ')

    echo ""
    echo "          dest:"
    echo "${dest}"
    echo ""
    echo "          message:"
    echo "${message}"
    echo ""
    echo "--------------------------------------------------"

    polkadot-js-api \
        --ws "${url?}" \
        --seed "${seed?}" \
        tx.polkadotXcm.send \
            "${dest}" \
            "${message}"
}

function send_xcm_trap_from_statemine() {
    local url=$1
    local seed=$2
    local bridge_hub_para_id=$3
    local target_network=$4
    local target_network_para_id=$5
    echo "  calling send_xcm_trap_from_statemine:"
    echo "      url: ${url}"
    echo "      seed: ${seed}"
    echo "      bridge_hub_para_id: ${bridge_hub_para_id}"
    echo "      params:"

    local dest=$(jq --null-input \
                    --arg bridge_hub_para_id "$bridge_hub_para_id" \
                    '{ "V3": { "parents": 1, "interior": { "X1": { "Parachain": $bridge_hub_para_id } } } }')

    local message=$(jq --null-input \
                       --arg target_network "$target_network" \
                       --arg target_network_para_id "$target_network_para_id" \
                       '
                       {
                          "V3": [
                            {
                              "ExportMessage": {
                                "network": $target_network,
                                "destination": {
                                  "X1": {
                                    "Parachain": $target_network_para_id
                                  }
                                },
                                "xcm": [
                                  {
                                    "Trap": 12345
                                  }
                                ]
                              }
                            }
                          ]
                        }
                        ')

    echo ""
    echo "          dest:"
    echo "${dest}"
    echo ""
    echo "          message:"
    echo "${message}"
    echo ""
    echo "--------------------------------------------------"

    polkadot-js-api \
        --ws "${url?}" \
        --seed "${seed?}" \
        tx.polkadotXcm.send \
            "${dest}" \
            "${message}"
}

function allow_assets_transfer_from_statemine() {
    local relay_url=$1
    local relay_chain_seed=$2
    local statemine_para_id=$3
    local statemine_para_endpoint=$4
    local bridge_hub_para_id=$5
    local statemint_para_network=$6
    local statemint_para_para_id=$7
    echo "  calling allow_assets_transfer_from_statemine:"
    echo "      relay_url: ${relay_url}"
    echo "      relay_chain_seed: ${relay_chain_seed}"
    echo "      statemine_para_id: ${statemine_para_id}"
    echo "      statemine_para_endpoint: ${statemine_para_endpoint}"
    echo "      bridge_hub_para_id: ${bridge_hub_para_id}"
    echo "      statemint_para_network: ${statemint_para_network}"
    echo "      statemint_para_para_id: ${statemint_para_para_id}"
    echo "      params:"

    # generate data for Transact on Statemine
    local bridge_config=$(jq --null-input \
                             --arg statemint_para_network "$statemint_para_network" \
                             --arg bridge_hub_para_id "$bridge_hub_para_id" \
                             --arg statemint_para_network "$statemint_para_network" \
                             --arg statemint_para_para_id "$statemint_para_para_id" \
        '
            {
                "bridgeLocation": {
                    "parents": 1,
                    "interior": {
                        "X1": { "Parachain": $bridge_hub_para_id }
                    }
                },
                "allowedTargetLocation": {
                    "parents": 2,
                    "interior": {
                        "X2": [
                            {
                                "GlobalConsensus": $statemint_para_network,
                            },
                            {
                                "Parachain": $statemint_para_para_id
                            }
                        ]
                    }
                }
            }
        '
    )
    local tmp_output_file=$(mktemp)
    generate_hex_encoded_call_data "add-bridge-config" "${statemine_para_endpoint}" "${tmp_output_file}" $statemint_para_network "$bridge_config"
    local hex_encoded_data=$(cat $tmp_output_file)

    local dest=$(jq --null-input \
                    --arg statemine_para_id "$statemine_para_id" \
                    '{ "V3": { "parents": 0, "interior": { "X1": { "Parachain": $statemine_para_id } } } }')

    local message=$(jq --null-input \
                       --argjson hex_encoded_data $hex_encoded_data \
                       '
                       {
                          "V3": [
                                  {
                                    "UnpaidExecution": {
                                        "weight_limit": "Unlimited"
                                    }
                                  },
                                  {
                                    "Transact": {
                                      "origin_kind": "Superuser",
                                      "require_weight_at_most": {
                                        "ref_time": 1000000000,
                                        "proof_size": 0,
                                      },
                                      "call": {
                                        "encoded": $hex_encoded_data
                                      }
                                    }
                                  }
                          ]
                        }
                        ')

    echo ""
    echo "          dest:"
    echo "${dest}"
    echo ""
    echo "          message:"
    echo "${message}"
    echo ""
    echo "--------------------------------------------------"

    polkadot-js-api \
        --ws "${relay_url?}" \
        --seed "${relay_chain_seed?}" \
        --sudo \
        tx.xcmPallet.send \
            "${dest}" \
            "${message}"
}

function remove_assets_transfer_from_statemine() {
    local relay_url=$1
    local relay_chain_seed=$2
    local statemine_para_id=$3
    local statemine_para_endpoint=$4
    local statemint_para_network=$5
    echo "  calling remove_assets_transfer_from_statemine:"
    echo "      relay_url: ${relay_url}"
    echo "      relay_chain_seed: ${relay_chain_seed}"
    echo "      statemine_para_id: ${statemine_para_id}"
    echo "      statemine_para_endpoint: ${statemine_para_endpoint}"
    echo "      statemint_para_network: ${statemint_para_network}"
    echo "      params:"

    local tmp_output_file=$(mktemp)
    generate_hex_encoded_call_data "remove-bridge-config" "${statemine_para_endpoint}" "${tmp_output_file}" $statemint_para_network
    local hex_encoded_data=$(cat $tmp_output_file)

    local dest=$(jq --null-input \
                    --arg statemine_para_id "$statemine_para_id" \
                    '{ "V3": { "parents": 0, "interior": { "X1": { "Parachain": $statemine_para_id } } } }')

    local message=$(jq --null-input \
                       --argjson hex_encoded_data $hex_encoded_data \
                       '
                       {
                          "V3": [
                                  {
                                    "UnpaidExecution": {
                                        "weight_limit": "Unlimited"
                                    }
                                  },
                                  {
                                    "Transact": {
                                      "origin_kind": "Superuser",
                                      "require_weight_at_most": {
                                        "ref_time": 1000000000,
                                        "proof_size": 0,
                                      },
                                      "call": {
                                        "encoded": $hex_encoded_data
                                      }
                                    }
                                  }
                          ]
                        }
                        ')

    echo ""
    echo "          dest:"
    echo "${dest}"
    echo ""
    echo "          message:"
    echo "${message}"
    echo ""
    echo "--------------------------------------------------"

    polkadot-js-api \
        --ws "${relay_url?}" \
        --seed "${relay_chain_seed?}" \
        --sudo \
        tx.xcmPallet.send \
            "${dest}" \
            "${message}"
}

# TODO: we need to fill sovereign account for bridge-hub, because, small ammouts does not match ExistentialDeposit, so no reserve pass
# SA for BH: MultiLocation { parents: 1, interior: X1(Parachain(1013)) } - 5Eg2fntRRwLinojmk3sh5xscp7F3S6Zzm5oDVtoLTALKiypR on Statemine

function transfer_asset_via_bridge() {
    local url=$1
    local seed=$2
    echo "  calling transfer_asset_via_bridge:"
    echo "      url: ${url}"
    echo "      seed: ${seed}"
    echo "      params:"


    local assets=$(jq --null-input \
        '
        {
            "V3": [
                {
                    "id": {
                        "Concrete": {
                            "parents": 1,
                            "interior": "Here"
                        }
                    },
                    "fun": {
                        "Fungible": 100000000
                    }
                }
            ]
        }
        '
    )


## TODO: decode some account to bytes: "id": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"

    local destination=$(jq --null-input \
        '
            {
                "V3": {
                    "parents": 2,
                    "interior": {
                        "X3": [
                            {
                                "GlobalConsensus": "Wococo"
                            },
                            {
                                "Parachain": 1000
                            },
                            {
                                "AccountId32": {
                                    "id": [28, 189, 45, 67, 83, 10, 68, 112, 90, 208, 136, 175, 49, 62, 24, 248, 11, 83, 239, 22, 179, 97, 119, 205, 75, 119, 184, 70, 242, 165, 240, 124]
                                }
                            }
                        ]
                    }
                }
            }
        '
    )

    echo ""
    echo "          assets:"
    echo "${assets}"
    echo ""
    echo "          destination:"
    echo "${destination}"
    echo ""
    echo "--------------------------------------------------"

#    local tmp_output_file=$(mktemp)
#    generate_hex_encoded_call_data "transfer-asset-via-bridge" "${url}" "${tmp_output_file}" "$assets" "$destination"
#    local hex_encoded_data=$(cat $tmp_output_file)

    polkadot-js-api \
        --ws "${url?}" \
        --seed "${seed?}" \
        tx.bridgeAssetsTransfer.transferAssetViaBridge \
            "${assets}" \
            "${destination}"
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

function init_ro_wo() {
    ensure_relayer

    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        ~/local_bridge_testing/bin/substrate-relay init-bridge rococo-to-bridge-hub-wococo \
	--source-host localhost \
	--source-port 9942 \
	--source-version-mode Auto \
	--target-host localhost \
	--target-port 8945 \
	--target-version-mode Auto \
	--target-signer //Bob
}

function init_wo_ro() {
    ensure_relayer

    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        ~/local_bridge_testing/bin/substrate-relay init-bridge wococo-to-bridge-hub-rococo \
        --source-host localhost \
        --source-port 9945 \
        --source-version-mode Auto \
        --target-host localhost \
        --target-port 8943 \
        --target-version-mode Auto \
        --target-signer //Bob
}

function run_relay() {
    ensure_relayer

    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        ~/local_bridge_testing/bin/substrate-relay relay-headers-and-messages bridge-hub-rococo-bridge-hub-wococo \
        --rococo-host localhost \
        --rococo-port 9942 \
        --rococo-version-mode Auto \
        --bridge-hub-rococo-host localhost \
        --bridge-hub-rococo-port 8943 \
        --bridge-hub-rococo-version-mode Auto \
        --bridge-hub-rococo-signer //Charlie \
        --wococo-headers-to-bridge-hub-rococo-signer //Bob \
        --wococo-parachains-to-bridge-hub-rococo-signer //Bob \
        --bridge-hub-rococo-transactions-mortality 4 \
        --wococo-host localhost \
        --wococo-port 9945 \
        --wococo-version-mode Auto \
        --bridge-hub-wococo-host localhost \
        --bridge-hub-wococo-port 8945 \
        --bridge-hub-wococo-version-mode Auto \
        --bridge-hub-wococo-signer //Charlie \
        --rococo-headers-to-bridge-hub-wococo-signer //Bob \
        --rococo-parachains-to-bridge-hub-wococo-signer //Bob \
        --bridge-hub-wococo-transactions-mortality 4 \
        --lane 00000001
}

case "$1" in
  start-rococo)
    ensure_binaries

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
    ensure_binaries

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
    init_ro_wo
    ;;
  init-wo-ro)
    # Init bridge Wococo->Rococo
    init_wo_ro
    ;;
  run-relay)
    init_ro_wo
    init_wo_ro
    run_relay
    ;;
  send-remark-local)
    ensure_polkadot_js_api
    send_xcm_transact_remark_from_statemine \
        "ws://127.0.0.1:9910" \
        "${STATEMINE_ACCOUNT_SEED_FOR_LOCAL}" \
        1013 \
        "Wococo" \
        1000 \
        "ws://127.0.0.1:9010"
    ;;
  send-trap-local)
    ensure_polkadot_js_api
    send_xcm_trap_from_statemine \
        "ws://127.0.0.1:9910" \
        "${STATEMINE_ACCOUNT_SEED_FOR_LOCAL}" \
        1013 \
        "Wococo" \
        1000
    ;;
  send-remark-rococo)
    ensure_polkadot_js_api
    send_xcm_transact_remark_from_statemine \
        "wss://ws-rococo-rockmine2-collator-node-0.parity-testnet.parity.io" \
        "${ROCKMINE2_ACCOUNT_SEED_FOR_ROCOCO}" \
        1013 \
        "Wococo" \
        1000 \
        "wss://ws-wococo-wockmint-collator-node-0.parity-testnet.parity.io"
    ;;
  send-trap-rococo)
    ensure_polkadot_js_api
    send_xcm_trap_from_statemine \
        "wss://ws-rococo-rockmine2-collator-node-0.parity-testnet.parity.io" \
        "${ROCKMINE2_ACCOUNT_SEED_FOR_ROCOCO}" \
        1013 \
        "Wococo" \
        1000
    ;;
  allow-transfer-on-statemine-local)
      ensure_polkadot_js_api
      allow_assets_transfer_from_statemine \
          "ws://127.0.0.1:9942" \
          "//Alice" \
          1000 \
          "ws://127.0.0.1:9910" \
          1013 \
          "Wococo" 1000
      ;;
  remove-assets-transfer-from-statemine-local)
      ensure_polkadot_js_api
      remove_assets_transfer_from_statemine \
          "ws://127.0.0.1:9942" \
          "//Alice" \
          1000 \
          "ws://127.0.0.1:9910" \
          "Wococo"
      ;;
  transfer-asset)
      ensure_polkadot_js_api
      transfer_asset_via_bridge \
          "ws://127.0.0.1:9910" \
          "//Alice"
      ;;
  stop)
    pkill -f polkadot
    pkill -f parachain
    ;;
  *) echo "A command is require. Supported commands: run-relay, send-trap-rococo/send-trap-local, send-remark-local/send-remark-rococo, allow-transfer-on-statemine-local/remove-assets-transfer-from-statemine-local, transfer-asset"; exit 1;;
esac
