pub mod bech32;
pub mod error;
pub mod http;
pub mod log;
pub mod opcodes;
pub mod wallet;

#[cfg(feature = "postgres")]
#[macro_use]
extern crate diesel;

#[cfg(feature = "postgres")]
pub mod handlers;
#[cfg(feature = "postgres")]
pub mod models;
#[cfg(feature = "postgres")]
pub mod schema;

pub mod auth;

use std::fs::read_to_string;

/// read seed from `file`
pub fn read_seed(file: &str) -> Result<String, std::io::Error> {
    read_to_string(file)
}
