use anyhow::Result;
use btc_address_server::{
    http,
    log::setup,
    read_seed,
    wallet::{Address, Multisig, Segwit},
};
use std::convert::TryInto;
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

    log::info!(
        "{}",
        Segwit::from_seed(&read_seed("seed.txt").expect("Readd seed.txt error"))
            .expect("Create address failed)")
    );

    log::info!(
        "{}",
        Multisig::new(
            3,
            3,
            vec![
                hex::decode("03d728ad6757d4784effea04d47baafa216cf474866c2d4dc99b1e8e3eb936e730")?
                    .try_into()
                    .expect("slice with incorrect length"),
                hex::decode("03aeb681df5ac19e449a872b9e9347f1db5a0394d2ec5caf2a9c143f86e232b0d9")?
                    .try_into()
                    .expect("slice with incorrect length"),
                hex::decode("02d83bba35a8022c247b645eed6f81ac41b7c1580de550e7e82c75ad63ee9ac2fd")?
                    .try_into()
                    .expect("slice with incorrect length"),
            ]
        )
        .expect("Create address failed)")
    );

    loop {}
}
