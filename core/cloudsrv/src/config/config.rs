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
    /// The hetu cloud service config by a toml file
    #[clap(long, short = 'c', default_value_t)]
    pub config_file: String,

    /// The configuration backend for the scheduler, see StateBackend::variants() for options. Default: Standalone
    #[clap(long, default_value = "Standalone")]
    pub config_backend: String,

    /// Namespace for the ballista cluster that this executor will join. Default: ballista
    #[clap(long, default_value = "ballista")]
    pub namespace: String,

    /// etcd urls for use when discovery mode is `etcd`. Default: localhost:2379
    #[clap(long, default_value = "localhost:2379")]
    pub etcd_urls: String,

    /// Local host name or IP address to bind to. Default: 0.0.0.0
    #[clap(long, default_value = "0.0.0.0")]
    pub bind_host: String,

    /// bind port. Default: 50050
    #[clap(long, default_value = "50050")]
    pub bind_port: u16,

    /// The scheduing policy for the scheduler, see TaskSchedulingPolicy::variants() for options. Default: PullStaged
    #[clap(long, default_value = "PullStaged")]
    pub scheduler_policy: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let args: Self = Config::parse();

        let mut builder: serfig::Builder<Self> = serfig::Builder::default();
        // Load local config file
        {
            let config_file = if !args.config_file.is_empty() {
                args.config_file.clone()
            } else if let Ok(path) = env::var("HETU_CLOUDSRV_CONFIG_FILE") {
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
    async fn hetu_cloud_service_load_config() {
        let conf: Result<Config> = Config::load();

        match conf {
            Ok(conf) => {
                assert_eq!(conf.bind_port, 50050);
            }
            Err(err) => {
                panic!("load config error: {}", err);
            }
        }
    }
}
