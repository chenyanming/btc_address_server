use anyhow::{Context, Result};
use btc_address_server::{http, log::setup};
use tokio::task;

use ring::{digest, pbkdf2};
use std::num::NonZeroU32;

#[tokio::main]
async fn main() -> Result<()> {
    // log
    setup();
    log::info!("Hello, world!");

    // http
    task::spawn(async move {
        http::start_http_server().await;
    });

    // mnemonic words -> 512 bits (64 bytes) Seed
    let mnemonic_words =
        "army van defense carry jealous true garbage claim echo media make crunch".as_bytes();
    let salt = "mnemonic".as_bytes();
    const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;
    let n_iter = NonZeroU32::new(2_048).unwrap();
    let mut pbkdf2_hash = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA512,
        n_iter,
        salt,
        mnemonic_words,
        &mut pbkdf2_hash,
    );
    // log::info!("Seed: {:x?}", pbkdf2_hash.as_ref());
    log::info!("Seed: {}", hex::encode(pbkdf2_hash));

    let master_private_key = &pbkdf2_hash.as_ref()[..32];
    log::info!("Master Private Key: {}", hex::encode(master_private_key));

    let master_chain_code = &pbkdf2_hash.as_ref()[32..CREDENTIAL_LEN];
    log::info!("Master Chain Code: {}", hex::encode(master_chain_code));

    loop {}
}
