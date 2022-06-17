#!/bin/bash

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

if [ -z "${DOCKER_REPO}" ]; then
  echo "DOCKER_REPO env var must be set"
  exit -1
fi
cargo install cross
# arm linux
cross build --release --target aarch64-unknown-linux-gnu
# mac osx
# cross build --release --target aarch64-apple-darwin
rm -rf temp-hetu-docker
mkdir temp-hetu-docker
cp target/aarch64-unknown-linux-gnu/release/hetu-query temp-hetu-docker
cp target/aarch64-unknown-linux-gnu/release/hetu-cloud-service temp-hetu-docker
cp target/aarch64-unknown-linux-gnu/release/tpch temp-hetu-docker
mkdir temp-hetu-docker/queries/
cp benchmarks/queries/*.sql temp-hetu-docker/queries/
docker buildx build --push -t $DOCKER_REPO/hetu-arm64 --platform=linux/arm64 -f dev/docker/hetu-arm64.Dockerfile temp-hetu-docker
rm -rf temp-hetu-docker