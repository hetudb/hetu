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

#![doc = include_str ! ("../README.md")]
pub fn print_version() {
    println!("Hetu cloud service version: {}", CLOUD_SERVICE_VERSION)
}

pub const CLOUD_SERVICE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod config;
pub mod meta;
pub mod scheduler;

pub mod api;
pub mod planner;
pub mod scheduler_server;
#[cfg(feature = "sled")]
pub mod standalone;
pub mod state;

#[cfg(test)]
pub mod test_utils;
