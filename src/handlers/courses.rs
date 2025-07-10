use crate::{
    common::{error::{AppError, ErrorResponse}, token::{AuthHeader, OptionalBearerClaims}},
    controllers::{
        self,
        courses::{CourseInfo, CourseProgress},
    },
    BasicState,
};
use axum::{
    extract::{Path, State},
    http::{StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::extract::Query;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct IdsStruct {
    ids: Vec<i32>,
}

// PUBLIC GET /courses - Get a list of all available courses (for main page)
#[utoipa::path(
    get,
    description = "Возвращает список публичных курсов (общий)",
    path = "/courses",
    responses(
        (status = 200, description = "Успешно", body = Vec<i32>),
        (status = 500, description = "Что-то случилось", body = ErrorResponse),
    )
)]
pub async fn get_all_courses(State(state): State<BasicState>) -> Result<Response, AppError> {
    let courses = controllers::courses::get_all_courses(&state).await?;
    let body = json!({
        "data": courses,
    });

    Ok((StatusCode::OK, axum::Json(body)).into_response())
}

// PUBLIC GET /courses - Get a list of all available courses (for main page)
#[utoipa::path(
    get,
    description = "Возвращает информацию о курсах",
    path = "/courses/ids",
    params (
        ("ids" = String, Query, description = "Массив из этих ids, т.е ?ids=1&ids=2&ids=3"),
        ("Authorization" = String, Header, description = "(Опционально) JWT")
    ),
    responses(
        (status = 200, description = "Успешно", body = Vec<CourseInfo>),
        (status = 400, description = "Нет токена в за", body = Vec<CourseInfo>),
        (status = 500, description = "Что-то случилось", body = ErrorResponse)
    )
)]
pub async fn get_courses_by_ids(
    State(state): State<BasicState>,
    auth_token: OptionalBearerClaims,
    Query(ids): Query<IdsStruct>,
) -> Result<Response, AppError> {
    let courses =
        controllers::courses::get_courses_by_ids(&state, ids.ids, auth_token.0).await?;
    let body = json!({
        "data": courses,
    });
    return Ok((StatusCode::OK, axum::Json(body)).into_response());
}

// PUBLIC GET /course/{course_id} - Get info about a single course
#[utoipa::path(
    get,
    description = "Возвращает информацию о куосе",
    path = "/courses/{course_id}",
    params (
        ("course_id" = String, Path, description = "Айди курса"),
        ("Authorization" = String, Header, description = "(Опционально) JWT")
    ),
    responses(
        (status = 200, description = "Успешно", body = CourseInfo),
        (status = 500, description = "Что-то случилось", body = ErrorResponse)
    )
)]
pub async fn get_course(
    State(state): State<BasicState>,
    auth_token: OptionalBearerClaims,
    Path(course_id): Path<i32>,
) -> Result<Response, AppError> {

    let course =
        controllers::courses::get_course(&state, course_id, auth_token.0).await?;
    let body = json!({
        "data": course,
    });

    return Ok((StatusCode::OK, axum::Json(body)).into_response());
}

#[utoipa::path(
    get,
    description = "Возвращает прогресс по курсу",
    path = "/courses/{course_id}/progress",
    params (
        ("Authorization" = String, Header, description = "JWT"),
        ("course_id" = String, Path, description = "Айди курса")
    ),
    responses(
        (status = 200, description = "Успешно", body = CourseProgress),
        (status = 500, description = "Что-то случилось", body = ErrorResponse)
    )
)]
pub async fn get_course_progress(
    State(state): State<BasicState>,
    Path(course_id): Path<i32>,
    auth_token: AuthHeader,
) -> Result<Response, AppError> {
    let user_id = auth_token.claims.id;
    let progress = controllers::courses::get_course_progress(&state, course_id, user_id).await?;
    let body = json!({
        "data": progress,
    });
    return Ok((StatusCode::OK, axum::Json(body)).into_response());
}

// Favourite section
#[utoipa::path(
    post,
    description = "Добавляет курс в избранное",
    path = "/courses/{course_id}/favourite",
    params (
        ("Authorization" = String, Header, description = "JWT"),
        ("course_id" = String, Path, description = "Айди курса")
    ),
    responses(
        (status = 200, description = "Успешно"),
        (status = 500, description = "Что-то случилось", body = ErrorResponse)
    )
)]
pub async fn add_course_to_favourite(
    State(state): State<BasicState>,
    Path(course_id): Path<i32>,
    auth_header: AuthHeader,
) -> Result<Response, AppError> {
    let user_id = auth_header.claims.id;
    controllers::courses::add_course_to_favourite(&state, user_id, course_id).await?;
    Ok((StatusCode::OK).into_response())
}

#[utoipa::path(
    get,
    description = "Возвращаются айдишники курсов в избранном",
    path = "/courses/{course_id}/favourite",
    params (
        ("Authorization" = String, Header, description = "JWT"),
        ("course_id" = String, Path, description = "Айди курса")
    ),
    responses(
        (status = 200, description = "Успешно", body = Vec<i32>),
        (status = 500, description = "Что-то случилось", body = ErrorResponse)
    )
)]
pub async fn get_favourite_courses(
    State(state): State<BasicState>,
    auth_header: AuthHeader,
) -> Result<Response, AppError> {
    let user_id = auth_header.claims.id;

    let ids = controllers::courses::get_favourite_courses(&state, user_id).await?;
    let body = json!({
        "data": ids,
    });
    Ok((StatusCode::OK, axum::Json(body)).into_response())
}


// Internal
#[utoipa::path(
    get,
    description = "Возвращаются айдишники курсов юзера",
    path = "/internal/courses/users/{user_id}",
    params (
        ("user_id" = String, Path, description = "Айди юзера, курсы которого зафетчить")
    ),
    responses(
        (status = 200, description = "Успешно", body = Vec<i32>),
        (status = 500, description = "Что-то случилось", body = ErrorResponse)
    )
)]
pub async fn get_user_courses(State(state): State<BasicState>, Path(user_id): Path<u32>) -> Result<Response, AppError> {
    let ids = controllers::courses::get_user_courses(&state, user_id).await?;
    Ok((StatusCode::OK, axum::Json(ids)).into_response())
}

// Internal
#[utoipa::path(
    get,
    description = "Возвращаются айдишники начатых курсов юзера",
    path = "/internal/courses/users/{user_id}/started",
    params (
        ("user_id" = String, Path, description = "Айди юзера, курсы которого зафетчить")
    ),
    responses(
        (status = 200, description = "Успешно", body = Vec<i32>),
        (status = 500, description = "Что-то случилось", body = ErrorResponse)
    )
)]
pub async fn get_user_courses_started(State(state): State<BasicState>, Path(user_id): Path<u32>) -> Result<Response, AppError> {
    let ids = controllers::courses::get_user_courses_started(&state, user_id).await?;
    Ok((StatusCode::OK, axum::Json(ids)).into_response())
}

#[utoipa::path(
    get,
    description = "Возвращаются айдишники законч курсов юзера",
    path = "/internal/courses/users/{user_id}/completed",
    params (
        ("user_id" = String, Path, description = "Айди юзера, курсы которого зафетчить")
    ),
    responses(
        (status = 200, description = "Успешно", body = Vec<i32>),
        (status = 500, description = "Что-то случилось", body = ErrorResponse)
    )
)]
pub async fn get_user_courses_completed(State(state): State<BasicState>, Path(user_id): Path<u32>) -> Result<Response, AppError> {
    let ids = controllers::courses::get_user_courses_completed(&state, user_id).await?;
    Ok((StatusCode::OK, axum::Json(ids)).into_response())
}