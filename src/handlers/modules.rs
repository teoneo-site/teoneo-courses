use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::{
    common,
    controllers::{self, module::ModuleInfo},
    handlers::{self, ErrorResponse, ErrorTypes},
    AppState,
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
        (status = 200, description = "Успешно. Ответ без поля 'theory'", body = ModuleInfo),
        (status = 500, description = "Не удалось зафетчить модули, что-то не так с БД", body = ErrorResponse)
    )
)]
pub async fn get_modules_for_course(
    State(state): State<AppState>,
    Path(course_id): Path<i32>,
) -> Result<Response, Response> {
    match controllers::module::get_modules_for_course(&state, course_id).await {
        Ok(modules) => {
            let mut json: serde_json::value::Value = serde_json::Value::Null;
            json["data"] = serde_json::Value::Array([].to_vec());

            for module in modules.into_iter() {
                if let Some(data_array) = json["data"].as_array_mut() {
                    let mut value: serde_json::Value =
                        serde_json::Value::Object(serde_json::Map::new());
                    value["id"] = module.id.into();
                    value["course_id"] = module.course_id.into();
                    value["title"] = module.title.into();
                    value["description"] = module.description.into();
                    data_array.push(value);
                }
            }
            return Ok((StatusCode::OK, axum::Json(json)).into_response());
        }
        Err(why) => {
            tracing::error!("Could not fetch modules for course: {}", why);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(handlers::ErrorResponse::new(
                    ErrorTypes::InternalError,
                &format!("Could not fetch modules, because: {}", why),
                )),
            )
                .into_response());
        }
    };
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
        (status = 500, description = "Не удалось зафетчить модули, что-то не так с БД", body = ErrorResponse)
    )
)]
pub async fn get_module(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((course_id, module_id)): Path<(i32, i32)>,
) -> Result<Response, Response> {
    let authorization_token = headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|s| s.split_whitespace().last())
            .unwrap_or("");

    let is_subscribed_to_course = if let Ok(user_id) = common::token::verify_jwt_token(authorization_token) {
        match controllers::course::verify_ownership(&state, user_id as i32, course_id).await {
            Ok(_) => true,
            Err(why) => {
                tracing::error!("verify_ownership failed: {}", why);
                false
            }
        }
    } else {
        false
    };
        

    match controllers::module::get_module(&state, course_id, module_id).await {
        Ok(module) => {
            let body = if is_subscribed_to_course {
                json!({
                    "data": module,
                })
            } else {
                let mut value: serde_json::Value =
                    serde_json::Value::Object(serde_json::Map::new());
                value["id"] = module_id.into();
                value["course_id"] = course_id.into();
                value["title"] = module.title.into();
                value["description"] = module.description.into();

                json!({
                    "data": value
                })
            };

            return Ok((StatusCode::OK, axum::Json(body)).into_response());
        }
        Err(why) => {
            tracing::error!("Could not get module: {}", why);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(handlers::ErrorResponse::new(
                    ErrorTypes::InternalError,
                    &format!("Could not fetch the module: {}", why),
                )), // Should not panic, because struct is always valid for converting into JSON
            )
                .into_response());
        }
    };
}
