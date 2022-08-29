#!/usr/bin/env bash

DIR=$(dirname -- "$0")

# This is the new version using `ruled-labels`
repo="$GITHUB_REPOSITORY"
pr="$GITHUB_PR"

pushd "$DIR/../ruled_labels" > /dev/null

# TODO: Fetch the labels for the PR under test
labels=("B0-Silent" "X1-Runtime" "P1" "D1-audited 👍")

ruled-labels --version
ruled-labels check --labels "${labels[@]}"
