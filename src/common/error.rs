use std::fmt::Display;

use axum::{http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub struct AppError(anyhow::Error);
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(ErrorResponse::new(
                ErrorTypes::InternalError,
                &format!("Something went wrong: {}", self.0),
            )),
        )
            .into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(value: E) -> Self {
        Self(value.into())
    }
}

pub mod helpers {
    use axum::{http::StatusCode, response::IntoResponse};
    // use reqwest::StatusCode;

    use crate::common::error::{ErrorResponse, ErrorTypes};

    pub fn error_response(
        status: StatusCode,
        error_type: ErrorTypes,
        error_msg: &str,
    ) -> axum::response::Response {
        (
            status,
            axum::Json(ErrorResponse::new(error_type, error_msg)),
        )
            .into_response()
    }
    #[macro_export]
    macro_rules! error_response {
        ($status:expr, $error_type:expr, $($arg:tt)*) => {
            crate::common::error::helpers::error_response($status, $error_type, &format!($($arg)*))
        };
    }
}

// Errors stuff

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    error_type: String,
    error_msg: String,
}

impl ErrorResponse {
    pub fn new(error_type: ErrorTypes, error_msg: &str) -> Self {
        Self {
            error_type: error_type.to_string(),
            error_msg: error_msg.to_owned(),
        }
    }
}

pub enum ErrorTypes {
    InternalError,
    JwtTokenExpired,
    MaxAttemptsSubmit,
    CourseNotOwned,
    NoAuthHeader,
}

impl Display for ErrorTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InternalError => write!(f, "server_internal_error"),
            Self::JwtTokenExpired => write!(f, "jwt_token_expired"),
            Self::MaxAttemptsSubmit => write!(f, "max_attempts_submit"),
            Self::CourseNotOwned => write!(f, "course_not_owned"),
            Self::NoAuthHeader => write!(f, "no_auth_header"),
        }
    }
}
