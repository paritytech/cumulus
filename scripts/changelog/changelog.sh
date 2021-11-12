#!/usr/bin/env bash

set -e

DIR=`dirname "$BASH_SOURCE"`
pushd $DIR

# The following is fake and comes from CI
source .env

# call for instance as:
# ./build-cl.sh statemine-v5.0.0
# you may set the ENV NO_CACHE to force a reload
OWNER=paritytech
REPO=cumulus
REF1=${1}
REF2=${2:-HEAD}

CL_FILE=$REPO.json

echo Using CL_FILE: $CL_FILE
echo Building changelog for $OWNER/$REPO between $REF1 and $REF2

export RUST_LOG=debug;

if [[ ${NO_CACHE} ]]; then
    echo NO_CACHE set
fi

# This is acting as cache so we don't spend time querying while testing
if [[ ${NO_CACHE} || ! -f "$CL_FILE" ]]; then
    echo Generating $CL_FILE
    changelogerator $OWNER/$REPO -f $REF1 -t $REF2 > $CL_FILE
else
    echo Re-using $CL_FILE
fi

# Here we compose all the pieces together into one
# single big json file.
jq \
    --slurpfile srtool_rococo digests/rococo-srtool-digest.json \
    --slurpfile srtool_shell digests/shell-srtool-digest.json \
    --slurpfile srtool_westmint digests/westmint-srtool-digest.json \
    --slurpfile srtool_statemint digests/statemint-srtool-digest.json \
    --slurpfile srtool_statemine digests/statemine-srtool-digest.json \
    --slurpfile cl $CL_FILE \
    -n '{ cl: $cl[0], srtool: [
        { name: "rococo", data: $srtool_rococo[0] },
        { name: "shell", data: $srtool_shell[0] },
        { name: "westmint", data: $srtool_westmint[0] },
        { name: "statemint", data: $srtool_statemint[0] },
        { name: "statemine", data: $srtool_statemine[0] }
    ] }' | tee context.json

tera --env --env-key env --include-path templates --template templates/template.md.tera context.json | tee release-notes-cumulus.md

popd
