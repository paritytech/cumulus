#!/bin/bash

# Helper script which outputs the diff to Substrate's `node-template`.
#
# Invoke with `SUBSTRATE_DIR=/path/to/substrate scripts/substrate-diff.sh`.

SUBSTRATE_DIR=${SUBSTRATE_DIR:-~/projects/substrate}
echo "Comparing with Substrate in directory $SUBSTRATE_DIR"

for FILE in `fd -e rs -e toml --search-path node/ --search-path runtime/`;
do
	DIFF=`diff $FILE $SUBSTRATE_DIR/bin/node-template/$FILE`
	EXIT_CODE=$?
	if [[ $EXIT_CODE -ne 0 ]]; then
		echo "Difference in: $FILE";
		echo "$DIFF"
	fi
done
