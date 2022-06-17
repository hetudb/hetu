<!---
  Licensed to the Apache Software Foundation (ASF) under one
  or more contributor license agreements.  See the NOTICE file
  distributed with this work for additional information
  regarding copyright ownership.  The ASF licenses this file
  to you under the Apache License, Version 2.0 (the
  "License"); you may not use this file except in compliance
  with the License.  You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

  Unless required by applicable law or agreed to in writing,
  software distributed under the License is distributed on an
  "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
  KIND, either express or implied.  See the License for the
  specific language governing permissions and limitations
  under the License.
-->

# Hetu Command-line Interface

[Hetu](df) is a distributed query execution framework, written in Rust, that uses Apache Arrow as its in-memory format.

The Hetu CLI allows SQL queries to be executed by an in-process DataFusion context, or by a distributed
Hetu context.

```ignore
USAGE:
    hetu-cli [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -q, --quiet      Reduce printing other than the results and work quietly
    -V, --version    Prints version information

OPTIONS:
    -c, --batch-size <batch-size>    The batch size of each query, or use DataFusion default
    -p, --data-path <data-path>      Path to your data, default to current directory
    -f, --file <file>...             Execute commands from file(s), then exit
        --format <format>            Output format [default: table]  [possible values: csv, tsv, table, json, ndjson]
        --host <host>                Hetu cloud service host
        --port <port>                Hetu cloud service port
```

## Example

Create a CSV file to query.

```bash,ignore
$ echo "1,2" > data.csv
```

```sql,ignore
$ hetu-cli

Hetu CLI v0.1.0

> CREATE EXTERNAL TABLE foo (a INT, b INT) STORED AS CSV LOCATION 'data.csv';
0 rows in set. Query took 0.001 seconds.

> SELECT * FROM foo;
+---+---+
| a | b |
+---+---+
| 1 | 2 |
+---+---+
1 row in set. Query took 0.017 seconds.
```

## Hetu-Cli

If you want to execute the SQL in hetu by `hetu-cli`, you must build/compile `hetu-cli` first.

```bash
cd hetu/client/cli
cargo build
```

The Hetu CLI can connect to a Hetu cloud service for query execution.

```bash
hetu-cli --host localhost --port 50050
```

[df]: https://www.hebudb.com
