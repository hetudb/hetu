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

//! Hetu Cloud Service Rust scheduler binary.

use log::info;

use hetu_cloudsrv::config::Config;
use hetu_cloudsrv::scheduler::SchedulerHandler;
use hetu_cloudsrv::CLOUD_SERVICE_VERSION;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let conf: Config = Config::load()?;

    // Scheduler RPC API and HTTP API service.
    {
        let mut scheduler_handler = SchedulerHandler::new(conf.clone());
        let listening = scheduler_handler.start().await?;
        info!(
            "Hetu Cloud Service v{} Rust Scheduler listening on {:?}",
            CLOUD_SERVICE_VERSION, listening
        );
    }
    Ok(())
}
