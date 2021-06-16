#!/bin/bash

source ./internal-scripts/common.sh

add_jobspec() {
  title "Adding Jobspec #$1 to Chainlink node..."

  CL_URL="http://localhost:669$1"

  login_cl "$CL_URL"

  ACCOUNT_ID=$2

  payload=$(
    cat <<EOF
{
  "initiators": [
    {
      "type": "external",
      "params": {
        "name": "test-ei",
        "body": {
          "endpoint": "substrate-node",
          "accountIds": ["${ACCOUNT_ID}"]
        }
      }
    }
  ],
  "tasks": [
    {
      "type": "httpget"
    },
    {
      "type": "jsonparse"
    },
    {
      "type": "multiply"
    },
    {
      "type": "substrate",
      "params": {
        "type": "int128"
      }
    }
  ]
}
EOF
  )

  JOBID=$(curl -s -b ./tmp/cookiefile -d "$payload" -X POST -H 'Content-Type: application/json' "$CL_URL/v2/specs" | jq -r '.data.id')
  echo "$JOBID" >> jobids.txt

  echo "Jobspec has been added to Chainlink node"
  title "Done adding jobspec #$1"
}
