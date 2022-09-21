#!/bin/bash

# Script for generating messages from a source chain to a target chain.
# Prerequisites: mounting the common folder in the docker container (Adding the following volume entry):
# - ./bridges/common:/common
# It can be used by executing `source /common/generate_messages.sh` in a different script,
# after setting the following variables:
# SOURCE_CHAIN
# TARGET_CHAIN
# MAX_SUBMIT_DELAY_S
# SEND_MESSAGE - the command that is executed to send a message
# MESSAGE_LANE
# SECONDARY_MESSAGE_LANE - optional
# SECONDARY_EXTRA_ARGS - optional, for example "--use-xcm-pallet"
# EXTRA_ARGS - for example "--use-xcm-pallet"
# REGULAR_PAYLOAD
# BATCH_PAYLOAD
# MAX_UNCONFIRMED_MESSAGES_AT_INBOUND_LANE

SECONDARY_MESSAGE_LANE=${SECONDARY_MESSAGE_LANE:-""}
SECONDARY_EXTRA_ARGS=${SECONDARY_EXTRA_ARGS:-""}

# Sleep a bit between messages
rand_sleep() {
	SUBMIT_DELAY_S=`shuf -i 0-$MAX_SUBMIT_DELAY_S -n 1`
	echo "Sleeping $SUBMIT_DELAY_S seconds..."
	sleep $SUBMIT_DELAY_S
	NOW=`date "+%Y-%m-%d %H:%M:%S"`
	echo "Woke up at $NOW"
}

# last time when we have been asking for conversion rate update
LAST_CONVERSION_RATE_UPDATE_TIME=0
# conversion rate override argument
CONVERSION_RATE_OVERRIDE="--conversion-rate-override metric"

# start sending large messages immediately
LARGE_MESSAGES_TIME=0
# start sending message packs in a hour
BUNCH_OF_MESSAGES_TIME=3600

while true
do
	rand_sleep

	# ask for latest conversion rate. We're doing that because otherwise we'll be facing
	# bans from the conversion rate provider
	if [ $SECONDS -ge $LAST_CONVERSION_RATE_UPDATE_TIME ]; then
		CONVERSION_RATE_OVERRIDE="--conversion-rate-override metric"
		CONVERSION_RATE_UPDATE_DELAY=`shuf -i 300-600 -n 1`
		LAST_CONVERSION_RATE_UPDATE_TIME=$((SECONDS + $CONVERSION_RATE_UPDATE_DELAY))
	fi

	# send regular message
	echo "Sending Message from $SOURCE_CHAIN to $TARGET_CHAIN"
	SEND_MESSAGE_OUTPUT=`$SEND_MESSAGE --lane $MESSAGE_LANE $EXTRA_ARGS $CONVERSION_RATE_OVERRIDE raw $REGULAR_PAYLOAD 2>&1`
	echo $SEND_MESSAGE_OUTPUT
	if [ "$CONVERSION_RATE_OVERRIDE" = "--conversion-rate-override metric" ]; then
		ACTUAL_CONVERSION_RATE_REGEX="conversion rate override: ([0-9\.]+)"
		if [[ $SEND_MESSAGE_OUTPUT =~ $ACTUAL_CONVERSION_RATE_REGEX ]]; then
			CONVERSION_RATE=${BASH_REMATCH[1]}
			echo "Read updated conversion rate: $CONVERSION_RATE"
			CONVERSION_RATE_OVERRIDE="--conversion-rate-override $CONVERSION_RATE"
		else
			echo "Error: unable to find conversion rate in send-message output. Will keep using on-chain rate"
			CONVERSION_RATE_OVERRIDE=""
		fi
	fi

	if [ ! -z $SECONDARY_MESSAGE_LANE ]; then
		echo "Sending Message from $SOURCE_CHAIN to $TARGET_CHAIN using secondary lane: $SECONDARY_MESSAGE_LANE"
		$SEND_MESSAGE \
			--lane $SECONDARY_MESSAGE_LANE \
			$SECONDARY_EXTRA_ARGS \
			$CONVERSION_RATE_OVERRIDE \
			raw $REGULAR_PAYLOAD
	fi

	# every other hour we're sending 3 large (size, weight, size+weight) messages
	if [ $SECONDS -ge $LARGE_MESSAGES_TIME ]; then
		LARGE_MESSAGES_TIME=$((SECONDS + 7200))

		rand_sleep
		echo "Sending Maximal Size Message from $SOURCE_CHAIN to $TARGET_CHAIN"
		$SEND_MESSAGE \
			--lane $MESSAGE_LANE \
			$CONVERSION_RATE_OVERRIDE \
			sized max
	fi

	# every other hour we're sending a bunch of small messages
	if [ $SECONDS -ge $BUNCH_OF_MESSAGES_TIME ]; then
		BUNCH_OF_MESSAGES_TIME=$((SECONDS + 7200))

		for i in $(seq 0 $MAX_UNCONFIRMED_MESSAGES_AT_INBOUND_LANE);
		do
			$SEND_MESSAGE \
				--lane $MESSAGE_LANE \
				$EXTRA_ARGS \
				$CONVERSION_RATE_OVERRIDE \
				raw $BATCH_PAYLOAD
		done

	fi
done
