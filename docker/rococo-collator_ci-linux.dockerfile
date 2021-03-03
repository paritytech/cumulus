FROM paritytech/ci-linux:production as builder
LABEL description="This is the build stage for Polkadot. Here we create the binary."

WORKDIR /cumulus

COPY . /cumulus

RUN cargo build --release

# ===== SECOND STAGE ======

FROM debian:buster-slim
LABEL description="This is the 2nd stage: a very small image where we copy the Polkadot binary."
RUN mkdir -p /usr/local/bin/cumulus
COPY --from=builder /cumulus/target/release/rococo-collator /usr/local/bin/rococo-collator
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
USER cumulus
EXPOSE 30333 9933 9944
VOLUME ["/data"]

# Tini allows us to avoid several Docker edge cases, see https://github.com/krallin/tini.
ENTRYPOINT ["tini", "--", "rococo-collator"]
