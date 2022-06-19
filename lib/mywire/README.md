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

# MySQL

Bindings for emulating a MySQL/MariaDB server.

When developing new databases or caching layers, it can be immensely useful to test your system
using existing applications. However, this often requires significant work modifying
applications to use your database over the existing ones. This crate solves that problem by
acting as a MySQL server, and delegating operations such as querying and query execution to
user-defined logic.

To start, implement `AsyncMysqlShim` for your backend, and create a `AsyncMysqlIntermediary` over an
instance of your backend and a connection stream. The appropriate methods will be called on
your backend whenever a client issues a `QUERY`, `PREPARE`, or `EXECUTE` command, and you will
have a chance to respond appropriately. For example, to write a shim that always responds to
all commands with a "no results" reply:

```rust
use std::io;

use mywire::*;
use tokio::net::TcpListener;

struct Backend;

#[async_trait::async_trait]
impl<W: io::Write + Send> AsyncMysqlShim<W> for Backend {
    type Error = io::Error;

    async fn on_prepare<'a>(
        &'a mut self,
        _: &'a str,
        info: StatementMetaWriter<'a, W>,
    ) -> io::Result<()> {
        info.reply(42, &[], &[])
    }

    async fn on_execute<'a>(
        &'a mut self,
        _: u32,
        _: opensrv_mysql::ParamParser,
        results: QueryResultWriter<'a, W>,
    ) -> io::Result<()> {
        results.completed(OkResponse::default())
    }

    async fn on_close(&mut self, _: u32) {}

    async fn on_query<'a>(
        &'a mut self,
        sql: &'a str,
        results: QueryResultWriter<'a, W>,
    ) -> io::Result<()> {
        println!("execute sql {:?}", sql);
        results.start(&[])?.finish()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:3306").await?;

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move { AsyncMysqlIntermediary::run_on(Backend, stream).await });
    }
}
```

This example can be exected with:
```
cargo run --example=serve_one
```

More examples can be found [here](examples).
