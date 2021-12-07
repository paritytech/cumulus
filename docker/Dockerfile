
FROM paritytech/ci-linux:production as builder

LABEL description="This is the build stage for kabocha. Here we create the binary."

ARG PROFILE=release
WORKDIR /kabocha

COPY . /kabocha/
#RUN  fallocate -l 1G /swapfile

RUN rustup uninstall nightly
RUN rustup install nightly
RUN rustup update nightly
RUN rustup target add wasm32-unknown-unknown --toolchain nightly
RUN cargo clean
RUN cargo update
#RUN cargo +nightly-2021-11-01 check 
RUN cargo build --release -j 1

# ===== SECOND STAGE ======

FROM debian:buster-slim
LABEL description="This is the 2nd stage: a very small image where we copy the kabocha binary."
ARG PROFILE=release
COPY --from=builder /kabocha/target/$PROFILE/parachain-collator /usr/local/bin
COPY /specs /specs/
RUN useradd -m -u 1000 -U -s /bin/sh -d /kabocha kabocha && \
	mkdir -p /kabocha/.local/share && \
	mkdir /data && \
	chown -R kabocha:kabocha /data && \
	ln -s /data /kabocha/.local/share/kabocha && \
	rm -rf /usr/bin /usr/sbin

USER kabocha
EXPOSE 30333-30343 9933-9960 8080 300
VOLUME ["/data"]

CMD ["/usr/local/bin/parachain-collator"]