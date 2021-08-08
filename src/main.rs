use anyhow::Result;
use btc_address_server::{http, log::setup};
use tokio::task;

#[tokio::main]
async fn main() -> Result<()> {
    // log
    setup();
    log::info!("Hello, world!");

    // http
    task::spawn(async move {
        if let Err(e) = http::start_http_server().await {
            log::error!("{:?}", e);
        }
    });

    loop {}
}
