use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;

use actix_web::dev::ServiceRequest;
use actix_web::Result;

use crate::error::ServiceError;

use alcoholic_jwt::{token_kid, validate, Validation, JWKS};
use serde::{Deserialize, Serialize};
// use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    company: String,
    exp: usize,
}

pub async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest> {
    let config = req
        .app_data::<Config>()
        .map(|data| data.clone())
        .unwrap_or_else(Default::default);
    log::debug!("Received token: {:?}", &credentials.token());
    match validate_token(credentials.token()).await {
        Ok(res) => {
            if res == true {
                Ok(req)
            } else {
                Err(AuthenticationError::from(config).into())
            }
        }
        Err(_) => Err(AuthenticationError::from(config).into()),
    }
}

pub async fn validate_token(token: &str) -> Result<bool, ServiceError> {
    let authority = std::env::var("AUTHORITY")
        .unwrap_or_else(|_| "https://dev-babm2h9u.us.auth0.com/".to_string());
    let jwks = fetch_jwks(&format!(
        "{}{}",
        authority.as_str(),
        ".well-known/jwks.json"
    ))
    .await
    .expect("failed to fetch jwks");
    let validations = vec![Validation::Issuer(authority), Validation::SubjectPresent];
    let kid = match token_kid(&token) {
        Ok(res) => res.expect("failed to decode kid"),
        Err(_) => return Err(ServiceError::JWKSFetchError),
    };
    let jwk = jwks.find(&kid).expect("Specified key not found in set");
    let res = validate(token, jwk, validations);
    Ok(res.is_ok())
}

async fn fetch_jwks(uri: &str) -> Result<JWKS, Box<dyn std::error::Error>> {
    let res = reqwest::get(uri).await?;
    let val = res.json::<JWKS>().await?;
    return Ok(val);
}

#[actix_rt::test]
async fn test_fetch_jwks() {
    let authority =
        std::env::var("AUTHORITY").expect(|_| "https://dev-babm2h9u.us.auth0.com/".to_string());
    let jwks = fetch_jwks(&format!(
        "{}{}",
        authority.as_str(),
        ".well-known/jwks.json"
    ))
    .await
    .expect("failed to fetch jwks");

    assert_eq!(format!("{:?}", jwks), "JWKS { keys: [JWK { kty: RSA, alg: Some(RS256), kid: Some(\"biM-mtmTqfrAmt8E8FdyY\"), n: \"sC7kTz6U8qvqA_TXDM3b2UykSapGAkj0kIfCwIW5JljnGQEKqVUIOpeF8h1UZWYAaT3rfyOYqGHPHqmFmf3TF_gDDEacxDDpK3lP5MQfo6iltCkiFMbLBTlpQvhOU9WalhuanAWFs6Dz6MUbEFy6Hv-k0vUGv6wyMeFkOczI8J-nxyWTv49SHpFHP2yTQFg31ACXuzHxIFWAMwtNrspKdG2h2fo66XEAZiZUtyxrBHWi05dbPg2pNH_QGbStIAyAyqlWiaksf9M6pjGc2uVwSydWbNvm4SmTPfw2Y17jQkxSfcB6wBJMLYMPIGukeZ-k9UHJALjQPEkLOGe_ttmd2Q\", e: \"AQAB\" }, JWK { kty: RSA, alg: Some(RS256), kid: Some(\"O03lUqKFkran1xQorn3jv\"), n: \"xJlIz_YLSpE6_ZPogzVgRYXCfiq4M49lGiiyia4xTsmxvb3RlttHwe0JByrxDhsbyQxHYGlVb5PfrolcS0yTEulr3h6KuHz9i5W93Xq_FURefV2mle2qNLWr6sDpmfrwp04CCFF0EUHIv9n911XtoB9u2iuKrPGVrRf0fPTCOU-5i0wcQ0c2EAwjwkrVjdrhxL_ydph2gxW58FVuhM4gLlWEJThoYM7GjfM-dLYhIoBF3j_-B7cywpiUhmEwyQuYhFpND_XxPEheFPEgLsA-E8IYWDouRBTUEYNU2hVsz3XY34u1TISio9r897oihvMsW5dj0aUEPO3yJcUdywXvfQ\", e: \"AQAB\" }] }".to_owned());
}

#[actix_rt::test]
async fn test_validate_token() {
    assert_eq!(validate_token("xxxxx").await.unwrap(), true)
}
