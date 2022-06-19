<!---
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
-->

# Hetu is a real-time OLAP database management system.

The goal is to achieve a system of high concurrency, real-time write and
analytical query optimization, with mixed workloads

- Simple:

  - Multi-Modal APIs for Rust/Python/Java/R/SQL
  - Cloud native serverless On S3 or local

- Feature-rich:

  - Extensive SQL Plus support
  - Transactions, persistence
  - External Direct Parquet & CSV & Json querying

- Fast
  - Vectorized engine in SIMD
  - Parallel query processing
  - Optimized for analytics
  - High-performance RowwiseRowBlock and RowwiseColumnarBlock format store: lstore and cstore

The foundational technologies in Hetu are:

- [Apache Arrow](https://arrow.apache.org/) memory model and compute kernels for efficient processing of data.
- [DataFusion Query Engine](https://github.com/apache/arrow-datafusion) for query execution
- [Apache Arrow Flight Protocol](https://arrow.apache.org/blog/2019/10/13/introducing-arrow-flight/) for efficient
  data transfer between processes.
- [Google Protocol Buffers](https://developers.google.com/protocol-buffers) for serializing query plans

Hetu can be deployed as a standalone cluster and also supports [Kubernetes](https://kubernetes.io/). In either
case, the scheduler can be configured to use [etcd](https://etcd.io/) as a backing store to (eventually) provide
redundancy in the case of a scheduler failing.

# Project Status and Roadmap

Hetu is currently a proof-of-concept based on datafusion and provides batch execution of SQL queries. Although it is already capable of
executing complex queries, it is not yet scalable or robust.

There is an excellent discussion in https://github.com/hetudb/hetu/issues/1 about the future of the project
and we encourage you to participate and add your feedback there if you are interested in using or contributing to
Hetu.

The current initiatives being considered are:

- Supports reliable raft-based meta storage
- Supports lightweight storage based on lsm-tree real-time writes and reads
- Continue to improve the current batch-based execution
- Add support for low-latency query execution based on a streaming model

# Getting Started

The easiest way to get started is to run one of the standalone or distributed [examples](./examples/README.md). After
that, refer to the [Getting Started Guide](client/rust/client/README.md).

## Architecture Overview

- Refer to the [developer documentation](docs/developer) for the [Architecture Overview](/docs/developer/architecture.md)

## Contribution Guide

Please see [Contribution Guide](CONTRIBUTING.md) for information about contributing to HetuDB.
