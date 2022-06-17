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
