FROM ubuntu:latest

COPY target/release/polkadot-collator /usr/local/bin/polkadot-collator

WORKDIR /polkadot

COPY run.sh run.sh
COPY rococo-local-cfde.json rococo-local-cfde.json

ENTRYPOINT ["bash", "run.sh"]