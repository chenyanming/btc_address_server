pub mod bech32;
pub mod http;
pub mod log;
pub mod opcodes;
pub mod wallet;

use std::fs::read_to_string;

/// read seed from `file`
pub fn read_seed(file: &str) -> Result<String, std::io::Error> {
    read_to_string(file)
}
