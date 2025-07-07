use axum::{
    extract::{Path, State},
    http::{StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::{
    common::{error::{AppError, ErrorResponse}, token::OptionalBearerClaims},
    controllers::{self, modules::{ModuleInfo, ModuleShortInfo}},
    BasicState,
};

// PUBLCI GET /course/{course_id}/modules - Get info course's modules (id, course_id, title)
#[utoipa::path(
    get,
    description = "Возвращает инфу о модулях",
    path = "/courses/{course_id}/modules",
    params (
        ("course_id" = String, Path, description = "Айди курса")
    ),
    responses(
        (status = 200, description = "Успешно. Ответ без поля 'theory'", body = ModuleShortInfo),
        (status = 500, description = "Что-то случилось", body = ErrorResponse)
    )
)]
pub async fn get_modules_for_course(
    State(state): State<BasicState>,
    Path(course_id): Path<i32>,
) -> Result<Response, AppError> {
    let modules = controllers::modules::get_modules_for_course(&state, course_id).await?;
    let json = json!({
        "data": modules,
    });
    Ok((StatusCode::OK, axum::Json(json)).into_response())
}

#[utoipa::path(
    get,
    description = "Возвращает информацию о модуле курса",
    path = "/courses/{course_id}/modules/{module_id}",
    params (
        ("Authorization" = String, Header, description = "(Optional) JWT"),
        ("course_id" = i32, Path, description = "Айди курса"),
        ("module_id" = i32, Path, description = "Айди модуля")
    ),
    responses(
        (status = 200, description = "Успешно", body = ModuleInfo),
        (status = 201, description = "(200) Успешно. Ответ без поля 'theory'. Без токена", body = ModuleInfo),
        (status = 500, description = "Что-то случилось", body = ErrorResponse)
    )
)]
pub async fn get_module(
    State(state): State<BasicState>,
    auth_token: OptionalBearerClaims,
    Path((course_id, module_id)): Path<(i32, i32)>,
) -> Result<Response, AppError> {
    let is_subscribed_to_course =
        if let Some(user_id) = auth_token.0 {
            if let Ok(_) = controllers::courses::verify_ownership(&state, user_id, course_id).await {
                true
            } else {
                false
            }
        } else {
            false
        };

    let module = controllers::modules::get_module(&state, course_id, module_id).await?;
    let body = if is_subscribed_to_course {
        json!({
            "data": module,
        })
    } else {
        let mut value: serde_json::Value = serde_json::Value::Object(serde_json::Map::new());
        value["id"] = module_id.into();
        value["course_id"] = course_id.into();
        value["title"] = module.title.into();
        value["description"] = module.description.into();

        json!({
            "data": value
        })
    };

    Ok((StatusCode::OK, axum::Json(body)).into_response())
}
