use actix_web::{error::ResponseError, HttpResponse};

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

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Internal Server Error")]
    InternalServerError,

    #[error("BadRequest: {0}")]
    BadRequest(String),

    #[error("JWKSFetchError")]
    JWKSFetchError,
}

// impl ResponseError trait allows to convert our errors into http responses with appropriate data
impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::InternalServerError => {
                HttpResponse::InternalServerError().json("Internal Server Error, Please try later")
            }
            ServiceError::BadRequest(ref message) => HttpResponse::BadRequest().json(message),
            ServiceError::JWKSFetchError => {
                HttpResponse::InternalServerError().json("Could not fetch JWKS")
            }
        }
    }
}
