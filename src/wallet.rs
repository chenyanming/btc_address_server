use crate::{bech32, opcodes, read_seed};
use anyhow::{Context, Result};

use bitvec::prelude::*;
use core::convert::TryFrom;
use ring::{digest, pbkdf2};
use ripemd160::{Digest, Ripemd160};
use secp256k1::{constants::PUBLIC_KEY_SIZE, Message, PublicKey, Secp256k1, SecretKey};
use std::fmt::Display;
use std::num::NonZeroU32;

type PubKey = [u8; PUBLIC_KEY_SIZE];

pub trait Address {
    /// comsume Self and return the address
    fn to_address(self) -> String;
}

pub struct Segwit {
    public_key: PubKey,
    address: String,
}

pub struct Multisig {
    public_keys: Vec<PubKey>,
    address: String,
}

impl Address for Segwit {
    fn to_address(self) -> String {
        self.address
    }
}

impl Address for Multisig {
    fn to_address(self) -> String {
        self.address
    }
}

/// Provide the common functions, e.g. create the public key and baisc bitcoin address generation function
mod wallet {
    use super::*;

    /// Create public key from `mnemonic_words` and `salt`
    pub fn new_public_key(mnemonic_words: &str, salt: &str) -> PubKey {
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
        log::debug!("Seed: {}", hex::encode(pbkdf2_hash));

        let master_private_key = &pbkdf2_hash.as_ref()[..32];
        log::debug!("Master Private Key: {}", hex::encode(master_private_key));

        let master_chain_code = &pbkdf2_hash.as_ref()[32..CREDENTIAL_LEN];
        log::debug!("Master Chain Code: {}", hex::encode(master_chain_code));

        let secp = Secp256k1::new();
        let secret_key =
            SecretKey::from_slice(master_private_key).expect("32 bytes, within curve order");
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        log::debug!(
            "1. Master Public Key: {}, len: {}",
            hex::encode(public_key.serialize()),
            public_key.serialize().len()
        );
        public_key.serialize()
    }

    /// Create a legacy bitcoin address, prefix with "1"
    pub fn new_legacy_address(version: u8, public_key: &[u8]) -> Result<String> {
        // mnemonic words -> 512 bits (64 bytes) Seed
        // let public_key = new_public_key(
        //     "army van defense carry jealous true garbage claim echo media make crunch",
        //     "mnemonic",
        // );
        log::debug!("1. Master Public Key: {}", hex::encode(public_key));

        let sha256 = digest::digest(&digest::SHA256, &public_key);
        log::debug!("2. SHA-256 hash of 1: {}", hex::encode(&sha256));

        let mut ripemd160 = Ripemd160::new();
        ripemd160.update(sha256);
        let ripemd160 = ripemd160.finalize();
        log::debug!("3. RIPEMD-160 hash of 2: {}", hex::encode(&ripemd160));

        let mut network = vec![];
        network.extend([version]);
        network.extend(ripemd160);
        log::debug!("4. Add network byte to 3: {}", hex::encode(&network));

        let sha256 = digest::digest(&digest::SHA256, &network);
        log::debug!("5. SHA-256 hash of 4: {}", hex::encode(&sha256));

        let sha256 = digest::digest(&digest::SHA256, sha256.as_ref());
        log::debug!("6. SHA-256 hash of 5: {}", hex::encode(&sha256));

        let first_four_bytes = &sha256.as_ref()[..4];
        log::debug!(
            "7. First four bytes of 6: {}",
            hex::encode(&first_four_bytes)
        );

        let mut final_result = vec![];
        final_result.extend(network);
        final_result.extend(first_four_bytes);

        log::debug!("8. Add 7 to the end of 4: {}", hex::encode(&final_result));

        let address = bs58::encode(&final_result);
        Ok(address.into_string())
    }
}

impl Segwit {
    /// Create a Hierarchical Deterministic (HD) Segregated Witness (SegWit) Bitcoin address from seed
    pub fn from_seed(seed: &str) -> Result<Self> {
        let public_key = wallet::new_public_key(seed, "mnemonic");
        Ok(Self::new(public_key)?)
    }

    /// Create a Hierarchical Deterministic (HD) Segregated Witness (SegWit) Bitcoin address from a public key
    pub fn new(public_key: PubKey) -> Result<Self> {
        // let public_key =
        //     hex::decode("0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")?;
        let sha256 = digest::digest(&digest::SHA256, &public_key);

        log::debug!("2. SHA-256 hash of 1: {}", hex::encode(&sha256));

        let mut ripemd160 = Ripemd160::new();
        ripemd160.update(sha256);
        let ripemd160 = ripemd160.finalize();
        log::debug!(
            "3. RIPEMD-160 hash of 2: {}, len: {}",
            hex::encode(&ripemd160),
            ripemd160.len()
        );

        let mut bv_source: BitArray<Msb0, [u8; 20]> = BitArray::new(ripemd160.into());
        let mut bv_target: Vec<u8> = vec![];
        for i in 0..32 {
            bv_target.push(bv_source[5 * i..5 * (i + 1)].load_be());
        }

        log::debug!("squash the bytes of 3: {}", hex::encode(&bv_target));

        let mut witness = vec![];
        witness.extend([0]);
        witness.extend(bv_target);
        log::debug!(
            "4. Add witness version byte to 3: {}",
            hex::encode(&witness)
        );

        let checksum = bech32::bech32_create_checksum("bc", &witness);
        log::debug!("5. Compute checksum of 4: {}", hex::encode(&checksum));

        witness.extend(checksum);
        log::debug!(
            "6. Append the checksum to result of 5: {}",
            hex::encode(&witness)
        );
        let witness_map = witness
            .into_iter()
            .map(|x| bech32::CHARSET[x as usize].to_string())
            .collect::<String>();

        log::debug!("6. Map each value to its corresponding character in Bech32Chars (qpzry9x8gf2tvdw0s3jn54khce6mua7l) of 5: {}", &witness_map);

        let address = "bc".to_string() + &bech32::SEP.to_string() + &witness_map;
        Ok(Self {
            public_key,
            address,
        })
    }
}

impl Display for Segwit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Segwit Address: {}", self.address)
    }
}

impl Multisig {
    /// Generate an n-out-of-m Multisignature (multi-sig) Pay-To-Script-Hash (P2SH) bitcoin address
    pub fn new(m: u8, n: u8, public_keys: Vec<PubKey>) -> Result<Self> {
        let mut redeem_script = Vec::new();
        redeem_script.extend([u8::from(opcodes::OpPushNum::try_from(m)?)]);
        public_keys.iter().for_each(|key| {
            redeem_script.push(0x21);
            redeem_script.extend(key);
        });
        redeem_script.extend([u8::from(opcodes::OpPushNum::try_from(n)?)]);
        redeem_script.push(opcodes::OP_CHECKMULTISIG);

        log::debug!("Redeem script: {:x?}", hex::encode(&redeem_script));

        let address = wallet::new_legacy_address(5, &redeem_script)?;
        Ok(Self {
            public_keys,
            address,
        })
    }
}

impl Display for Multisig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Multisig Address: {}", self.address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn test_new_legacy_address() {
        assert_eq!(
            wallet::new_legacy_address(
                0,
                &wallet::new_public_key(
                    "army van defense carry jealous true garbage claim echo media make crunch",
                    "mnemonic",
                )
            )
            .unwrap(),
            "15izCzAjLZtMZChHsVrVQ1GmJ5psPRGL6C".to_string(),
        );
    }

    #[test]
    fn test_new_segwit_address() {
        assert_eq!(
            Segwit::new(
                hex::decode("0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")
                    .unwrap()
                    .try_into()
                    .unwrap()
            )
            .unwrap()
            .to_address(),
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string(),
        );
    }

    #[test]
    fn test_new_multisig_p2sh_address() {
        assert_eq!(
            Multisig::new(
                3,
                3,
                vec![
                    hex::decode(
                        "03d728ad6757d4784effea04d47baafa216cf474866c2d4dc99b1e8e3eb936e730"
                    )
                    .unwrap()
                    .try_into()
                    .unwrap(),
                    hex::decode(
                        "03aeb681df5ac19e449a872b9e9347f1db5a0394d2ec5caf2a9c143f86e232b0d9"
                    )
                    .unwrap()
                    .try_into()
                    .unwrap(),
                    hex::decode(
                        "02d83bba35a8022c247b645eed6f81ac41b7c1580de550e7e82c75ad63ee9ac2fd"
                    )
                    .unwrap()
                    .try_into()
                    .unwrap(),
                ]
            )
            .unwrap()
            .to_address(),
            "3Bzxiixsr6ZKyJk9H5MLc52R7LZw3uzBuy".to_string(),
        );
    }
}
