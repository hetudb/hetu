// Copyright 2021 HetuDB.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! After running this, you should be able to run:
//!
//! ```console
//! $ echo "SELECT * FROM foo" | mysql -h 127.0.0.1 -u root --table
//! $
//! ```

use futures_util::future::{AbortHandle, AbortRegistration, Abortable};
use futures_util::StreamExt;
use log::{error, info};
use std::future::Future;
use std::io;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::base::{Runtime, Thread, TrySpawn};
use crate::config::Config;
use crate::session::HetuContext;
use crate::utils::DFQueryResultWriter;
use hetu_error::{HetuError, Result};
use hetu_mywire::*;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::TcpListenerStream;

struct Backend<W: std::io::Write> {
    ctx: Arc<HetuContext>,
    generic_hold: PhantomData<W>,
}

#[async_trait::async_trait]
impl<W: io::Write + Send + Sync> AsyncMysqlShim<W> for Backend<W> {
    type Error = HetuError;

    async fn on_init<'a>(
        &'a mut self,
        schema: &'a str,
        writer: InitWriter<'a, W>,
    ) -> Result<()> {
        println!("use {}", schema);
        writer.ok()?;
        Ok(())
    }

    async fn on_prepare<'a>(
        &'a mut self,
        _: &'a str,
        info: StatementMetaWriter<'a, W>,
    ) -> Result<()> {
        info.reply(42, &[], &[])?;
        Ok(())
    }

    async fn on_execute<'a>(
        &'a mut self,
        _id: u32,
        _param: ParamParser<'a>,
        writer: QueryResultWriter<'a, W>,
    ) -> Result<()> {
        writer.error(
            ErrorKind::ER_UNKNOWN_ERROR,
            "Execute is not support in HetuDB.".as_bytes(),
        )?;
        Ok(())
    }

    async fn on_close(&mut self, _: u32) {}

    async fn on_query<'a>(
        &'a mut self,
        sql: &'a str,
        results: QueryResultWriter<'a, W>,
    ) -> Result<()> {
        println!("execute sql {:?}", sql);
        // TODO: sql command dispatch
        let mut writer = DFQueryResultWriter::create(results);

        // create a plan to run a SQL query
        let df_result = self.ctx.sql(sql).await;
        match df_result {
            Ok(df) => {
                let records_result = df.collect().await;
                match records_result {
                    Ok(records) => {
                        let result = Ok((records, String::from("ExtraInfo")));
                        let mut write_result = writer.write(result);
                        if let Err(hetu_error) = write_result {
                            write_result = Err(hetu_error);
                        }
                        write_result
                    }
                    Err(err) => {
                        let result = Err(HetuError::from(err));
                        println!("DataFusionError: {}", result.as_ref().err().unwrap());
                        let mut write_result = writer.write(result);
                        if let Err(hetu_error) = write_result {
                            write_result = Err(hetu_error);
                        }
                        write_result
                    }
                }
            }
            Err(err) => {
                let result = Err(HetuError::from(err));
                println!("DataFusionError: {}", result.as_ref().err().unwrap());
                let mut write_result = writer.write(result);
                if let Err(hetu_error) = write_result {
                    write_result = Err(hetu_error);
                }
                write_result
            }
        }
    }

    /// authenticate method for the specified plugin
    async fn authenticate(
        &self,
        _auth_plugin: &str,
        username: &[u8],
        _salt: &[u8],
        _auth_data: &[u8],
    ) -> bool {
        username == "root".as_bytes()
    }

    fn version(&self) -> &str {
        // 5.1.10 because that's what Ruby's ActiveRecord requires
        "5.7.25-HetuDB-v0.1.0-alpha HetuDB Server (Apache License 2.0) \nCommunity Edition, MySQL 5.7 compatiible"
    }

    fn connect_id(&self) -> u32 {
        u32::from_le_bytes([0x08, 0x00, 0x00, 0x00])
    }

    fn default_auth_plugin(&self) -> &str {
        "mysql_native_password"
    }

    fn auth_plugin_for_username(&self, _user: &[u8]) -> &str {
        "mysql_native_password"
    }

    fn salt(&self) -> [u8; 20] {
        let bs = ";X,po_k}>o6^Wz!/kM}N".as_bytes();
        let mut scramble: [u8; 20] = [0; 20];
        for i in 0..20 {
            scramble[i] = bs[i];
            if scramble[i] == b'\0' || scramble[i] == b'$' {
                scramble[i] = scramble[i] + 1;
            }
        }
        scramble
    }
}

pub type ListeningStream = Abortable<TcpListenerStream>;

pub struct MySQLHandler {
    context: Arc<HetuContext>,
    hostname: String,
    port: i32,
    abort_handle: AbortHandle,
    abort_registration: Option<AbortRegistration>,
    join_handle: Option<JoinHandle<()>>,
}

impl MySQLHandler {
    pub fn create(conf: Config, context: Arc<HetuContext>) -> MySQLHandler {
        let (abort_handle, registration) = AbortHandle::new_pair();
        MySQLHandler {
            context,
            hostname: conf.mysql_handler_host,
            port: conf.mysql_handler_port,
            abort_handle,
            abort_registration: Some(registration),
            join_handle: None,
        }
    }

    pub async fn start(&mut self) -> Result<SocketAddr> {
        match self.abort_registration.take() {
            None => Err(HetuError::Internal(String::from(
                "MySQLHandler already running.",
            ))),
            Some(registration) => {
                let rejected_rt = Arc::new(Runtime::with_worker_threads(
                    1,
                    Some("mysql-handler".to_string()),
                )?);
                let listening = format!("{}:{}", self.hostname, self.port);
                let (stream, listener) = Self::listener_tcp(listening).await?;
                let stream = Abortable::new(stream, registration);
                self.join_handle =
                    Some(tokio::spawn(self.listen_loop(stream, rejected_rt)));
                Ok(listener)
            }
        }
    }

    pub async fn shutdown(&mut self, graceful: bool) {
        if !graceful {
            return;
        }

        self.abort_handle.abort();

        if let Some(join_handle) = self.join_handle.take() {
            if let Err(error) = join_handle.await {
                error!("An unexpected error occurred during the shutdown  MySQLHandler, cause {}", error);
            }
        }
    }

    async fn listener_tcp(listening: String) -> Result<(TcpListenerStream, SocketAddr)> {
        let listener = tokio::net::TcpListener::bind(&listening)
            .await
            .map_err(|e| HetuError::Internal(format!("{} {}", &listening, e)))?;
        let listener_addr = listener.local_addr()?;
        Ok((TcpListenerStream::new(listener), listener_addr))
    }

    fn listen_loop(
        &self,
        stream: ListeningStream,
        rt: Arc<Runtime>,
    ) -> impl Future<Output = ()> {
        let context = self.context.clone();
        stream.for_each(move |accept_socket| {
            let executor = rt.clone();
            let ctx = context.clone();
            async move {
                match accept_socket {
                    Err(error) => error!("Broken session connection: {}", error),
                    Ok(socket) => MySQLHandler::accept_socket(ctx, executor, socket),
                };
            }
        })
    }

    pub fn accept_socket(
        context: Arc<HetuContext>,
        executor: Arc<Runtime>,
        socket: TcpStream,
    ) {
        executor.spawn(async move {
            if let Err(error) = Self::run_on_stream(context, socket) {
                error!("Unexpected error occurred during query: {:?}", error);
            };
        });
    }

    fn run_on_stream(context: Arc<HetuContext>, stream: TcpStream) -> Result<()> {
        let blocking_stream = Self::convert_stream(stream)?;

        let non_blocking_stream = TcpStream::from_std(blocking_stream)?;
        let query_executor =
            Runtime::with_worker_threads(1, Some("mysql-query-executor".to_string()))?;
        Thread::spawn(move || {
            let join_handle = query_executor.spawn(async move {
                let interactive_worker = Backend {
                    // create local execution context
                    // ctx: SessionContext::with_config(
                    //     SessionConfig::new().with_information_schema(true),
                    // ),
                    ctx: context,
                    generic_hold: Default::default(),
                };

                let opts = IntermediaryOptions {
                    process_use_statement_on_query: true,
                };
                AsyncMysqlIntermediary::run_with_options(
                    interactive_worker,
                    non_blocking_stream,
                    &opts,
                )
                .await
            });
            let _ = futures::executor::block_on(join_handle);
        });
        info!("Start MySQL Handler successful.");
        Ok(())
    }

    // TODO: move to ToBlockingStream trait
    fn convert_stream(stream: TcpStream) -> Result<std::net::TcpStream> {
        let stream = stream.into_std()?;
        // .map_err(Err(HetuError::Internal(String::from("Cannot to convert Tokio TcpStream to Std TcpStream"))))?;
        stream.set_nonblocking(false)?;
        // .map_err(Err(HetuError::Internal(String::from("Cannot to convert Tokio TcpStream to Std TcpStream"))))?;
        Ok(stream)
    }
}
