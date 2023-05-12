#!/bin/bash

# import common functions
source "$(dirname "$0")"/bridges_rococo_wococo.sh "import"

# Address: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
# AccountId: [212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125]
STATEMINE_ACCOUNT_SEED_FOR_LOCAL="//Alice"
# Address: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
# AccountId: [212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125]
WESTMINT_ACCOUNT_ADDRESS_FOR_LOCAL="5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"

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
        --lane 00000001
}

case "$1" in
  run-relay)
    init_ksm_dot
    init_dot_ksm
    run_relay
    ;;
  allow-transfers-local)
      # this allows send transfers on statemine (by governance-like)
      ./$0 "allow-transfer-on-statemine-local"
      # this allows receive transfers on westmint (by governance-like)
      ./$0 "allow-transfer-on-westmint-local"
      ;;
  allow-transfer-on-statemine-local)
      ensure_polkadot_js_api
      allow_assets_transfer_send \
          "ws://127.0.0.1:9942" \
          "//Alice" \
          1000 \
          "ws://127.0.0.1:9910" \
          1002 \
          "Polkadot" 1000
      ;;
  allow-transfer-on-westmint-local)
      ensure_polkadot_js_api
      allow_assets_transfer_receive \
          "ws://127.0.0.1:9945" \
          "//Alice" \
          1000 \
          "ws://127.0.0.1:9010" \
          1002 \
          "Kusama" \
          1000
      # drip SovereignAccount for `MultiLocation { parents: 2, interior: X2(GlobalConsensus(Kusama), Parachain(1000)) }` => 5HnhuKCrvrJ9EqnV5mmkJ9kZPiBEH1aWXA8w4maTkXp86g4T
      # use sp_core::crypto::Ss58Codec;
      # println!("{}",
      #     frame_support::sp_runtime::AccountId32::new(
      #         GlobalConsensusParachainConvert::<[u8; 32]>::convert_ref(
      #             MultiLocation { parents: 2, interior: X2(GlobalConsensus(Kusama), Parachain(1000)) }).unwrap()
      #		).to_ss58check_with_version(42_u16.into())
      # );
      transfer_balance \
          "ws://127.0.0.1:9010" \
          "//Alice" \
          "5HnhuKCrvrJ9EqnV5mmkJ9kZPiBEH1aWXA8w4maTkXp86g4T" \
          $((1000000000 + 50000000000 * 20)) # ExistentialDeposit + maxTargetLocationFee * 20
      # create foreign assets for native Statemine token
      force_create_foreign_asset \
          "ws://127.0.0.1:9945" \
          "//Alice" \
          1000 \
          "ws://127.0.0.1:9010" \
          "Kusama" \
          "5HnhuKCrvrJ9EqnV5mmkJ9kZPiBEH1aWXA8w4maTkXp86g4T"
      ;;
  remove-assets-transfer-from-statemine-local)
      ensure_polkadot_js_api
      remove_assets_transfer_send \
          "ws://127.0.0.1:9942" \
          "//Alice" \
          1000 \
          "ws://127.0.0.1:9910" \
          "Polkadot"
      ;;
  transfer-asset-from-statemine-local)
      ensure_polkadot_js_api
      transfer_asset_via_bridge \
          "ws://127.0.0.1:9910" \
          "$STATEMINE_ACCOUNT_SEED_FOR_LOCAL" \
          "$WESTMINT_ACCOUNT_ADDRESS_FOR_LOCAL" \
          "Polkadot"
      ;;
  ping-via-bridge-from-statemine-local)
      ensure_polkadot_js_api
      ping_via_bridge \
          "ws://127.0.0.1:9910" \
          "$STATEMINE_ACCOUNT_SEED_FOR_LOCAL" \
          "$WESTMINT_ACCOUNT_ADDRESS_FOR_LOCAL" \
          "Polkadot"
      ;;
  drip)
      transfer_balance \
          "ws://127.0.0.1:9010" \
          "//Alice" \
          "5HnhuKCrvrJ9EqnV5mmkJ9kZPiBEH1aWXA8w4maTkXp86g4T" \
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
          - allow-transfers-local
              - allow-transfer-on-statemine-local
              - allow-transfer-on-westmint-local
              - remove-assets-transfer-from-statemine-local
          - transfer-asset-from-statemine-local
          - ping-via-bridge-from-statemine-local";
    exit 1
    ;;
esac
