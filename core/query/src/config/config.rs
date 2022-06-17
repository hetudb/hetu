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

use std::env;

use clap::Parser;
use serde::Deserialize;
use serde::Serialize;
use serfig::collectors::{from_env, from_file, from_self};
use serfig::parsers::Toml;

use hetu_error::{HetuError, Result};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, Parser)]
#[clap(author, version, about, long_about = None)]
#[serde(default)]
pub struct Config {
    /// The hetu query config by a toml file
    #[clap(long, short = 'c', default_value_t)]
    pub config_file: String,

    /// The hetu cloud service host
    #[clap(long, default_value = "localhost")]
    pub scheduler_host: String,

    /// The hetu cloud service port
    #[clap(long, default_value = "50050")]
    pub scheduler_port: u16,

    /// The hetu query local IP address to bind to.
    #[clap(long, default_value = "0.0.0.0")]
    pub bind_host: String,

    /// The hetu query bind port
    #[clap(long, default_value = "50051")]
    pub bind_port: u16,

    /// Host name or IP address to register with scheduler so that other executors can connect to this executor. If none is provided, the scheduler will use the connecting IP address to communicate with the executor.
    #[clap(long, default_value = "localhost")]
    pub external_host: String,

    /// The hetu query bind grpc service port
    #[clap(long, default_value = "50052")]
    pub bind_grpc_port: u16,

    /// The hetu query directory for temporary IPC files
    #[clap(long, default_value = "")]
    pub work_dir: String,

    /// The hetu query max concurrent tasks.
    #[clap(long, default_value = "4")]
    pub concurrent_tasks: usize,

    /// The task scheduing policy for the scheduler, see TaskSchedulingPolicy::variants() for options. Default: PullStaged
    #[clap(long, default_value = "PullStaged")]
    pub task_scheduling_policy: String,

    /// Enable periodic cleanup of work_dir directories.
    #[clap(long, parse(try_from_str = true_or_false), default_value_t)]
    pub executor_cleanup_enable: bool,

    /// Controls the interval in seconds , which the worker cleans up old job dirs on the local machine.
    #[clap(long, default_value = "1800")]
    pub executor_cleanup_interval: u64,

    /// The number of seconds to retain job directories on each worker 604800 (7 days, 7 * 24 * 3600), In other words, after job done, how long the resulting data is retained
    #[clap(long, default_value = "604800")]
    pub executor_cleanup_ttl: u64,

    /// Hetu hetu query mysql handler local host name or IP address to bind to. Default: 127.0.0.1
    #[clap(long, default_value = "127.0.0.1")]
    pub mysql_handler_host: String,

    /// Hetu hetu query mysql handler local bind port. Default: 3307
    #[clap(long, default_value = "3307")]
    pub mysql_handler_port: i32,
}

fn true_or_false(s: &str) -> Result<bool> {
    match s {
        "true" => Ok(true),
        _ => Ok(false),
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let args: Self = Config::parse();

        let mut builder: serfig::Builder<Self> = serfig::Builder::default();
        // Load local config file
        {
            let config_file = if !args.config_file.is_empty() {
                args.config_file.clone()
            } else if let Ok(path) = env::var("HETU_QUERY_CONFIG_FILE") {
                path
            } else {
                "".to_string()
            };

            println!("config_file: {}", config_file);
            builder = builder.collect(from_file(Toml, &config_file));
        }

        // Load env
        builder = builder.collect(from_env());

        // load args
        builder = builder.collect(from_self(args));

        let b_conf = builder.build();
        let conf = match b_conf {
            Ok(config) => Ok(config),
            Err(err) => Err(HetuError::Internal(err.to_string())),
        };
        conf
    }
}

#[cfg(test)]
mod test {
    use crate::config::config::Config;
    use hetu_error::Result;

    #[tokio::test]
    async fn hetu_query_load_config() {
        let conf: Result<Config> = Config::load();

        match conf {
            Ok(conf) => {
                assert_eq!(conf.bind_port, 50051);
            }
            Err(err) => {
                panic!("load config error: {}", err);
            }
        }
    }
}
