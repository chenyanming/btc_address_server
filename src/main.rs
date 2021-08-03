use btc_address_server::{http, log::setup};
use anyhow::{Context, Result};
use tokio::task;

#[tokio::main]
async fn main() -> Result<()> {
    // log
    setup();
    log::info!("Hello, world!");

    // http
    task::spawn(async move {
        http::start_http_server().await;
    });

    loop {}
}
