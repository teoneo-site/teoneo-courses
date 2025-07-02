use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::headers::authorization::{Bearer, Credentials};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use anyhow::anyhow;

use crate::common::error::{AppError, ErrorResponse, ErrorTypes};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub id: u32,
    pub exp: i64,
}


#[derive(Serialize, Deserialize)]
pub struct AuthHeader {
    pub claims: Claims,
    pub token: String,
}

impl<S: std::marker::Sync> FromRequestParts<S> for AuthHeader {
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
                tracing::error!("{}", why);
                (
                    StatusCode::BAD_REQUEST,
                    axum::Json(ErrorResponse::new(
                        ErrorTypes::NoAuthHeader,
                        "No auth header",
                    )),
                )
                    .into_response()
            })?;

        let claims = decode::<Claims>(
            token,
            &DecodingKey::from_secret(std::env::var("SECRET_WORD_JWT").unwrap().as_ref()),
            &Validation::default(),
        )
        .map_err(|err| {
            tracing::error!("Could not validate: {}", err);
            (
                StatusCode::UNAUTHORIZED,
                axum::Json(ErrorResponse::new(
                    ErrorTypes::JwtTokenExpired,
                    "Token update requested",
                )),
            )
                .into_response()
        })?
        .claims;

        Ok(AuthHeader {
            claims,
            token: token.to_owned(),
        })
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

pub struct OptionalBearerClaims(pub Option<u32>);

impl<S> FromRequestParts<S> for OptionalBearerClaims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers.get(AUTHORIZATION);

        match auth_header {
            Some(header_value) => {
                // Try to parse the header as a Bearer token
                let bearer =
                    Bearer::decode(header_value).ok_or(anyhow!("Could not decode bearer"))?;

                match verify_jwt_token(bearer.token()) {
                    Ok(user_id) => Ok(OptionalBearerClaims(Some(user_id))),
                    Err(_) => Ok(OptionalBearerClaims(None))
                }
            }
            None => Ok(OptionalBearerClaims(None)),
        }
    }
}