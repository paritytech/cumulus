#!/bin/bash

set -e

res=`curl -s http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d '{
    "jsonrpc":"2.0",
    "id":1,
    "method":"system_health",
    "params": [
]
}'  | jq -r '.result.isSyncing'`

[ 'x'$res == 'xfalse' ]