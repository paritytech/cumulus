#!/bin/bash

# A script to udpate bridges repo as subtree to Cumulus
# Usage:
#       ./scripts/bridges_update_subtree.sh fetch
#       ./scripts/bridges_update_subtree.sh fetch

set -e

BRIDGES_BRANCH="${BRANCH:-master}"
BRIDGES_TARGET_DIR="${TARGET_DIR:-bridges}"

# the script is able to work only on clean git copy
[[ -z "$(git status --porcelain)" ]] || {
    echo >&2 "The git copy must be clean (stash all your changes):";
    git status --porcelain
    exit 1;
}

function fetch() {
    local bridges_remote=$(git remote -v | grep "parity-bridges-common.git (fetch)" | head -n1 | awk '{print $1;}')
    if [ -z "$bridges_remote" ]; then
        echo ""
        echo "Adding new remote: 'bridges' repo..."
        echo ""
        echo "... check your YubiKey ..."
        git remote add -f bridges git@github.com:paritytech/parity-bridges-common.git
        bridges_remote="bridges"
    else
        echo ""
        echo "Fetching remote: '${bridges_remote}' repo..."
        echo ""
        echo "... check your YubiKey ..."
        git fetch ${bridges_remote} --prune
    fi

    echo ""
    echo "Syncing/updating subtree with remote branch '${bridges_remote}/$BRIDGES_BRANCH' to target directory: '$BRIDGES_TARGET_DIR'"
    echo ""
    echo "... check your YubiKey ..."
    git subtree pull --prefix=$BRIDGES_TARGET_DIR ${bridges_remote} $BRIDGES_BRANCH --squash
    echo ""
    echo ""
    echo ""
    echo ".. if there are any conflict, please, resolve and then 'git merge --continue'"
}

function remove() {
    # remove unneded stuff
    rm -R bridges/.config
    rm -R bridges/deployments
    rm -R bridges/docs
    rm -R bridges/fuzz
    rm -R bridges/.github
    rm -R bridges/.maintain
    rm -R bridges/relays
    rm -R bridges/scripts
    rm -R bridges/bin/millau/node
    rm -R bridges/bin/rialto
    rm -R bridges/bin/rialto-parachain
    rm -R bridges/bin/.keep

    # remove all file from top directory
    find ./bridges -maxdepth 1 -type f -exec rm "{}" \;


    git add --all
    # TODO: add some specific message
    git commit --amend -S -m "updating bridges subtree"
}

case "$1" in
    fetch)
        fetch
        ;;
    remove)
        remove
        ;;
    all)
        fetch
        remove
        ;;
esac
