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

//! Hetu scheduler handler.

use crate::scheduler_server::externalscaler::external_scaler_server::ExternalScalerServer;
use anyhow::{Context, Result};
use futures::future::{self, Either, TryFutureExt};
use hyper::{server::conn::AddrStream, service::make_service_fn, Server};
use std::convert::Infallible;
use std::{net::SocketAddr, sync::Arc};
use tonic::transport::server::Connected;
use tonic::transport::Server as TonicServer;
use tower::Service;

use crate::api::{get_routes, EitherBody, Error};
#[cfg(feature = "etcd")]
use crate::state::backend::etcd::EtcdClient;
#[cfg(feature = "sled")]
use crate::state::backend::standalone::StandaloneClient;
use datafusion_proto::protobuf::LogicalPlanNode;
use hetu_core::{
    print_version,
    serde::protobuf::{scheduler_grpc_server::SchedulerGrpcServer, PhysicalPlanNode},
};

use crate::scheduler_server::SchedulerServer;
use crate::state::backend::{StateBackend, StateBackendClient};

use hetu_core::config::TaskSchedulingPolicy;
use hetu_core::serde::BallistaCodec;
use log::info;

use datafusion::execution::context::default_session_builder;

use crate::config::Config;
use crate::CLOUD_SERVICE_VERSION;

pub struct SchedulerHandler {
    conf: Config,
}

impl SchedulerHandler {
    pub fn new(conf: Config) -> SchedulerHandler {
        SchedulerHandler { conf }
    }

    pub async fn start(&mut self) -> Result<SocketAddr> {
        let conf = self.conf.clone();
        // start scheduler
        env_logger::init();

        if false {
            print_version();
            std::process::exit(0);
        }

        let namespace = conf.namespace;
        let bind_host = conf.bind_host;
        let port = conf.bind_port;

        let addr = format!("{}:{}", bind_host, port);
        let addr = addr.parse()?;

        let config_backend: StateBackend = match conf.config_backend.as_str() {
            "Etcd" => StateBackend::Etcd,
            _ => StateBackend::Standalone,
        };

        let client: Arc<dyn StateBackendClient> = match config_backend {
            #[cfg(not(any(feature = "sled", feature = "etcd")))]
            _ => std::compile_error!(
            "To build the scheduler enable at least one config backend feature (`etcd` or `sled`)"
        ),
            #[cfg(feature = "etcd")]
            StateBackend::Etcd => {
                let etcd = etcd_client::Client::connect(&[conf.etcd_urls], None)
                    .await
                    .context("Could not connect to etcd")?;
                Arc::new(EtcdClient::new(etcd))
            }
            #[cfg(not(feature = "etcd"))]
            StateBackend::Etcd => {
                unimplemented!(
                    "build the scheduler with the `etcd` feature to use the etcd config backend"
                )
            }
            #[cfg(feature = "sled")]
            StateBackend::Standalone => {
                // TODO: Use a real file and make path is configurable
                Arc::new(
                    StandaloneClient::try_new_temporary()
                        .context("Could not create standalone config backend")?,
                )
            }
            #[cfg(not(feature = "sled"))]
            StateBackend::Standalone => {
                unimplemented!(
                    "build the scheduler with the `sled` feature to use the standalone config backend"
                )
            }
        };

        let policy: TaskSchedulingPolicy = match conf.scheduler_policy.as_str() {
            "PushStaged" => hetu_core::config::TaskSchedulingPolicy::PushStaged,
            _ => hetu_core::config::TaskSchedulingPolicy::PullStaged,
        };

        start_server(client, namespace, addr, policy).await?;
        Ok(addr)
    }
}

async fn start_server(
    config_backend: Arc<dyn StateBackendClient>,
    namespace: String,
    addr: SocketAddr,
    policy: TaskSchedulingPolicy,
) -> Result<()> {
    info!(
        "Hetu Cloud Service v{} Scheduler listening on {:?}",
        CLOUD_SERVICE_VERSION, addr
    );
    // Should only call SchedulerServer::new() once in the process
    info!(
        "Starting Scheduler grpc server with task scheduling policy of {:?}",
        policy
    );
    let mut scheduler_server: SchedulerServer<LogicalPlanNode, PhysicalPlanNode> =
        match policy {
            TaskSchedulingPolicy::PushStaged => SchedulerServer::new_with_policy(
                config_backend.clone(),
                namespace.clone(),
                policy,
                BallistaCodec::default(),
                default_session_builder,
            ),
            _ => SchedulerServer::new(
                config_backend.clone(),
                namespace.clone(),
                BallistaCodec::default(),
            ),
        };

    scheduler_server.init().await?;

    Server::bind(&addr)
        .serve(make_service_fn(move |request: &AddrStream| {
            let scheduler_grpc_server =
                SchedulerGrpcServer::new(scheduler_server.clone());

            let keda_scaler = ExternalScalerServer::new(scheduler_server.clone());

            let mut tonic = TonicServer::builder()
                .add_service(scheduler_grpc_server)
                .add_service(keda_scaler)
                .into_service();
            let mut warp = warp::service(get_routes(scheduler_server.clone()));

            let connect_info = request.connect_info();
            future::ok::<_, Infallible>(tower::service_fn(
                move |req: hyper::Request<hyper::Body>| {
                    // Set the connect info from hyper to tonic
                    let (mut parts, body) = req.into_parts();
                    parts.extensions.insert(connect_info.clone());
                    let req = http::Request::from_parts(parts, body);

                    let header = req.headers().get(hyper::header::ACCEPT);
                    if header.is_some() && header.unwrap().eq("application/json") {
                        return Either::Left(
                            warp.call(req)
                                .map_ok(|res| res.map(EitherBody::Left))
                                .map_err(Error::from),
                        );
                    }
                    Either::Right(
                        tonic
                            .call(req)
                            .map_ok(|res| res.map(EitherBody::Right))
                            .map_err(Error::from),
                    )
                },
            ))
        }))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Could not start grpc server")
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
