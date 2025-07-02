use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    common::{error::{AppError, ErrorResponse}, token::AuthHeader},
    controllers::{
        self,
        users::{UserInfo, UserInfoFull, UserStats},
    },
    AppState,
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValueInfo {
    Courses,
    User,
    All,
}

#[derive(Serialize, Deserialize)]
pub struct UserInfoQuery {
    value: ValueInfo,
}

async fn handle_result<T: Serialize>(
    fut: impl std::future::Future<Output = anyhow::Result<T>>,
) -> Result<Response, AppError> {
    let mut json_obj = serde_json::Value::Object(serde_json::Map::new());
    let resp = fut.await?;
    json_obj["data"] = serde_json::to_value(&resp).unwrap(); // Why would it panic?
    Ok((StatusCode::OK, axum::Json(json_obj)).into_response())
}


#[utoipa::path(
    get,
    description = "Используется для получения информации о юзере в зависимости от параметра (username, email, owned_courses). Если",
    path = "/user/info",
    params (
        ("value" = String, Query, description = "Принимает courses, user, all"),
         ("Authorization" = String, Header, description = "JWT")
    ),
    responses(
        (status = 200, description = "При значении all", body = UserInfoFull),
        (status = 201, description = "(200) При значении user", body = UserInfo),
        (status = 203, description = "(200) При значении courses", body = Vec<i32>),
        (status = 500, description = "Что-то случилось", body = ErrorResponse),
    )
)]
pub async fn get_user_info_and_courses(
    State(state): State<AppState>,
    auth_header: AuthHeader,
    Query(value): Query<UserInfoQuery>,
) -> Result<Response, AppError> {
    let user_id = auth_header.claims.id;
    match value.value {
        ValueInfo::All => {
            handle_result(controllers::users::get_user_info_all(&state, user_id)).await
        }
        ValueInfo::Courses => {
            handle_result(controllers::users::get_courses_info(&state, user_id)).await
        }
        ValueInfo::User => handle_result(controllers::users::get_user_info(&state, user_id)).await,
    }
}

#[utoipa::path(
    get,
    path = "/user/stats",
    description = "Возвращает статистику пользователя",
    params(
        ("Authorization" = String, Header, description = "JWT")
    ),
    responses(
        (status = 200, description = "Успешно", body = UserStats),
        (status = 500, description = "Что-то случилось", body = ErrorResponse),
    )
)]
pub async fn get_user_stats(
    State(state): State<AppState>,
    auth_header: AuthHeader,
) -> Result<Response, AppError> {
    let user_id = auth_header.claims.id;
    let stats = controllers::users::get_user_stats(&state, user_id).await?;
    Ok((StatusCode::OK, axum::Json(stats)).into_response())
}
