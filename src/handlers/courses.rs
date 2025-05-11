use axum::{
    extract::{Path, State},
    http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::MySqlPool;

use crate::{
    controllers,
    handlers::{self, ErrorTypes, ResponseBody},
    AppState,
};

// PUBLIC GET /courses - Get a list of all available courses (for main page)
pub async fn get_all_courses(State(state): State<AppState>) -> Result<Response, Response> {
    match controllers::course::get_all_courses(&state.pool).await {
        Ok(courses) => {
            let body = json!({
                "data": courses,
            })
            .to_string();

            return Ok(ResponseBody::new(StatusCode::OK, None, body).into_response());
        }
        Err(why) => {
            eprintln!("Why co: {}", why);

            return Err(ResponseBody::new(
                StatusCode::BAD_REQUEST,
                None,
                handlers::ErrorResponse::new(ErrorTypes::InternalError, "Could not fetch courses"),
            )
            .into_response());
        }
    };
}

// PUBLCI GET /course/{course_id} - Get info about a single course
pub async fn get_course(
    State(state): State<AppState>,
    Path(course_id): Path<i32>,
) -> Result<Response, Response> {
    match controllers::course::get_course(&state.pool, course_id).await {
        Ok(course) => {
            let body = json!({
                "data": course,
            })
            .to_string();

            return Ok(ResponseBody::new(StatusCode::OK, None, body).into_response());
        }
        Err(why) => {
            eprintln!("Why co: {}", why);

            return Err(ResponseBody::new(
                StatusCode::BAD_REQUEST,
                None,
                handlers::ErrorResponse::new(
                    ErrorTypes::InternalError,
                    "Could not fetch the course",
                ),
            )
            .into_response());
        }
    };
}
