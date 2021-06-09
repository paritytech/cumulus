#!/bin/bash
# A simple script to run the commands based on polkadot/polkadot-collator

command=$1
shift

echo "the command is ${command}"
if [ "${command}" = "polkadot" ]; then
    polkadot $@
else
    polkadot-collator $@
fi