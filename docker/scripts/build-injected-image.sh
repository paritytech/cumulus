#!/usr/bin/env bash

OWNER=${OWNER:-parity}
IMAGE_NAME=${IMAGE_NAME:-polkadot-parachain}
TAGS=${TAGS[@]:-latest}
IFS=',' read -r -a TAG_ARRAY <<< "$TAGS"
TAG_ARGS=" "

echo "The image ${IMAGE_NAME} will be tagged with ${TAG_ARRAY[*]}"
for tag in "${TAG_ARRAY[@]}"; do
  TAG_ARGS+="--tag ${OWNER}/${IMAGE_NAME}:${tag} "
done

echo "$TAG_ARGS"

docker build --no-cache \
    --build-arg IMAGE_NAME=$IMAGE_NAME \
    ${TAG_ARGS} \
    -f ./docker/injected.Dockerfile \
    . && docker images
