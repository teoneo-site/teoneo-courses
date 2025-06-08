use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::Query;
use serde::Deserialize;
use serde_json::json;
use crate::{
    common::token::Claims, controllers, db, handlers::{self, ErrorTypes}, AppState
};

#[derive(Deserialize)]
pub struct IdsStruct {
    ids: Vec<i32>,
}

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

// PUBLIC GET /courses - Get a list of all available courses (for main page)
pub async fn get_courses_by_ids(State(state): State<AppState>, Query(ids): Query<IdsStruct>) -> Result<Response, Response> {
    match controllers::course::get_courses_by_ids(&state, ids.ids).await {
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

// PUBLIC GET /course/{course_id} - Get info about a single course
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

pub async fn get_course_progress(State(state): State<AppState>, Path(course_id): Path<i32>, claims: Claims) -> Result<Response, Response> {
    let user_id = claims.id;
    match controllers::course::get_course_progress(&state, course_id, user_id).await {
        Ok(progress) => {
            let body = json!({
                "data": progress,
            });
            return Ok((StatusCode::OK, axum::Json(body)).into_response())
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
    }
}