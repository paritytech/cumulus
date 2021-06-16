#!/usr/bin/env bash

set -e

echo "*** Config EI ***"

cd $(dirname ${BASH_SOURCE[0]})/..
cat <<EOF | docker exec -i chainlink-node sh
chainlink admin login -f /run/secrets/apicredentials
chainlink initiators create substrate http://external-initiator:8080/jobs > credentials
EOF
docker cp chainlink-node:/home/root/credentials credentials
cat credentials | awk 'NR==5' | awk -F[' '] '{print "EI_IC_ACCESSKEY="$6} {print "EI_IC_SECRET="$8} {print "EI_CI_ACCESSKEY="$10} {print "EI_CI_SECRET="$12}' > external_initiator.env
rm -rf credentials