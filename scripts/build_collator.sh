#!/usr/bin/env bash

set -e

cd "$(cd "$(dirname "$0")" && git rev-parse --show-toplevel)"

time docker build \
    -f ./docker/test-parachain-collator.dockerfile \
    -t test-parachain-collator:latest .
