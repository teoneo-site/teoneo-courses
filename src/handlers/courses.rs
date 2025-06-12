use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::extract::Query;
use serde::Deserialize;
use serde_json::json;
use crate::{
    common::{self, token::Claims}, controllers, db, handlers::{self, ErrorTypes}, AppState
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
                StatusCode::INTERNAL_SERVER_ERROR,
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
pub async fn get_courses_by_ids(State(state): State<AppState>, headers: HeaderMap, Query(ids): Query<IdsStruct>) -> Result<Response, Response> {
    let authorization_token = 
       headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.split_whitespace().last())
        .unwrap_or("");

    match common::token::verify_jwt_token(authorization_token) {
        Ok(user_id) => {
            match controllers::course::get_courses_by_ids_expanded(&state, ids.ids, user_id).await {
                Ok(courses) => {
                    let body = json!({
                        "data": courses,
                    });

                    return Ok((StatusCode::OK, axum::Json(body)).into_response());
                }
                Err(why) => {
                    eprintln!("Why co: {}", why);
                    
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(handlers::ErrorResponse::new(
                            ErrorTypes::InternalError,
                            "Could not fetch courses",
                        )),
                    )
                        .into_response());
                }
            };
        }
        Err(_) => {
            match controllers::course::get_courses_by_ids_basic(&state, ids.ids).await {
                Ok(courses) => {
                    let body = json!({
                        "data": courses,
                    });

                    return Ok((StatusCode::OK, axum::Json(body)).into_response());
                }
                Err(why) => {
                    eprintln!("Why co: {}", why);
                    
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(handlers::ErrorResponse::new(
                            ErrorTypes::InternalError,
                            "Could not fetch courses",
                        )),
                    )
                        .into_response());
                }
            };
        }
    }
}

// PUBLIC GET /course/{course_id} - Get info about a single course
pub async fn get_course(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(course_id): Path<i32>,
) -> Result<Response, Response> {
    let authorization_token = 
       headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.split_whitespace().last())
        .unwrap_or("");

    match common::token::verify_jwt_token(authorization_token) {
        Ok(user_id) => {
            match controllers::course::get_course_extended(&state, course_id, user_id).await {
                Ok(course) => {
                    let body = json!({
                        "data": course,
                    });

                    return Ok((StatusCode::OK, axum::Json(body)).into_response());
                }
                Err(why) => {
                    eprintln!("Why co: {}", why);

                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(handlers::ErrorResponse::new(
                            ErrorTypes::InternalError,
                            "Could not fetch the course",
                        )),
                    )
                        .into_response());
                }
            };
        }
        Err(_) => {
            match controllers::course::get_course_basic(&state, course_id).await {
                Ok(course) => {
                    let body = json!({
                        "data": course,
                    });

                    return Ok((StatusCode::OK, axum::Json(body)).into_response());
                }
                Err(why) => {
                    eprintln!("Why co: {}", why);

                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(handlers::ErrorResponse::new(
                            ErrorTypes::InternalError,
                            "Could not fetch the course",
                        )),
                    )
                        .into_response());
                }
            };
        }
    }
    
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
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(handlers::ErrorResponse::new(
                    ErrorTypes::InternalError,
                    "Could not fetch the course",
                )),
            )
                .into_response());
        }
    }
}

// Favourite section
pub async fn add_course_to_favourite(State(state): State<AppState>, Path(course_id): Path<i32>, claims: Claims) -> Result<Response, Response> {
    let user_id = claims.id;
    match controllers::course::add_course_to_favourite(&state, user_id, course_id).await {
        Ok(_) => {
            return Ok((StatusCode::OK).into_response())
        }
        Err(why) => {
            eprintln!("Could not favour a course: {}", why);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(handlers::ErrorResponse::new(
                    ErrorTypes::InternalError,
                    "Could not add a course to favourites",
                )),
            )
                .into_response());
        }
    }
}

pub async fn get_favourite_courses(State(state): State<AppState>, claims: Claims) -> Result<Response, Response> {
    let user_id = claims.id;
    match controllers::course::get_favourite_courses(&state, user_id).await {
        Ok(ids) => {
            let body = json!({
                "data": ids,
            });
            return Ok((StatusCode::OK, axum::Json(body)).into_response())
        }
        Err(why) => {
            eprintln!("Could not get favourite courses: {}", why);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(handlers::ErrorResponse::new(
                    ErrorTypes::InternalError,
                    "Could not fetch favourite courses",
                )),
            )
                .into_response());
        }
    }
}