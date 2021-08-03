use chrono::Local;
use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::prelude::*;

/// Setup the log
pub fn setup() {
    let _ = Builder::from_default_env()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {} [{}:{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                { buf.default_styled_level(record.level()) },
                record.module_path().unwrap_or("<unnamed>"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .target(Target::Stdout) // default is stderr
        .init();
}
