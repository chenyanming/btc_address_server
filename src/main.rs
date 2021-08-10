use btc_address_server::{http, log::setup};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // log
    setup();
    log::info!("Hello, world!");

    // http
    http::start_http_server().await?;

    loop {}
}
