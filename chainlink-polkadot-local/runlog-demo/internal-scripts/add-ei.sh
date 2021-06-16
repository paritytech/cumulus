#!/bin/bash

source ./internal-scripts/common.sh

add_ei() {
  title "Adding External Initiator #$1 to Chainlink node..."

  CL_URL="http://localhost:669$1"

  login_cl "$CL_URL"

  payload=$(
    cat <<EOF
{
  "name": "test-ei",
  "url": "http://runlog-demo_external-initiator-node$1_1:8080/jobs"
}
EOF
  )

  result=$(curl -s -b ./tmp/cookiefile -d "$payload" -X POST -H 'Content-Type: application/json' "$CL_URL/v2/external_initiators")

  EI_IC_ACCESSKEY=$(jq -r '.data.attributes.incomingAccessKey' <<<"$result")
  EI_IC_SECRET=$(jq -r '.data.attributes.incomingSecret' <<<"$result")
  EI_CI_ACCESSKEY=$(jq -r '.data.attributes.outgoingToken' <<<"$result")
  EI_CI_SECRET=$(jq -r '.data.attributes.outgoingSecret' <<<"$result")

  run_ei "$1" "$EI_CI_ACCESSKEY" "$EI_CI_SECRET" "$EI_IC_ACCESSKEY" "$EI_IC_SECRET"

  echo "EI has been added to Chainlink node"
  title "Done adding EI #$1"
}
