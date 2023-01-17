#!/bin/bash

function fetch() {
    # TODO: check if already existing remote bridges
    #       if not add one: git remote add -f bridges git@github.com:paritytech/parity-bridges-common.git

    BRIDGES_BRANCH="${BRANCH:-master}"
    echo "Syncing with branch: '$BRIDGES_BRANCH'"

    # rm -R bridges
    # git add --all
    # echo "... check YubiKey"
    # git commit -S -m "updating bridges subtree"
    # echo "... check YubiKey"
    #git subtree add --prefix=bridges bridges $BRIDGES_BRANCH --squash

    # OR

    echo "... check YubiKey"
    git fetch bridges --prune
    git subtree pull --prefix=bridges bridges $BRIDGES_BRANCH --squash
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
