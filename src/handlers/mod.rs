use std::fmt::Display;

use serde::{Deserialize, Serialize};

// Types of handlers
pub mod courses;
pub mod modules;
pub mod tasks;

// Errors stuff
#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    error_type: String,
    error_msg: String,
}

impl ErrorResponse {
    pub fn new(error_type: &str, error_msg: &str) -> Self {
        Self {
            error_type: error_type.to_string(),
            error_msg: error_msg.to_owned(),
        }
    }
}

// pub trait IntoErrorResponse {
//     fn into_error_response(&self) -> ErrorResponse;
// }

pub enum ErrorTypes {
    InternalError,
    JwtTokenExpired,
    MaxAttemptsSubmit,
}

impl Display for ErrorTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InternalError => write!(f, "server_internal_error"),
            Self::JwtTokenExpired => write!(f, "jwt_token_expired"),
            Self::MaxAttemptsSubmit => write!(f, "max_attempts_submit"),
        }
    }
}
