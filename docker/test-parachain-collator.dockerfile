FROM rust:buster as builder

RUN apt-get update && apt-get install time clang libclang-dev llvm -y
RUN rustup toolchain install nightly
RUN rustup target add wasm32-unknown-unknown --toolchain nightly
RUN command -v wasm-gc || cargo +nightly install --git https://github.com/alexcrichton/wasm-gc --force

WORKDIR /paritytech/cumulus

COPY . .

RUN time cargo build --release -p cumulus-test-parachain-collator

# the collator stage is normally built once, cached, and then ignored, but can
# be specified with the --target build flag. This adds some extra tooling to the
# image, which is required for a launcher script. The script simply adds two
# arguments to the list passed in:
#
#   --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/PEER_ID
#
# with the appropriate ID for both Alice and Bob
FROM debian:buster-slim as collator
RUN apt-get update && apt-get install jq curl bash -y && \
    curl -sSo /wait-for-it.sh https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh && \
    chmod +x /wait-for-it.sh
COPY --from=builder \
    /paritytech/cumulus/target/release/cumulus-test-parachain-collator /usr/bin
COPY ./inject_bootnodes.sh /usr/bin
CMD ["/usr/bin/inject_bootnodes.sh"]


FROM debian:buster-slim
COPY --from=builder \
    /paritytech/cumulus/target/release/cumulus-test-parachain-collator /usr/bin

CMD ["/usr/bin/cumulus-test-parachain-collator"]
