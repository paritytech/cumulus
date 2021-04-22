FROM ubuntu:latest

COPY target/release/rococo-collator /usr/local/bin/rococo-collator

WORKDIR /polkadot

COPY run.sh run.sh

ENTRYPOINT ["bash", "run.sh"]