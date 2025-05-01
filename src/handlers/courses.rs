use axum::{
    extract::{Path, State},
    http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::MySqlPool;

use crate::{
    controllers,
    handlers::{self, ErrorTypes},
};

// PUBLIC GET /courses - Get a list of all available courses (for main page)
pub async fn get_all_courses(State(state): State<MySqlPool>) -> Result<Response, Response> {
    match controllers::course::get_all_courses(&state).await {
        Ok(courses) => {
            let body = json!({
                "data": courses,
            })
            .to_string();
            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

            return Ok((StatusCode::OK, headers, body).into_response());
        }
        Err(why) => {
            eprintln!("Why: {}", why);

            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            return Err((
                StatusCode::BAD_REQUEST,
                headers,
                serde_json::to_string_pretty(&handlers::ErrorResponse::new(
                    &ErrorTypes::InternalError.to_string(),
                    "Could not fetch courses",
                ))
                .unwrap(), // Should not panic, because struct is always valid for converting into JSON
            )
                .into_response());
        }
    };
}

// PUBLCI GET /course/{course_id} - Get info about a single course
pub async fn get_course(
    State(state): State<MySqlPool>,
    Path(course_id): Path<i32>,
) -> Result<Response, Response> {
    match controllers::course::get_course(&state, course_id).await {
        Ok(course) => {
            let body = json!({
                "data": course,
            })
            .to_string();
            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            return Ok((StatusCode::OK, headers, body).into_response());
        }
        Err(why) => {
            eprintln!("Why: {}", why);

            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            return Err((
                StatusCode::BAD_REQUEST,
                headers,
                serde_json::to_string_pretty(&handlers::ErrorResponse::new(
                    &ErrorTypes::InternalError.to_string(),
                    "Could not fetch the course",
                ))
                .unwrap(), // Should not panic, because struct is always valid for converting into JSON
            )
                .into_response());
        }
    };
}
