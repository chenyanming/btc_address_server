use anyhow::{Context, Result};
use btc_address_server::{bech32, http, log::setup, opcodes};
use tokio::task;

use bitvec::prelude::*;
use core::convert::TryFrom;
use ring::{digest, pbkdf2};
use ripemd160::{Digest, Ripemd160};
use secp256k1::{constants::PUBLIC_KEY_SIZE, Message, PublicKey, Secp256k1, SecretKey};
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

    log::info!(
        "Base58 encoding of 8 (aka bitcoin address): {}",
        generate_legacy_address(
            0,
            &generate_public_key(
                "army van defense carry jealous true garbage claim echo media make crunch",
                "mnemonic",
            )[..]
        )?
    );
    log::info!(
        "Bech32_encoded address consists of 3 parts: HRP + Separator + Data: {}",
        generate_segwit_address(&hex::decode(
            "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
        )?)?
    );

    log::info!(
        "Multisig p2sh address: {}",
        generate_multisig_p2sh_address(
            3,
            3,
            vec![
                &hex::decode("03d728ad6757d4784effea04d47baafa216cf474866c2d4dc99b1e8e3eb936e730")?,
                &hex::decode("03aeb681df5ac19e449a872b9e9347f1db5a0394d2ec5caf2a9c143f86e232b0d9")?,
                &hex::decode("02d83bba35a8022c247b645eed6f81ac41b7c1580de550e7e82c75ad63ee9ac2fd")?,
            ]
        )?
    );

    loop {}
}

fn generate_public_key(mnemonic_words: &str, salt: &str) -> [u8; PUBLIC_KEY_SIZE] {
    // mnemonic words -> 512 bits (64 bytes) Seed
    let mnemonic_words = mnemonic_words.as_bytes();
    let salt = salt.as_bytes();
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

    let secp = Secp256k1::new();
    let secret_key =
        SecretKey::from_slice(master_private_key).expect("32 bytes, within curve order");
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
    log::info!(
        "1. Master Public Key: {}, len: {}",
        hex::encode(public_key.serialize()),
        public_key.serialize().len()
    );
    public_key.serialize()
}

fn generate_legacy_address(version: u8, public_key: &[u8]) -> Result<String> {
    // mnemonic words -> 512 bits (64 bytes) Seed
    // let public_key = generate_public_key(
    //     "army van defense carry jealous true garbage claim echo media make crunch",
    //     "mnemonic",
    // );
    log::info!("1. Master Public Key: {}", hex::encode(public_key));

    let sha256 = digest::digest(&digest::SHA256, &public_key);
    log::info!("2. SHA-256 hash of 1: {}", hex::encode(&sha256));

    let mut ripemd160 = Ripemd160::new();
    ripemd160.update(sha256);
    let ripemd160 = ripemd160.finalize();
    log::info!("3. RIPEMD-160 hash of 2: {}", hex::encode(&ripemd160));

    let mut network = vec![];
    network.extend([version]);
    network.extend(ripemd160);
    log::info!("4. Add network byte to 3: {}", hex::encode(&network));

    let sha256 = digest::digest(&digest::SHA256, &network);
    log::info!("5. SHA-256 hash of 4: {}", hex::encode(&sha256));

    let sha256 = digest::digest(&digest::SHA256, sha256.as_ref());
    log::info!("6. SHA-256 hash of 5: {}", hex::encode(&sha256));

    let first_four_bytes = &sha256.as_ref()[..4];
    log::info!(
        "7. First four bytes of 6: {}",
        hex::encode(&first_four_bytes)
    );

    let mut final_result = vec![];
    final_result.extend(network);
    final_result.extend(first_four_bytes);

    log::info!("8. Add 7 to the end of 4: {}", hex::encode(&final_result));

    let address = bs58::encode(&final_result);
    Ok(address.into_string())
}

fn generate_segwit_address(public_key: &[u8]) -> Result<String> {
    // let public_key =
    //     hex::decode("0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")?;
    let sha256 = digest::digest(&digest::SHA256, &public_key);

    log::info!("2. SHA-256 hash of 1: {}", hex::encode(&sha256));

    let mut ripemd160 = Ripemd160::new();
    ripemd160.update(sha256);
    let ripemd160 = ripemd160.finalize();
    log::info!(
        "3. RIPEMD-160 hash of 2: {}, len: {}",
        hex::encode(&ripemd160),
        ripemd160.len()
    );

    let mut bv_source: BitArray<Msb0, [u8; 20]> = BitArray::new(ripemd160.into());
    let mut bv_target: Vec<u8> = vec![];
    for i in 0..32 {
        bv_target.push(bv_source[5 * i..5 * (i + 1)].load_be());
    }

    log::info!("squash the bytes of 3: {}", hex::encode(&bv_target));

    let mut witness = vec![];
    witness.extend([0]);
    witness.extend(bv_target);
    log::info!(
        "4. Add witness version byte to 3: {}",
        hex::encode(&witness)
    );

    let checksum = bech32::bech32_create_checksum("bc", &witness);
    log::info!("5. Compute checksum of 4: {}", hex::encode(&checksum));

    witness.extend(checksum);
    log::info!(
        "6. Append the checksum to result of 5: {}",
        hex::encode(&witness)
    );
    let witness_map = witness
        .into_iter()
        .map(|x| bech32::CHARSET[x as usize].to_string())
        .collect::<String>();

    log::info!("6. Map each value to its corresponding character in Bech32Chars (qpzry9x8gf2tvdw0s3jn54khce6mua7l) of 5: {}", &witness_map);

    let address = "bc".to_string() + &bech32::SEP.to_string() + &witness_map;
    Ok(address)
}

fn generate_multisig_p2sh_address(m: u8, n: u8, public_keys: Vec<&[u8]>) -> Result<String> {
    let mut redeem_script = vec![];
    redeem_script.extend([u8::from(opcodes::OpPushNum::try_from(m)?)]);
    public_keys.into_iter().for_each(|key| {
        redeem_script.push(0x21);
        redeem_script.extend(key);
    });
    redeem_script.extend([u8::from(opcodes::OpPushNum::try_from(n)?)]);
    redeem_script.push(opcodes::OP_CHECKMULTISIG);

    log::info!("Redeem script: {:x?}", hex::encode(&redeem_script));

    generate_legacy_address(5, &redeem_script)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_legacy_address() {
        assert_eq!(
            generate_legacy_address(
                0,
                &generate_public_key(
                    "army van defense carry jealous true garbage claim echo media make crunch",
                    "mnemonic",
                )[..]
            )
            .unwrap(),
            "15izCzAjLZtMZChHsVrVQ1GmJ5psPRGL6C".to_string(),
        );
    }

    #[test]
    fn test_generate_bech32_address() {
        assert_eq!(
            generate_segwit_address(
                &hex::decode("0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")
                    .unwrap()
            )
            .unwrap(),
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string(),
        );
    }

    #[test]
    fn test_generate_multisig_p2sh_address() {
        assert_eq!(
            generate_multisig_p2sh_address(
                3,
                3,
                vec![
                    &hex::decode("03d728ad6757d4784effea04d47baafa216cf474866c2d4dc99b1e8e3eb936e730").unwrap(),
                    &hex::decode("03aeb681df5ac19e449a872b9e9347f1db5a0394d2ec5caf2a9c143f86e232b0d9").unwrap(),
                    &hex::decode("02d83bba35a8022c247b645eed6f81ac41b7c1580de550e7e82c75ad63ee9ac2fd").unwrap(),
                ]
            )
            .unwrap(),
            "3Bzxiixsr6ZKyJk9H5MLc52R7LZw3uzBuy".to_string(),
        );
    }
}
