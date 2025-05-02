use chrono::{Duration, Utc};
use jsonwebtoken::{decode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Claims {
    id: u32,
    exp: i64,
}

pub fn verify_refresh_token(token: &str) -> anyhow::Result<u32> {
    let validation = Validation::default();

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(std::env::var("SECRET_WORD_REFRESH").unwrap().as_ref()),
        &validation,
    )?;
    Ok(claims.claims.id)
}

pub fn verify_jwt_token(token: &str) -> anyhow::Result<u32> {
    let validation = Validation::default();

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(std::env::var("SECRET_WORD_JWT").unwrap().as_ref()),
        &validation,
    )?;
    Ok(claims.claims.id)
}
