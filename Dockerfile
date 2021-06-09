FROM ubuntu:latest

COPY target/release/polkadot-collator /usr/local/bin/polkadot-collator
COPY target/release/polkadot /usr/local/bin/polkadot

WORKDIR /polkadot

COPY rococo-single-custom.json rococo-single-custom.json
COPY run.sh run.sh
ENTRYPOINT ["bash", "run.sh"]