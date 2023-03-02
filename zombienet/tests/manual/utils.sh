#!/bin/bash

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
}

function get_runtime_version() {
    local url=$1
    echo "...getting runtime version from '$url'"
    polkadot-js-api \
        --ws "${url?}" \
        query.system.lastRuntimeUpgrade

    polkadot-js-api \
        --ws "${url?}" \
        query.polkadotXcm.safeXcmVersion
}

function get_account_data() {
    local url=$1
    local account=$2
    echo "...getting data for '$account' from '$url'"
    polkadot-js-api \
        --ws "${url?}" \
        query.system.account \
            "${account}"

    polkadot-js-api \
        --ws "${url?}" \
        query.assets.account 1 "${account}"

    polkadot-js-api \
        --ws "${url?}" \
        query.uniques.asset 1 1 | jq --arg owner ${account} 'select(.asset.owner==$owner)'
}

function create_asset() {
    local url=$1
    local account=$2
    local seed=$3
    echo "...creating assets for '$account' from '$url'"
    polkadot-js-api \
        --ws "${url?}" \
        --seed "${seed}" \
        tx.assets.create 1 "${account}" 1
}

function mint_asset() {
    local url=$1
    local seed=$2
    local to_account=$3
    echo "...minting asset for '$to_account' from '$url'"
    polkadot-js-api \
        --ws "${url?}" \
        --seed "${seed}" \
        tx.assets.mint 1 "${to_account}" 5
}

function create_unique() {
    local url=$1
    local account=$2
    local seed=$3
    echo "...creating unique for '$account' from '$url'"
    polkadot-js-api \
        --ws "${url?}" \
        --seed "${seed}" \
        tx.uniques.create 1 "${account}"
}

function mint_unique() {
    local url=$1
    local seed=$2
    local to_account=$3
    echo "...minting unique for '$to_account' from '$url'"
    polkadot-js-api \
        --ws "${url?}" \
        --seed "${seed}" \
        tx.uniques.mint 1 1 "${to_account}"
}

function set_price_unique() {
    local url=$1
    local seed=$2
    local to_account=$3
    echo "...setting price unique for '$to_account' from '$url'"
    polkadot-js-api \
        --ws "${url?}" \
        --seed "${seed}" \
        tx.uniques.setPrice 1 1 9 "${to_account}"
}

function buy_unique() {
    local url=$1
    local seed=$2
    echo "...buying unique from '$url'"
    polkadot-js-api \
        --ws "${url?}" \
        --seed "${seed}" \
        tx.uniques.buyItem 1 1 10
}

function authorize_upgrade() {
    local url=$1
    local seed=$2
    local hash=$3
    echo "...calling authorize_upgrade from '$url', hash: $hash"
    polkadot-js-api \
        --ws "${url?}" \
        --sudo \
        --seed "${seed}" \
      tx.parachainSystem.authorizeUpgrade $hash
}

case "$1" in
  data)
    ensure_polkadot_js_api
    get_account_data "$2" "$3"
    ;;
  version)
    ensure_polkadot_js_api
    get_runtime_version "$2" "$3"
    ;;
  create-asset)
    ensure_polkadot_js_api
    create_asset "$2" "$3" "$4"
    ;;
  mint-asset)
    ensure_polkadot_js_api
    mint_asset "$2" "$3" "$4"
    ;;
  create-unique)
    ensure_polkadot_js_api
    create_unique "$2" "$3" "$4"
    ;;
  mint-unique)
    ensure_polkadot_js_api
    mint_unique "$2" "$3" "$4"
    ;;
  set-price-unique)
    ensure_polkadot_js_api
    set_price_unique "$2" "$3" "$4"
    ;;
  buy-unique)
    ensure_polkadot_js_api
    buy_unique "$2" "$3" "$4"
    ;;
  authorize-upgrade)
    ensure_polkadot_js_api
    authorize_upgrade "$2" "$3" "$4"
    ;;
esac
