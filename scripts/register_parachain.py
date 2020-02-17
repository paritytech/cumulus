#!/usr/bin/env python3

"""
Register a testing parachain collator on a local test network.

This is _not_ a general-purpose script; it is tightly coupled to
../docker/docker-compose.yml and related scripts and files.
"""

from pathlib import Path
from time import sleep

from substrateinterface import SubstrateInterface


def wait_for_file(path, tries=10, delay=1):
    """
    Wait for a file to exist in the local filesystem.

    Strictly speaking, there's a bit of a race condition here: it is not
    impossible that the other container will still be writing this file when we
    query it. Unfortunately, there's no great way to ensure that the file is
    completely written just by looking at the filesystem, and in docker's
    world, we can't just iterate /proc/*/fd/* to see if there's an open
    descriptor.

    In practice, this is unlikely to pose a problem.
    """
    p = Path(path)
    for _ in range(tries):
        if p.exists():
            if not p.is_file():
                raise Exception(f"{path} was not a normal file")
            if p.stat().st_size > 0:
                return
        sleep(delay)
    raise Exception(f"{path} was not ready after {delay * tries} seconds")


def register_parachain():
    wait_for_file("/runtime/cumulus_test_parachain_runtime.compact.wasm")
    wait_for_file("/genesis/genesis-state")

    si = SubstrateInterface(url="http://172.28.1.1:9933")
    print(si.get_chain_head())


if __name__ == "__main__":
    register_parachain()
