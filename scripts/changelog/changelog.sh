#!/usr/bin/env bash

set -e

DIR=`dirname "$BASH_SOURCE"`
pushd $DIR

# The following is fake and shoud come from CI
source .env

# call for instance as:
# ./changelog.sh 0.9.11 0.9.12 statemine-v5.0.0
OWNER=paritytech
REPO=cumulus

# Refs used to build the refs for the substrate and polkadot repos
# For instance: 0.9.11 and 0.9.12
V1=${1}
V2=${2}

# Ref for the start of the changelog in the cumulus repo
REF1=${3}
REF2=${4:-HEAD}

# you may set the ENV NO_CACHE to force fetching from Github
# NO_CACHE=1

CUMULUS=$REPO.json

SUBSTRATE=substrate.json
REF1_SUBSTRATE=polkadot-v$V1
REF2_SUBSTRATE=polkadot-v$V2

POLKADOT=polkadot.json
REF1_POKADOT=v$V1
REF2_POKADOT=v$V2

echo Using CUMULUS: $CUMULUS
echo Building changelog for $OWNER/$REPO between $REF1 and $REF2

export RUST_LOG=debug;

if [[ ${NO_CACHE} ]]; then
    echo NO_CACHE set
fi

# This is acting as cache so we don't spend time querying while testing
if [[ ${NO_CACHE} || ! -f "$CUMULUS" ]]; then
    echo Fetching data for Cumulus into $CUMULUS
    changelogerator $OWNER/$REPO -f $REF1 -t $REF2 > $CUMULUS
else
    echo Re-using $CUMULUS
fi

if [[ ${NO_CACHE} || ! -f "$POLKADOT" ]]; then
    echo Fetching data for Polkadot into $POLKADOT
    changelogerator $OWNER/polkadot -f v$V1 -t v$V2 > $POLKADOT
else
    echo Re-using $POLKADOT
fi

if [[ ${NO_CACHE} || ! -f "$SUBSTRATE" ]]; then
    echo Fetching data for Substrate into $SUBSTRATE
    changelogerator $OWNER/substrate -f polkadot-v$V1 -t polkadot-v$V2 > $SUBSTRATE
else
    echo Re-using $SUBSTRATE
fi

# Here we compose all the pieces together into one
# single big json file.
jq \
    --slurpfile cumulus $CUMULUS \
    --slurpfile substrate $SUBSTRATE \
    --slurpfile polkadot $POLKADOT \
    --slurpfile srtool_rococo digests/rococo-srtool-digest.json \
    --slurpfile srtool_shell digests/shell-srtool-digest.json \
    --slurpfile srtool_westmint digests/westmint-srtool-digest.json \
    --slurpfile srtool_statemint digests/statemint-srtool-digest.json \
    --slurpfile srtool_statemine digests/statemine-srtool-digest.json \
    -n '{
            cumulus: $cumulus[0],
            substrate: $substrate[0],
            polkadot: $polkadot[0],
        srtool: [
        { name: "rococo", data: $srtool_rococo[0] },
        { name: "shell", data: $srtool_shell[0] },
        { name: "westmint", data: $srtool_westmint[0] },
        { name: "statemint", data: $srtool_statemint[0] },
        { name: "statemine", data: $srtool_statemine[0] }
    ] }' | tee context.json

tera --env --env-key env --include-path templates --template templates/template.md.tera context.json | tee release-notes-cumulus.md

popd
