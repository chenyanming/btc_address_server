use anyhow::Result;
use btc_address_server::{http, log::setup};
use tokio::task;

#[actix_web::main]
async fn main() -> Result<()> {
    // log
    setup();
    log::info!("Hello, world!");

    // http
    http::start_http_server().await?;

    loop {}
}
