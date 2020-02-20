FROM node:latest AS pjs

# It would be great to depend on a more stable tag, but we need some
# as-yet-unreleased features.
RUN yarn global add @polkadot/api-cli@0.10.0-beta.14

ENTRYPOINT [ "polkadot-js-api" ]
CMD [ "--version" ]

# To use the pjs build stage to access the blockchain from the host machine:
#
#   docker build -f docker/parachain-registrar.dockerfile --target pjs -t parachain-registrar:pjs .
#   alias pjs='docker run --rm --net cumulus_testing_net parachain-registrar:pjs --ws ws://172.28.1.1:9944'
#
# Then, as long as the chain is running, you can use the polkadot-js-api CLI like:
#
#   pjs query.sudo.key

FROM pjs
# the only thing left to do is to actually run the transaction.
COPY ./scripts/register_para.sh /usr/bin
# unset the previous stage's entrypoint
ENTRYPOINT []
CMD [ "/usr/bin/register_para.sh" ]
