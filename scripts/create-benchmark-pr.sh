#!/usr/bin/env bash

set -Eeu -o pipefail
shopt -s inherit_errexit

PR_TITLE="$1"
HEAD_REF="$2"

ORG="paritytech"
REPO="$CI_PROJECT_NAME"
BASE_REF="$CI_COMMIT_BRANCH"

WEIGHTS_COMPARISON_URL_PARTS=(
  "https://weights.tasty.limo/compare?"
  "repo=$REPO&"
  "threshold=30&"
  "path_pattern=**%2Fweights%2F*.rs&"
  "method=guess-worst&"
  "ignore_errors=true&"
  "unit=time&"
  "old=$BASE_REF&"
  "new=$HEAD_REF"
)
printf -v WEIGHTS_COMPARISON_URL %s "${WEIGHTS_COMPARISON_URL_PARTS[@]}"

PAYLOAD="$(jq -n \
  --arg title "$PR_TITLE" \
  --arg body "
This PR is generated automatically by CI. (Once merged please backport to master and node release branch.)

Compare the weights with \`$BASE_REF\`: $WEIGHTS_COMPARISON_URL
" \
  --arg base "$BASE_REF" \
  --arg head "$HEAD_REF" \
  '{
      title: $title,
      body: $body,
      head: $head,
      base: $base
   }'
)"

echo "PAYLOAD: $PAYLOAD"

curl \
  -H "Authorization: token $GITHUB_TOKEN" \
  -X POST \
  -d "$PAYLOAD" \
  "https://api.github.com/repos/$ORG/$REPO/pulls"
