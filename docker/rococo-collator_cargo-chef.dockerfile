### Planning Stage ###
FROM rust:1.50-buster as planner
WORKDIR /cumulus
# We only pay the installation cost once, 
# it will be cached from the second build onwards
# To ensure a reproducible build consider pinning 
# the cargo-chef version with `--version X.X.X`
# https://github.com/LukeMathWalker/cargo-chef
RUN cargo install cargo-chef 
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

### Caching Stage ###
FROM rust:1.50-buster as cacher
RUN apt-get update && \
		apt-get install -y cmake pkg-config libssl-dev git clang llvm
WORKDIR /cumulus
RUN cargo install cargo-chef
ENV RUST_BACKTRACE=1
# Install latest version of nightly
RUN rustup toolchain install nightly && \
  	rustup target add wasm32-unknown-unknown --toolchain nightly && \
  	rustup default stable
COPY --from=planner /cumulus/recipe.json recipe.json
RUN rustc --version
RUN cargo chef cook --release --recipe-path recipe.json

### Building Stage ###
FROM rust:1.50-buster as builder
RUN apt-get update && \
		apt-get install -y cmake pkg-config libssl-dev git clang llvm
WORKDIR /cumulus
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /cumulus/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
ENV RUST_BACKTRACE=1
RUN rustup toolchain install nightly && \
  	rustup target add wasm32-unknown-unknown --toolchain nightly && \
  	rustup default stable
RUN cargo build --release

### Runtime Stage ###
FROM debian:buster-slim as runtime
RUN apt-get update --fix-missing && \
    apt-get install -y tini && \
    rm -rf /var/lib/apt/lists/*
# Non-root user for security purposes.
# UIDs below 10,000 are a security risk, as a container breakout could result
# in the container being ran as a more privileged user on the host kernel with
# the same UID.
# Static GID/UID is also useful for chown'ing files outside the container where
# such a user does not exist.
RUN groupadd --gid 10001 cumulus && \
    useradd  --home-dir /home/cumulus \
             --create-home \
             --shell /bin/bash \
             --gid cumulus \
             --groups cumulus \
             --uid 10000 cumulus 
RUN mkdir -p /home/cumulus/.local/share && \
	mkdir /home/cumulus/data && \
	chown -R cumulus:cumulus /home/cumulus && \
	ln -s /home/cumulus/data /home/cumulus/.local/share

WORKDIR /home/cumulus
COPY --from=builder /cumulus/target/release/rococo-collator /usr/local/bin/rococo-collator

# Tini allows us to avoid several Docker edge cases, see https://github.com/krallin/tini.
ENTRYPOINT ["tini", "--", "rococo-collator"]

# Use the non-root user to run our application
USER cumulus
EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]
