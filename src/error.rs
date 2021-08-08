use thiserror::Error;
#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Push Number Exceeds")]
    NumberOfKeysExceeds,
    #[error("M shoud not be larger than N")]
    InvalidM,
}
