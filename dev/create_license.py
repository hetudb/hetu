#!/usr/bin/python
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

import json
import subprocess

subprocess.check_output(["cargo", "install", "cargo-license"])
data = subprocess.check_output(
    [
        "cargo",
        "license",
        "--avoid-build-deps",
        "--avoid-dev-deps",
        "--do-not-bundle",
        "--json",
    ]
)
data = json.loads(data)

result = """
  Copyright 2021 HetuDB.
  
  Licensed under the Apache License, Version 2.0 (the "License");
  you may not use this file except in compliance with the License.
  You may obtain a copy of the License at
  
      http://www.apache.org/licenses/LICENSE-2.0
  
  Unless required by applicable law or agreed to in writing, software
  distributed under the License is distributed on an "AS IS" BASIS,
  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
  See the License for the specific language governing permissions and
  limitations under the License
"""
result += "\n------------------\n\n"
result += "This software is built and contains the following software:\n\n"
result += "(automatically generated via [cargo-license](https://crates.io/crates/cargo-license))\n\n"
for item in data:
    license = item["license"]
    name = item["name"]
    version = item["version"]
    repository = item["repository"]
    result += "------------------\n\n"
    result += f"### {name} {version}\n* source: [{repository}]({repository})\n* license: {license}\n\n"

with open("LICENSE.txt", "w") as f:
    f.write(result)
