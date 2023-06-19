#!/bin/bash

# import common functions
source "$(dirname "$0")"/bridges_rococo_wococo.sh "import"

# Address: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
# AccountId: [212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125]
ASSET_HUB_KUSAMA_ACCOUNT_SEED_FOR_LOCAL="//Alice"
# Address: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
# AccountId: [212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125]
ASSET_HUB_POLKADOT_ACCOUNT_ADDRESS_FOR_LOCAL="5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"

# SovereignAccount for `MultiLocation { parents: 2, interior: X2(GlobalConsensus(Rococo), Parachain(1000)) }` => 5DLdHR78ujzS93zCVeyZB1qRFjBCdMnJwnpSBSRZ6jMX8R5y
#
# use sp_core::crypto::Ss58Codec;
# println!("{}",
#     frame_support::sp_runtime::AccountId32::new(
#         GlobalConsensusParachainConvertsFor::<UniversalLocation, [u8; 32]>::convert_ref(
#             MultiLocation { parents: 2, interior: X2(GlobalConsensus(Kusama), Parachain(1000)) }).unwrap()
#		).to_ss58check_with_version(42_u16.into())
# );
KUSAMA_STATEMINE_1000_SOVEREIGN_ACCOUNT="5DLdHR78ujzS93zCVeyZB1qRFjBCdMnJwnpSBSRZ6jMX8R5y"

function init_ksm_dot() {
    ensure_relayer

    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        ~/local_bridge_testing/bin/substrate-relay init-bridge kusama-to-bridge-hub-polkadot \
	--source-host localhost \
	--source-port 9942 \
	--source-version-mode Auto \
	--target-host localhost \
	--target-port 8945 \
	--target-version-mode Auto \
	--target-signer //Bob
}

function init_dot_ksm() {
    ensure_relayer

    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        ~/local_bridge_testing/bin/substrate-relay init-bridge polkadot-to-bridge-hub-kusama \
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
        ~/local_bridge_testing/bin/substrate-relay relay-headers-and-messages bridge-hub-kusama-bridge-hub-polkadot \
        --kusama-host localhost \
        --kusama-port 9942 \
        --kusama-version-mode Auto \
        --bridge-hub-kusama-host localhost \
        --bridge-hub-kusama-port 8943 \
        --bridge-hub-kusama-version-mode Auto \
        --bridge-hub-kusama-signer //Charlie \
        --polkadot-headers-to-bridge-hub-kusama-signer //Bob \
        --polkadot-parachains-to-bridge-hub-kusama-signer //Bob \
        --bridge-hub-kusama-transactions-mortality 4 \
        --polkadot-host localhost \
        --polkadot-port 9945 \
        --polkadot-version-mode Auto \
        --bridge-hub-polkadot-host localhost \
        --bridge-hub-polkadot-port 8945 \
        --bridge-hub-polkadot-version-mode Auto \
        --bridge-hub-polkadot-signer //Charlie \
        --kusama-headers-to-bridge-hub-polkadot-signer //Bob \
        --kusama-parachains-to-bridge-hub-polkadot-signer //Bob \
        --bridge-hub-polkadot-transactions-mortality 4 \
        --lane 00000000
}

case "$1" in
  run-relay)
    init_ksm_dot
    init_dot_ksm
    run_relay
    ;;
  transfer-asset-from-asset-hub-kusama-local)
      ensure_polkadot_js_api
      transfer_asset_via_bridge \
          "ws://127.0.0.1:9910" \
          "$ASSET_HUB_KUSAMA_ACCOUNT_SEED_FOR_LOCAL" \
          "$ASSET_HUB_POLKADOT_ACCOUNT_ADDRESS_FOR_LOCAL" \
          "Polkadot"
      ;;
  drip)
      transfer_balance \
          "ws://127.0.0.1:9010" \
          "//Alice" \
          "$KUSAMA_STATEMINE_1000_SOVEREIGN_ACCOUNT" \
          $((1000000000 + 50000000000 * 20))
      ;;
  stop)
    pkill -f polkadot
    pkill -f parachain
    ;;
  *)
    echo "A command is require. Supported commands for:
    Local (zombienet) run:
          - run-relay
          - transfer-asset-from-asset-hub-kusama-local";
    exit 1
    ;;
esac
