use thiserror::Error;
#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Push Number Exceeds")]
    NumberOfKeysExceeds,
    #[error("M should not be larger than N")]
    InvalidM,
    #[error("N should not be zero")]
    EmptyN,
    #[error("N is larger than the total number of public keys")]
    LargeN,
    #[error("N is less than the total number of public keys")]
    InvalidN,
}
