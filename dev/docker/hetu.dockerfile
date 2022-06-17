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
ARG RELEASE_FLAG=--release
ARG VERSION=0.1.0

FROM hetu-base:$VERSION AS base
WORKDIR /tmp/hetu
RUN apt-get -y install cmake
RUN cargo install cargo-chef --version 0.1.34

# this does not seem to work and we end up downloading and compiling everything twice

#FROM base as planner
#ADD Cargo.toml .
#COPY hetu ./hetu/
#COPY hetu-cli ./hetu-cli/
#COPY examples ./examples/
#COPY benchmarks ./benchmarks/
#RUN cargo chef prepare --recipe-path recipe.json
#
#FROM base as cacher
#COPY --from=planner /tmp/hetu/recipe.json recipe.json
#RUN cargo chef cook $RELEASE_FLAG --recipe-path recipe.json

FROM base as builder
RUN mkdir /tmp/hetu/hetu
RUN mkdir /tmp/hetu/hetu-cli
RUN mkdir /tmp/hetu/examples
RUN mkdir /tmp/hetu/benchmarks
ADD Cargo.toml .
COPY hetu ./hetu/
COPY hetu-cli ./hetu-cli/
COPY examples ./examples/
COPY benchmarks ./benchmarks/
#COPY --from=cacher /tmp/hetu/target target
ARG RELEASE_FLAG=--release

# force build.rs to run to generate configure_me code.
ENV FORCE_REBUILD='true'
RUN cargo build $RELEASE_FLAG

# put the executor on /executor (need to be copied from different places depending on FLAG)
ENV RELEASE_FLAG=${RELEASE_FLAG}
RUN if [ -z "$RELEASE_FLAG" ]; then mv /tmp/hetu/target/debug/hetu-query /query; else mv /tmp/hetu/target/release/hetu-query /query; fi

# put the scheduler on /scheduler (need to be copied from different places depending on FLAG)
ENV RELEASE_FLAG=${RELEASE_FLAG}
RUN if [ -z "$RELEASE_FLAG" ]; then mv /tmp/hetu/target/debug/hetu-cloud-service /hetu-cloud-service; else mv /tmp/hetu/target/release/hetu-cloud-service /hetu-cloud-service; fi

# put the tpch on /tpch (need to be copied from different places depending on FLAG)
ENV RELEASE_FLAG=${RELEASE_FLAG}
RUN if [ -z "$RELEASE_FLAG" ]; then mv /tmp/hetu/target/debug/tpch /hetu-tpch; else mv /tmp/hetu/target/release/tpch /hetu-tpch; fi

# Copy the binary into a new container for a smaller docker image
FROM hetu-base:$VERSION

COPY --from=builder /hetu-query /

COPY --from=builder /hetu-cloud-service /

COPY --from=builder /hetu-tpch /

ADD benchmarks/run.sh /
RUN mkdir /queries
COPY benchmarks/queries/ /queries/

ENV RUST_LOG=info
ENV RUST_BACKTRACE=full

CMD ["/hetu-query"]
