FROM phusion/baseimage:focal-1.0.0
LABEL description="This is the 2nd stage: a very small image where we copy the Substrate binary."

RUN apt-get update && \
apt-get install -y jq

RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	useradd -m -u 1000 -U -s /bin/sh -d /encointer encointer && \
	mkdir -p /encointer/.local/share/encointer-collator && \
	chown -R encointer:encointer /encointer/.local && \
	ln -s /encointer/.local/share/encointer-collator /data

COPY encointer-collator /usr/local/bin
COPY ./scripts/healthcheck9933.sh /usr/local/bin

RUN chmod +x /usr/local/bin/encointer-collator
RUN chmod +x /usr/local/bin/healthcheck9933.sh

# checks
RUN ldd /usr/local/bin/encointer-collator && \
	/usr/local/bin/encointer-collator --version

# Shrinking
#RUN rm -rf /usr/lib/python* && \
#	rm -rf /usr/bin /usr/sbin /usr/share/man

#USER encointer
EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/encointer-collator"]
