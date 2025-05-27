use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use crate::{
    controllers,
    handlers::{self, ErrorTypes},
    AppState,
};

// PUBLIC GET /courses - Get a list of all available courses (for main page)
pub async fn get_all_courses(State(state): State<AppState>) -> Result<Response, Response> {
    match controllers::course::get_all_courses(&state).await {
        Ok(courses) => {
            let body = json!({
                "data": courses,
            });

            return Ok((StatusCode::OK, axum::Json(body)).into_response());
        }
        Err(why) => {
            eprintln!("Why co: {}", why);
            
            return Err((
                StatusCode::BAD_REQUEST,
                axum::Json(handlers::ErrorResponse::new(
                    ErrorTypes::InternalError,
                    "Could not fetch courses",
                )),
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
    match controllers::course::get_course(&state, course_id).await {
        Ok(course) => {
            let body = json!({
                "data": course,
            });

            return Ok((StatusCode::OK, axum::Json(body)).into_response());
        }
        Err(why) => {
            eprintln!("Why co: {}", why);

            return Err((
                StatusCode::BAD_REQUEST,
                axum::Json(handlers::ErrorResponse::new(
                    ErrorTypes::InternalError,
                    "Could not fetch the course",
                )),
            )
                .into_response());
        }
    };
}
