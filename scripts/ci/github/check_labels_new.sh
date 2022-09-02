#!/usr/bin/env bash

DIR=$(dirname -- "$0")

# This is the new version using `ruled-labels`
api_base="https://api.github.com/repos"
repo="$GITHUB_REPOSITORY"
pr_id="$GITHUB_PR"

echo "repo: $repo"
echo "pr_ir: $pr_id"

pushd "$DIR/../ruled_labels" > /dev/null

# Fetch the labels for the PR under test
labels="$(curl -H "Authorization: token $GITHUB_PR_TOKEN" -s "$api_base/$repo/pulls/$pr_id" | jq ".labels | .[] | .name" | tr '\n' ' ')"

echo "labels: $labels"

ruled-labels --version
cmd='ruled-labels check --labels '${labels[@]}
echo "cmd: $cmd"
bash -c "$cmd"
# ruled-labels check --labels ${labels[@]}
