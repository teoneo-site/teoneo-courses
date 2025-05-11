use axum::{
    extract::{Path, State},
    http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::MySqlPool;

use crate::{
    common, controllers,
    handlers::{self, ErrorTypes},
    AppState,
};

use super::ResponseBody;

// PUBLCI GET /course/{course_id}/modules - Get info course's modules (id, course_id, title)
pub async fn get_modules_for_course(
    State(state): State<AppState>,
    Path(course_id): Path<i32>,
) -> Result<Response, Response> {
    match controllers::module::get_modules_for_course(&state.pool, course_id).await {
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
                    data_array.push(value);
                }
            }
            return Ok(ResponseBody::new(StatusCode::OK, None, json).into_response());
        }
        Err(why) => {
            eprintln!("Why mo: {}", why);
            return Err(ResponseBody::new(
                StatusCode::BAD_REQUEST,
                None,
                handlers::ErrorResponse::new(ErrorTypes::InternalError, "Could not fetch modules"),
            )
            .into_response());
        }
    };
}

pub async fn get_module(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((course_id, module_id)): Path<(i32, i32)>,
) -> Result<Response, Response> {
    let token = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.split_whitespace().last())
        .unwrap_or("");

    let is_subscribed_to_course = match common::token::verify_jwt_token(token) {
        Ok(user_id) => {
            // Check ownership TODO: API for verifying ownership of a course
            true
        }
        Err(why) => {
            eprintln!("Why mo kilka: {}", why);
            false // If user isnt logged in its okay, he'll see public part of the module
        }
    };

    match controllers::module::get_module(&state.pool, course_id, module_id).await {
        Ok(module) => {
            let body = if is_subscribed_to_course {
                json!({
                    "data": module,
                })
                .to_string()
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
                .to_string()
            };

            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            return Ok((StatusCode::OK, headers, body).into_response());
        }
        Err(why) => {
            eprintln!("Why mo cock: {}", why);

            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            return Err((
                StatusCode::BAD_REQUEST,
                headers,
                serde_json::to_string_pretty(&handlers::ErrorResponse::new(
                    ErrorTypes::InternalError,
                    "Could not fetch the module",
                ))
                .unwrap(), // Should not panic, because struct is always valid for converting into JSON
            )
                .into_response());
        }
    };
}
