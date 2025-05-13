use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::{
    common::{self, token::Claims}, controllers, db, handlers::{self, ErrorTypes}, AppState
};

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
            return Ok((StatusCode::OK, axum::Json(json)).into_response());
        }
        Err(why) => {
            eprintln!("Why mo: {}", why);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(handlers::ErrorResponse::new(ErrorTypes::InternalError, "Could not fetch modules")),
            )
                .into_response());
        }
    };
}

pub async fn get_module(
    State(state): State<AppState>,
    claims: Claims,
    Path((course_id, module_id)): Path<(i32, i32)>,
) -> Result<Response, Response> {
    let is_subscribed_to_course = match controllers::course::verify_ownership(&state.pool, claims.id as i32, course_id).await {
        Ok(val) => val,
        Err(why) => {
            eprintln!("Why ver ownership failed: {}", why);
            false // user will see public part of the module
        }
    };

    match controllers::module::get_module(&state.pool, course_id, module_id).await {
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
            eprintln!("Why mo cock: {}", why);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(handlers::ErrorResponse::new(
                    ErrorTypes::InternalError,
                    "Could not fetch the module",
                )), // Should not panic, because struct is always valid for converting into JSON
            )
                .into_response());
        }
    };
}
