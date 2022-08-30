#!/usr/bin/env bash

DIR=$(dirname -- "$0")

# This is the new version using `ruled-labels`
api_base="https://api.github.com/repos"
repo="$GITHUB_REPOSITORY"
pr_id="$GITHUB_PR"

pushd "$DIR/../ruled_labels" > /dev/null

# Fetch the labels for the PR under test
labels=$(curl -H "Authorization: token $GITHUB_PR_TOKEN" -s "$api_base/$repo/pulls/$pr_id" | jq ".labels | .[] " | grep '"name":' | tr -d ',' | sed 's/  "name": //g' | tr -d '"' | awk '{ORS=" "} {print$NF}')

ruled-labels --version
ruled-labels check --labels "${labels}"
