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

//! Hetu Query Rust executor binary.

use chrono::{DateTime, Duration, Utc};
use hetu_query::protocol::MySQLHandler;
use hetu_query::session::HetuContext;
use std::sync::Arc;
use std::time::Duration as Core_Duration;

use anyhow::{Context, Result};
use arrow_flight::flight_service_server::FlightServiceServer;
use hetu_query::{execution_loop, executor_server, HETU_QUERY_SERVICE_VERSION};
use log::{error, info};
use tempfile::TempDir;
use tokio::fs::ReadDir;
use tokio::{fs, time};
use tonic::transport::Server;
use uuid::Uuid;

use datafusion::execution::runtime_env::{RuntimeConfig, RuntimeEnv};
use datafusion_proto::protobuf::LogicalPlanNode;
use hetu_core::config::{BallistaConfig, TaskSchedulingPolicy};
use hetu_core::error::BallistaError;
use hetu_core::serde::protobuf::{
    executor_registration, scheduler_grpc_client::SchedulerGrpcClient,
    ExecutorRegistration, PhysicalPlanNode,
};
use hetu_core::serde::scheduler::ExecutorSpecification;
use hetu_core::serde::BallistaCodec;
use hetu_core::BALLISTA_VERSION;
use hetu_query::config::Config;
use hetu_query::executor::Executor;
use hetu_query::flight_service::BallistaFlightService;
use hetu_query::metrics::LoggingMetricsCollector;

#[cfg(feature = "snmalloc")]
#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let conf: Config = Config::load()?;
    // MySQL handler.
    {
        println!("Config: {:?}", conf);
        let config = BallistaConfig::builder()
            .set("ballista.shuffle.partitions", "2")
            .set("ballista.with_information_schema", "true")
            .build()?;

        let context = Arc::new(
            HetuContext::remote(
                conf.scheduler_host.as_str(),
                conf.scheduler_port,
                &config,
            )
            .await?,
        );

        let mut mysql_handler = MySQLHandler::create(conf.clone(), context.clone());
        let listening = mysql_handler.start().await?;
        info!("Hetu Query Service v{} Rust MySQL Handler listening on {}, Usage: mysql -h {} -P {} -u root",
                     HETU_QUERY_SERVICE_VERSION,
                     listening,
                     conf.mysql_handler_host,
                     conf.mysql_handler_port);
    }

    let external_host = Some(conf.external_host);
    let bind_host = conf.bind_host;
    let port = conf.bind_port;
    let grpc_port = conf.bind_grpc_port;

    let addr = format!("{}:{}", bind_host, port);
    let addr = addr
        .parse()
        .with_context(|| format!("Could not parse address: {}", addr))?;

    let scheduler_host = conf.scheduler_host;
    let scheduler_port = conf.scheduler_port;
    let scheduler_url = format!("http://{}:{}", scheduler_host, scheduler_port);

    let work_dir = if conf.work_dir.is_empty() {
        TempDir::new()?
            .into_path()
            .into_os_string()
            .into_string()
            .unwrap()
    } else {
        conf.work_dir
    };
    info!("Running with config:");
    info!("work_dir: {}", work_dir);
    info!("concurrent_tasks: {}", conf.concurrent_tasks);

    let executor_meta = ExecutorRegistration {
        id: Uuid::new_v4().to_string(), // assign this executor a unique ID
        optional_host: external_host
            .clone()
            .map(executor_registration::OptionalHost::Host),
        port: port as u32,
        grpc_port: grpc_port as u32,
        specification: Some(
            ExecutorSpecification {
                task_slots: conf.concurrent_tasks as u32,
            }
            .into(),
        ),
    };

    let config = RuntimeConfig::new().with_temp_file_path(work_dir.clone());
    let runtime = Arc::new(RuntimeEnv::new(config).map_err(|_| {
        BallistaError::Internal("Failed to init Executor RuntimeEnv".to_owned())
    })?);

    let metrics_collector = Arc::new(LoggingMetricsCollector::default());

    let executor = Arc::new(Executor::new(
        executor_meta,
        &work_dir,
        runtime,
        metrics_collector,
    ));

    let scheduler = SchedulerGrpcClient::connect(scheduler_url)
        .await
        .context("Could not connect to scheduler")?;

    let default_codec: BallistaCodec<LogicalPlanNode, PhysicalPlanNode> =
        BallistaCodec::default();

    let scheduler_policy: TaskSchedulingPolicy =
        match conf.task_scheduling_policy.as_str() {
            "PushStaged" => hetu_core::config::TaskSchedulingPolicy::PushStaged,
            _ => hetu_core::config::TaskSchedulingPolicy::PullStaged,
        };

    let cleanup_ttl = conf.executor_cleanup_ttl;

    if conf.executor_cleanup_enable {
        let mut interval_time =
            time::interval(Core_Duration::from_secs(conf.executor_cleanup_interval));
        tokio::spawn(async move {
            loop {
                interval_time.tick().await;
                if let Err(e) =
                    clean_shuffle_data_loop(&work_dir, cleanup_ttl as i64).await
                {
                    error!("Ballista executor fail to clean_shuffle_data {:?}", e)
                }
            }
        });
    }

    match scheduler_policy {
        TaskSchedulingPolicy::PushStaged => {
            tokio::spawn(executor_server::startup(
                scheduler,
                executor.clone(),
                default_codec,
            ));
        }
        _ => {
            tokio::spawn(execution_loop::poll_loop(
                scheduler,
                executor.clone(),
                default_codec,
            ));
        }
    }

    // Arrow flight service
    {
        let service = BallistaFlightService::new(executor.clone());
        let server = FlightServiceServer::new(service);
        info!(
            "Ballista v{} Rust Executor listening on {:?}",
            BALLISTA_VERSION, addr
        );
        let server_future =
            tokio::spawn(Server::builder().add_service(server).serve(addr));
        server_future
            .await
            .context("Tokio error")?
            .context("Could not start executor server")?;
    }

    Ok(())
}

/// This function will scheduled periodically for cleanup executor.
/// Will only clean the dir under work_dir not include file
async fn clean_shuffle_data_loop(work_dir: &str, seconds: i64) -> Result<()> {
    let mut dir = fs::read_dir(work_dir).await?;
    let mut to_deleted = Vec::new();
    let mut need_delete_dir;
    while let Some(child) = dir.next_entry().await? {
        if let Ok(metadata) = child.metadata().await {
            // only delete the job dir
            if metadata.is_dir() {
                let dir = fs::read_dir(child.path()).await?;
                match check_modified_time_in_dirs(vec![dir], seconds).await {
                    Ok(x) => match x {
                        true => {
                            need_delete_dir = child.path().into_os_string();
                            to_deleted.push(need_delete_dir)
                        }
                        false => {}
                    },
                    Err(e) => {
                        error!("Fail in clean_shuffle_data_loop {:?}", e)
                    }
                }
            }
        } else {
            error!("Can not get metadata from file: {:?}", child)
        }
    }
    info!(
        "The work_dir {:?} that have not been modified for {:?} seconds will be deleted",
        &to_deleted, seconds
    );
    for del in to_deleted {
        fs::remove_dir_all(del).await?;
    }
    Ok(())
}

/// Determines if a directory all files are older than cutoff seconds.
async fn check_modified_time_in_dirs(
    mut vec: Vec<ReadDir>,
    ttl_seconds: i64,
) -> Result<bool> {
    let cutoff = Utc::now() - Duration::seconds(ttl_seconds);

    while !vec.is_empty() {
        let mut dir = vec.pop().unwrap();
        while let Some(child) = dir.next_entry().await? {
            let meta = child.metadata().await?;
            if meta.is_dir() {
                let dir = fs::read_dir(child.path()).await?;
                // check in next loop
                vec.push(dir);
            } else {
                let modified_time: DateTime<Utc> =
                    meta.modified().map(chrono::DateTime::from)?;
                if modified_time > cutoff {
                    // if one file has been modified in ttl we won't delete the whole dir
                    return Ok(false);
                }
            }
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use crate::clean_shuffle_data_loop;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::time::Duration;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_executor_clean_up() {
        let work_dir = TempDir::new().unwrap().into_path();
        let job_dir = work_dir.as_path().join("job_id");
        let file_path = job_dir.as_path().join("tmp.csv");
        let data = "Jorge,2018-12-13T12:12:10.011Z\n\
                    Andrew,2018-11-13T17:11:10.011Z";
        fs::create_dir(job_dir).unwrap();
        File::create(&file_path)
            .expect("creating temp file")
            .write_all(data.as_bytes())
            .expect("writing data");

        let work_dir_clone = work_dir.clone();

        let count1 = fs::read_dir(work_dir.clone()).unwrap().count();
        assert_eq!(count1, 1);
        let mut handles = vec![];
        handles.push(tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(2)).await;
            clean_shuffle_data_loop(work_dir_clone.to_str().unwrap(), 1)
                .await
                .unwrap();
        }));
        futures::future::join_all(handles).await;
        let count2 = fs::read_dir(work_dir.clone()).unwrap().count();
        assert_eq!(count2, 0);
    }
}
