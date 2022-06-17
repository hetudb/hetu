#!/usr/bin/env python

#
# Copyright 2021 HetuDB.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License
#

# Script that updates verions for hetu crates, locally
#
# dependencies:
# pip install tomlkit

import re
import os
import argparse
from pathlib import Path
import tomlkit

crates = {
    # core
    'hetu-cloud-service': 'core/cloudsrv/Cargo.toml',
    'hetu-query': 'core/query/Cargo.toml',
    'hetu-metabase': 'core/metabase/Cargo.toml',
    'hetu-core': 'core/core/Cargo.toml',

    # common
    'hetu-error': 'common/error/Cargo.toml',

    # cli
    'hetu-client': 'client/rust/client/Cargo.toml',
    'hetu-cli': 'client/rust/cli/Cargo.toml',

    # benchmarks
    'hetu-benchmarks': 'benchmarks/Cargo.toml',

    # examples
    'hetu-examples': 'examples/Cargo.toml',

    # lib
    'hetu-mywire': 'lib/mywire/Cargo.toml',
    'hetu-cstore': 'lib/cstore/Cargo.toml',
    'hetu-lstore': 'lib/lstore/Cargo.toml',

    # extension
    'hetu-cdc': 'extension/cdc/Cargo.toml',
    'hetu-streaming': 'extension/streaming/Cargo.toml',

    # plugin
    'hetu-tpch': 'plugin/tpch/Cargo.toml',
    'hetu-tpcds': 'plugin/tpcds/Cargo.toml',

    # service
    'hetu-proxy': 'plugin/proxy/Cargo.toml',

    # testing
    'hetu-benchmark': 'testing/benchmark/Cargo.toml',

    # storage
    'hetu-datanode': 'storage/datanode',
}

def update_hetu_version(cargo_toml: str, new_version: str):
    print(f'updating {cargo_toml}')
    with open(cargo_toml) as f:
        data = f.read()

    doc = tomlkit.parse(data)
    doc.get('package')['version'] = new_version

    with open(cargo_toml, 'w') as f:
        f.write(tomlkit.dumps(doc))


def update_downstream_versions(cargo_toml: str, new_version: str):
    with open(cargo_toml) as f:
        data = f.read()

    doc = tomlkit.parse(data)

    for crate in crates.keys():
        df_dep = doc.get('dependencies', {}).get(crate)
        # skip crates that pin hetu using git hash
        if df_dep is not None and df_dep.get('version') is not None:
            print(f'updating {crate} dependency in {cargo_toml}')
            df_dep['version'] = new_version

        df_dep = doc.get('dev-dependencies', {}).get(crate)
        if df_dep is not None and df_dep.get('version') is not None:
            print(f'updating {crate} dev-dependency in {cargo_toml}')
            df_dep['version'] = new_version

    with open(cargo_toml, 'w') as f:
        f.write(tomlkit.dumps(doc))


def update_docs(path: str, new_version: str):
    print(f"updating docs in {path}")
    with open(path, 'r+') as fd:
        content = fd.read()
        fd.seek(0)
        content = re.sub(r'hetu = "(.+)"', f'hetu = "{new_version}"', content)
        fd.write(content)


def main():
    parser = argparse.ArgumentParser(
        description=(
            'Update hetu crate version and corresponding version pins '
            'in downstream crates.'
        ))
    parser.add_argument('new_version', type=str, help='new hetu version')
    args = parser.parse_args()

    new_version = args.new_version
    repo_root = Path(__file__).parent.parent.absolute()

    print(f'Updating hetu crate versions in {repo_root} to {new_version}')
    for cargo_toml in crates.values():
        update_hetu_version(cargo_toml, new_version)

    print(f'Updating hetu dependency versions in {repo_root} to {new_version}')
    for cargo_toml in crates.values():
        update_downstream_versions(cargo_toml, new_version)

    update_docs("README.md", new_version)


if __name__ == "__main__":
    main()
