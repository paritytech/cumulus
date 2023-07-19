#!/bin/sh
set -eux

time docker build . -t local/substrate-relay --build-arg=PROJECT=substrate-relay
time docker build . -t local/rialto-bridge-node --build-arg=PROJECT=rialto-bridge-node
time docker build . -t local/millau-bridge-node --build-arg=PROJECT=millau-bridge-node
time docker build . -t local/rialto-parachain-collator --build-arg=PROJECT=rialto-parachain-collator
