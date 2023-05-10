#!/usr/bin/env python3

"""
Simple script to upgrade the hardcoded chain-spec.json files.

This the main purpose is that to automate migration of values from the old files to the new files that can not
be inserted by the rust code, e.g., the `bootNodes`.

Usage: ./scripts/update_hardcoded_specs.py <--regenesis>

Optionally define if the `genesis` field of the chain-spec should also be migrated. This field should be set as follows:
*   True, if a completely new chain-spec shall be created. This will create a new genesis state, which is not compatible
    with chains running on the old chain-spec.
*   False, if we only want to change other fields that are relevant to the node (i.e., the client) only, but not
    the runtime. For instance if we update the substrate/polkadot.
"""

import argparse
import json
import os
import subprocess

SPECS = [
    "integritee-rococo",
    "integritee-westend",
    "integritee-kusama",
    "integritee-polkadot",
    "integritee-moonbase",
    "shell-kusama", # enable if you want to change the data for registration.
]
COLLATOR = "target/release/integritee-collator"
RES_DIR = "polkadot-parachains/chain-specs"


def main(regenesis: bool):
    for chain_spec in SPECS:

        ret = subprocess.call(
            f'scripts/dump_wasm_state_and_spec.sh {chain_spec}-fresh {COLLATOR} {RES_DIR}',
            stdout=subprocess.PIPE,
            shell=True
        )

        print(ret)

        orig_file = f'{RES_DIR}/{chain_spec}.json'
        new_file_base = f'{RES_DIR}/{chain_spec}-fresh'

        if not os.path.exists(orig_file):
            print(f"no previous spec found for {orig_file}, not migrating previous settings")
            os.popen(f"cp {new_file_base}-raw.json {orig_file}")
        else:
            with open(orig_file, 'r+') as spec_orig_file:
                orig_json = json.load(spec_orig_file)

                # migrate old values to new spec
                with open(f'{new_file_base}-raw.json', 'r+') as spec_new_file:
                    new_json = json.load(spec_new_file)

                    new_json["bootNodes"] = orig_json["bootNodes"]

                    if not regenesis:
                        new_json["genesis"] = orig_json["genesis"]

                    # go to beginning of the file to overwrite
                    spec_orig_file.seek(0)
                    json.dump(new_json, spec_orig_file, indent=2)
                    spec_orig_file.truncate()

            # remove side-products
            os.remove(f'{new_file_base}.json')
            os.remove(f'{new_file_base}-raw.json')
            os.remove(f'{new_file_base}-raw-unsorted.json')
            os.remove(f'{new_file_base}.state')
            os.remove(f'{new_file_base}.wasm')


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--regenesis', help='Overwrite genesis state in chain spec. Use this for resetting chains entirely', action='store_true')

    args = parser.parse_args()
    print(f'Updating chain specs. Preserving bootnodes. (re-genesis == {args.regenesis})')

    main(args.regenesis)
