use axum::{
    extract::FromRequestParts,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::common::error::{ErrorResponse, ErrorTypes};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub id: u32,
    pub exp: i64,
}

impl<S: std::marker::Sync> FromRequestParts<S> for Claims {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _: &S,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|s| s.split_whitespace().last())
            .ok_or("Missing header")
            .map_err(|why| {
                eprintln!("{}", why);
                (
                    StatusCode::BAD_REQUEST,
                    axum::Json(ErrorResponse::new(
                        ErrorTypes::NoAuthHeader,
                        "No auth header",
                    )),
                )
                    .into_response()
            })?;

        Ok(decode::<Claims>(
            token,
            &DecodingKey::from_secret(std::env::var("SECRET_WORD_JWT").unwrap().as_ref()),
            &Validation::default(),
        )
        .map_err(|err| {
            eprintln!("Could not validate: {}", err);
            (
                StatusCode::UNAUTHORIZED,
                axum::Json(ErrorResponse::new(
                    ErrorTypes::JwtTokenExpired,
                    "Token update requested",
                )),
            )
                .into_response()
        })?
        .claims)
    }
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
