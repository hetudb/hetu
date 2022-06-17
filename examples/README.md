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

# Hetu Examples

This directory contains examples for executing distributed queries with Hetu.

# Standalone Examples

The standalone example is the easiest to get started with. Hetu supports a standalone mode where a scheduler
and executor are started in-process.

```bash
cargo run --example standalone_sql --features="hetu-client/standalone"
```

### Source code for standalone SQL example

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let config = BallistaConfig::builder()
        .set("ballista.shuffle.partitions", "1")
        .build()?;

    let ctx = HetuContext::standalone(&config, 2).await?;

    ctx.register_csv(
        "test",
        "testdata/aggregate_test_100.csv",
        CsvReadOptions::new(),
    )
    .await?;

    let df = ctx.sql("select count(1) from test").await?;

    df.show().await?;
    Ok(())
}

```

# Distributed Examples

For background information on the Hetu architecture, refer to
the [Hetu Query README](../client/rust/client/README.md).

## Start a standalone cluster

From the root of the project, build release binaries.

```bash
cargo build --release
```

Start a Hetu scheduler process in a new terminal session.

```bash
RUST_LOG=info ./target/release/hetu-cloud-service
```

Start one or more Hetu executor processes in new terminal sessions. When starting more than one
executor, a unique port number must be specified for each executor.

```bash
RUST_LOG=info ./target/release/hetu-executor -c 2 -p 50051
RUST_LOG=info ./target/release/hetu-executor -c 2 -p 50052
```

## Running the examples

The examples can be run using the `cargo run --bin` syntax.

## Distributed SQL Example

```bash
cargo run --release --bin sql
```

### Source code for distributed SQL example

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let config = BallistaConfig::builder()
        .set("ballista.shuffle.partitions", "4")
        .build()?;
    let ctx = HetuContext::remote("localhost", 50050, &config).await?;

    let filename = "testdata/alltypes_plain.parquet";

    // define the query using the DataFrame trait
    let df = ctx
        .read_parquet(filename, ParquetReadOptions::default())
        .await?
        .select_columns(&["id", "bool_col", "timestamp_col"])?
        .filter(col("id").gt(lit(1)))?;

    // print the results
    df.show().await?;

    Ok(())
}
```

## Distributed DataFrame Example

```bash
cargo run --release --bin dataframe
```

### Source code for distributed DataFrame example

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let config = BallistaConfig::builder()
        .set("ballista.shuffle.partitions", "4")
        .build()?;
    let ctx = HetuContext::remote("localhost", 50050, &config).await?;

    let filename = "testdata/alltypes_plain.parquet";

    // define the query using the DataFrame trait
    let df = ctx
        .read_parquet(filename, ParquetReadOptions::default())
        .await?
        .select_columns(&["id", "bool_col", "timestamp_col"])?
        .filter(col("id").gt(lit(1)))?;

    // print the results
    df.show().await?;

    Ok(())
}
```
