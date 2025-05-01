use serde::{Deserialize, Serialize};

pub mod courses;

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

pub trait IntoErrorResponse {
    fn into_error_response(&self) -> ErrorResponse;
}
