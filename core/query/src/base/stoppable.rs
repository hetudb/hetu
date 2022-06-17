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

use hetu_error::HetuError;
use tokio::sync::broadcast;

/// A task that can be started and stopped.
#[async_trait::async_trait]
pub trait Stoppable {
    /// Start working without blocking the calling thread.
    /// When returned, it should have been successfully started.
    /// Otherwise an Err() should be returned.
    ///
    /// Calling `start()` on a started task should get an error.
    async fn start(&mut self) -> Result<(), HetuError>;

    /// Blocking stop. It should not return until everything is cleaned up.
    ///
    /// In case a graceful `stop()` had blocked for too long,
    /// the caller submit a FORCE stop by sending a `()` to `force`.
    /// An impl should either close everything at once, or just ignore the `force` signal if it does not support force stop.
    ///
    /// Calling `stop()` twice should get an error.
    async fn stop(
        &mut self,
        mut force: Option<broadcast::Receiver<()>>,
    ) -> Result<(), HetuError>;
}
