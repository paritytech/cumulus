#!/usr/bin/env bash
__dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
${__dir}/benchmarks-ci.sh assets statemine target/production
${__dir}/benchmarks-ci.sh assets statemint target/production
${__dir}/benchmarks-ci.sh assets westmint target/production
