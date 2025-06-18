use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::extract::Query;
use serde::Deserialize;
use serde_json::json;
use utoipa::ToSchema;
use crate::{
    common::{self, token::Claims}, controllers::{self, course::{BasicCourseInfo, CourseProgress, ExtendedCourseInfo}}, db, handlers::{self, ErrorResponse, ErrorTypes}, AppState
};


#[derive(Deserialize)]
pub struct IdsStruct {
    ids: Vec<i32>,
}

#[derive(ToSchema)]
struct CoursesReturnVal {
    #[schema(example = json!([1, 2, 3]))]
    data: Vec<i32>
}

// PUBLIC GET /courses - Get a list of all available courses (for main page)
#[utoipa::path(
    get,
    description = "Возвращает список публичных курсов (общий)",
    path = "/courses",
    responses(
        (status = 200, description = "Успешно", body = CoursesReturnVal),
        (status = 500, description = "Курсы пустые (в БД)", body = ErrorResponse),
    )
)]
pub async fn get_all_courses(State(state): State<AppState>) -> Result<Response, Response> {
    match controllers::course::get_all_courses(&state).await {
        Ok(courses) => {
            let body = json!({
                "data": courses,
            });

            return Ok((StatusCode::OK, axum::Json(body)).into_response());
        }
        Err(why) => {
            tracing::error!("Could not fetch all courses: {}", why);
            
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(handlers::ErrorResponse::new(
                    ErrorTypes::InternalError,
                    "Could not fetch courses, because it is (probably) empty",
                )),
            )
                .into_response());
        }
    };
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
        (status = 200, description = "Успешно", body = Vec<BasicCourseInfo>),
        (status = 201, description = "(200) Успешно (При наличии токена)", body = Vec<ExtendedCourseInfo>),
        (status = 500, description = "Не удалось зафетчить курсы, что-то не так с БД", body = ErrorResponse)
    )
)]
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
                    tracing::error!("Could not get courses by ids extended: {}", why);
                    
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(handlers::ErrorResponse::new(
                            ErrorTypes::InternalError,
                            &format!("Could not fetch courses, because: {}", why),
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
                    tracing::error!("Could not get courses by ids basic: {}", why);
                    
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(handlers::ErrorResponse::new(
                            ErrorTypes::InternalError,
                            &format!("Could not fetch courses, What's up with the database?: {}", why),
                        )),
                    )
                        .into_response());
                }
            };
        }
    }
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
        (status = 200, description = "Успешно", body = BasicCourseInfo),
        (status = 201, description = "Успешно (При наличии токена)", body = ExtendedCourseInfo),
        (status = 500, description = "Не удалось зафетчить курс, что-то не так с БД", body = ErrorResponse)
    )
)]
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
                    tracing::error!("Could not get course extended info: {}", why);

                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(handlers::ErrorResponse::new(
                            ErrorTypes::InternalError,
                            &format!("Could not fetch the course, because: {}", why),
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
                    tracing::error!("Could not get course basicinfo: {}", why);

                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(handlers::ErrorResponse::new(
                            ErrorTypes::InternalError,
                           &format!("Could not fetch the course, because: {}", why),
                        )),
                    )
                        .into_response());
                }
            };
        }
    }
    
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
        (status = 500, description = "Не удалось зафетчить прогресс, что-то не так с БД", body = ErrorResponse)
    )
)]
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
            tracing::error!("Why could not fetch course progress: {}", why);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(handlers::ErrorResponse::new(
                    ErrorTypes::InternalError,
                    &format!("Could not fetch the course, because: {}", why),
                )),
            )
                .into_response());
        }
    }
}

// Favourite section
#[utoipa::path(
    post,
    description = "Добавляет курс в избранное",
    path = "/courses/{course_id}/favour",
    params (
        ("Authorization" = String, Header, description = "JWT"),
        ("course_id" = String, Path, description = "Айди курса")
    ),
    responses(
        (status = 200, description = "Успешно"),
        (status = 500, description = "Не удалось добавить курс в избранное, что-то не так с БД", body = ErrorResponse)
    )
)]
pub async fn add_course_to_favourite(State(state): State<AppState>, Path(course_id): Path<i32>, claims: Claims) -> Result<Response, Response> {
    let user_id = claims.id;
    match controllers::course::add_course_to_favourite(&state, user_id, course_id).await {
        Ok(_) => {
            return Ok((StatusCode::OK).into_response())
        }
        Err(why) => {
            tracing::error!("Could not favour a course: {}", why);
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


#[derive(ToSchema)]
struct FavouriteResponse {
    #[schema(example = json!([1, 2, 3]))]
    data: Vec<i32>
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
        (status = 200, description = "Успешно", body = FavouriteResponse),
        (status = 500, description = "Не удалось добавить курс в избранное, что-то не так с БД", body = ErrorResponse)
    )
)]
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
            tracing::error!("Could not get favourite courses: {}", why);
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