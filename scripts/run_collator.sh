#!/usr/bin/env bash

set -e

cd "$(cd "$(dirname "$0")" && git rev-parse --show-toplevel)"

docker-compose build
docker-compose up -d
